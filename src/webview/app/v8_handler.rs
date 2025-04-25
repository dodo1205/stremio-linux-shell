use cef::{rc::*, *};

use crate::webview::constants::{IPC_MESSAGE, IPC_RECEIVER};

use super::utils;

pub struct WebViewV8Handler {
    object: *mut RcImpl<cef_dll_sys::_cef_v8_handler_t, Self>,
}

impl WebViewV8Handler {
    pub fn new() -> V8Handler {
        V8Handler::new(Self {
            object: std::ptr::null_mut(),
        })
    }

    fn is_handler(name: Option<&CefString>, value: &str) -> bool {
        name.is_some_and(|name| {
            let handler_name = CefString::from(value);
            name.as_slice() == handler_name.as_slice()
        })
    }

    fn handler_data(arguments: Option<&[Option<impl ImplV8Value>]>) -> Option<CefStringUtf16> {
        arguments.and_then(|arguments| {
            arguments.first().and_then(|value| {
                value
                    .as_ref()
                    .map(|value| value.get_string_value())
                    .map(|value| CefString::from(&value))
            })
        })
    }

    fn send_ipc_message(data: CefStringUtf16) {
        if let Some(context) = v8_context_get_current_context() {
            if let Some(mut browser) = context.get_browser() {
                utils::send_process_message(Some(&mut browser), IPC_MESSAGE, Some(&data));
            }
        }
    }
}

impl Rc for WebViewV8Handler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapV8Handler for WebViewV8Handler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_v8_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewV8Handler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self { object }
    }
}

impl ImplV8Handler for WebViewV8Handler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_v8_handler_t {
        self.object.cast()
    }

    fn execute(
        &self,
        name: Option<&CefString>,
        _object: Option<&mut impl ImplV8Value>,
        arguments: Option<&[Option<impl ImplV8Value>]>,
        _retval: Option<&mut Option<impl ImplV8Value>>,
        _exception: Option<&mut CefString>,
    ) -> ::std::os::raw::c_int {
        if Self::is_handler(name, IPC_RECEIVER) {
            if let Some(data) = Self::handler_data(arguments) {
                Self::send_ipc_message(data);

                return 1;
            }
        }

        0
    }
}
