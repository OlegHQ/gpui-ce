//! Blade Texture Test Example
//!
//! This example demonstrates using `paint_blade_texture()` to display
//! an external Blade GPU texture in GPUI. This API enables zero-copy
//! rendering from external renderers (like isos-render) directly into GPUI.
//!
//! Run with: `cargo run --example blade_texture_test --features macos-blade`

#[path = "prelude.rs"]
mod example_prelude;

use gpui::{
    App, Application, Bounds, Colors, Context, Corners, Render, Window, WindowBounds,
    WindowOptions, canvas, div, point, prelude::*, px, size,
};

#[cfg(all(target_os = "macos", feature = "macos-blade"))]
use gpui::{BladeContext, gpu};

/// Stores our test texture state
#[cfg(all(target_os = "macos", feature = "macos-blade"))]
struct BladeTextureDemo {
    /// The Blade GPU context (shared with GPUI)
    blade_ctx: BladeContext,
    /// The test texture we create
    texture: gpu::Texture,
    /// View into the texture for rendering
    texture_view: gpu::TextureView,
}

#[cfg(all(target_os = "macos", feature = "macos-blade"))]
impl BladeTextureDemo {
    fn new(_cx: &mut Context<Self>) -> Self {
        // Create the Blade GPU context
        let blade_ctx = BladeContext::new().expect("Failed to create BladeContext");
        let gpu = &blade_ctx.gpu;

        // Create a test texture with a color gradient
        let width = 256u32;
        let height = 256u32;

        // Create the texture
        let texture = gpu.create_texture(gpu::TextureDesc {
            name: "test-texture",
            format: gpu::TextureFormat::Rgba8Unorm,
            size: gpu::Extent {
                width,
                height,
                depth: 1,
            },
            dimension: gpu::TextureDimension::D2,
            array_layer_count: 1,
            mip_level_count: 1,
            usage: gpu::TextureUsage::COPY | gpu::TextureUsage::RESOURCE,
        });

        // Create view for the texture
        let texture_view = gpu.create_texture_view(
            texture,
            gpu::TextureViewDesc {
                name: "test-texture-view",
                format: gpu::TextureFormat::Rgba8Unorm,
                dimension: gpu::ViewDimension::D2,
                subresources: &gpu::TextureSubresources::default(),
            },
        );

        // Generate gradient pixel data (red-green gradient)
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = (x as f32 / width as f32 * 255.0) as u8; // R
                pixels[idx + 1] = (y as f32 / height as f32 * 255.0) as u8; // G
                pixels[idx + 2] = 128; // B
                pixels[idx + 3] = 255; // A
            }
        }

        // Upload pixel data to texture
        // Create a staging buffer for the upload
        let buffer = gpu.create_buffer(gpu::BufferDesc {
            name: "staging",
            size: pixels.len() as u64,
            memory: gpu::Memory::Upload,
        });

        // Copy data to the buffer
        unsafe {
            let ptr = gpu.map_buffer(buffer);
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), ptr, pixels.len());
            gpu.unmap_buffer(buffer);
        }

        // Create command encoder and copy buffer to texture
        let mut encoder = gpu.create_command_encoder(gpu::CommandEncoderDesc {
            name: "texture-upload",
            buffer_count: 1,
        });
        encoder.start();
        encoder.init_texture(texture);

        if let mut pass = encoder.transfer("upload") {
            pass.copy_buffer_to_texture(
                gpu::BufferPiece { buffer, offset: 0 },
                gpu::TexturePiece {
                    texture,
                    mip_level: 0,
                    array_layer: 0,
                    origin: [0, 0, 0],
                },
                gpu::Extent {
                    width,
                    height,
                    depth: 1,
                },
            );
        }

        let sync = gpu.submit(&mut encoder);
        gpu.wait_for(&sync, !0);

        // Clean up staging buffer
        gpu.destroy_buffer(buffer);

        Self {
            blade_ctx,
            texture,
            texture_view,
        }
    }
}

#[cfg(all(target_os = "macos", feature = "macos-blade"))]
impl Drop for BladeTextureDemo {
    fn drop(&mut self) {
        self.blade_ctx.gpu.destroy_texture_view(self.texture_view);
        self.blade_ctx.gpu.destroy_texture(self.texture);
    }
}

#[cfg(all(target_os = "macos", feature = "macos-blade"))]
impl Render for BladeTextureDemo {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::for_appearance(window);
        let texture_view = self.texture_view;

        div()
            .id("main")
            .size_full()
            .p_6()
            .bg(colors.background)
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(colors.text)
                    .child("Blade Texture Test"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_muted)
                    .child("Displaying an external Blade GPU texture via paint_blade_texture()"),
            )
            .child(
                div()
                    .w(px(280.))
                    .h(px(280.))
                    .rounded_lg()
                    .bg(colors.surface)
                    .border_1()
                    .border_color(colors.border)
                    .p_3()
                    .child(
                        canvas(
                            move |_, _, _| {},
                            move |bounds, _, window, _cx| {
                                // Calculate the texture bounds (256x256 centered in the canvas)
                                let texture_bounds = Bounds {
                                    origin: bounds.origin,
                                    size: size(px(256.), px(256.)),
                                };

                                // Paint the blade texture with rounded corners
                                window.paint_blade_texture(
                                    texture_bounds,
                                    Corners::all(px(8.)),
                                    texture_view,
                                    1.0,
                                );
                            },
                        )
                        .size_full(),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child("The gradient texture is created on the GPU and rendered directly without CPU copies."),
            )
    }
}

// Fallback for non-Blade platforms
#[cfg(not(all(target_os = "macos", feature = "macos-blade")))]
struct BladeTextureDemo;

#[cfg(not(all(target_os = "macos", feature = "macos-blade")))]
impl BladeTextureDemo {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

#[cfg(not(all(target_os = "macos", feature = "macos-blade")))]
impl Render for BladeTextureDemo {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::for_appearance(window);

        div()
            .id("main")
            .size_full()
            .p_6()
            .bg(colors.background)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_xl()
                    .text_color(colors.error)
                    .child("Blade texture rendering requires macos-blade feature"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(colors.text_muted)
                    .mt_2()
                    .child("Run with: cargo run --example blade_texture_test --features macos-blade"),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(350.), px(450.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| BladeTextureDemo::new(cx)),
        )
        .expect("Failed to open window");

        example_prelude::init_example(cx, "Blade Texture Test");
    });
}
