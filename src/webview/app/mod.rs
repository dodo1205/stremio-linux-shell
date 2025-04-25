mod browser_process_handler;
mod client;
mod render_process_handler;
mod utils;
mod v8_handler;

use browser_process_handler::WebViewBrowserProcessHandler;
use cef::{rc::*, *};
use render_process_handler::WebViewRenderProcessHandler;

use crate::constants::CMD_SWITCHES;

use super::SharedBrowser;

pub struct WebViewApp {
    object: *mut RcImpl<cef_dll_sys::_cef_app_t, Self>,
    browser: SharedBrowser,
}

impl WebViewApp {
    pub fn new(browser: SharedBrowser) -> App {
        App::new(Self {
            object: std::ptr::null_mut(),
            browser,
        })
    }
}

impl WrapApp for WebViewApp {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_app_t, Self>) {
        self.object = object;
    }
}

impl Clone for WebViewApp {
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

impl Rc for WebViewApp {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplApp for WebViewApp {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_app_t {
        self.object.cast()
    }

    fn on_before_command_line_processing(
        &self,
        _process_type: Option<&CefString>,
        command_line: Option<&mut impl ImplCommandLine>,
    ) {
        if let Some(line) = command_line {
            CMD_SWITCHES.iter().for_each(|switch| {
                line.append_switch(Some(&CefString::from(switch.to_owned())));
            });
        }
    }

    fn get_browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(WebViewBrowserProcessHandler::new(self.browser.clone()))
    }

    fn get_render_process_handler(&self) -> Option<RenderProcessHandler> {
        Some(WebViewRenderProcessHandler::new())
    }
}
