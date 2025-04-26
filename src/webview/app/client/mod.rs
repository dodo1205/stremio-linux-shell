mod display_handler;
mod lifespan_handler;
mod load_handler;
mod render_handler;

use display_handler::WebViewDisplayHandler;
use lifespan_handler::WebViewLifeSpanHandler;
use load_handler::WebViewLoadHandler;
use render_handler::WebViewRenderHandler;

use crate::{
    WebViewEvent, cef_impl,
    webview::{
        SENDER,
        constants::{IPC_MESSAGE, READY_MESSAGE},
    },
};

cef_impl!(
    prefix = "WebView",
    name = Client,
    sys_type = cef_dll_sys::cef_client_t,
    {
        fn get_display_handler(&self) -> Option<DisplayHandler> {
            Some(WebViewDisplayHandler::new())
        }

        fn get_render_handler(&self) -> Option<RenderHandler> {
            Some(WebViewRenderHandler::new())
        }

        fn get_life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(WebViewLifeSpanHandler::new())
        }

        fn get_load_handler(&self) -> Option<LoadHandler> {
            Some(WebViewLoadHandler::new())
        }

        fn on_process_message_received(
            &self,
            _browser: Option<&mut impl ImplBrowser>,
            _frame: Option<&mut impl ImplFrame>,
            _source_process: ProcessId,
            message: Option<&mut impl ImplProcessMessage>,
        ) -> ::std::os::raw::c_int {
            if let Some(message) = message {
                let name = CefString::from(&message.get_name());

                let ready_message_name = CefString::from(READY_MESSAGE);
                if name.as_slice() == ready_message_name.as_slice() {
                    if let Some(sender) = SENDER.get() {
                        sender.send(WebViewEvent::Ready).ok();
                    }
                }

                let ipc_message_name = CefString::from(IPC_MESSAGE);
                if name.as_slice() == ipc_message_name.as_slice() {
                    let arguments = message.get_argument_list().unwrap();
                    let data = CefString::from(&arguments.get_string(0));

                    if let Some(sender) = SENDER.get() {
                        sender.send(WebViewEvent::Ipc(data.to_string())).ok();
                    }
                }
            }

            Default::default()
        }
    }
);
