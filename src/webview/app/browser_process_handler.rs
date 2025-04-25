use cef::{rc::*, *};

use crate::{
    shared::RENDERER,
    webview::{SharedBrowser, app::client::WebViewClient},
};

pub struct WebViewBrowserProcessHandler {
    object: *mut RcImpl<cef_dll_sys::_cef_browser_process_handler_t, Self>,
    browser: SharedBrowser,
}

impl WebViewBrowserProcessHandler {
    pub fn new(browser: SharedBrowser) -> BrowserProcessHandler {
        BrowserProcessHandler::new(Self {
            object: std::ptr::null_mut(),
            browser,
        })
    }
}

impl Rc for WebViewBrowserProcessHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl WrapBrowserProcessHandler for WebViewBrowserProcessHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_browser_process_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewBrowserProcessHandler {
    fn clone(&self) -> Self {
        let object = unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
            rc_impl
        };

        let browser = self.browser.clone();

        Self { object, browser }
    }
}

impl ImplBrowserProcessHandler for WebViewBrowserProcessHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_browser_process_handler_t {
        self.object.cast()
    }

    fn on_context_initialized(&self) {
        if let Some(lock) = RENDERER.get() {
            if let Ok(renderer) = lock.read() {
                let mut client = WebViewClient::new();

                let url = CefString::from("about:blank");

                let window_info = WindowInfo {
                    windowless_rendering_enabled: 1,
                    // external_begin_frame_enabled: 1,
                    // shared_texture_enabled: 1,
                    ..Default::default()
                };

                let settings = BrowserSettings {
                    windowless_frame_rate: renderer.refresh_rate as i32,
                    ..Default::default()
                };

                // assert_eq!(
                //     browser_host_create_browser(
                //         Some(&window_info),
                //         Some(&mut client),
                //         Some(&url),
                //         Some(&settings),
                //         Option::<&mut DictionaryValue>::None,
                //         Option::<&mut RequestContext>::None,
                //     ),
                //     1
                // )

                let browser = browser_host_create_browser_sync(
                    Some(&window_info),
                    Some(&mut client),
                    Some(&url),
                    Some(&settings),
                    Option::<&mut DictionaryValue>::None,
                    Option::<&mut RequestContext>::None,
                )
                .expect("Failed to create browser sync");

                let mut guard = self.browser.lock().unwrap();
                *guard = Some(browser);
            }
        }
    }
}
