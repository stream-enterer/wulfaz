use super::WidgetId;
use super::node::ZTier;
use super::tree::WidgetTree;
use super::widget::Widget;

impl WidgetTree {
    /// Find the topmost widget whose rect contains the point (x, y).
    /// Walks back-to-front through draw order: highest Z-tier first,
    /// last inserted within tier first.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<WidgetId> {
        let draw_order = self.roots_draw_order();
        for &root in draw_order.iter().rev() {
            if let Some(hit) = self.hit_test_node(root, x, y) {
                return Some(hit);
            }
        }
        None
    }

    fn hit_test_node(&self, id: WidgetId, x: f32, y: f32) -> Option<WidgetId> {
        let node = self.arena.get(id)?;
        if !node.rect.contains(x, y) {
            return None;
        }
        // Respect clip rect — widgets scrolled off-screen are invisible and not clickable.
        if let Some(clip) = &node.clip_rect
            && !clip.contains(x, y)
        {
            return None;
        }
        // Children drawn on top — check last child first.
        for &child in node.children.iter().rev() {
            if let Some(hit) = self.hit_test_node(child, x, y) {
                return Some(hit);
            }
        }
        Some(id)
    }

    /// Collect all focusable widgets in tree order (depth-first).
    /// Currently only Buttons are focusable.
    pub fn focusable_widgets(&self) -> Vec<WidgetId> {
        let mut result = Vec::new();
        for root in self.roots_draw_order() {
            self.collect_focusable(root, &mut result);
        }
        result
    }

    /// Collect focusable widgets only from roots at or above `min_tier`.
    /// Used for modal focus scoping — when a modal is open, Tab only cycles
    /// through widgets in the modal layer and above.
    pub fn focusable_widgets_in_tier(&self, min_tier: ZTier) -> Vec<WidgetId> {
        let mut result = Vec::new();
        // Sort roots by tier (same order as roots_draw_order).
        let mut sorted = self.roots.clone();
        sorted.sort_by_key(|(_, tier)| *tier);
        for (root, tier) in sorted {
            if tier >= min_tier {
                self.collect_focusable(root, &mut result);
            }
        }
        result
    }

    fn collect_focusable(&self, id: WidgetId, out: &mut Vec<WidgetId>) {
        if let Some(node) = self.arena.get(id) {
            if matches!(
                node.widget,
                Widget::Button { .. } | Widget::ScrollList { .. } | Widget::ScrollView { .. }
            ) {
                out.push(id);
            }
            for &child in &node.children {
                self.collect_focusable(child, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{FontFamily, HeuristicMeasurer, Position, Size, Sizing};
    use super::*;

    #[test]
    fn z_tier_hit_test_highest_tier_wins() {
        let mut tree = WidgetTree::new();
        // Panel covering the full area.
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.2; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(panel, Sizing::Fixed(200.0), Sizing::Fixed(200.0));

        // Overlay covering part of the same area.
        let overlay = tree.insert_root_with_tier(
            Widget::Panel {
                bg_color: [0.5; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
            ZTier::Overlay,
        );
        tree.set_position(overlay, Position::Fixed { x: 10.0, y: 10.0 });
        tree.set_sizing(overlay, Sizing::Fixed(50.0), Sizing::Fixed(50.0));

        tree.layout(
            Size {
                width: 400.0,
                height: 400.0,
            },
            &mut HeuristicMeasurer,
        );

        // Hit at overlay position -> overlay wins (higher tier).
        assert_eq!(tree.hit_test(20.0, 20.0), Some(overlay));
        // Hit outside overlay but inside panel -> panel wins.
        assert_eq!(tree.hit_test(100.0, 100.0), Some(panel));
    }

    #[test]
    fn focusable_widgets_in_tier_filters_by_tier() {
        let mut tree = WidgetTree::new();

        // Panel-tier button.
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        let panel_btn = tree.insert(
            panel,
            Widget::Button {
                text: "Panel".into(),
                color: [1.0; 4],
                bg_color: [0.3; 4],
                border_color: [0.8; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
            },
        );

        // Modal-tier button.
        let modal = tree.insert_root_with_tier(
            Widget::Panel {
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
            ZTier::Modal,
        );
        let modal_btn = tree.insert(
            modal,
            Widget::Button {
                text: "Modal".into(),
                color: [1.0; 4],
                bg_color: [0.3; 4],
                border_color: [0.8; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
            },
        );

        // All focusable (min_tier = Panel) should include both.
        let all = tree.focusable_widgets_in_tier(ZTier::Panel);
        assert!(all.contains(&panel_btn));
        assert!(all.contains(&modal_btn));

        // Modal-scoped should only include the modal button.
        let modal_only = tree.focusable_widgets_in_tier(ZTier::Modal);
        assert!(!modal_only.contains(&panel_btn));
        assert!(modal_only.contains(&modal_btn));
    }
}
