//! Mini-map panel (UI-407).
//!
//! Small overview panel in the bottom-right showing a downscaled tile map
//! with viewport indicator. Click-to-navigate and drag-to-pan.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{Edges, FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

/// Minimap display size in screen pixels.
const MINIMAP_DISPLAY_W: f32 = 128.0;
const MINIMAP_DISPLAY_H: f32 = 96.0;

/// Minimap pixel dimensions (integer).
const MINIMAP_W: usize = 128;
const MINIMAP_H: usize = 96;

/// Info needed to build the minimap.
pub struct MinimapInfo {
    pub map_width: u32,
    pub map_height: u32,
    pub camera_x: f32,
    pub camera_y: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}

/// Compact padding for the minimap frame (px). Smaller than standard
/// panel padding since this is a navigation widget, not a content window.
const MINIMAP_PAD: f32 = 4.0;

/// Build the minimap panel (UI-407).
///
/// Returns `(panel_root, map_area)`. The minimap texture is rendered
/// separately via the sprite pipeline; this builds the UI frame.
pub fn build_minimap(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &MinimapInfo,
) -> (WidgetId, WidgetId) {
    // Root panel — Fit on both axes. measure_node now propagates Fixed-sized
    // children through Columns, so the panel shrink-wraps correctly.
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.gold,
        border_width: theme.panel_border_width,
        shadow_width: 2.0,
    });
    tree.set_sizing(panel, Sizing::Fit, Sizing::Fit);
    tree.set_padding(panel, Edges::all(MINIMAP_PAD));

    // Position at bottom-right (estimate dimensions for placement).
    // Label height uses font_data_size + 4px headroom for ascent+descent.
    let est_w = MINIMAP_DISPLAY_W + MINIMAP_PAD * 2.0;
    let est_h = MINIMAP_DISPLAY_H + MINIMAP_PAD * 2.0 + (theme.font_data_size + 4.0) + 2.0;
    let px = info.screen_width - est_w - 8.0;
    let py = info.screen_height - est_h - 8.0;
    tree.set_position(panel, Position::Fixed { x: px, y: py });

    // Frame column.
    let col = tree.insert(
        panel,
        Widget::Column {
            gap: 2.0,
            align: CrossAlign::Center,
        },
    );

    // Title label
    tree.insert(
        col,
        Widget::Label {
            text: "Map".to_string(),
            color: theme.text_medium,
            font_size: theme.font_data_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    // Minimap area: a panel that serves as the click target for navigation.
    // The actual texture is rendered by the sprite pipeline on top of this.
    let map_area = tree.insert(
        col,
        Widget::Panel {
            bg_color: [0.15, 0.12, 0.10, 1.0], // dark background for map
            border_color: theme.panel_border_color,
            border_width: 1.0,
            shadow_width: 0.0,
        },
    );
    tree.set_sizing(
        map_area,
        Sizing::Fixed(MINIMAP_DISPLAY_W),
        Sizing::Fixed(MINIMAP_DISPLAY_H),
    );

    (panel, map_area)
}

/// Convert a click position within the minimap to world coordinates.
pub fn minimap_click_to_world(
    click_x: f32,
    click_y: f32,
    minimap_rect_x: f32,
    minimap_rect_y: f32,
    map_width: u32,
    map_height: u32,
) -> (f32, f32) {
    let rel_x = (click_x - minimap_rect_x) / MINIMAP_DISPLAY_W;
    let rel_y = (click_y - minimap_rect_y) / MINIMAP_DISPLAY_H;
    let world_x = rel_x * map_width as f32;
    let world_y = rel_y * map_height as f32;
    (world_x, world_y)
}

/// Minimap texture (128×96 RGBA).
///
/// Blank base image. Per-frame, the base is copied and a dynamic viewport
/// indicator is stamped on top.
pub struct MinimapTexture {
    base: Vec<u8>,
    frame: Vec<u8>,
}

impl Default for MinimapTexture {
    fn default() -> Self {
        Self::new()
    }
}

impl MinimapTexture {
    /// Create a blank minimap texture.
    pub fn new() -> Self {
        let pixel_count = MINIMAP_W * MINIMAP_H;
        let base = vec![0u8; pixel_count * 4];
        let frame = base.clone();
        MinimapTexture { base, frame }
    }

    /// Set a single pixel in the frame buffer with bounds checking.
    fn set_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x >= 0 && x < MINIMAP_W as i32 && y >= 0 && y < MINIMAP_H as i32 {
            let idx = (y as usize * MINIMAP_W + x as usize) * 4;
            self.frame[idx..idx + 4].copy_from_slice(&color);
        }
    }

    /// Stamp the viewport indicator onto the frame buffer.
    ///
    /// Draws a rectangle whose size matches the actual viewport extent on the
    /// minimap. Small viewports (< 7px either dimension) are filled solid;
    /// larger ones draw a 1px hollow outline.
    pub fn render_frame(
        &mut self,
        cam_center_x: f32,
        cam_center_y: f32,
        viewport_cols: f32,
        viewport_rows: f32,
        map_w: u32,
        map_h: u32,
    ) {
        self.frame.copy_from_slice(&self.base);

        if map_w == 0 || map_h == 0 {
            return;
        }

        // Camera center → minimap pixel coordinates.
        let mx = (cam_center_x * MINIMAP_W as f32 / map_w as f32).round() as i32;
        let my = (cam_center_y * MINIMAP_H as f32 / map_h as f32).round() as i32;

        // Viewport extent in minimap pixels (minimum 5).
        let vp_w = (viewport_cols * MINIMAP_W as f32 / map_w as f32)
            .ceil()
            .max(5.0) as i32;
        let vp_h = (viewport_rows * MINIMAP_H as f32 / map_h as f32)
            .ceil()
            .max(5.0) as i32;

        // Top-left corner of viewport rect.
        let left = mx - vp_w / 2;
        let top = my - vp_h / 2;

        let outline: [u8; 4] = [0, 0, 0, 255];
        let indicator: [u8; 4] = [255, 255, 255, 255];

        // 1px dark outline (border ring outside the viewport rect).
        for dy in -1..=vp_h {
            for dx in -1..=vp_w {
                if dx >= 0 && dx < vp_w && dy >= 0 && dy < vp_h {
                    continue; // skip interior
                }
                self.set_pixel(left + dx, top + dy, outline);
            }
        }

        // White viewport indicator.
        if vp_w < 7 || vp_h < 7 {
            // Small viewport — fill solid.
            for dy in 0..vp_h {
                for dx in 0..vp_w {
                    self.set_pixel(left + dx, top + dy, indicator);
                }
            }
        } else {
            // Hollow rectangle: 1px border.
            for dx in 0..vp_w {
                self.set_pixel(left + dx, top, indicator);
                self.set_pixel(left + dx, top + vp_h - 1, indicator);
            }
            for dy in 1..vp_h - 1 {
                self.set_pixel(left, top + dy, indicator);
                self.set_pixel(left + vp_w - 1, top + dy, indicator);
            }
        }
    }

    /// Current frame buffer, ready for GPU upload (128×96 RGBA).
    pub fn pixels(&self) -> &[u8] {
        &self.frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Downscale factor from world to minimap pixels.
    fn minimap_scale(map_width: u32, map_height: u32) -> (f32, f32) {
        (
            MINIMAP_DISPLAY_W / map_width as f32,
            MINIMAP_DISPLAY_H / map_height as f32,
        )
    }

    #[test]
    fn minimap_builds_successfully() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = MinimapInfo {
            map_width: 6309,
            map_height: 4753,
            camera_x: 3000.0,
            camera_y: 2000.0,
            viewport_w: 80.0,
            viewport_h: 60.0,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let (root, map_area) = build_minimap(&mut tree, &theme, &info);
        assert!(tree.get(root).is_some());
        assert!(tree.get(map_area).is_some());
    }

    #[test]
    fn click_to_world_center() {
        // Click at minimap center should map to world center.
        let (wx, wy) = minimap_click_to_world(
            64.0, // center of 128px
            48.0, // center of 96px
            0.0, 0.0, 6309, 4753,
        );
        assert!((wx - 3154.5).abs() < 1.0);
        assert!((wy - 2376.5).abs() < 1.0);
    }

    #[test]
    fn minimap_scale_values() {
        let (sx, sy) = minimap_scale(6309, 4753);
        assert!(sx > 0.0 && sx < 1.0);
        assert!(sy > 0.0 && sy < 1.0);
    }

    #[test]
    fn render_frame_stamps_viewport_marker() {
        let mut tex = MinimapTexture::new();
        // Small viewport → solid fill, center pixel should be white.
        tex.render_frame(64.0, 64.0, 1.0, 1.0, 128, 128);

        // Marker center at minimap pixel (64, 48).
        let idx = (48 * MINIMAP_W + 64) * 4;
        assert_eq!(
            &tex.frame[idx..idx + 4],
            &[255, 255, 255, 255],
            "Viewport marker center should be white"
        );
    }

    #[test]
    fn viewport_indicator_grows_with_zoom() {
        let mut tex = MinimapTexture::new();

        // Large viewport (half the map) → hollow rect.
        tex.render_frame(128.0, 128.0, 128.0, 128.0, 256, 256);

        // vp_w = ceil(128 * 128 / 256) = 64, vp_h = ceil(128 * 96 / 256) = 48.
        // Center pixel (64, 48) should be transparent (hollow interior).
        let idx = (48 * MINIMAP_W + 64) * 4;
        assert_ne!(
            &tex.frame[idx..idx + 4],
            &[255u8, 255, 255, 255],
            "Hollow rect interior should not be white"
        );

        // Top-left of the rect should be white (border).
        // left = 64 - 64/2 = 32, top = 48 - 48/2 = 24.
        let edge_idx = (24 * MINIMAP_W + 32) * 4;
        assert_eq!(
            &tex.frame[edge_idx..edge_idx + 4],
            &[255, 255, 255, 255],
            "Viewport rect edge should be white"
        );
    }
}
