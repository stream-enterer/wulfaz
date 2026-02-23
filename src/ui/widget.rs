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
        /// Per-child height overrides (UI-501). Empty = fixed-height mode.
        item_heights: Vec<f32>,
        /// Text shown when the list has no children. None = no empty state.
        empty_text: Option<String>,
    },

    /// Horizontal or vertical bar showing a 0.0..=1.0 fraction (UI-200).
    /// Always stretch-width (fills parent). Height from field.
    ProgressBar {
        fraction: f32,          // 0.0..=1.0, clamped in draw
        fg_color: [f32; 4],     // foreground (filled portion) sRGB RGBA
        bg_color: [f32; 4],     // background (unfilled portion) sRGB RGBA
        border_color: [f32; 4], // border on background rect sRGB RGBA
        border_width: f32,      // border width in pixels
        height: f32,            // bar height in pixels
    },

    /// Thin line divider between sections (UI-201).
    /// Horizontal: width = parent, height = thickness.
    /// Vertical: width = thickness, height = parent.
    Separator {
        color: [f32; 4],  // sRGB RGBA
        thickness: f32,   // pixels
        horizontal: bool, // true = horizontal divider, false = vertical
    },

    /// Sprite icon from the atlas (UI-202c).
    /// `sprite` is the atlas region name. Renders as a square quad.
    Icon {
        sprite: String,         // atlas region name
        size: f32,              // display size in pixels (square)
        tint: Option<[f32; 4]>, // optional tint multiply (sRGB RGBA)
    },

    /// Checkbox/toggle with label (UI-203).
    /// Draws a small bordered box (16x16 default) with a checkmark icon when checked,
    /// and a text label to the right.
    Checkbox {
        checked: bool,
        label: String,
        color: [f32; 4], // text + border color sRGB RGBA
        font_size: f32,  // pixels
    },

    /// Drop-down select widget (UI-204).
    /// Closed: button showing `options[selected]` with down-arrow.
    /// Open: emits option labels below the trigger rect.
    /// Builder manages open/close state and rebuilds each frame.
    Dropdown {
        selected: usize,
        options: Vec<String>,
        open: bool,
        color: [f32; 4],    // text color sRGB RGBA
        bg_color: [f32; 4], // background sRGB RGBA
        font_size: f32,     // pixels
    },

    /// Horizontal slider widget (UI-205).
    /// Thin track with a draggable thumb positioned at `(value - min) / (max - min)`.
    Slider {
        value: f32,
        min: f32,
        max: f32,
        track_color: [f32; 4], // track bar sRGB RGBA
        thumb_color: [f32; 4], // thumb sRGB RGBA
        width: f32,            // total track width in pixels
    },

    /// Single-line text input field (UI-206).
    /// Draws background, text content (or placeholder if empty),
    /// and a cursor line when focused.
    TextInput {
        text: String,
        cursor_pos: usize,   // byte offset within text
        color: [f32; 4],     // text color sRGB RGBA
        bg_color: [f32; 4],  // background sRGB RGBA
        font_size: f32,      // pixels
        placeholder: String, // shown when text is empty
        focused: bool,       // whether cursor is visible
    },

    /// Collapsible section with header and toggleable children (UI-304).
    /// Header row shows a triangle indicator + label. Click toggles `expanded`.
    /// When collapsed, children are not measured, laid out, or drawn.
    Collapsible {
        header: String,
        expanded: bool,
        color: [f32; 4], // text + indicator color sRGB RGBA
        font_size: f32,  // pixels
    },

    /// Tab container with a tab bar and content area (UI-301).
    /// Tab bar is drawn by the widget. Children are the active tab's content,
    /// laid out Column-style below the tab bar.
    TabContainer {
        tabs: Vec<String>,
        active: usize,
        tab_color: [f32; 4],    // inactive tab background sRGB RGBA
        active_color: [f32; 4], // active tab background sRGB RGBA
        font_size: f32,         // pixels
    },

    /// Invisible spacer that fills remaining space in Row/Column (UI-601).
    /// In a Row, absorbs remaining width. In a Column, absorbs remaining height.
    /// Emits no draw commands. Used for push-to-end and centering layouts.
    Expand,
}
