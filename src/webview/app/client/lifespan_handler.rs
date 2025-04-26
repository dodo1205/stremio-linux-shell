use crate::cef_impl;

cef_impl!(
    prefix = "WebView",
    name = LifeSpanHandler,
    sys_type = cef_dll_sys::cef_life_span_handler_t,
    {
        fn on_before_close(&self, _browser: Option<&mut impl ImplBrowser>) {
            shutdown();
        }
    }
);
