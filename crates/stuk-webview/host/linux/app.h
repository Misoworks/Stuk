#ifndef STUK_CEF_HOST_APP_H_
#define STUK_CEF_HOST_APP_H_

#include "include/cef_app.h"

class StukApp : public CefApp, public CefBrowserProcessHandler {
 public:
  StukApp();

  CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override {
    return this;
  }

  void OnBeforeCommandLineProcessing(
      const CefString& process_type,
      CefRefPtr<CefCommandLine> command_line) override;
  void OnContextInitialized() override;
  bool OnAlreadyRunningAppRelaunch(
      CefRefPtr<CefCommandLine> command_line,
      const CefString& current_directory) override;
  CefRefPtr<CefClient> GetDefaultClient() override;

 private:
  IMPLEMENT_REFCOUNTING(StukApp);
};

#endif
