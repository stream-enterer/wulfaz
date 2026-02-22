use super::draw::{FontFamily, TextSpan};

/// Content description for a tooltip (UI-W04).
/// Stored on WidgetNode, built into widget subtrees on hover.
#[derive(Debug, Clone)]
pub enum TooltipContent {
    /// Simple text tooltip (rendered as body-font label).
    Text(String),
    /// Custom widget list with optional nested tooltips per child.
    Custom(Vec<(Widget, Option<TooltipContent>)>),
}

/// Cross-axis alignment for Row and Column containers (UI-100, UI-101).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum CrossAlign {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

/// Flat enum widget identity (DD-1).
/// Closed set — we know all widget types. No trait objects.
#[derive(Debug, Clone)]
pub enum Widget {
    /// Horizontal auto-layout container (UI-100).
    /// Children laid out left-to-right with `gap` spacing.
    /// Row itself emits no draw commands (transparent container).
    Row {
        gap: f32,          // pixels between children
        align: CrossAlign, // vertical alignment of children within row height
    },

    /// Vertical auto-layout container (UI-101).
    /// Children laid out top-to-bottom with `gap` spacing.
    /// Column itself emits no draw commands (transparent container).
    Column {
        gap: f32,          // pixels between children
        align: CrossAlign, // horizontal alignment of children within column width
    },

    /// Container with background, border, and optional inner shadow.
    Panel {
        bg_color: [f32; 4],     // sRGB RGBA
        border_color: [f32; 4], // sRGB RGBA
        border_width: f32,      // pixels
        shadow_width: f32,      // pixels
    },

    /// Single-line or multi-line text.
    /// When `wrap` is true and parent provides a width constraint,
    /// text breaks at word boundaries (UI-102).
    Label {
        text: String,
        color: [f32; 4], // sRGB RGBA
        font_size: f32,  // pixels
        font_family: FontFamily,
        wrap: bool, // false = single-line (default), true = word-wrap
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

    /// Scrollable vertical list with virtual scrolling (UI-W03).
    /// Children are laid out vertically, each at `item_height` pixels tall.
    /// Only children within the visible viewport are measured/drawn.
    ScrollList {
        bg_color: [f32; 4],        // background sRGB RGBA
        border_color: [f32; 4],    // border sRGB RGBA
        border_width: f32,         // pixels
        item_height: f32,          // fixed height per child item (pixels)
        scroll_offset: f32,        // scroll position (pixels from top, 0 = top)
        scrollbar_color: [f32; 4], // scrollbar thumb sRGB RGBA
        scrollbar_width: f32,      // scrollbar track width (pixels)
    },
}
