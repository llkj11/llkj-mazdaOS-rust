//! Platform-neutral native UI with bundled, antialiased typography and read-only observations.
mod clock;
mod graphics;
mod preferences;
mod settings;
mod sound;
mod stock_assets;
mod typography;
mod ui;

pub use clock::{ClockPreferences, TimeFormat};
pub use graphics::{Color, Framebuffer, Rect, Renderer};
pub use preferences::{Preferences, Theme, Units};
pub use settings::{DisplayMode, DisplayPower, DisplayPreferences};
pub use sound::{SoundControl, SoundPreferences, SoundProfile};
pub use stock_assets::{Bitmap, StockAssets};
pub use typography::{fit_text, install_reference_font, text_width};
pub use ui::{MediaIntent, Screen, UiModel};
pub const DISPLAY_WIDTH: usize = 800;
pub const DISPLAY_HEIGHT: usize = 480;

fn measurement_label(value: f32, unit: &str, max_characters: usize) -> String {
    let unavailable = format!("-- {unit}");
    if !value.is_finite() {
        return unavailable;
    }
    let label = format!("{value:.0} {unit}");
    if label.chars().count() > max_characters {
        unavailable
    } else {
        label
    }
}
