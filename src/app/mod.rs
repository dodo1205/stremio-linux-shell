mod adapters;
mod utils;

use std::{
    ffi::CString,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use ashpd::{WindowIdentifier, desktop::open_uri};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, Version},
    display::GetGlDisplay,
    prelude::GlDisplay,
};
use glutin_winit::DisplayBuilder;
use tracing::error;
use url::Url;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, MouseButton, Touch, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::ModifiersState,
    platform::wayland::WindowAttributesExtWayland,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{CursorIcon, Fullscreen, Window, WindowAttributes},
};

use crate::{
    constants::{APP_ID, WINDOW_SIZE},
    shared::{
        GL_CONTEXT, GL_SURFACE,
        types::{Cursor, MouseDelta, MousePosition, WindowSize},
    },
};

const CONTEXT_API: ContextApi = ContextApi::OpenGl(Some(Version::new(3, 3)));

pub enum AppEvent {
    Ready,
    Resized(WindowSize),
    Focused(bool),
    Minimized(bool),
    MouseMoved((MousePosition, bool)),
    MouseWheel(MouseDelta),
    MouseInput((ElementState, MouseButton)),
    TouchInput(Touch),
    KeyboardInput((KeyEvent, ModifiersState)),
}

pub struct App {
    window: Option<Window>,
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
    modifiers_state: ModifiersState,
    hovered: bool,
}

impl App {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<AppEvent>();

        Self {
            window: None,
            sender,
            receiver,
            modifiers_state: ModifiersState::empty(),
            hovered: false,
        }
    }

    pub fn events<T: FnMut(AppEvent)>(&self, handler: T) {
        self.receiver.try_iter().for_each(handler);
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        if let Some(window) = self.window.as_ref() {
            if let Ok(icon) = TryInto::<CursorIcon>::try_into(cursor) {
                window.set_cursor(icon);
                window.set_cursor_visible(true);
            } else {
                window.set_cursor_visible(false);
            }
        }
    }

    pub fn set_fullscreen(&self, state: bool) {
        if let Some(window) = self.window.as_ref() {
            let fullscreen = match state {
                true => Some(Fullscreen::Borderless(None)),
                false => None,
            };

            window.set_fullscreen(fullscreen);
        }
    }

    pub fn get_refresh_rate(&self) -> u32 {
        if let Some(window) = self.window.as_ref() {
            for monitor in window.available_monitors() {
                if let Some(m_hz) = monitor.refresh_rate_millihertz() {
                    return m_hz / 1000;
                }
            }
        }

        30
    }

    pub async fn open_url(&self, url: Url) {
        if let Some(window) = self.window.as_ref() {
            if let (Ok(window), Ok(display)) = (window.window_handle(), window.display_handle()) {
                let window_handle = window.as_raw();
                let display_handle = display.as_raw();
                let window_identifier =
                    WindowIdentifier::from_raw_handle(&window_handle, Some(&display_handle)).await;

                let request = open_uri::OpenFileRequest::default().identifier(window_identifier);

                request
                    .send_uri(&url)
                    .await
                    .map_err(|e| error!("Failed to open uri: {e}"))
                    .ok();
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.window.take();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("Stremio")
            .with_name(APP_ID, APP_ID)
            .with_decorations(true)
            .with_resizable(true)
            .with_min_inner_size(PhysicalSize::new(900, 600))
            .with_inner_size(PhysicalSize::<u32>::from(WINDOW_SIZE));

        let template_builder = ConfigTemplateBuilder::new();

        let (window, config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(event_loop, template_builder, utils::config_picker)
            .expect("Failed to build display");

        let surface = utils::create_window_surface(&config, &window);

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(CONTEXT_API)
            .build(None);

        let context = utils::create_context(&config, &context_attributes);

        gl::load_with(|name| {
            let name = CString::new(name).unwrap();
            context.display().get_proc_address(&name) as *const _
        });

        GL_CONTEXT.get_or_init(|| Mutex::new(Some(context)));
        GL_SURFACE.get_or_init(|| Mutex::new(Some(surface)));

        self.window = window;

        self.sender.send(AppEvent::Ready).ok();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers_state = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.sender
                    .send(AppEvent::KeyboardInput((event, self.modifiers_state)))
                    .ok();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.sender
                    .send(AppEvent::MouseMoved((position.into(), self.hovered)))
                    .ok();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.sender.send(AppEvent::MouseWheel(delta.into())).ok();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.sender.send(AppEvent::MouseInput((state, button))).ok();
            }
            WindowEvent::CursorEntered { .. } => {
                self.hovered = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.hovered = false;
            }
            WindowEvent::Touch(touch) => {
                self.sender.send(AppEvent::TouchInput(touch)).ok();
            }
            WindowEvent::Resized(size) => {
                self.sender.send(AppEvent::Resized(size.into())).ok();
            }
            WindowEvent::Focused(state) => {
                self.sender.send(AppEvent::Focused(state)).ok();

                if let Some(window) = self.window.as_ref() {
                    let minimized = window.is_minimized().unwrap_or(false);
                    self.sender.send(AppEvent::Minimized(minimized)).ok();
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}
