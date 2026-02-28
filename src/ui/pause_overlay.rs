use super::WidgetId;
use super::geometry::{Position, Sizing};
use super::tree::WidgetTree;
use super::widget::Widget;

/// Build a fullscreen semi-transparent overlay when the simulation is paused (UI-105).
/// Inserted as the last root so it draws on top of the tile map but below UI panels
/// that are added after it. Returns the overlay's WidgetId.
pub fn build_pause_overlay(tree: &mut WidgetTree, screen_w: f32, screen_h: f32) -> WidgetId {
    let overlay = tree.insert_root(Widget::Panel {
        bg_color: [0.0, 0.0, 0.0, 0.15],
        border_color: [0.0; 4],
        border_width: 0.0,
        shadow_width: 0.0,
    });
    tree.set_position(overlay, Position::Fixed { x: 0.0, y: 0.0 });
    tree.set_sizing(overlay, Sizing::Fixed(screen_w), Sizing::Fixed(screen_h));
    overlay
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{DrawList, HeuristicMeasurer, Size};

    #[test]
    fn pause_overlay_emits_fullscreen_panel() {
        let mut tree = WidgetTree::new();
        let overlay = build_pause_overlay(&mut tree, 800.0, 600.0);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(overlay).unwrap();
        assert!((rect.width - 800.0).abs() < 0.1);
        assert!((rect.height - 600.0).abs() < 0.1);

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        assert_eq!(draw_list.panels.len(), 1);
        let p = &draw_list.panels[0];
        assert!(
            (p.bg_color[3] - 0.15).abs() < 0.01,
            "overlay alpha should be 0.15"
        );
    }
}
