use cef::{rc::*, *};

use crate::webview::{SENDER, WebViewEvent};

pub struct WebViewDisplayHandler {
    object: *mut RcImpl<cef_dll_sys::_cef_display_handler_t, Self>,
}

impl WebViewDisplayHandler {
    pub fn new() -> DisplayHandler {
        DisplayHandler::new(Self {
            object: std::ptr::null_mut(),
        })
    }
}

impl Rc for WebViewDisplayHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapDisplayHandler for WebViewDisplayHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_display_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewDisplayHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self { object }
    }
}

impl ImplDisplayHandler for WebViewDisplayHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_display_handler_t {
        self.object.cast()
    }

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
