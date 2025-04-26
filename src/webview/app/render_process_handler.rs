use crate::{
    cef_impl,
    webview::{
        app::v8_handler::WebViewV8Handler,
        constants::{IPC_RECEIVER, READY_MESSAGE},
    },
};

use super::utils;

cef_impl!(
    prefix = "WebView",
    name = RenderProcessHandler,
    sys_type = cef_dll_sys::cef_render_process_handler_t,
    {
        fn on_browser_created(
            &self,
            browser: Option<&mut impl ImplBrowser>,
            _extra_info: Option<&mut impl ImplDictionaryValue>,
        ) {
            utils::send_process_message(browser, READY_MESSAGE, None);
        }

        fn on_context_created(
            &self,
            _browser: Option<&mut impl ImplBrowser>,
            _frame: Option<&mut impl ImplFrame>,
            context: Option<&mut impl ImplV8Context>,
        ) {
            let name = CefString::from(IPC_RECEIVER);
            let mut handler = WebViewV8Handler::new();

            let mut value = v8_value_create_function(Some(&name), Some(&mut handler))
                .expect("Failed to create a value for function");

            if let Some(context) = context {
                if let Some(global) = context.get_global() {
                    global.set_value_bykey(
                        Some(&name),
                        Some(&mut value),
                        V8Propertyattribute::default(),
                    );
                }
            }
        }
    }
);
