#ifndef STUK_CEF_HOST_HANDLER_H_
#define STUK_CEF_HOST_HANDLER_H_

#include <list>
#include <vector>

#include "include/cef_client.h"

class StukHandler : public CefClient,
                    public CefDragHandler,
                    public CefDisplayHandler,
                    public CefLifeSpanHandler,
                    public CefLoadHandler,
                    public CefRequestHandler {
 public:
  StukHandler();
  ~StukHandler() override;

  static StukHandler* GetInstance();

  CefRefPtr<CefDragHandler> GetDragHandler() override { return this; }
  CefRefPtr<CefDisplayHandler> GetDisplayHandler() override { return this; }
  CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override { return this; }
  CefRefPtr<CefLoadHandler> GetLoadHandler() override { return this; }
  CefRefPtr<CefRequestHandler> GetRequestHandler() override { return this; }

  void OnDraggableRegionsChanged(
      CefRefPtr<CefBrowser> browser,
      CefRefPtr<CefFrame> frame,
      const std::vector<CefDraggableRegion>& regions) override;
  void OnTitleChange(CefRefPtr<CefBrowser> browser,
                     const CefString& title) override;
  void OnAfterCreated(CefRefPtr<CefBrowser> browser) override;
  bool DoClose(CefRefPtr<CefBrowser> browser) override;
  void OnBeforeClose(CefRefPtr<CefBrowser> browser) override;
  void OnLoadError(CefRefPtr<CefBrowser> browser,
                   CefRefPtr<CefFrame> frame,
                   ErrorCode errorCode,
                   const CefString& errorText,
                   const CefString& failedUrl) override;
  bool OnBeforeBrowse(CefRefPtr<CefBrowser> browser,
                      CefRefPtr<CefFrame> frame,
                      CefRefPtr<CefRequest> request,
                      bool user_gesture,
                      bool is_redirect) override;

 private:
  using BrowserList = std::list<CefRefPtr<CefBrowser>>;
  BrowserList browsers_;
  bool closing_ = false;

  IMPLEMENT_REFCOUNTING(StukHandler);
};

#endif
