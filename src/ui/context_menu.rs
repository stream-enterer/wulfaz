use super::draw::FontFamily;
use super::theme::Theme;
use super::widget::Widget;
use super::{Edges, Position, Size, Sizing, WidgetId, WidgetTree, ZTier};

/// A single context menu item.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: String,
    pub enabled: bool,
}

/// Context menu spawned on right-click (UI-303).
///
/// Manages a single context menu overlay. The menu is a Column of clickable
/// labels inserted at `ZTier::Overlay` so it draws above panels but below
/// modals. Only one context menu can be open at a time.
pub struct ContextMenu {
    /// Root widget id of the open menu, if any.
    root: Option<WidgetId>,
}

impl ContextMenu {
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Whether a context menu is currently open.
    pub fn is_open(&self) -> bool {
        self.root.is_some()
    }

    /// The root widget id, if open.
    pub fn root_id(&self) -> Option<WidgetId> {
        self.root
    }

    /// Open a context menu at the given screen position.
    /// Any existing menu is dismissed first.
    /// Items are rendered as a Column of labels with on_click callbacks.
    pub fn open(
        &mut self,
        tree: &mut WidgetTree,
        theme: &Theme,
        x: f32,
        y: f32,
        screen: Size,
        items: &[MenuItem],
    ) {
        let screen_w = screen.width;
        let screen_h = screen.height;
        // Close existing menu if any.
        self.dismiss(tree);

        let font_size = theme.font_body_size;
        let item_h = font_size + 8.0; // text + vertical padding
        let char_w = font_size * 0.6;
        let pad = 8.0;

        // Compute menu width from widest label.
        let max_label_len = items.iter().map(|i| i.label.len()).max().unwrap_or(0);
        let menu_w = max_label_len as f32 * char_w + pad * 2.0;
        let menu_h = items.len() as f32 * item_h + pad * 2.0;

        // Clamp position to screen bounds.
        let cx = if x + menu_w > screen_w {
            (screen_w - menu_w).max(0.0)
        } else {
            x
        };
        let cy = if y + menu_h > screen_h {
            (screen_h - menu_h).max(0.0)
        } else {
            y
        };

        // Create menu root panel at Overlay tier.
        let root = tree.insert_root_with_tier(
            Widget::Panel {
                bg_color: theme.bg_parchment,
                border_color: theme.panel_border_color,
                border_width: 1.0,
                shadow_width: 0.0,
            },
            ZTier::Overlay,
        );
        tree.set_position(root, Position::Fixed { x: cx, y: cy });
        tree.set_sizing(root, Sizing::Fixed(menu_w), Sizing::Fit);
        tree.set_padding(root, Edges::all(pad));

        // Add menu items as clickable labels inside a Column layout.
        let col = tree.insert(
            root,
            Widget::Column {
                gap: 0.0,
                align: super::widget::CrossAlign::Start,
            },
        );
        tree.set_sizing(col, Sizing::Percent(1.0), Sizing::Fit);

        for item in items {
            let color = if item.enabled {
                theme.text_medium
            } else {
                theme.disabled
            };
            let btn = tree.insert(
                col,
                Widget::Label {
                    text: item.label.clone(),
                    color,
                    font_size,
                    font_family: FontFamily::default(),
                    wrap: false,
                },
            );
            tree.set_sizing(btn, Sizing::Percent(1.0), Sizing::Fixed(item_h));
            tree.set_padding(
                btn,
                Edges {
                    top: 2.0,
                    right: 0.0,
                    bottom: 2.0,
                    left: 0.0,
                },
            );
            if item.enabled {
                tree.set_on_click(btn, super::UiAction::ContextAction(item.action.clone()));
            }
        }

        self.root = Some(root);
    }

    /// Dismiss the context menu, removing it from the tree.
    pub fn dismiss(&mut self, tree: &mut WidgetTree) {
        if let Some(root) = self.root.take() {
            tree.remove(root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{HeuristicMeasurer, Size, WidgetTree};

    fn default_theme() -> Theme {
        Theme::default()
    }

    fn sample_items() -> Vec<MenuItem> {
        vec![
            MenuItem {
                label: "Attack".into(),
                action: "ctx::attack".into(),
                enabled: true,
            },
            MenuItem {
                label: "Follow".into(),
                action: "ctx::follow".into(),
                enabled: true,
            },
            MenuItem {
                label: "Recruit".into(),
                action: "ctx::recruit".into(),
                enabled: false,
            },
        ]
    }

    #[test]
    fn open_creates_menu_root() {
        let mut tree = WidgetTree::new();
        let mut ctx = ContextMenu::new();
        let theme = default_theme();

        assert!(!ctx.is_open());
        ctx.open(
            &mut tree,
            &theme,
            100.0,
            100.0,
            Size {
                width: 800.0,
                height: 600.0,
            },
            &sample_items(),
        );
        assert!(ctx.is_open());
        assert!(ctx.root_id().is_some());

        // Root is at Overlay tier.
        let root = ctx.root_id().unwrap();
        assert_eq!(tree.z_tier(root), Some(ZTier::Overlay));
    }

    #[test]
    fn dismiss_removes_menu() {
        let mut tree = WidgetTree::new();
        let mut ctx = ContextMenu::new();
        let theme = default_theme();

        ctx.open(
            &mut tree,
            &theme,
            100.0,
            100.0,
            Size {
                width: 800.0,
                height: 600.0,
            },
            &sample_items(),
        );
        let root = ctx.root_id().unwrap();
        ctx.dismiss(&mut tree);

        assert!(!ctx.is_open());
        assert!(tree.get(root).is_none(), "menu root removed from tree");
    }

    #[test]
    fn open_replaces_existing_menu() {
        let mut tree = WidgetTree::new();
        let mut ctx = ContextMenu::new();
        let theme = default_theme();

        ctx.open(
            &mut tree,
            &theme,
            50.0,
            50.0,
            Size {
                width: 800.0,
                height: 600.0,
            },
            &sample_items(),
        );
        let first_root = ctx.root_id().unwrap();

        ctx.open(
            &mut tree,
            &theme,
            200.0,
            200.0,
            Size {
                width: 800.0,
                height: 600.0,
            },
            &sample_items(),
        );
        let second_root = ctx.root_id().unwrap();

        assert_ne!(first_root, second_root);
        assert!(tree.get(first_root).is_none(), "first menu removed");
        assert!(tree.get(second_root).is_some(), "second menu exists");
    }

    #[test]
    fn enabled_items_have_on_click() {
        let mut tree = WidgetTree::new();
        let mut ctx = ContextMenu::new();
        let theme = default_theme();

        ctx.open(
            &mut tree,
            &theme,
            100.0,
            100.0,
            Size {
                width: 800.0,
                height: 600.0,
            },
            &sample_items(),
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        // Walk the tree to find labels with on_click.
        let root = ctx.root_id().unwrap();
        let root_node = tree.get(root).unwrap();
        let col_id = root_node.children[0];
        let col_node = tree.get(col_id).unwrap();

        // 3 items: Attack (enabled), Follow (enabled), Recruit (disabled).
        assert_eq!(col_node.children.len(), 3);

        let attack = tree.get(col_node.children[0]).unwrap();
        assert!(attack.on_click.is_some(), "enabled item has on_click");
        assert!(
            matches!(&attack.on_click, Some(crate::ui::UiAction::ContextAction(s)) if s == "ctx::attack")
        );

        let recruit = tree.get(col_node.children[2]).unwrap();
        assert!(recruit.on_click.is_none(), "disabled item has no on_click");
    }

    #[test]
    fn menu_clamped_to_screen_bounds() {
        let mut tree = WidgetTree::new();
        let mut ctx = ContextMenu::new();
        let theme = default_theme();

        // Open near bottom-right corner.
        ctx.open(
            &mut tree,
            &theme,
            790.0,
            590.0,
            Size {
                width: 800.0,
                height: 600.0,
            },
            &sample_items(),
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let root = ctx.root_id().unwrap();
        let node = tree.get(root).unwrap();
        // Should be clamped so it doesn't extend past the screen.
        assert!(
            node.rect.x + node.rect.width <= 800.0 + 1.0,
            "menu x clamped: {} + {} <= 800",
            node.rect.x,
            node.rect.width
        );
    }
}
