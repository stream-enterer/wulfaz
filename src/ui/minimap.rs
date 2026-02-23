//! Mini-map panel (UI-407).
//!
//! Small overview panel in the bottom-right showing a downscaled tile map
//! with viewport indicator. Click-to-navigate teleports the camera.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{Edges, FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

/// Default minimap display size in pixels.
const MINIMAP_DISPLAY_W: f32 = 128.0;
const MINIMAP_DISPLAY_H: f32 = 96.0;

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
/// Returns the panel root ID. The minimap texture itself is rendered
/// separately via the sprite pipeline; this builds the UI frame.
pub fn build_minimap(tree: &mut WidgetTree, theme: &Theme, info: &MinimapInfo) -> WidgetId {
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
    tree.set_on_click(map_area, "minimap::click");

    // Viewport indicator info as tooltip
    let vp_info = format!(
        "Camera: ({:.0}, {:.0})\nView: {:.0}x{:.0}",
        info.camera_x, info.camera_y, info.viewport_w, info.viewport_h
    );
    tree.set_tooltip(map_area, Some(super::widget::TooltipContent::Text(vp_info)));

    panel
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
        let root = build_minimap(&mut tree, &theme, &info);
        assert!(tree.get(root).is_some());
    }

    #[test]
    fn click_to_world_center() {
        // Click at minimap center should map to world center
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
}
