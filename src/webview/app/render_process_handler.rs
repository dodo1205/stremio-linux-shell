use cef::{rc::*, *};

use crate::webview::{
    app::v8_handler::WebViewV8Handler,
    constants::{IPC_RECEIVER, READY_MESSAGE},
};

use super::utils;

pub struct WebViewRenderProcessHandler {
    object: *mut RcImpl<cef_dll_sys::cef_render_process_handler_t, Self>,
}

impl WebViewRenderProcessHandler {
    pub fn new() -> RenderProcessHandler {
        RenderProcessHandler::new(Self {
            object: std::ptr::null_mut(),
        })
    }
}

impl Rc for WebViewRenderProcessHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapRenderProcessHandler for WebViewRenderProcessHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_render_process_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewRenderProcessHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self { object }
    }
}

impl ImplRenderProcessHandler for WebViewRenderProcessHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::cef_render_process_handler_t {
        self.object.cast()
    }

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
