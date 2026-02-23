use super::draw::FontFamily;

/// Centralized visual style constants (DD-2).
///
/// Single global theme. No runtime switching. Widgets read from Theme
/// at construction time instead of hardcoding colors.
#[derive(Debug, Clone)]
pub struct Theme {
    // -- Color palette (sRGB RGBA) --
    /// Parchment background: #D4B896
    pub bg_parchment: [f32; 4],
    /// Gold accent: #C8A850
    pub gold: [f32; 4],
    /// High-contrast text (headers, emphasis): #F0E6D2
    pub text_light: [f32; 4],
    /// Default body text: #D8C8A8
    pub text_medium: [f32; 4],
    /// Secondary/metadata text: #A09078
    pub text_low: [f32; 4],
    /// Danger/warning red accent: #C04040
    pub danger: [f32; 4],
    /// Disabled/inactive grey: #808080
    pub disabled: [f32; 4],

    // -- Semantic text colors (UI-700) --
    /// Positive values (health OK, good outcomes): green.
    pub text_positive: [f32; 4],
    /// Negative values (damage, bad outcomes): red.
    pub text_negative: [f32; 4],
    /// Caution/mixed values: gold.
    pub text_warning: [f32; 4],

    // -- Panel defaults --
    /// Default panel border color (gold).
    pub panel_border_color: [f32; 4],
    /// Default panel border width in pixels.
    pub panel_border_width: f32,
    /// Default panel inner shadow width in pixels.
    pub panel_shadow_width: f32,

    // -- Font defaults --
    /// Header font family.
    pub font_header_family: FontFamily,
    /// Header font size in pixels (16pt).
    pub font_header_size: f32,
    /// Body font family.
    pub font_body_family: FontFamily,
    /// Body font size in pixels (12pt).
    pub font_body_size: f32,
    /// Data/terminal font family.
    pub font_data_family: FontFamily,
    /// Data/terminal font size in pixels (9pt).
    pub font_data_size: f32,

    // -- Spacing defaults --
    /// Default panel padding in pixels.
    pub panel_padding: f32,
    /// Vertical gap between stacked labels in pixels.
    pub label_gap: f32,
    /// Button internal horizontal padding in pixels.
    pub button_pad_h: f32,
    /// Button internal vertical padding in pixels.
    pub button_pad_v: f32,

    // -- ScrollList defaults --
    /// Scrollbar thumb width in pixels.
    pub scrollbar_width: f32,
    /// Scrollbar thumb color (sRGB RGBA).
    pub scrollbar_color: [f32; 4],
    /// Default scroll list item height in pixels.
    pub scroll_item_height: f32,
    /// Alpha tint for alternating (odd-indexed) scroll list rows.
    pub scroll_row_alt_alpha: f32,

    // -- Status bar defaults (UI-I01a) --
    /// Status bar background color (sRGB RGBA).
    pub status_bar_bg: [f32; 4],
    /// Status bar horizontal padding (pixels).
    pub status_bar_padding_h: f32,
    /// Status bar vertical padding (pixels).
    pub status_bar_padding_v: f32,

    // -- Tooltip defaults (UI-W04) --
    /// Hover delay before showing tooltip (milliseconds).
    pub tooltip_delay_ms: u64,
    /// Fast-show window after a tooltip is dismissed (milliseconds).
    /// Within this window, new tooltips appear instantly.
    pub tooltip_fast_window_ms: u64,
    /// Horizontal offset from cursor to tooltip (pixels).
    pub tooltip_offset_x: f32,
    /// Vertical offset from cursor to tooltip (pixels).
    pub tooltip_offset_y: f32,
    /// Per-nesting-level position offset (pixels).
    pub tooltip_nesting_offset: f32,
    /// Internal padding for tooltip panels (pixels).
    pub tooltip_padding: f32,
    /// Tooltip background color (sRGB RGBA).
    pub tooltip_bg_color: [f32; 4],
    /// Tooltip border color (sRGB RGBA).
    pub tooltip_border_color: [f32; 4],
    /// Tooltip border width (pixels).
    pub tooltip_border_width: f32,
    /// Tooltip inner shadow width (pixels).
    pub tooltip_shadow_width: f32,
    /// Maximum tooltip width in pixels. Text wraps at this boundary.
    pub tooltip_max_width: f32,

    // -- Map overlay defaults (UI-I02) --
    /// Hover tile highlight color (semi-transparent).
    pub overlay_hover: [f32; 4],
    /// Selected entity tile highlight color (semi-transparent).
    pub overlay_selection: [f32; 4],
    /// Wander target tile highlight color (semi-transparent).
    pub overlay_path: [f32; 4],

    // -- Progress bar defaults (UI-200) --
    /// Default progress bar height in pixels.
    pub progress_bar_height: f32,
    /// Default progress bar border width in pixels.
    pub progress_bar_border_width: f32,
    /// Health bar foreground color (green).
    pub progress_bar_health_fg: [f32; 4],
    /// Health bar background color (dark).
    pub progress_bar_health_bg: [f32; 4],

    // -- Separator defaults (UI-201) --
    /// Default separator color (sRGB RGBA).
    pub separator_color: [f32; 4],
    /// Default separator thickness in pixels.
    pub separator_thickness: f32,

    // -- Tab container defaults (UI-301) --
    /// Active tab background color (sRGB RGBA).
    pub tab_active_color: [f32; 4],
    /// Inactive tab background color (sRGB RGBA).
    pub tab_inactive_color: [f32; 4],
    /// Tab bar row height in pixels.
    pub tab_bar_height: f32,

    // -- Animation defaults (UI-W05) --
    /// Hover tooltip fade-in duration (milliseconds).
    pub anim_tooltip_fade_ms: u64,
    /// Inspector panel slide-in duration (milliseconds).
    pub anim_inspector_slide_ms: u64,
    /// Button hover highlight transition duration (milliseconds).
    pub anim_hover_highlight_ms: u64,
    /// Button hover highlight alpha (0.0 = transparent, 1.0 = opaque).
    pub anim_hover_highlight_alpha: f32,
    /// Panel hide (slide-out/fade-out) duration (milliseconds).
    pub anim_panel_hide_ms: u64,

    // -- Scaling / accessibility (UI-504) --
    /// Global UI scale factor (0.5..=2.0). Default 1.0.
    pub ui_scale: f32,
    /// High-contrast mode: thicker borders, full-alpha text.
    pub high_contrast: bool,
}

impl Theme {
    /// Scale a pixel value by `ui_scale`, rounded to nearest integer.
    pub fn s(&self, px: f32) -> f32 {
        (px * self.ui_scale).round()
    }

    /// Scale a font size by `ui_scale`, clamped to minimum 6px.
    pub fn font_size(&self, base: f32) -> f32 {
        (base * self.ui_scale).round().max(6.0)
    }

    /// Border width, thicker in high-contrast mode.
    pub fn border_width(&self) -> f32 {
        self.panel_border_width + if self.high_contrast { 1.0 } else { 0.0 }
    }

    /// Text alpha: 1.0 in high-contrast, unchanged otherwise.
    pub fn text_alpha(&self) -> f32 {
        if self.high_contrast { 1.0 } else { 0.85 }
    }
}

/// Convert a hex color (#RRGGBB) to sRGB [f32; 4] with alpha 1.0.
const fn hex(r: u8, g: u8, b: u8) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
}

/// Convert a hex color with custom alpha.
const fn hex_a(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a]
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // DD-2 palette
            bg_parchment: hex_a(0x3E, 0x2C, 0x1C, 0.95),
            gold: hex(0xC8, 0xA8, 0x50),
            text_light: hex(0xF0, 0xE6, 0xD2),
            text_medium: hex(0xD8, 0xC8, 0xA8),
            text_low: hex(0xA0, 0x90, 0x78),
            danger: hex(0xC0, 0x40, 0x40),
            disabled: hex(0x80, 0x80, 0x80),

            // Semantic text colors (UI-700)
            text_positive: hex(0x40, 0xA0, 0x40), // green (matches progress_bar_health_fg)
            text_negative: hex(0xC0, 0x40, 0x40), // red (matches danger)
            text_warning: hex(0xC8, 0xA8, 0x50),  // gold (matches gold)

            // Panel defaults
            panel_border_color: hex(0xC8, 0xA8, 0x50), // gold
            panel_border_width: 2.0,
            panel_shadow_width: 6.0,

            // Font defaults (DD-2)
            font_header_family: FontFamily::Serif,
            font_header_size: 16.0,
            font_body_family: FontFamily::Serif,
            font_body_size: 12.0,
            font_data_family: FontFamily::Mono,
            font_data_size: 9.0,

            // Spacing
            panel_padding: 12.0,
            label_gap: 4.0,
            button_pad_h: 8.0,
            button_pad_v: 4.0,

            // ScrollList defaults
            scrollbar_width: 6.0,
            scrollbar_color: hex_a(0xC8, 0xA8, 0x50, 0.5), // gold at 50% alpha
            scroll_item_height: 20.0,                      // pixels
            scroll_row_alt_alpha: 0.04,                    // subtle alternation

            // Status bar defaults (UI-I01a)
            status_bar_bg: hex_a(0x32, 0x24, 0x16, 0.98), // darker brown
            status_bar_padding_h: 8.0,
            status_bar_padding_v: 4.0,

            // Tooltip defaults (UI-W04)
            tooltip_delay_ms: 300,
            tooltip_fast_window_ms: 500,
            tooltip_offset_x: 8.0,
            tooltip_offset_y: 8.0,
            tooltip_nesting_offset: 4.0,
            tooltip_padding: 8.0,
            tooltip_bg_color: hex_a(0x32, 0x24, 0x16, 0.98), // darker brown
            tooltip_border_color: hex(0xC8, 0xA8, 0x50),     // gold
            tooltip_border_width: 1.0,
            tooltip_shadow_width: 4.0,
            tooltip_max_width: 400.0,

            // Map overlay defaults (UI-I02)
            overlay_hover: hex_a(0xF0, 0xE6, 0xD2, 0.15), // light parchment, subtle
            overlay_selection: hex_a(0xC8, 0xA8, 0x50, 0.35), // gold, prominent
            overlay_path: hex_a(0x60, 0xA0, 0x60, 0.25),  // muted green

            // Progress bar defaults (UI-200)
            progress_bar_height: 8.0,
            progress_bar_border_width: 1.0,
            progress_bar_health_fg: hex(0x40, 0xA0, 0x40), // green
            progress_bar_health_bg: hex_a(0x55, 0x44, 0x30, 0.6), // medium brown

            // Separator defaults (UI-201)
            separator_color: hex_a(0xC8, 0xA8, 0x50, 0.3), // gold at 30% alpha
            separator_thickness: 1.0,

            // Tab container defaults (UI-301)
            tab_active_color: hex_a(0x3E, 0x2C, 0x1C, 0.95), // matches bg
            tab_inactive_color: hex_a(0x2A, 0x1E, 0x12, 0.7), // darker brown
            tab_bar_height: 24.0,

            // Animation defaults (UI-W05)
            anim_tooltip_fade_ms: 150,
            anim_inspector_slide_ms: 200,
            anim_hover_highlight_ms: 200,
            anim_hover_highlight_alpha: 0.3,
            anim_panel_hide_ms: 150,

            // Scaling / accessibility (UI-504)
            ui_scale: 1.0,
            high_contrast: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_palette_matches_dd2() {
        let t = Theme::default();

        // Dark brown bg #3E2C1C
        assert!((t.bg_parchment[0] - 0x3E as f32 / 255.0).abs() < 0.01);
        assert!((t.bg_parchment[1] - 0x2C as f32 / 255.0).abs() < 0.01);
        assert!((t.bg_parchment[2] - 0x1C as f32 / 255.0).abs() < 0.01);
        assert!((t.bg_parchment[3] - 0.95).abs() < 0.01);

        // Gold #C8A850
        assert!((t.gold[0] - 0.784).abs() < 0.01);
        assert!((t.gold[1] - 0.659).abs() < 0.01);
        assert!((t.gold[2] - 0.314).abs() < 0.01);

        // Text light #F0E6D2
        assert!((t.text_light[0] - 0.941).abs() < 0.01);
        assert!((t.text_light[1] - 0.902).abs() < 0.01);
        assert!((t.text_light[2] - 0.824).abs() < 0.01);

        // Danger #C04040
        assert!((t.danger[0] - 0.753).abs() < 0.01);
        assert!((t.danger[1] - 0.251).abs() < 0.01);
        assert!((t.danger[2] - 0.251).abs() < 0.01);

        // Font families
        assert_eq!(t.font_header_family, FontFamily::Serif);
        assert_eq!(t.font_body_family, FontFamily::Serif);
        assert_eq!(t.font_data_family, FontFamily::Mono);

        // Font sizes
        assert!((t.font_header_size - 16.0).abs() < 0.01);
        assert!((t.font_body_size - 12.0).abs() < 0.01);
        assert!((t.font_data_size - 9.0).abs() < 0.01);
    }

    #[test]
    fn hex_conversion() {
        let white = hex(0xFF, 0xFF, 0xFF);
        assert!((white[0] - 1.0).abs() < 0.001);
        assert!((white[1] - 1.0).abs() < 0.001);
        assert!((white[2] - 1.0).abs() < 0.001);
        assert!((white[3] - 1.0).abs() < 0.001);

        let black = hex(0x00, 0x00, 0x00);
        assert!(black[0].abs() < 0.001);
        assert!(black[1].abs() < 0.001);
        assert!(black[2].abs() < 0.001);

        let half_alpha = hex_a(0x80, 0x80, 0x80, 0.5);
        assert!((half_alpha[3] - 0.5).abs() < 0.001);
    }

    #[test]
    fn overlay_colors_are_semi_transparent() {
        let t = Theme::default();
        assert!(t.overlay_hover[3] > 0.0 && t.overlay_hover[3] < 1.0);
        assert!(t.overlay_selection[3] > 0.0 && t.overlay_selection[3] < 1.0);
        assert!(t.overlay_path[3] > 0.0 && t.overlay_path[3] < 1.0);
        // Selection should be more visible than hover.
        assert!(t.overlay_selection[3] > t.overlay_hover[3]);
    }

    #[test]
    fn scale_default_identity() {
        let t = Theme::default();
        assert!((t.ui_scale - 1.0).abs() < f32::EPSILON);
        assert!(!t.high_contrast);
        // s() at scale 1.0 returns input unchanged (after rounding)
        assert!((t.s(12.0) - 12.0).abs() < f32::EPSILON);
        assert!((t.font_size(16.0) - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scale_doubles_values() {
        let mut t = Theme::default();
        t.ui_scale = 2.0;
        assert!((t.s(12.0) - 24.0).abs() < f32::EPSILON);
        assert!((t.s(8.0) - 16.0).abs() < f32::EPSILON);
        assert!((t.font_size(9.0) - 18.0).abs() < f32::EPSILON);
        assert!((t.font_size(16.0) - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn font_size_min_clamp() {
        let mut t = Theme::default();
        t.ui_scale = 0.5;
        // 0.5 * 9.0 = 4.5, clamped to 6.0
        assert!((t.font_size(9.0) - 6.0).abs() < f32::EPSILON);
        // 0.5 * 16.0 = 8.0, above minimum
        assert!((t.font_size(16.0) - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn high_contrast_border_and_alpha() {
        let mut t = Theme::default();
        let normal_border = t.border_width();
        let normal_alpha = t.text_alpha();

        t.high_contrast = true;
        assert!(t.border_width() > normal_border);
        assert!((t.text_alpha() - 1.0).abs() < f32::EPSILON);
        assert!(t.text_alpha() >= normal_alpha);
    }
}
