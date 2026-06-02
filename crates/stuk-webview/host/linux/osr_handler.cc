#include "osr_handler.h"

#include <algorithm>
#include <atomic>
#include <cerrno>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <sstream>
#include <string>
#include <thread>
#include <utility>
#include <vector>

#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#include "include/cef_app.h"
#include "include/cef_browser.h"
#include "include/cef_parser.h"
#include "include/cef_task.h"
#include "include/internal/cef_types.h"
#include "include/wrapper/cef_helpers.h"

namespace {
StukOsrHandler* g_instance = nullptr;
std::atomic<bool> g_bridge_reader_started{false};

int SwitchInt(CefRefPtr<CefCommandLine> command_line,
              const std::string& name,
              int fallback) {
  const std::string value = command_line->GetSwitchValue(name);
  if (value.empty()) {
    return fallback;
  }
  return std::atoi(value.c_str());
}

float SwitchFloat(CefRefPtr<CefCommandLine> command_line,
                  const std::string& name,
                  float fallback) {
  const std::string value = command_line->GetSwitchValue(name);
  if (value.empty()) {
    return fallback;
  }
  return std::atof(value.c_str());
}

std::vector<std::string> Split(const std::string& value, char separator) {
  std::vector<std::string> parts;
  std::stringstream stream(value);
  std::string item;
  while (std::getline(stream, item, separator)) {
    parts.push_back(item);
  }
  return parts;
}

std::vector<std::string> BridgeCommands(CefRefPtr<CefCommandLine> command_line) {
  std::vector<std::string> commands;
  for (const auto& item :
       Split(std::string(command_line->GetSwitchValue("stuk-bridge-commands")), ',')) {
    if (!item.empty()) {
      commands.push_back(item);
    }
  }
  return commands;
}

std::string DecodeUriComponent(const std::string& value) {
  return CefURIDecode(
             value, true,
             static_cast<cef_uri_unescape_rule_t>(
                 UU_SPACES | UU_URL_SPECIAL_CHARS_EXCEPT_PATH_SEPARATORS |
                 UU_REPLACE_PLUS_WITH_SPACE))
      .ToString();
}

std::string QueryValue(const std::string& url, const std::string& name) {
  const size_t query_start = url.find('?');
  if (query_start == std::string::npos) {
    return "";
  }
  const std::string needle = name + "=";
  size_t cursor = query_start + 1;
  while (cursor < url.size()) {
    const size_t next = url.find('&', cursor);
    const size_t end = next == std::string::npos ? url.size() : next;
    const std::string part = url.substr(cursor, end - cursor);
    if (part.rfind(needle, 0) == 0) {
      return DecodeUriComponent(part.substr(needle.size()));
    }
    if (next == std::string::npos) {
      break;
    }
    cursor = next + 1;
  }
  return "";
}

std::string BridgeRequestId(const std::string& url) {
  const std::string prefix = "stuk://bridge/";
  if (url.rfind(prefix, 0) != 0) {
    return "";
  }
  const size_t start = prefix.size();
  const size_t end = url.find_first_of("?#", start);
  return DecodeUriComponent(
      url.substr(start, end == std::string::npos ? std::string::npos : end - start));
}

std::string UrlOrigin(const std::string& url) {
  const size_t scheme_end = url.find("://");
  if (scheme_end == std::string::npos) {
    return "null";
  }
  const std::string scheme = url.substr(0, scheme_end);
  if (scheme == "file" || scheme == "about" || scheme == "devtools") {
    return scheme + "://";
  }
  const size_t authority_start = scheme_end + 3;
  const size_t authority_end = url.find_first_of("/?#", authority_start);
  const std::string authority = url.substr(
      authority_start,
      authority_end == std::string::npos ? std::string::npos
                                         : authority_end - authority_start);
  return authority.empty() ? "null" : scheme + "://" + authority;
}

std::string JsString(const std::string& value) {
  std::string output = "\"";
  for (char c : value) {
    switch (c) {
      case '\\':
        output += "\\\\";
        break;
      case '"':
        output += "\\\"";
        break;
      case '\n':
        output += "\\n";
        break;
      case '\r':
        output += "\\r";
        break;
      case '\t':
        output += "\\t";
        break;
      default:
        output += c;
        break;
    }
  }
  output += "\"";
  return output;
}

std::string JsArray(const std::set<std::string>& values) {
  std::string output = "[";
  bool first = true;
  for (const auto& value : values) {
    if (!first) {
      output += ",";
    }
    output += JsString(value);
    first = false;
  }
  output += "]";
  return output;
}

std::string BridgeInstallScript(const std::set<std::string>& commands) {
  return "(function(){"
         "if(window.stuk&&window.stuk.bridge&&window.stuk.bridge.__native)return;"
         "const commands=new Set(" +
         JsArray(commands) +
         ");"
         "const pending=new Map();let nextId=1;"
         "window.__stukBridgeResolve=function(id,ok,payload){"
         "const entry=pending.get(String(id));if(!entry)return;"
         "pending.delete(String(id));"
         "if(ok){entry.resolve(payload);}else{entry.reject(new Error((payload&&payload.message)||'Stuk bridge command failed'));}"
         "};"
         "window.stuk=window.stuk||{};"
         "window.stuk.bridge={__native:true,commands:Array.from(commands),invoke(name,params={}){"
         "if(!commands.has(name))return Promise.reject(new Error('Stuk bridge command not registered: '+name));"
         "const id=String(nextId++);"
         "const payload=encodeURIComponent(JSON.stringify(params));"
         "const url='stuk://bridge/'+encodeURIComponent(id)+'?name='+encodeURIComponent(name)+'&payload='+payload;"
         "return new Promise((resolve,reject)=>{"
         "pending.set(id,{resolve,reject});"
         "setTimeout(()=>{if(pending.has(id)){pending.delete(id);reject(new Error('Stuk bridge command timed out: '+name));}},60000);"
         "window.location.href=url;"
         "});"
         "}};"
         "})();";
}

std::string DataUri(const std::string& body) {
  return "data:text/html;base64," +
         CefURIEncode(CefBase64Encode(body.data(), body.size()), false)
             .ToString();
}

bool ParseBridgeResponse(const std::string& line,
                         std::string* browser_id,
                         std::string* request_id,
                         bool* ok,
                         std::string* payload) {
  const std::string prefix = "STUK_BRIDGE_RESPONSE\t";
  if (line.rfind(prefix, 0) != 0) {
    return false;
  }
  std::vector<std::string> parts;
  size_t cursor = prefix.size();
  while (parts.size() < 3) {
    const size_t next = line.find('\t', cursor);
    if (next == std::string::npos) {
      return false;
    }
    parts.push_back(line.substr(cursor, next - cursor));
    cursor = next + 1;
  }
  *browser_id = parts[0];
  *request_id = parts[1];
  *ok = parts[2] == "ok";
  *payload = line.substr(cursor);
  return true;
}

class BridgeResponseTask : public CefTask {
 public:
  BridgeResponseTask(std::string browser_id,
                     std::string request_id,
                     bool ok,
                     std::string payload)
      : browser_id_(std::move(browser_id)),
        request_id_(std::move(request_id)),
        ok_(ok),
        payload_(std::move(payload)) {}

  void Execute() override {
    if (StukOsrHandler* handler = StukOsrHandler::GetInstance()) {
      handler->ResolveBridgeResponse(browser_id_, request_id_, ok_, payload_);
    }
  }

 private:
  const std::string browser_id_;
  const std::string request_id_;
  const bool ok_;
  const std::string payload_;

  IMPLEMENT_REFCOUNTING(BridgeResponseTask);
};

void StartBridgeReader() {
  bool expected = false;
  if (!g_bridge_reader_started.compare_exchange_strong(expected, true)) {
    return;
  }
  std::thread([] {
    std::string line;
    while (std::getline(std::cin, line)) {
      std::string browser_id;
      std::string request_id;
      std::string payload;
      bool ok = false;
      if (ParseBridgeResponse(line, &browser_id, &request_id, &ok, &payload)) {
        CefPostTask(TID_UI,
                    new BridgeResponseTask(browser_id, request_id, ok, payload));
      }
    }
  }).detach();
}

class OsrCommandTask : public CefTask {
 public:
  OsrCommandTask(CefRefPtr<StukOsrHandler> handler, std::string line)
      : handler_(handler), line_(std::move(line)) {}

  void Execute() override {
    handler_->HandleControlLine(line_);
  }

 private:
  CefRefPtr<StukOsrHandler> handler_;
  const std::string line_;

  IMPLEMENT_REFCOUNTING(OsrCommandTask);
};

class QuitTask : public CefTask {
 public:
  void Execute() override {
    CefQuitMessageLoop();
  }

 private:
  IMPLEMENT_REFCOUNTING(QuitTask);
};

void PutU32(std::vector<char>* buffer, size_t offset, uint32_t value) {
  (*buffer)[offset + 0] = static_cast<char>(value & 0xff);
  (*buffer)[offset + 1] = static_cast<char>((value >> 8) & 0xff);
  (*buffer)[offset + 2] = static_cast<char>((value >> 16) & 0xff);
  (*buffer)[offset + 3] = static_cast<char>((value >> 24) & 0xff);
}

void PutI32(std::vector<char>* buffer, size_t offset, int32_t value) {
  PutU32(buffer, offset, static_cast<uint32_t>(value));
}

bool SendAll(int fd, const char* bytes, size_t len) {
  size_t sent = 0;
  while (sent < len) {
    const ssize_t result = send(fd, bytes + sent, len - sent, MSG_NOSIGNAL);
    if (result <= 0) {
      return false;
    }
    sent += static_cast<size_t>(result);
  }
  return true;
}

int KeyCodeForName(const std::string& key) {
  if (key.size() == 1) {
    unsigned char c = key[0];
    if (c >= 'a' && c <= 'z') {
      return c - 'a' + 'A';
    }
    return c;
  }
  if (key.rfind("Key", 0) == 0 && key.size() == 4) {
    return key[3];
  }
  if (key == "Enter") return 13;
  if (key == "Backspace") return 8;
  if (key == "Tab") return 9;
  if (key == "Escape") return 27;
  if (key == " " || key == "Space") return 32;
  if (key == "ArrowLeft") return 37;
  if (key == "ArrowUp") return 38;
  if (key == "ArrowRight") return 39;
  if (key == "ArrowDown") return 40;
  if (key == "Delete") return 46;
  if (key == "Home") return 36;
  if (key == "End") return 35;
  if (key == "PageUp") return 33;
  if (key == "PageDown") return 34;
  return 0;
}

std::u16string Utf8ToUtf16(const std::string& value) {
  std::u16string output;
  for (size_t i = 0; i < value.size();) {
    uint32_t cp = static_cast<unsigned char>(value[i++]);
    if ((cp & 0x80) == 0) {
    } else if ((cp & 0xe0) == 0xc0 && i < value.size()) {
      const uint32_t b1 = static_cast<unsigned char>(value[i++]);
      cp = ((cp & 0x1f) << 6) | (b1 & 0x3f);
    } else if ((cp & 0xf0) == 0xe0 && i + 1 < value.size()) {
      const uint32_t b1 = static_cast<unsigned char>(value[i++]);
      const uint32_t b2 = static_cast<unsigned char>(value[i++]);
      cp = ((cp & 0x0f) << 12) | ((b1 & 0x3f) << 6) | (b2 & 0x3f);
    } else if ((cp & 0xf8) == 0xf0 && i + 2 < value.size()) {
      const uint32_t b1 = static_cast<unsigned char>(value[i++]);
      const uint32_t b2 = static_cast<unsigned char>(value[i++]);
      const uint32_t b3 = static_cast<unsigned char>(value[i++]);
      cp = ((cp & 0x07) << 18) | ((b1 & 0x3f) << 12) |
           ((b2 & 0x3f) << 6) | (b3 & 0x3f);
    } else {
      continue;
    }
    if (cp <= 0xffff) {
      output.push_back(static_cast<char16_t>(cp));
    } else {
      cp -= 0x10000;
      output.push_back(static_cast<char16_t>(0xd800 + (cp >> 10)));
      output.push_back(static_cast<char16_t>(0xdc00 + (cp & 0x3ff)));
    }
  }
  return output;
}

cef_mouse_button_type_t MouseButtonFromString(const std::string& value) {
  if (value == "right") return MBT_RIGHT;
  if (value == "middle") return MBT_MIDDLE;
  return MBT_LEFT;
}

std::string CursorName(cef_cursor_type_t type) {
  switch (type) {
    case CT_HAND:
      return "pointer";
    case CT_IBEAM:
      return "text";
    case CT_CROSS:
      return "crosshair";
    case CT_MOVE:
      return "move";
    case CT_WAIT:
      return "wait";
    case CT_HELP:
      return "help";
    case CT_NOTALLOWED:
    case CT_NODROP:
      return "not-allowed";
    case CT_EASTWESTRESIZE:
    case CT_COLUMNRESIZE:
      return "ew-resize";
    case CT_NORTHSOUTHRESIZE:
    case CT_ROWRESIZE:
      return "ns-resize";
    case CT_NORTHEASTRESIZE:
      return "ne-resize";
    case CT_NORTHWESTRESIZE:
      return "nw-resize";
    case CT_SOUTHEASTRESIZE:
      return "se-resize";
    case CT_SOUTHWESTRESIZE:
      return "sw-resize";
    default:
      return "default";
  }
}
}  // namespace

StukOsrHandler::StukOsrHandler(std::string socket_path,
                               int width,
                               int height,
                               float scale,
                               std::vector<std::string> bridge_commands,
                               bool transparent_background)
    : socket_path_(std::move(socket_path)),
      width_(std::max(1, width)),
      height_(std::max(1, height)),
      scale_(std::max(0.25f, scale)),
      bridge_commands_(bridge_commands.begin(), bridge_commands.end()),
      transparent_background_(transparent_background) {
  if (!g_instance) {
    g_instance = this;
  }
  ConnectSocket();
  StartCommandReader();
  if (!bridge_commands_.empty()) {
    StartBridgeReader();
  }
}

StukOsrHandler::~StukOsrHandler() {
  if (socket_fd_ >= 0) {
    close(socket_fd_);
  }
  if (g_instance == this) {
    g_instance = nullptr;
  }
}

StukOsrHandler* StukOsrHandler::GetInstance() {
  return g_instance;
}

bool StukOsrHandler::ConnectSocket() {
  socket_fd_ = socket(AF_UNIX, SOCK_STREAM, 0);
  if (socket_fd_ < 0) {
    return false;
  }
  sockaddr_un addr{};
  addr.sun_family = AF_UNIX;
  if (socket_path_.size() >= sizeof(addr.sun_path)) {
    return false;
  }
  std::strncpy(addr.sun_path, socket_path_.c_str(), sizeof(addr.sun_path) - 1);
  if (connect(socket_fd_, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) != 0) {
    close(socket_fd_);
    socket_fd_ = -1;
    return false;
  }
  return true;
}

void StukOsrHandler::StartCommandReader() {
  if (socket_fd_ < 0) {
    return;
  }
  const int fd = socket_fd_;
  CefRefPtr<StukOsrHandler> self(this);
  std::thread([self, fd] {
    std::string pending;
    char buffer[2048];
    while (true) {
      const ssize_t n = recv(fd, buffer, sizeof(buffer), 0);
      if (n <= 0) {
        break;
      }
      pending.append(buffer, static_cast<size_t>(n));
      size_t newline = 0;
      while ((newline = pending.find('\n')) != std::string::npos) {
        std::string line = pending.substr(0, newline);
        pending.erase(0, newline + 1);
        CefPostTask(TID_UI, new OsrCommandTask(self, line));
      }
    }
  }).detach();
}

bool StukOsrHandler::SendMessage(uint32_t kind,
                                 uint32_t width,
                                 uint32_t height,
                                 int32_t x,
                                 int32_t y,
                                 const void* payload,
                                 uint32_t payload_len) {
  if (socket_fd_ < 0) {
    return false;
  }
  std::lock_guard<std::mutex> lock(socket_mutex_);
  std::vector<char> header(28, 0);
  header[0] = 'S';
  header[1] = 'K';
  header[2] = 'O';
  header[3] = 'R';
  PutU32(&header, 4, kind);
  PutU32(&header, 8, width);
  PutU32(&header, 12, height);
  PutI32(&header, 16, x);
  PutI32(&header, 20, y);
  PutU32(&header, 24, payload_len);
  return SendAll(socket_fd_, header.data(), header.size()) &&
         (payload_len == 0 ||
          SendAll(socket_fd_, static_cast<const char*>(payload), payload_len));
}

bool StukOsrHandler::OnCursorChange(CefRefPtr<CefBrowser> browser,
                                    CefCursorHandle cursor,
                                    cef_cursor_type_t type,
                                    const CefCursorInfo& custom_cursor_info) {
  const std::string name = CursorName(type);
  SendMessage(4, 0, 0, 0, 0, name.data(), static_cast<uint32_t>(name.size()));
  return true;
}

void StukOsrHandler::OnAfterCreated(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  browsers_.push_back(browser);
  browser_ = browser;
  browser->GetHost()->SetWindowlessFrameRate(60);
}

bool StukOsrHandler::DoClose(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  if (browsers_.size() == 1) {
    closing_ = true;
  }
  return false;
}

void StukOsrHandler::OnBeforeClose(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  for (auto it = browsers_.begin(); it != browsers_.end(); ++it) {
    if ((*it)->IsSame(browser)) {
      browsers_.erase(it);
      break;
    }
  }
  if (browsers_.empty()) {
    CefQuitMessageLoop();
  }
}

void StukOsrHandler::OnLoadError(CefRefPtr<CefBrowser> browser,
                                 CefRefPtr<CefFrame> frame,
                                 ErrorCode errorCode,
                                 const CefString& errorText,
                                 const CefString& failedUrl) {
  CEF_REQUIRE_UI_THREAD();
  if (errorCode == ERR_ABORTED) {
    return;
  }
  std::stringstream body;
  body << "<!doctype html><meta charset=\"utf-8\"><body style=\"margin:0;"
          "font:14px system-ui;background:#111;color:#eee;padding:24px\">"
       << "<h2>Failed to load</h2><p>" << std::string(failedUrl) << "</p><p>"
       << std::string(errorText) << "</p></body>";
  frame->LoadURL(DataUri(body.str()));
}

void StukOsrHandler::OnLoadStart(CefRefPtr<CefBrowser> browser,
                                 CefRefPtr<CefFrame> frame,
                                 TransitionType transition_type) {
  CEF_REQUIRE_UI_THREAD();
  if (frame->IsMain()) {
    InstallTransparentBackground(frame);
    InstallBridge(browser, frame);
  }
}

void StukOsrHandler::OnLoadEnd(CefRefPtr<CefBrowser> browser,
                               CefRefPtr<CefFrame> frame,
                               int httpStatusCode) {
  CEF_REQUIRE_UI_THREAD();
  if (frame->IsMain()) {
    InstallTransparentBackground(frame);
    InstallBridge(browser, frame);
  }
}

bool StukOsrHandler::OnBeforeBrowse(CefRefPtr<CefBrowser> browser,
                                    CefRefPtr<CefFrame> frame,
                                    CefRefPtr<CefRequest> request,
                                    bool user_gesture,
                                    bool is_redirect) {
  CEF_REQUIRE_UI_THREAD();
  const std::string url = request->GetURL();
  return HandleWindowCommand(browser, url) || HandleBridgeCommand(browser, frame, url);
}

bool StukOsrHandler::GetScreenInfo(CefRefPtr<CefBrowser> browser,
                                   CefScreenInfo& screen_info) {
  screen_info.device_scale_factor = scale_;
  screen_info.depth = 32;
  screen_info.depth_per_component = 8;
  screen_info.rect = CefRect(0, 0, width_, height_);
  screen_info.available_rect = CefRect(0, 0, width_, height_);
  return true;
}

void StukOsrHandler::GetViewRect(CefRefPtr<CefBrowser> browser, CefRect& rect) {
  rect = CefRect(0, 0, width_, height_);
}

void StukOsrHandler::OnPopupShow(CefRefPtr<CefBrowser> browser, bool show) {
  if (!show) {
    SendMessage(3, 0, 0, 0, 0, nullptr, 0);
  }
}

void StukOsrHandler::OnPopupSize(CefRefPtr<CefBrowser> browser,
                                 const CefRect& rect) {
  popup_rect_ = rect;
}

void StukOsrHandler::OnPaint(CefRefPtr<CefBrowser> browser,
                             PaintElementType type,
                             const RectList& dirtyRects,
                             const void* buffer,
                             int width,
                             int height) {
  const uint32_t kind = type == PET_POPUP ? 2 : 1;
  const int32_t x = type == PET_POPUP ? popup_rect_.x : 0;
  const int32_t y = type == PET_POPUP ? popup_rect_.y : 0;
  const uint32_t payload_len = static_cast<uint32_t>(width * height * 4);
  SendMessage(kind, width, height, x, y, buffer, payload_len);
}

void StukOsrHandler::HandleControlLine(const std::string& line) {
  CEF_REQUIRE_UI_THREAD();
  const auto parts = Split(line, '\t');
  if (parts.empty() || !browser_) {
    return;
  }
  CefRefPtr<CefBrowserHost> host = browser_->GetHost();
  if (parts[0] == "resize" && parts.size() >= 4) {
    width_ = std::max(1, std::atoi(parts[1].c_str()));
    height_ = std::max(1, std::atoi(parts[2].c_str()));
    scale_ = std::max(0.25f, static_cast<float>(std::atof(parts[3].c_str())));
    host->NotifyScreenInfoChanged();
    host->WasResized();
  } else if (parts[0] == "mouse_move" && parts.size() >= 5) {
    CefMouseEvent event;
    event.x = std::atoi(parts[1].c_str());
    event.y = std::atoi(parts[2].c_str());
    event.modifiers = std::strtoul(parts[3].c_str(), nullptr, 10);
    host->SendMouseMoveEvent(event, std::atoi(parts[4].c_str()) != 0);
  } else if (parts[0] == "mouse_click" && parts.size() >= 7) {
    CefMouseEvent event;
    event.x = std::atoi(parts[1].c_str());
    event.y = std::atoi(parts[2].c_str());
    const auto button = MouseButtonFromString(parts[3]);
    event.modifiers = std::strtoul(parts[4].c_str(), nullptr, 10);
    const bool up = std::atoi(parts[5].c_str()) != 0;
    const int click_count = std::max(1, std::atoi(parts[6].c_str()));
    host->SendMouseClickEvent(event, button, up, click_count);
  } else if (parts[0] == "mouse_wheel" && parts.size() >= 6) {
    CefMouseEvent event;
    event.x = std::atoi(parts[1].c_str());
    event.y = std::atoi(parts[2].c_str());
    const int dx = std::atoi(parts[3].c_str());
    const int dy = std::atoi(parts[4].c_str());
    event.modifiers = std::strtoul(parts[5].c_str(), nullptr, 10);
    host->SendMouseWheelEvent(event, dx, dy);
  } else if (parts[0] == "key" && parts.size() >= 6) {
    const bool pressed = std::atoi(parts[1].c_str()) != 0;
    const std::string key = DecodeUriComponent(parts[2]);
    const std::string text = DecodeUriComponent(parts[3]);
    const uint32_t modifiers = std::strtoul(parts[4].c_str(), nullptr, 10);
    const int key_code = KeyCodeForName(key);
    CefKeyEvent event;
    event.type = pressed ? KEYEVENT_RAWKEYDOWN : KEYEVENT_KEYUP;
    event.modifiers = modifiers;
    event.windows_key_code = key_code;
    event.native_key_code = key_code;
    host->SendKeyEvent(event);
    if (pressed && !text.empty()) {
      for (char16_t ch : Utf8ToUtf16(text)) {
        CefKeyEvent char_event;
        char_event.type = KEYEVENT_CHAR;
        char_event.modifiers = modifiers;
        char_event.windows_key_code = ch;
        char_event.native_key_code = ch;
        char_event.character = ch;
        char_event.unmodified_character = ch;
        host->SendKeyEvent(char_event);
      }
    }
  } else if (parts[0] == "focus" && parts.size() >= 2) {
    host->SetFocus(std::atoi(parts[1].c_str()) != 0);
  } else if (parts[0] == "close") {
    host->CloseBrowser(false);
    CefPostDelayedTask(TID_UI, new QuitTask, 250);
  }
}

bool StukOsrHandler::HandleWindowCommand(CefRefPtr<CefBrowser> browser,
                                         const std::string& url) {
  const std::string prefix = "stuk://window/";
  if (url.rfind(prefix, 0) != 0) {
    return false;
  }
  std::string command = url.substr(prefix.size());
  const size_t query = command.find_first_of("?#");
  if (query != std::string::npos) {
    command = command.substr(0, query);
  }
  if (command == "close") {
    browser->GetHost()->CloseBrowser(false);
    CefPostDelayedTask(TID_UI, new QuitTask, 250);
  }
  return true;
}

bool StukOsrHandler::HandleBridgeCommand(CefRefPtr<CefBrowser> browser,
                                         CefRefPtr<CefFrame> frame,
                                         const std::string& url) {
  const std::string prefix = "stuk://bridge/";
  if (url.rfind(prefix, 0) != 0) {
    return false;
  }
  const std::string request_id = BridgeRequestId(url);
  const std::string command = QueryValue(url, "name");
  const std::string payload = QueryValue(url, "payload");
  const std::string browser_id = std::to_string(browser->GetIdentifier());
  const std::string origin = UrlOrigin(frame ? std::string(frame->GetURL()) : "");
  if (request_id.empty() || command.empty()) {
    ResolveBridgeResponse(browser_id, request_id, false,
                          "{\"message\":\"Malformed Stuk bridge request\"}");
    return true;
  }
  if (!bridge_commands_.contains(command)) {
    ResolveBridgeResponse(
        browser_id, request_id, false,
        "{\"message\":\"Stuk bridge command is not allowlisted\"}");
    return true;
  }
  std::cout << "STUK_BRIDGE_REQUEST\t" << browser_id << "\t" << request_id
            << "\t" << origin << "\t" << command << "\t"
            << (payload.empty() ? "{}" : payload)
            << std::endl;
  return true;
}

void StukOsrHandler::InstallBridge(CefRefPtr<CefBrowser> browser,
                                   CefRefPtr<CefFrame> frame) {
  if (!bridge_commands_.empty()) {
    frame->ExecuteJavaScript(BridgeInstallScript(bridge_commands_), frame->GetURL(),
                             0);
  }
}

void StukOsrHandler::InstallTransparentBackground(CefRefPtr<CefFrame> frame) {
  if (!transparent_background_) {
    return;
  }
  frame->ExecuteJavaScript(
      "(function(){"
      "if(document.documentElement){document.documentElement.style.background='transparent';}"
      "if(document.body){document.body.style.background='transparent';}"
      "if(!document.querySelector('style[data-stuk-transparent-background]')){"
      "const style=document.createElement('style');"
      "style.setAttribute('data-stuk-transparent-background','');"
      "style.textContent='html,body{background:transparent!important;}';"
      "document.head&&document.head.appendChild(style);"
      "}"
      "})();",
      frame->GetURL(), 0);
}

void StukOsrHandler::ResolveBridgeResponse(const std::string& browser_id,
                                           const std::string& request_id,
                                           bool ok,
                                           const std::string& payload) {
  CEF_REQUIRE_UI_THREAD();
  const int expected_id = std::atoi(browser_id.c_str());
  CefRefPtr<CefBrowser> target;
  for (auto& browser : browsers_) {
    if (browser->GetIdentifier() == expected_id) {
      target = browser;
      break;
    }
  }
  if (!target || request_id.empty()) {
    return;
  }
  const std::string script =
      "window.__stukBridgeResolve&&window.__stukBridgeResolve(" +
      JsString(request_id) + "," + (ok ? "true" : "false") + "," +
      (payload.empty() ? "null" : payload) + ");";
  target->GetMainFrame()->ExecuteJavaScript(script, target->GetMainFrame()->GetURL(),
                                            0);
}

void CreateStukOsrBrowser(CefRefPtr<CefCommandLine> command_line) {
  const std::string url_value = command_line->GetSwitchValue("url");
  const std::string url =
      url_value.empty() ? "about:blank" : std::string(url_value);
  const int width = std::max(1, SwitchInt(command_line, "stuk-width", 800));
  const int height = std::max(1, SwitchInt(command_line, "stuk-height", 600));
  const float scale = SwitchFloat(command_line, "stuk-scale", 1.0f);
  const std::string socket_path = command_line->GetSwitchValue("stuk-osr-socket");

  CefBrowserSettings browser_settings;
  browser_settings.windowless_frame_rate = 60;
  if (command_line->HasSwitch("stuk-transparent")) {
    browser_settings.background_color = CefColorSetARGB(0, 0, 0, 0);
  }

  CefWindowInfo window_info;
  window_info.SetAsWindowless(kNullWindowHandle);
  CefRefPtr<StukOsrHandler> handler(new StukOsrHandler(
      socket_path, width, height, scale, BridgeCommands(command_line),
      command_line->HasSwitch("stuk-transparent")));
  CefBrowserHost::CreateBrowser(window_info, handler, url, browser_settings,
                                nullptr, nullptr);
}
