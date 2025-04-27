mod app;
mod constants;
mod ipc;
mod player;
mod server;
mod shared;
mod webview;

use app::{App, AppEvent};
use clap::Parser;
use constants::{DATA_PATH, STARTUP_URL, URI_SCHEME, WINDOW_SIZE};
use glutin::{display::GetGlDisplay, surface::GlSurface};
use ipc::{IpcEvent, IpcEventMpv};
use player::{Player, PlayerEvent};
use server::Server;
use shared::{drop_gl, drop_renderer, with_gl, with_renderer_read, with_renderer_write};
use std::{
    fs, num::NonZeroU32, path::Path, process::ExitCode, rc::Rc, sync::mpsc::channel, thread,
    time::Duration,
};
use webview::{WebView, WebViewEvent};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
};

enum BridgeEvent {
    Draw,
}

#[derive(Parser, Debug)]
#[command(version, ignore_errors(true))]
struct Args {
    /// Open dev tools
    #[arg(short, long)]
    dev: bool,
    /// Startup url
    #[arg(short, long, default_value = STARTUP_URL)]
    url: String,
    /// Open a deeplink
    #[arg(short, long)]
    open: Option<String>,
    /// Disable server
    #[arg(short, long)]
    no_server: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let expanded_data_path = shellexpand::tilde(&DATA_PATH).to_string();
    let data_path = Path::new(&expanded_data_path);
    fs::create_dir_all(data_path).expect("Failed to create data directory");

    let mut webview = WebView::new(data_path);
    if webview.should_exit() {
        return ExitCode::SUCCESS;
    }

    let mut server = Server::new(data_path);
    if !args.no_server {
        server.setup().expect("Failed to setup server");
        server.start().expect("Failed to start server");
    }

    let mut app = App::new();
    let mut player = Player::new();

    let (bridge_tx, bridge_rx) = channel::<BridgeEvent>();

    let mut event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    loop {
        let timeout = Some(Duration::ZERO);
        let status = event_loop.pump_app_events(timeout, &mut app);

        if let PumpStatus::Exit(exit_code) = status {
            server.stop().expect("Failed to stop server");
            webview.stop();
            drop_renderer();
            drop_gl();

            break ExitCode::from(exit_code as u8);
        }

        app.events(|event| match event {
            AppEvent::Ready => {
                let refresh_rate = app.get_refresh_rate();

                with_gl(|surface, _| {
                    shared::create_renderer(WINDOW_SIZE, refresh_rate);
                    player.setup(Rc::new(surface.display()));
                });

                webview.start();
            }
            AppEvent::Resized(size) => {
                with_gl(|surface, context| {
                    surface.resize(
                        context,
                        NonZeroU32::new(size.0 as u32).unwrap(),
                        NonZeroU32::new(size.1 as u32).unwrap(),
                    );

                    with_renderer_write(|renderer| {
                        renderer.resize(size.0, size.1);
                    });

                    webview.resized();
                    webview.repaint();
                });
            }
            AppEvent::MouseMoved(position) => {
                webview.mouse_moved_event(position);
            }
            AppEvent::MouseWheel(delta) => {
                webview.mouse_wheel_event(delta);
            }
            AppEvent::MouseInput((state, button)) => {
                webview.mouse_input_event(state, button);
            }
            AppEvent::TouchInput(touch) => {
                webview.touch_event(touch);
            }
            AppEvent::KeyboardInput(key_event) => {
                webview.keyboard_input_event(key_event);
            }
        });

        webview.events(|event| match event {
            WebViewEvent::Ready => {
                webview.navigate(&args.url);
                webview.dev_tools(args.dev);
            }
            WebViewEvent::Loaded => {
                if let Some(deeplink) = &args.open {
                    if deeplink.starts_with(URI_SCHEME) {
                        let message =
                            ipc::create_response(IpcEvent::OpenMedia(deeplink.to_string()));
                        webview.post_message(message);
                    }
                }
            }
            WebViewEvent::Paint => {
                bridge_tx.send(BridgeEvent::Draw).ok();
            }
            WebViewEvent::Cursor(cursor) => {
                app.set_cursor(cursor);
            }
            WebViewEvent::Ipc(data) => ipc::parse_request(data, |event| match event {
                IpcEvent::Init(id) => {
                    let message = ipc::create_response(IpcEvent::Init(id));
                    webview.post_message(message);
                }
                IpcEvent::Fullscreen(state) => {
                    app.set_fullscreen(state);

                    let message = ipc::create_response(IpcEvent::Fullscreen(state));
                    webview.post_message(message);
                }
                IpcEvent::Mpv(event) => match event {
                    IpcEventMpv::Observe(name) => {
                        player.observe_property(name);
                    }
                    IpcEventMpv::Command((name, args)) => {
                        player.command(name, args);
                    }
                    IpcEventMpv::Set(property) => {
                        player.set_property(property);
                    }
                    _ => {}
                },
                _ => {}
            }),
        });

        player.events(|event| match event {
            PlayerEvent::Update => {
                bridge_tx.send(BridgeEvent::Draw).ok();
            }
            PlayerEvent::PropertyChange(property) => {
                let message = ipc::create_response(IpcEvent::Mpv(IpcEventMpv::Change(property)));
                webview.post_message(message);
            }
        });

        bridge_rx.try_iter().for_each(|event| match event {
            BridgeEvent::Draw => {
                with_gl(|surface, context| {
                    let should_render = player.should_render();

                    with_renderer_read(|renderer| {
                        if should_render {
                            player.render(renderer.fbo, renderer.width, renderer.height);
                        }

                        renderer.draw();
                    });

                    surface
                        .swap_buffers(context)
                        .expect("Failed to swap buffers");

                    if should_render {
                        player.report_swap();
                    }
                });
            }
        });

        thread::sleep(Duration::from_millis(1));
    }
}
