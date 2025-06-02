use egui::{ColorImage, IconData, TextureHandle, TextureOptions};
use image::RgbaImage;
use std::sync::OnceLock;

/// The icon bytes embedded directly in the binary
pub static ICON_BYTES: &[u8] = include_bytes!("../../assets/icon.png");

/// Lazily loaded app icon image
static APP_ICON_IMAGE: OnceLock<RgbaImage> = OnceLock::new();

/// Get the app icon image, loading it if necessary
fn get_app_icon_image() -> &'static RgbaImage {
    APP_ICON_IMAGE.get_or_init(|| {
        // Load the image from the embedded bytes only once
        image::load_from_memory(ICON_BYTES)
            .expect("Failed to load icon from embedded data")
            .into_rgba8()
    })
}

/// Load the embedded icon data into an egui icon
#[must_use]
pub fn load_app_icon() -> IconData {
    let image = get_app_icon_image();
    let width = image.width();
    let height = image.height();
    let rgba = image.clone().into_raw();

    IconData {
        rgba,
        width: width as _,
        height: height as _,
    }
}

/// Load the app icon as a texture for display in UI
pub fn load_app_icon_texture(ctx: &egui::Context) -> TextureHandle {
    let image = get_app_icon_image();
    let width = image.width();
    let height = image.height();
    let size = [width as _, height as _];
    let pixels = image.as_flat_samples();

    // Create a texture from the image data
    ctx.load_texture(
        "app_icon",
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
        TextureOptions::default(),
    )
}
