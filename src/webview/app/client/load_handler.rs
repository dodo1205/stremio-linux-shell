use cef::{rc::*, *};

use crate::webview::{
    SENDER, WebViewEvent,
    constants::{IPC_RECEIVER, IPC_SENDER, PRELOAD_SCRIPT},
};

pub struct WebViewLoadHandler {
    object: *mut RcImpl<cef_dll_sys::_cef_load_handler_t, Self>,
}

impl WebViewLoadHandler {
    pub fn new() -> LoadHandler {
        LoadHandler::new(Self {
            object: std::ptr::null_mut(),
        })
    }
}

impl Rc for WebViewLoadHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapLoadHandler for WebViewLoadHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_load_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewLoadHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self { object }
    }
}

impl ImplLoadHandler for WebViewLoadHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_load_handler_t {
        self.object.cast()
    }

    fn on_load_start(
        &self,
        _browser: Option<&mut impl ImplBrowser>,
        frame: Option<&mut impl ImplFrame>,
        _transition_type: TransitionType,
    ) {
        if let Some(frame) = frame {
            if frame.is_main() == 1 {
                let script = PRELOAD_SCRIPT
                    .replace("IPC_SENDER", IPC_SENDER)
                    .replace("IPC_RECEIVER", IPC_RECEIVER);
                let code = CefString::from(script.as_str());
                frame.execute_java_script(Some(&code), None, 0);
            }
        }
    }

    fn on_load_end(
        &self,
        _browser: Option<&mut impl ImplBrowser>,
        frame: Option<&mut impl ImplFrame>,
        http_status_code: ::std::os::raw::c_int,
    ) {
        if let Some(frame) = frame {
            if frame.is_main() == 1 && http_status_code == 200 {
                if let Some(sender) = SENDER.get() {
                    sender.send(WebViewEvent::Loaded).ok();
                }
            }
        }
    }
}
