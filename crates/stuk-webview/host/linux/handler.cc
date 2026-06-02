#include "handler.h"

#include <atomic>
#include <cstdlib>
#include <iostream>
#include <sstream>
#include <string>
#include <thread>
#include <utility>
#include <vector>

#include "include/cef_app.h"
#include "include/cef_parser.h"
#include "include/cef_task.h"
#include "include/views/cef_browser_view.h"
#include "include/views/cef_window.h"
#include "include/wrapper/cef_helpers.h"

namespace {
StukHandler* g_instance = nullptr;
std::atomic<bool> g_bridge_reader_started{false};

class QuitMessageLoopTask : public CefTask {
 public:
  void Execute() override {
    CefQuitMessageLoop();
  }

 private:
  IMPLEMENT_REFCOUNTING(QuitMessageLoopTask);
};

void ScheduleQuitMessageLoopFallback() {
  CefPostDelayedTask(TID_UI, new QuitMessageLoopTask, 150);
}

std::string DataUri(const std::string& body) {
  return "data:text/html;base64," +
         CefURIEncode(CefBase64Encode(body.data(), body.size()), false)
             .ToString();
}

CefRefPtr<CefWindow> WindowForBrowser(CefRefPtr<CefBrowser> browser) {
  CefRefPtr<CefBrowserView> browser_view =
      CefBrowserView::GetForBrowser(browser);
  return browser_view ? browser_view->GetWindow() : nullptr;
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
  if (authority.empty()) {
    return "null";
  }
  return scheme + "://" + authority;
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
    if (StukHandler* handler = StukHandler::GetInstance()) {
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
      if (!ParseBridgeResponse(line, &browser_id, &request_id, &ok, &payload)) {
        continue;
      }
      CefPostTask(TID_UI,
                  new BridgeResponseTask(browser_id, request_id, ok, payload));
    }
  }).detach();
}

bool HandleWindowCommand(CefRefPtr<CefBrowser> browser,
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

  CefRefPtr<CefWindow> window = WindowForBrowser(browser);
  if (!window) {
    return true;
  }

  if (command == "close") {
    if (!window->IsClosed()) {
      window->Hide();
    }
    browser->GetHost()->CloseBrowser(false);
    ScheduleQuitMessageLoopFallback();
  } else if (command == "minimize") {
    window->Minimize();
  } else if (command == "maximize") {
    window->Maximize();
  } else if (command == "restore") {
    window->Restore();
  } else if (command == "toggle-maximize") {
    if (window->IsMaximized()) {
      window->Restore();
    } else {
      window->Maximize();
    }
  }

  return true;
}

}  // namespace

StukHandler::StukHandler(std::vector<std::string> bridge_commands,
                         bool transparent_background)
    : bridge_commands_(bridge_commands.begin(), bridge_commands.end()),
      transparent_background_(transparent_background) {
  if (!g_instance) {
    g_instance = this;
  }
  if (!bridge_commands_.empty()) {
    StartBridgeReader();
  }
}

StukHandler::~StukHandler() {
  if (g_instance == this) {
    g_instance = nullptr;
  }
}

StukHandler* StukHandler::GetInstance() {
  return g_instance;
}

void StukHandler::OnTitleChange(CefRefPtr<CefBrowser> browser,
                                const CefString& title) {}

void StukHandler::OnDraggableRegionsChanged(
    CefRefPtr<CefBrowser> browser,
    CefRefPtr<CefFrame> frame,
    const std::vector<CefDraggableRegion>& regions) {
  CEF_REQUIRE_UI_THREAD();
  CefRefPtr<CefWindow> window = WindowForBrowser(browser);
  if (window) {
    window->SetDraggableRegions(regions);
  }
}

void StukHandler::OnAfterCreated(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  browsers_.push_back(browser);
}

bool StukHandler::DoClose(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  if (browsers_.size() == 1) {
    closing_ = true;
  }
  return false;
}

void StukHandler::OnBeforeClose(CefRefPtr<CefBrowser> browser) {
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

void StukHandler::OnLoadError(CefRefPtr<CefBrowser> browser,
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

void StukHandler::OnLoadStart(CefRefPtr<CefBrowser> browser,
                              CefRefPtr<CefFrame> frame,
                              TransitionType transition_type) {
  CEF_REQUIRE_UI_THREAD();
  if (frame->IsMain()) {
    InstallTransparentBackground(frame);
    InstallBridge(browser, frame);
  }
}

void StukHandler::OnLoadEnd(CefRefPtr<CefBrowser> browser,
                            CefRefPtr<CefFrame> frame,
                            int httpStatusCode) {
  CEF_REQUIRE_UI_THREAD();
  if (frame->IsMain()) {
    InstallTransparentBackground(frame);
    InstallBridge(browser, frame);
  }
}

bool StukHandler::OnBeforeBrowse(CefRefPtr<CefBrowser> browser,
                                 CefRefPtr<CefFrame> frame,
                                 CefRefPtr<CefRequest> request,
                                 bool user_gesture,
                                 bool is_redirect) {
  CEF_REQUIRE_UI_THREAD();
  const std::string url = request->GetURL();
  return HandleWindowCommand(browser, url) || HandleBridgeCommand(browser, frame, url);
}

bool StukHandler::HandleBridgeCommand(CefRefPtr<CefBrowser> browser,
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

void StukHandler::InstallBridge(CefRefPtr<CefBrowser> browser,
                                CefRefPtr<CefFrame> frame) {
  if (bridge_commands_.empty()) {
    return;
  }
  frame->ExecuteJavaScript(BridgeInstallScript(bridge_commands_), frame->GetURL(),
                           0);
}

void StukHandler::InstallTransparentBackground(CefRefPtr<CefFrame> frame) {
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

void StukHandler::ResolveBridgeResponse(const std::string& browser_id,
                                        const std::string& request_id,
                                        bool ok,
                                        const std::string& payload) {
  CEF_REQUIRE_UI_THREAD();
  CefRefPtr<CefBrowser> target;
  const int expected_id = std::atoi(browser_id.c_str());
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
