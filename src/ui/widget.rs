use super::draw::{FontFamily, TextSpan};

/// Flat enum widget identity (DD-1).
/// Closed set — we know all widget types. No trait objects.
#[derive(Debug, Clone)]
pub enum Widget {
    /// Container with background, border, and optional inner shadow.
    Panel {
        bg_color: [f32; 4],     // sRGB RGBA
        border_color: [f32; 4], // sRGB RGBA
        border_width: f32,      // pixels
        shadow_width: f32,      // pixels
    },

    /// Single-line or multi-line text.
    Label {
        text: String,
        color: [f32; 4], // sRGB RGBA
        font_size: f32,  // pixels
        font_family: FontFamily,
    },

    /// Clickable element with text and background.
    Button {
        text: String,
        color: [f32; 4],        // text color sRGB RGBA
        bg_color: [f32; 4],     // background sRGB RGBA
        border_color: [f32; 4], // border sRGB RGBA
        font_size: f32,         // pixels
        font_family: FontFamily,
    },

    /// Mixed-style text block (DD-4, UI-R01).
    /// Each span carries its own color and font family.
    /// Font size is shared across all spans.
    RichText {
        spans: Vec<TextSpan>,
        font_size: f32, // pixels — shared across all spans
    },
}
