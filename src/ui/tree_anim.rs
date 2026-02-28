use super::WidgetId;
use super::tree::WidgetTree;
use super::widget::Widget;

impl WidgetTree {
    // ------------------------------------------------------------------
    // Animation helpers (UI-W05)
    // ------------------------------------------------------------------

    /// Multiply all color alpha channels in a subtree by `opacity`.
    /// Used for fade-in/fade-out animations on tooltip and panel roots.
    pub fn set_subtree_opacity(&mut self, root: WidgetId, opacity: f32) {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if let Some(node) = self.arena.get_mut(id) {
                Self::apply_opacity(&mut node.widget, opacity);
                stack.extend(node.children.iter().copied());
            }
        }
    }

    /// Apply opacity multiplier to all color fields on a single widget.
    fn apply_opacity(widget: &mut Widget, opacity: f32) {
        match widget {
            // Row, Column, and Expand have no colors to fade.
            Widget::Row { .. } | Widget::Column { .. } | Widget::Expand => {}
            Widget::Panel {
                bg_color,
                border_color,
                ..
            } => {
                bg_color[3] *= opacity;
                border_color[3] *= opacity;
            }
            Widget::Label { color, .. } => {
                color[3] *= opacity;
            }
            Widget::Button {
                color,
                bg_color,
                border_color,
                ..
            } => {
                color[3] *= opacity;
                bg_color[3] *= opacity;
                border_color[3] *= opacity;
            }
            Widget::RichText { spans, .. } => {
                for span in spans {
                    span.color[3] *= opacity;
                }
            }
            Widget::ScrollList {
                bg_color,
                border_color,
                scrollbar_color,
                ..
            } => {
                bg_color[3] *= opacity;
                border_color[3] *= opacity;
                scrollbar_color[3] *= opacity;
            }
            Widget::ProgressBar {
                fg_color,
                bg_color,
                border_color,
                ..
            } => {
                fg_color[3] *= opacity;
                bg_color[3] *= opacity;
                border_color[3] *= opacity;
            }
            Widget::Separator { color, .. } => {
                color[3] *= opacity;
            }
            Widget::Icon { tint, .. } => {
                if let Some(t) = tint {
                    t[3] *= opacity;
                }
            }
            Widget::Checkbox { color, .. } => {
                color[3] *= opacity;
            }
            Widget::Dropdown {
                color, bg_color, ..
            } => {
                color[3] *= opacity;
                bg_color[3] *= opacity;
            }
            Widget::Slider {
                track_color,
                thumb_color,
                ..
            } => {
                track_color[3] *= opacity;
                thumb_color[3] *= opacity;
            }
            Widget::TextInput {
                color, bg_color, ..
            } => {
                color[3] *= opacity;
                bg_color[3] *= opacity;
            }
            Widget::Collapsible { color, .. } => {
                color[3] *= opacity;
            }
            Widget::TabContainer {
                tab_color,
                active_color,
                ..
            } => {
                tab_color[3] *= opacity;
                active_color[3] *= opacity;
            }
            Widget::ScrollView {
                scrollbar_color, ..
            } => {
                scrollbar_color[3] *= opacity;
            }
        }
    }

    /// Set the background color alpha of a specific widget.
    /// Used for button hover highlight animations.
    pub fn set_widget_bg_alpha(&mut self, id: WidgetId, alpha: f32) {
        if let Some(node) = self.arena.get_mut(id) {
            match &mut node.widget {
                Widget::Button { bg_color, .. } => {
                    bg_color[3] = alpha;
                }
                Widget::Panel { bg_color, .. } => {
                    bg_color[3] = alpha;
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::tree::WidgetTree;
    use crate::ui::widget::{CrossAlign, Widget};
    use crate::ui::{FontFamily, Sizing};

    #[test]
    fn set_subtree_opacity_scales_all_colors() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [1.0, 1.0, 1.0, 0.8],
            border_color: [1.0, 1.0, 1.0, 1.0],
            border_width: 2.0,
            shadow_width: 0.0,
        });
        let label = tree.insert(
            panel,
            Widget::Label {
                text: "Test".into(),
                color: [1.0, 1.0, 1.0, 1.0],
                font_size: 12.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.set_subtree_opacity(panel, 0.5);

        // Panel colors should be halved.
        let p = tree.get(panel).unwrap();
        if let Widget::Panel {
            bg_color,
            border_color,
            ..
        } = &p.widget
        {
            assert!((bg_color[3] - 0.4).abs() < 1e-6); // 0.8 * 0.5
            assert!((border_color[3] - 0.5).abs() < 1e-6); // 1.0 * 0.5
        }

        // Child label color should also be halved.
        let l = tree.get(label).unwrap();
        if let Widget::Label { color, .. } = &l.widget {
            assert!((color[3] - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn set_widget_bg_alpha() {
        let mut tree = WidgetTree::new();
        let btn = tree.insert_root(Widget::Button {
            text: "X".into(),
            color: [1.0; 4],
            bg_color: [0.0, 0.0, 0.0, 0.0],
            border_color: [1.0; 4],
            font_size: 12.0,
            font_family: FontFamily::default(),
        });

        tree.set_widget_bg_alpha(btn, 0.3);

        let node = tree.get(btn).unwrap();
        if let Widget::Button { bg_color, .. } = &node.widget {
            assert!((bg_color[3] - 0.3).abs() < 1e-6);
        }
    }

    #[test]
    fn dropdown_apply_opacity() {
        let mut tree = WidgetTree::new();
        let root = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(root, Sizing::Fixed(200.0), Sizing::Fixed(200.0));
        let dd = tree.insert(
            root,
            Widget::Dropdown {
                selected: 0,
                options: vec!["X".into()],
                open: false,
                color: [1.0, 1.0, 1.0, 1.0],
                bg_color: [0.5, 0.5, 0.5, 1.0],
                font_size: 14.0,
            },
        );
        tree.set_subtree_opacity(root, 0.5);
        let node = tree.get(dd).unwrap();
        match &node.widget {
            Widget::Dropdown {
                color, bg_color, ..
            } => {
                assert!((color[3] - 0.5).abs() < 0.01, "text alpha should be 0.5");
                assert!((bg_color[3] - 0.5).abs() < 0.01, "bg alpha should be 0.5");
            }
            _ => panic!("expected Dropdown"),
        }
    }

    #[test]
    fn tab_container_apply_opacity() {
        let mut tree = WidgetTree::new();
        let tc = tree.insert_root(Widget::TabContainer {
            tabs: vec!["X".into()],
            active: 0,
            tab_color: [0.5, 0.5, 0.5, 1.0],
            active_color: [0.8, 0.8, 0.8, 1.0],
            font_size: 14.0,
        });
        tree.set_subtree_opacity(tc, 0.5);
        let node = tree.get(tc).unwrap();
        if let Widget::TabContainer {
            tab_color,
            active_color,
            ..
        } = &node.widget
        {
            assert!((tab_color[3] - 0.5).abs() < 0.01);
            assert!((active_color[3] - 0.5).abs() < 0.01);
        } else {
            panic!("expected TabContainer");
        }
    }
}
