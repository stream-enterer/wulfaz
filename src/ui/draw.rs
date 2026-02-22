/// Font family selection for multi-font rendering (DD-2, DD-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontFamily {
    /// Libertinus Mono — data, terminal, map grid.
    #[default]
    Mono,
    /// Libertinus Serif — body text, headers.
    Serif,
}

impl FontFamily {
    /// cosmic-text family name for Attrs.
    pub fn family_name(self) -> &'static str {
        match self {
            FontFamily::Mono => "Libertinus Mono",
            FontFamily::Serif => "Libertinus Serif",
        }
    }
}

/// Intermediate draw command for a panel quad.
/// Consumed by `PanelRenderer::add_panel()`.
#[derive(Debug, Clone)]
pub struct PanelCommand {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub bg_color: [f32; 4],     // sRGB RGBA
    pub border_color: [f32; 4], // sRGB RGBA
    pub border_width: f32,
    pub shadow_width: f32,
}

/// Intermediate draw command for a text run.
/// Consumed by `FontRenderer::prepare_text_with_font()`.
#[derive(Debug, Clone)]
pub struct TextCommand {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: [f32; 4], // sRGB RGBA
    pub font_size: f32,
    pub font_family: FontFamily,
}

/// A styled text span within a RichText widget (DD-4).
/// Each span carries its own color and font family.
/// Font size is shared across all spans (set on the widget/command).
#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub color: [f32; 4], // sRGB RGBA
    pub font_family: FontFamily,
}

/// Intermediate draw command for rich text with mixed styles.
/// Consumed by `FontRenderer::prepare_rich_text()`.
#[derive(Debug, Clone)]
pub struct RichTextCommand {
    pub spans: Vec<TextSpan>,
    pub x: f32,
    pub y: f32,
    pub font_size: f32, // shared across all spans
}

/// Collects draw commands from the widget tree.
/// Decouples widget logic from GPU renderers.
pub struct DrawList {
    pub panels: Vec<PanelCommand>,
    pub texts: Vec<TextCommand>,
    pub rich_texts: Vec<RichTextCommand>,
}

impl DrawList {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            texts: Vec::new(),
            rich_texts: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.panels.clear();
        self.texts.clear();
        self.rich_texts.clear();
    }
}
