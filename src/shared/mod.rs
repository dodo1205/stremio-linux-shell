mod renderer;
pub mod types;

use std::sync::{Mutex, MutexGuard, RwLock};

use glutin::{
    context::{NotCurrentContext, PossiblyCurrentContext},
    prelude::{NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{Surface, WindowSurface},
};
use once_cell::sync::OnceCell;
use renderer::Renderer;

pub static RENDERER: OnceCell<RwLock<Option<Renderer>>> = OnceCell::new();

pub fn create_renderer(default_size: (i32, i32), refresh_rate: u32) {
    RENDERER.get_or_init(|| RwLock::new(Some(Renderer::new(default_size, refresh_rate))));
}

pub static GL_SURFACE: OnceCell<Mutex<Surface<WindowSurface>>> = OnceCell::new();
pub static GL_CONTEXT: OnceCell<Mutex<Option<NotCurrentContext>>> = OnceCell::new();

pub fn with_gl<T: FnMut(MutexGuard<Surface<WindowSurface>>, &PossiblyCurrentContext)>(
    mut handler: T,
) {
    if let Some(surface) = GL_SURFACE.get() {
        if let Ok(surface) = surface.lock() {
            if let Some(context) = GL_CONTEXT.get() {
                if let Ok(mut guard) = context.lock() {
                    if let Some(context) = guard.take() {
                        let current_context = context
                            .make_current(&surface)
                            .expect("Failed to make context current");

                        handler(surface, &current_context);

                        let not_current_context = current_context
                            .make_not_current()
                            .expect("Failed to make context not current");

                        *guard = Some(not_current_context);
                    }
                }
            }
        };
    }
}

pub fn with_renderer_read<T: FnOnce(&Renderer)>(handler: T) {
    if let Some(lock) = RENDERER.get() {
        if let Ok(renderer) = lock.read() {
            if let Some(renderer) = renderer.as_ref() {
                handler(renderer)
            }
        }
    }
}

pub fn with_renderer_write<T: FnOnce(&mut Renderer)>(handler: T) {
    if let Some(lock) = RENDERER.get() {
        if let Ok(mut renderer) = lock.write() {
            if let Some(renderer) = renderer.as_mut() {
                handler(renderer)
            }
        }
    }
}

pub fn drop_renderer() {
    if let Some(lock) = RENDERER.get() {
        if let Ok(mut renderer) = lock.write() {
            renderer.take();
        }
    }
}
