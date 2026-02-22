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
/// Consumed by `FontRenderer::prepare_text()`.
#[derive(Debug, Clone)]
pub struct TextCommand {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: [f32; 4], // sRGB RGBA
    pub font_size: f32,
}

/// Collects draw commands from the widget tree.
/// Decouples widget logic from GPU renderers.
pub struct DrawList {
    pub panels: Vec<PanelCommand>,
    pub texts: Vec<TextCommand>,
}

impl DrawList {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.panels.clear();
        self.texts.clear();
    }
}
