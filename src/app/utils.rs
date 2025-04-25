use glutin::{
    config::{Config, GlConfig},
    context::{ContextAttributes, NotCurrentContext},
    display::GetGlDisplay,
    prelude::GlDisplay,
    surface::{Surface, WindowSurface},
};
use glutin_winit::GlWindow;
use winit::window::Window;

pub fn create_window_surface(config: &Config, window: &Option<Window>) -> Surface<WindowSurface> {
    let surface_attributes = window
        .as_ref()
        .expect("Failed to get window")
        .build_surface_attributes(Default::default())
        .expect("Failed to build surface attributes");

    unsafe {
        config
            .display()
            .create_window_surface(config, &surface_attributes)
            .expect("Failed to create surface")
    }
}

pub fn create_context(
    config: &Config,
    context_attributes: &ContextAttributes,
) -> NotCurrentContext {
    unsafe {
        config
            .display()
            .create_context(config, context_attributes)
            .expect("Failed to create shared context")
    }
}

pub fn config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            if config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .expect("Failed to pick a config")
}
