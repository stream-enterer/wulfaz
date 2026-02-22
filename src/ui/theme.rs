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
    /// Light text (on dark/parchment backgrounds): #F0E6D2
    pub text_light: [f32; 4],
    /// Dark text (on light backgrounds): #3C2A1A
    pub text_dark: [f32; 4],
    /// Danger/warning red: #C04040
    pub danger: [f32; 4],
    /// Disabled/inactive grey: #808080
    pub disabled: [f32; 4],

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

    // -- Map overlay defaults (UI-I02) --
    /// Hover tile highlight color (semi-transparent).
    pub overlay_hover: [f32; 4],
    /// Selected entity tile highlight color (semi-transparent).
    pub overlay_selection: [f32; 4],
    /// Wander target tile highlight color (semi-transparent).
    pub overlay_path: [f32; 4],
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
            bg_parchment: hex_a(0xD4, 0xB8, 0x96, 0.95),
            gold: hex(0xC8, 0xA8, 0x50),
            text_light: hex(0xF0, 0xE6, 0xD2),
            text_dark: hex(0x3C, 0x2A, 0x1A),
            danger: hex(0xC0, 0x40, 0x40),
            disabled: hex(0x80, 0x80, 0x80),

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

            // Status bar defaults (UI-I01a)
            status_bar_bg: hex_a(0xC0, 0xA8, 0x80, 0.98), // darker parchment (same as tooltip)
            status_bar_padding_h: 8.0,
            status_bar_padding_v: 4.0,

            // Tooltip defaults (UI-W04)
            tooltip_delay_ms: 300,
            tooltip_fast_window_ms: 500,
            tooltip_offset_x: 8.0,
            tooltip_offset_y: 8.0,
            tooltip_nesting_offset: 4.0,
            tooltip_padding: 8.0,
            tooltip_bg_color: hex_a(0xC0, 0xA8, 0x80, 0.98), // slightly darker parchment
            tooltip_border_color: hex(0xC8, 0xA8, 0x50),     // gold
            tooltip_border_width: 1.0,
            tooltip_shadow_width: 4.0,

            // Map overlay defaults (UI-I02)
            overlay_hover: hex_a(0xF0, 0xE6, 0xD2, 0.15), // light parchment, subtle
            overlay_selection: hex_a(0xC8, 0xA8, 0x50, 0.35), // gold, prominent
            overlay_path: hex_a(0x60, 0xA0, 0x60, 0.25),  // muted green
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_palette_matches_dd2() {
        let t = Theme::default();

        // Parchment #D4B896 → 212/255, 184/255, 150/255
        assert!((t.bg_parchment[0] - 0.831).abs() < 0.01);
        assert!((t.bg_parchment[1] - 0.722).abs() < 0.01);
        assert!((t.bg_parchment[2] - 0.588).abs() < 0.01);
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
}
