use cef::{rc::*, *};

use crate::{
    WebViewEvent,
    shared::{RENDERER, with_gl},
    webview::SENDER,
};

pub struct WebViewRenderHandler {
    object: *mut RcImpl<cef_dll_sys::_cef_render_handler_t, Self>,
}

impl WebViewRenderHandler {
    pub fn new() -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
        })
    }
}

impl Rc for WebViewRenderHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapRenderHandler for WebViewRenderHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_render_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewRenderHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        Self { object }
    }
}

impl ImplRenderHandler for WebViewRenderHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_render_handler_t {
        self.object.cast()
    }

    fn get_view_rect(&self, _browser: Option<&mut impl ImplBrowser>, rect: Option<&mut Rect>) {
        if let Some(lock) = RENDERER.get() {
            if let Ok(renderer) = lock.read() {
                if let Some(rect) = rect {
                    *rect = Rect {
                        x: 0,
                        y: 0,
                        width: renderer.width,
                        height: renderer.height,
                    };
                }
            }
        }
    }

    fn on_paint(
        &self,
        _browser: Option<&mut impl ImplBrowser>,
        _type_: PaintElementType,
        _dirty_rects_count: usize,
        dirty_rects: Option<&Rect>,
        buffer: *const u8,
        width: ::std::os::raw::c_int,
        height: ::std::os::raw::c_int,
    ) {
        with_gl(|_, _| {
            if let Some(lock) = RENDERER.get() {
                if let Ok(mut renderer) = lock.write() {
                    if let Some(dirty) = dirty_rects {
                        renderer.paint(dirty.x, dirty.y, dirty.width, dirty.height, buffer, width);
                    } else {
                        renderer.paint(0, 0, width, height, buffer, width);
                    }
                }
            }
        });

        if let Some(sender) = SENDER.get() {
            sender.send(WebViewEvent::Paint).ok();
        }
    }
}
