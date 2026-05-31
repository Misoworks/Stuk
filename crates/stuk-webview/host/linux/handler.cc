#include "handler.h"

#include <sstream>
#include <string>

#include "include/cef_app.h"
#include "include/cef_parser.h"
#include "include/views/cef_browser_view.h"
#include "include/views/cef_window.h"
#include "include/wrapper/cef_helpers.h"

namespace {
StukHandler* g_instance = nullptr;

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

StukHandler::StukHandler() {
  if (!g_instance) {
    g_instance = this;
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

bool StukHandler::OnBeforeBrowse(CefRefPtr<CefBrowser> browser,
                                 CefRefPtr<CefFrame> frame,
                                 CefRefPtr<CefRequest> request,
                                 bool user_gesture,
                                 bool is_redirect) {
  CEF_REQUIRE_UI_THREAD();
  return HandleWindowCommand(browser, request->GetURL());
}
