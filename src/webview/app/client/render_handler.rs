use crate::{
    WebViewEvent, cef_impl,
    shared::{with_gl, with_renderer_read, with_renderer_write},
    webview::SENDER,
};

cef_impl!(
    prefix = "WebView",
    name = RenderHandler,
    sys_type = cef_dll_sys::cef_render_handler_t,
    {
        fn get_view_rect(&self, _browser: Option<&mut impl ImplBrowser>, rect: Option<&mut Rect>) {
            with_renderer_read(|renderer| {
                if let Some(rect) = rect {
                    *rect = Rect {
                        x: 0,
                        y: 0,
                        width: renderer.width,
                        height: renderer.height,
                    };
                }
            });
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
                with_renderer_write(|mut renderer| {
                    if let Some(dirty) = dirty_rects {
                        renderer.paint(dirty.x, dirty.y, dirty.width, dirty.height, buffer, width);
                    } else {
                        renderer.paint(0, 0, width, height, buffer, width);
                    }
                });
            });

            if let Some(sender) = SENDER.get() {
                sender.send(WebViewEvent::Paint).ok();
            }
        }
    }
);
