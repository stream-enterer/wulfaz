//! Mini-map panel (UI-407).
//!
//! Small overview panel in the bottom-right showing a downscaled tile map
//! with viewport indicator. Click-to-navigate and drag-to-pan.

use std::collections::HashMap;

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{Edges, FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};
use crate::tile_map::TileMap;

/// Minimap display size in screen pixels.
const MINIMAP_DISPLAY_W: f32 = 256.0;
const MINIMAP_DISPLAY_H: f32 = 192.0;

/// Minimap pixel dimensions (integer).
const MINIMAP_W: usize = 256;
const MINIMAP_H: usize = 192;

/// 3×5 bitmap font for digits 0-9.
/// Each digit is 5 rows of 3 pixels. Each row stored as a u8
/// where bits 2/1/0 represent left/middle/right pixels.
const DIGIT_BITMAPS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111], // 0
    [0b010, 0b110, 0b010, 0b010, 0b111], // 1
    [0b111, 0b001, 0b111, 0b100, 0b111], // 2
    [0b111, 0b001, 0b111, 0b001, 0b111], // 3
    [0b101, 0b101, 0b111, 0b001, 0b001], // 4
    [0b111, 0b100, 0b111, 0b001, 0b111], // 5
    [0b111, 0b100, 0b111, 0b101, 0b111], // 6
    [0b111, 0b001, 0b001, 0b001, 0b001], // 7
    [0b111, 0b101, 0b111, 0b101, 0b111], // 8
    [0b111, 0b101, 0b111, 0b001, 0b111], // 9
];

/// Blit a single 3×5 digit into an RGBA buffer.
fn blit_digit(buf: &mut [u8], stride: usize, x: i32, y: i32, digit: u8, color: [u8; 4]) {
    let bitmap = &DIGIT_BITMAPS[digit as usize % 10];
    for (row, &bits) in bitmap.iter().enumerate() {
        let py = y + row as i32;
        if py < 0 || py >= MINIMAP_H as i32 {
            continue;
        }
        for col in 0..3i32 {
            if bits & (1 << (2 - col)) != 0 {
                let px = x + col;
                if px >= 0 && px < stride as i32 {
                    let idx = (py as usize * stride + px as usize) * 4;
                    buf[idx..idx + 4].copy_from_slice(&color);
                }
            }
        }
    }
}

/// Blit a 1-3 digit number centered at (cx, cy) with drop shadow.
fn blit_number(
    buf: &mut [u8],
    stride: usize,
    cx: i32,
    cy: i32,
    n: u8,
    shadow: [u8; 4],
    fg: [u8; 4],
) {
    let digits: Vec<u8> = if n >= 100 {
        vec![n / 100, (n / 10) % 10, n % 10]
    } else if n >= 10 {
        vec![n / 10, n % 10]
    } else {
        vec![n]
    };

    // Total width: 3px per digit + 1px gap between digits.
    let total_w = digits.len() as i32 * 3 + (digits.len() as i32 - 1);
    let start_x = cx - total_w / 2;
    let start_y = cy - 2; // center 5px height

    for (i, &d) in digits.iter().enumerate() {
        let dx = start_x + i as i32 * 4; // 3px digit + 1px gap
        // Shadow at (+1, +1)
        blit_digit(buf, stride, dx + 1, start_y + 1, d, shadow);
        // Foreground at (0, 0)
        blit_digit(buf, stride, dx, start_y, d, fg);
    }
}

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

/// Build the minimap panel (UI-407).
///
/// Returns `(panel_root, map_area)`. The minimap texture is rendered
/// separately via the sprite pipeline; this builds the UI frame.
pub fn build_minimap(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &MinimapInfo,
) -> (WidgetId, WidgetId) {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.gold,
        border_width: theme.panel_border_width,
        shadow_width: 2.0,
    });
    let panel_w = MINIMAP_DISPLAY_W + theme.panel_padding * 2.0;
    let panel_h =
        MINIMAP_DISPLAY_H + theme.panel_padding * 2.0 + theme.font_data_size + theme.label_gap;
    tree.set_sizing(panel, Sizing::Fixed(panel_w), Sizing::Fixed(panel_h));
    tree.set_padding(panel, Edges::all(theme.panel_padding));

    // Position at bottom-right
    let px = info.screen_width - panel_w - 8.0;
    let py = info.screen_height - panel_h - 8.0;
    tree.set_position(panel, Position::Fixed { x: px, y: py });

    let col = tree.insert(
        panel,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Center,
        },
    );
    tree.set_position(col, Position::Fixed { x: 0.0, y: 0.0 });
    tree.set_sizing(col, Sizing::Fixed(MINIMAP_DISPLAY_W), Sizing::Fit);

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
    // Viewport indicator info as tooltip
    let vp_info = format!(
        "Camera: ({:.0}, {:.0})\nView: {:.0}x{:.0}",
        info.camera_x, info.camera_y, info.viewport_w, info.viewport_h
    );
    tree.set_tooltip(map_area, Some(super::widget::TooltipContent::Text(vp_info)));

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

/// Downscale factor from world to minimap pixels.
pub fn minimap_scale(map_width: u32, map_height: u32) -> (f32, f32) {
    (
        MINIMAP_DISPLAY_W / map_width as f32,
        MINIMAP_DISPLAY_H / map_height as f32,
    )
}

/// Pre-computed minimap texture (256×192 RGBA).
///
/// Generated once from TileMap data. Per-frame, the base image is copied
/// and a dynamic viewport indicator is stamped on top.
pub struct MinimapTexture {
    base: Vec<u8>,
    frame: Vec<u8>,
}

impl MinimapTexture {
    /// Generate the static minimap base from tile data.
    ///
    /// Samples quartier IDs at minimap resolution, draws border lines where
    /// adjacent pixels belong to different quartiers, and blits ID numbers
    /// at each quartier's centroid.
    pub fn generate(tiles: &TileMap) -> Self {
        let map_w = tiles.width();
        let map_h = tiles.height();
        let pixel_count = MINIMAP_W * MINIMAP_H;

        // Zeroed = transparent (dark panel bg shows through for river/edges).
        let mut base = vec![0u8; pixel_count * 4];

        // Sample quartier IDs at minimap resolution.
        let mut qid_grid = vec![0u8; pixel_count];
        // Per-quartier centroid accumulator: (sum_x, sum_y, count).
        let mut centroids: HashMap<u8, (u64, u64, u64)> = HashMap::new();

        for py in 0..MINIMAP_H {
            for px in 0..MINIMAP_W {
                let wx = px * map_w / MINIMAP_W;
                let wy = py * map_h / MINIMAP_H;
                let qid = tiles.get_quartier_id(wx, wy).unwrap_or(0);
                qid_grid[py * MINIMAP_W + px] = qid;
                if qid > 0 {
                    let entry = centroids.entry(qid).or_insert((0, 0, 0));
                    entry.0 += px as u64;
                    entry.1 += py as u64;
                    entry.2 += 1;
                }
            }
        }

        // Draw borders: pixel is a border if its right or bottom neighbor
        // has a different non-zero quartier ID.
        let border_color: [u8; 4] = [200, 180, 140, 255]; // muted gold
        for py in 0..MINIMAP_H {
            for px in 0..MINIMAP_W {
                let qid = qid_grid[py * MINIMAP_W + px];
                if qid == 0 {
                    continue;
                }
                let mut is_border = false;
                if px + 1 < MINIMAP_W {
                    let right = qid_grid[py * MINIMAP_W + px + 1];
                    if right != qid && right != 0 {
                        is_border = true;
                    }
                }
                if !is_border && py + 1 < MINIMAP_H {
                    let bottom = qid_grid[(py + 1) * MINIMAP_W + px];
                    if bottom != qid && bottom != 0 {
                        is_border = true;
                    }
                }
                if is_border {
                    let idx = (py * MINIMAP_W + px) * 4;
                    base[idx..idx + 4].copy_from_slice(&border_color);
                }
            }
        }

        // Blit ID numbers at quartier centroids.
        let label_fg: [u8; 4] = [220, 200, 170, 255];
        let label_shadow: [u8; 4] = [30, 25, 20, 255];
        for (&qid, &(sum_x, sum_y, count)) in &centroids {
            if count < 20 {
                continue; // quartier too small for a label
            }
            let cx = (sum_x / count) as i32;
            let cy = (sum_y / count) as i32;
            blit_number(&mut base, MINIMAP_W, cx, cy, qid, label_shadow, label_fg);
        }

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

    /// Current frame buffer, ready for GPU upload (256×192 RGBA).
    pub fn pixels(&self) -> &[u8] {
        &self.frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            128.0, // center of 256px
            96.0,  // center of 192px
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
    fn minimap_texture_border_and_labels() {
        let mut tiles = TileMap::new(128, 128);
        // Left half = quartier 1, right half = quartier 2.
        for y in 0..128 {
            for x in 0..128 {
                let qid = if x < 64 { 1 } else { 2 };
                tiles.set_quartier_id(x, y, qid);
            }
        }

        let tex = MinimapTexture::generate(&tiles);

        // At 256×192 minimap for a 128-wide map: each minimap pixel covers
        // 0.5 tiles horizontally. Pixel 127 → tile 63 (qid=1), pixel 128 →
        // tile 64 (qid=2). Border at x=127.
        let border_color = [200u8, 180, 140, 255];
        for py in 10..160 {
            let idx = (py * MINIMAP_W + 127) * 4;
            assert_eq!(
                &tex.base[idx..idx + 4],
                &border_color,
                "Expected border at (127, {py})"
            );
        }

        // Centroid of quartier 1 ≈ (63, 95), quartier 2 ≈ (191, 95).
        // Verify non-zero/non-border pixels exist near centroids (label text).
        let has_label_near = |cx: usize, cy: usize| -> bool {
            for dy in 0..8 {
                for dx in 0..8 {
                    let px = cx.wrapping_sub(4) + dx;
                    let py = cy.wrapping_sub(4) + dy;
                    if px < MINIMAP_W && py < MINIMAP_H {
                        let idx = (py * MINIMAP_W + px) * 4;
                        if tex.base[idx + 3] > 0 && tex.base[idx..idx + 4] != border_color {
                            return true;
                        }
                    }
                }
            }
            false
        };

        assert!(
            has_label_near(63, 95),
            "Expected label near quartier 1 centroid"
        );
        assert!(
            has_label_near(191, 95),
            "Expected label near quartier 2 centroid"
        );
    }

    #[test]
    fn render_frame_stamps_viewport_marker() {
        let mut tiles = TileMap::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                tiles.set_quartier_id(x, y, 1);
            }
        }
        let mut tex = MinimapTexture::generate(&tiles);
        // Small viewport → solid fill, center pixel should be white.
        tex.render_frame(64.0, 64.0, 1.0, 1.0, 128, 128);

        // Marker center at minimap pixel (128, 96).
        let idx = (96 * MINIMAP_W + 128) * 4;
        assert_eq!(
            &tex.frame[idx..idx + 4],
            &[255, 255, 255, 255],
            "Viewport marker center should be white"
        );
    }

    #[test]
    fn viewport_indicator_grows_with_zoom() {
        let mut tiles = TileMap::new(256, 256);
        for y in 0..256 {
            for x in 0..256 {
                tiles.set_quartier_id(x, y, 1);
            }
        }
        let mut tex = MinimapTexture::generate(&tiles);

        // Large viewport (half the map) → hollow rect.
        tex.render_frame(128.0, 128.0, 128.0, 128.0, 256, 256);

        // vp_w = ceil(128 * 256 / 256) = 128, vp_h = ceil(128 * 192 / 256) = 96.
        // Center pixel (128, 96) should be transparent (hollow interior).
        let idx = (96 * MINIMAP_W + 128) * 4;
        // Interior is base texture — not white.
        assert_ne!(
            &tex.frame[idx..idx + 4],
            &[255u8, 255, 255, 255],
            "Hollow rect interior should not be white"
        );

        // Top-left of the rect should be white (border).
        // left = 128 - 128/2 = 64, top = 96 - 96/2 = 48.
        let edge_idx = (48 * MINIMAP_W + 64) * 4;
        assert_eq!(
            &tex.frame[edge_idx..edge_idx + 4],
            &[255, 255, 255, 255],
            "Viewport rect edge should be white"
        );
    }
}
