#[cfg(target_os = "macos")]
mod apple_compat;
mod blade_atlas;
mod blade_context;
mod blade_renderer;

#[cfg(target_os = "macos")]
pub(crate) use apple_compat::*;
pub(crate) use blade_atlas::*;
pub use blade_context::*;
pub(crate) use blade_renderer::*;

/// Re-export blade_graphics for external renderers that need to create
/// textures on the shared GPU context.
pub use blade_graphics as gpu;
