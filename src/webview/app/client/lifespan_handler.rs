use std::os;

use url::Url;

use crate::{
    cef_impl,
    webview::{SENDER, WebViewEvent},
};

cef_impl!(
    prefix = "WebView",
    name = LifeSpanHandler,
    sys_type = cef_dll_sys::cef_life_span_handler_t,
    {
        fn on_before_close(&self, _browser: Option<&mut impl ImplBrowser>) {
            shutdown();
        }

        fn on_before_popup(
            &self,
            _browser: Option<&mut impl ImplBrowser>,
            _frame: Option<&mut impl ImplFrame>,
            _popup_id: ::std::os::raw::c_int,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: ::std::os::raw::c_int,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<impl ImplClient>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<impl ImplDictionaryValue>>,
            _no_javascript_access: Option<&mut ::std::os::raw::c_int>,
        ) -> os::raw::c_int {
            if let Some(target_url) = target_url {
                let target_url = target_url.to_string();

                if let Ok(url) = Url::parse(&target_url) {
                    if let Some(sender) = SENDER.get() {
                        sender.send(WebViewEvent::Open(url)).ok();
                    }
                }
            }

            true.into()
        }
    }
);
