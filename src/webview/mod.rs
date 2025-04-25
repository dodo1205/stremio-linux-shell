mod adapters;
mod app;
mod constants;

use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use adapters::{NativeKeyCode, WindowsKeyCode};
use app::WebViewApp;
use cef::{
    App, Browser, BrowserHost, BrowserSettings, CefString, Client, Frame, ImplBrowser,
    ImplBrowserHost, ImplCommandLine, ImplFrame, LogSeverity, Settings, api_hash, args::Args,
    execute_process, initialize,
};
use cef_dll_sys::{cef_key_event_type_t, cef_log_severity_t, cef_mouse_button_type_t};
use constants::IPC_SENDER;
use once_cell::sync::OnceCell;
use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::{Key, PhysicalKey},
};

use crate::shared::types::{Cursor, MouseDelta, MousePosition};

static SENDER: OnceCell<Sender<WebViewEvent>> = OnceCell::new();

type SharedBrowser = Arc<Mutex<Option<Browser>>>;

pub enum WebViewEvent {
    Ready,
    Loaded,
    Paint,
    Cursor(Cursor),
    Ipc(String),
}

pub struct WebView {
    args: Args,
    settings: Settings,
    app: App,
    browser: SharedBrowser,
    mouse_position: MousePosition,
    receiver: Receiver<WebViewEvent>,
}

impl WebView {
    pub fn new(data_path: &Path) -> Self {
        let _ = api_hash(cef_dll_sys::CEF_API_VERSION_LAST, 0);

        let args = Args::new();

        let (sender, receiver) = channel::<WebViewEvent>();
        SENDER.get_or_init(|| sender);

        let browser = Arc::new(Mutex::new(None));
        let app = WebViewApp::new(browser.clone());

        let cache_path = data_path.join("cef").join("cache");
        let log_path = data_path.join("cef").join("log");

        let settings = Settings {
            no_sandbox: 1,
            windowless_rendering_enabled: 1,
            multi_threaded_message_loop: 1,
            cache_path: cache_path.to_str().unwrap().into(),
            log_file: log_path.to_str().unwrap().into(),
            log_severity: LogSeverity::from(cef_log_severity_t::LOGSEVERITY_VERBOSE),
            ..Default::default()
        };

        Self {
            args,
            settings,
            app,
            browser,
            mouse_position: Default::default(),
            receiver,
        }
    }

    fn browser_host(&self) -> Option<BrowserHost> {
        let lock = self.browser.try_lock();
        if let Ok(guard) = lock {
            if let Some(browser) = guard.as_ref() {
                return browser.get_host();
            }
        }

        None
    }

    fn main_frame(&self) -> Option<Frame> {
        let lock = self.browser.try_lock();
        if let Ok(guard) = lock {
            if let Some(browser) = guard.as_ref() {
                return browser.get_main_frame();
            }
        }

        None
    }

    pub fn should_exit(&mut self) -> bool {
        let ret = execute_process(
            Some(self.args.as_main_args()),
            Some(&mut self.app),
            std::ptr::null_mut(),
        );

        let cmd = self.args.as_cmd_line().unwrap();

        let switch = CefString::from("type");
        let is_browser_process = cmd.has_switch(Some(&switch)) != 1;

        if is_browser_process {
            // println!("launch browser process");
            assert!(ret == -1, "cannot execute browser process");
            false
        } else {
            // let process_type = CefString::from(&cmd.get_switch_value(Some(&switch)));
            // println!("launch process {process_type}");
            assert!(ret >= 0, "cannot execute non-browser process");
            true
        }
    }

    pub fn start(&mut self) {
        assert_eq!(
            initialize(
                Some(self.args.as_main_args()),
                Some(&self.settings),
                Some(&mut self.app),
                std::ptr::null_mut(),
            ),
            1
        );
    }

    pub fn stop(&self) {
        if let Some(host) = self.browser_host() {
            host.close_browser(0);
        }
    }

    pub fn events<T: Fn(WebViewEvent)>(&self, handler: T) {
        self.receiver.try_iter().for_each(handler);
    }

    pub fn navigate(&self, url: &str) {
        if let Some(main_frame) = self.main_frame() {
            let url = CefString::from(url);
            main_frame.load_url(Some(&url));
        }
    }

    pub fn dev_tools(&self, state: bool) {
        if let Some(host) = self.browser_host() {
            if state {
                host.show_dev_tools(
                    None,
                    Option::<&mut Client>::None,
                    Option::<&BrowserSettings>::None,
                    None,
                );
            } else {
                host.close_dev_tools();
            }
        }
    }

    pub fn post_message(&self, message: String) {
        if let Some(main_frame) = self.main_frame() {
            let script = format!("{}('{}')", IPC_SENDER, message);
            let code = CefString::from(script.as_str());
            main_frame.execute_java_script(Some(&code), None, 0);
        }
    }

    pub fn resized(&mut self) {
        if let Some(host) = self.browser_host() {
            host.was_hidden(0);
            host.was_resized();
        }
    }

    pub fn mouse_moved_event(&mut self, position: MousePosition) {
        self.mouse_position = position;

        if let Some(host) = self.browser_host() {
            let event = self.mouse_position.into();
            host.send_mouse_move_event(Some(&event), 0);
        }
    }

    pub fn mouse_wheel_event(&self, MouseDelta(delta_x, delta_y): MouseDelta) {
        if let Some(host) = self.browser_host() {
            let event = self.mouse_position.into();
            host.send_mouse_wheel_event(Some(&event), delta_x, delta_y);
        }
    }

    pub fn mouse_input_event(&self, state: ElementState, button: MouseButton) {
        let lock = self.browser.try_lock();
        if let Ok(guard) = lock {
            if let Some(browser) = guard.as_ref() {
                let mouse_up = match state {
                    ElementState::Pressed => false,
                    ElementState::Released => true,
                };

                let button_type = match button {
                    MouseButton::Back if mouse_up => {
                        browser.go_back();
                        None
                    }
                    MouseButton::Forward if mouse_up => {
                        browser.go_forward();
                        None
                    }
                    MouseButton::Left => Some(cef_mouse_button_type_t::MBT_LEFT),
                    MouseButton::Right => Some(cef_mouse_button_type_t::MBT_RIGHT),
                    MouseButton::Middle => Some(cef_mouse_button_type_t::MBT_MIDDLE),
                    _ => None,
                };

                if let Some(button_type) = button_type {
                    if let Some(host) = browser.get_host() {
                        let event = self.mouse_position.into();

                        host.send_mouse_click_event(
                            Some(&event),
                            button_type.into(),
                            mouse_up.into(),
                            1,
                        );
                    }
                }
            }
        }
    }

    pub fn keyboard_input_event(&self, key_event: KeyEvent) {
        if let Some(host) = self.browser_host() {
            if let PhysicalKey::Code(code) = key_event.physical_key {
                if let (Ok(WindowsKeyCode(windows_key_code)), Ok(NativeKeyCode(native_key_code))) =
                    (code.try_into(), code.try_into())
                {
                    let event_type = match key_event.state.is_pressed() {
                        true => cef_key_event_type_t::KEYEVENT_KEYDOWN.into(),
                        false => cef_key_event_type_t::KEYEVENT_KEYUP.into(),
                    };

                    let event = cef::KeyEvent {
                        type_: event_type,
                        windows_key_code,
                        native_key_code,
                        ..Default::default()
                    };

                    host.send_key_event(Some(&event));
                }
            }

            if key_event.state.is_pressed() {
                if let Key::Character(character) = key_event.logical_key {
                    let event = cef::KeyEvent {
                        type_: cef_key_event_type_t::KEYEVENT_CHAR.into(),
                        character: character.as_str().chars().next().map(|c| c as u16).unwrap(),
                        ..Default::default()
                    };

                    host.send_key_event(Some(&event));
                }
            }
        }
    }
}
