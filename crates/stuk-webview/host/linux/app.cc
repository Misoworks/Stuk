#include "app.h"

#include <algorithm>
#include <cstdlib>
#include <string>
#include <utility>

#include "handler.h"
#include "include/cef_browser.h"
#include "include/cef_command_line.h"
#include "include/views/cef_browser_view.h"
#include "include/views/cef_window.h"
#include "include/wrapper/cef_helpers.h"

namespace {

int SwitchInt(CefRefPtr<CefCommandLine> command_line,
              const std::string& name,
              int fallback) {
  const std::string value = command_line->GetSwitchValue(name);
  if (value.empty()) {
    return fallback;
  }
  return std::atoi(value.c_str());
}

CefWindowHandle ParentWindow(CefRefPtr<CefCommandLine> command_line) {
  const std::string value = command_line->GetSwitchValue("stuk-parent-window");
  if (value.empty()) {
    return kNullWindowHandle;
  }
  return static_cast<CefWindowHandle>(std::strtoull(value.c_str(), nullptr, 0));
}

class StukBrowserViewDelegate : public CefBrowserViewDelegate {
 public:
  CefRefPtr<CefBrowserViewDelegate> GetDelegateForPopupBrowserView(
      CefRefPtr<CefBrowserView> browser_view,
      const CefBrowserSettings& settings,
      CefRefPtr<CefClient> client,
      bool is_devtools) override {
    return this;
  }

  cef_runtime_style_t GetBrowserRuntimeStyle() override {
    return CEF_RUNTIME_STYLE_ALLOY;
  }

 private:
  IMPLEMENT_REFCOUNTING(StukBrowserViewDelegate);
};

class StukWindowDelegate : public CefWindowDelegate {
 public:
  StukWindowDelegate(CefRefPtr<CefBrowserView> browser_view,
                     std::string title,
                     int width,
                     int height,
                     bool frameless)
      : browser_view_(browser_view),
        title_(std::move(title)),
        width_(width),
        height_(height),
        frameless_(frameless) {}

  void OnWindowCreated(CefRefPtr<CefWindow> window) override {
    window->SetTitle(title_);
    window->AddChildView(browser_view_);
    window->CenterWindow(CefSize(width_, height_));
    window->Show();
  }

  void OnWindowDestroyed(CefRefPtr<CefWindow> window) override {
    browser_view_ = nullptr;
  }

  bool CanClose(CefRefPtr<CefWindow> window) override {
    CefRefPtr<CefBrowser> browser = browser_view_ ? browser_view_->GetBrowser() : nullptr;
    return browser ? browser->GetHost()->TryCloseBrowser() : true;
  }

  CefSize GetPreferredSize(CefRefPtr<CefView> view) override {
    return CefSize(width_, height_);
  }

  CefRect GetInitialBounds(CefRefPtr<CefWindow> window) override {
    return frameless_ ? CefRect(0, 0, width_, height_) : CefRect();
  }

  bool IsFrameless(CefRefPtr<CefWindow> window) override {
    return frameless_;
  }

  cef_runtime_style_t GetWindowRuntimeStyle() override {
    return CEF_RUNTIME_STYLE_ALLOY;
  }

 private:
  CefRefPtr<CefBrowserView> browser_view_;
  const std::string title_;
  const int width_;
  const int height_;
  const bool frameless_;

  IMPLEMENT_REFCOUNTING(StukWindowDelegate);
};

void CreateStukBrowserWindow(CefRefPtr<CefCommandLine> command_line) {
  const std::string url_value = command_line->GetSwitchValue("url");
  const std::string url =
      url_value.empty() ? "about:blank" : std::string(url_value);
  const std::string title_value = command_line->GetSwitchValue("stuk-title");
  const std::string title =
      title_value.empty() ? "Stuk" : std::string(title_value);
  const int x = SwitchInt(command_line, "stuk-x", 0);
  const int y = SwitchInt(command_line, "stuk-y", 0);
  const int width = std::max(1, SwitchInt(command_line, "stuk-width", 800));
  const int height = std::max(1, SwitchInt(command_line, "stuk-height", 600));
  const bool frameless = command_line->HasSwitch("stuk-frameless");

  CefBrowserSettings browser_settings;
  if (command_line->HasSwitch("stuk-transparent")) {
    browser_settings.background_color = CefColorSetARGB(0, 0, 0, 0);
  }
  CefWindowInfo window_info;
  window_info.runtime_style = CEF_RUNTIME_STYLE_ALLOY;

  CefWindowHandle parent = ParentWindow(command_line);
  CefRefPtr<StukHandler> handler(new StukHandler);
  if (parent != kNullWindowHandle) {
    window_info.SetAsChild(parent, CefRect(x, y, width, height));
    CefBrowserHost::CreateBrowser(window_info, handler, url, browser_settings,
                                  nullptr, nullptr);
  } else {
    CefRefPtr<CefBrowserView> browser_view = CefBrowserView::CreateBrowserView(
        handler, url, browser_settings, nullptr, nullptr,
        new StukBrowserViewDelegate());
    CefWindow::CreateTopLevelWindow(
        new StukWindowDelegate(browser_view, title, width, height, frameless));
  }
}

}  // namespace

StukApp::StukApp() = default;

void StukApp::OnBeforeCommandLineProcessing(
    const CefString& process_type,
    CefRefPtr<CefCommandLine> command_line) {
  command_line->AppendSwitch("disable-vulkan");
  const std::string ozone_platform =
      command_line->GetSwitchValue("stuk-ozone-platform");
  if (!ozone_platform.empty()) {
    command_line->AppendSwitchWithValue("ozone-platform", ozone_platform);
    command_line->AppendSwitchWithValue("ozone-platform-hint", ozone_platform);
  }
  command_line->AppendSwitchWithValue(
      "disable-features", "Vulkan,DefaultANGLEVulkan,VulkanFromANGLE");
  command_line->AppendSwitch("disable-gpu");
  if (command_line->HasSwitch("stuk-transparent")) {
    command_line->AppendSwitch("enable-transparent-visuals");
    command_line->AppendSwitch("transparent-painting-enabled");
    command_line->AppendSwitchWithValue("default-background-color", "0x00000000");
  }
}

void StukApp::OnContextInitialized() {
  CEF_REQUIRE_UI_THREAD();
  CefRefPtr<CefCommandLine> command_line =
      CefCommandLine::GetGlobalCommandLine();
  CreateStukBrowserWindow(command_line);
}

bool StukApp::OnAlreadyRunningAppRelaunch(
    CefRefPtr<CefCommandLine> command_line,
    const CefString& current_directory) {
  CEF_REQUIRE_UI_THREAD();
  CreateStukBrowserWindow(command_line);
  return true;
}

CefRefPtr<CefClient> StukApp::GetDefaultClient() {
  return StukHandler::GetInstance();
}
