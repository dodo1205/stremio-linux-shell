use cef::{rc::*, *};

pub struct WebViewLifeSpanHandler {
    object: *mut RcImpl<cef_dll_sys::_cef_life_span_handler_t, Self>,
}

impl WebViewLifeSpanHandler {
    pub fn new() -> LifeSpanHandler {
        LifeSpanHandler::new(Self {
            object: std::ptr::null_mut(),
        })
    }
}

impl Rc for WebViewLifeSpanHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapLifeSpanHandler for WebViewLifeSpanHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_life_span_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewLifeSpanHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self { object }
    }
}

impl ImplLifeSpanHandler for WebViewLifeSpanHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_life_span_handler_t {
        self.object.cast()
    }

    fn on_before_close(&self, _browser: Option<&mut impl ImplBrowser>) {
        shutdown();
    }
}
