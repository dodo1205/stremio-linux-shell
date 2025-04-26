use crate::{
    WebViewEvent, cef_impl,
    shared::{RENDERER, with_gl},
    webview::SENDER,
};

cef_impl!(
    prefix = "WebView",
    name = RenderHandler,
    sys_type = cef_dll_sys::cef_render_handler_t,
    {
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
                            renderer.paint(
                                dirty.x,
                                dirty.y,
                                dirty.width,
                                dirty.height,
                                buffer,
                                width,
                            );
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
);
