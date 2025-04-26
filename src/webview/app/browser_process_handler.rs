use crate::{
    cef_impl,
    shared::RENDERER,
    webview::{BROWSER, app::client::WebViewClient},
};

cef_impl!(
    prefix = "WebView",
    name = BrowserProcessHandler,
    sys_type = cef_dll_sys::cef_browser_process_handler_t,
    {
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

                    BROWSER.get_or_init(|| {
                        browser_host_create_browser_sync(
                            Some(&window_info),
                            Some(&mut client),
                            Some(&url),
                            Some(&settings),
                            Option::<&mut DictionaryValue>::None,
                            Option::<&mut RequestContext>::None,
                        )
                        .expect("Failed to create browser sync")
                    });
                }
            }
        }
    }
);
