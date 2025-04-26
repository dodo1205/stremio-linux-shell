use crate::{
    cef_impl,
    webview::{SENDER, WebViewEvent},
};

cef_impl!(
    prefix = "WebView",
    name = DisplayHandler,
    sys_type = cef_dll_sys::cef_display_handler_t,
    {
        fn on_cursor_change(
            &self,
            _browser: Option<&mut impl ImplBrowser>,
            _cursor: ::std::os::raw::c_ulong,
            type_: CursorType,
            _custom_cursor_info: Option<&CursorInfo>,
        ) -> std::os::raw::c_int {
            if let Some(sender) = SENDER.get() {
                sender.send(WebViewEvent::Cursor(type_.into())).ok();
                return 1;
            }

            0
        }
    }
);
