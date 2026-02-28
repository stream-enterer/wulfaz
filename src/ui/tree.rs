use slotmap::SlotMap;

use super::WidgetId;
use super::action::UiAction;
use super::geometry::{Constraints, Edges, Position, Rect, Size, Sizing};
use super::node::{WidgetNode, ZTier};
use super::widget::{self, Widget};

/// Arena-backed retained widget tree.
pub struct WidgetTree {
    pub(crate) arena: SlotMap<WidgetId, WidgetNode>,
    pub(crate) roots: Vec<(WidgetId, ZTier)>,
    /// Alpha for alternating ScrollList row tint (from Theme).
    pub(crate) scroll_row_alt_alpha: f32,
    /// Border width for control widgets (buttons, checkboxes, etc.).
    /// Set from `Theme::control_border()`.
    pub(crate) control_border_width: f32,
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            arena: SlotMap::with_key(),
            roots: Vec::new(),
            scroll_row_alt_alpha: 0.04,
            control_border_width: 1.0,
        }
    }

    /// Set the alternating row tint alpha from the Theme.
    pub fn set_scroll_row_alt_alpha(&mut self, alpha: f32) {
        self.scroll_row_alt_alpha = alpha;
    }

    /// Set the control border width from the Theme.
    pub fn set_control_border_width(&mut self, width: f32) {
        self.control_border_width = width;
    }

    /// Insert a widget as a root (no parent) at the default Panel tier.
    pub fn insert_root(&mut self, widget: Widget) -> WidgetId {
        self.insert_root_with_tier(widget, ZTier::Panel)
    }

    /// Number of widgets currently in the arena (UI-505).
    pub fn widget_count(&self) -> usize {
        self.arena.len()
    }

    /// Insert a widget as a root at the specified Z-tier (UI-307).
    pub fn insert_root_with_tier(&mut self, widget: Widget, z_tier: ZTier) -> WidgetId {
        let padding = Self::default_padding(&widget);
        let id = self.arena.insert(WidgetNode {
            widget,
            parent: None,
            children: Vec::new(),
            position: Position::default(),
            width: Sizing::default(),
            height: Sizing::default(),
            padding,
            margin: Edges::ZERO,
            rect: Rect::default(),
            measured: Size::default(),
            tooltip: None,
            constraints: None,
            clip_rect: None,
            on_click: None,
        });
        self.roots.push((id, z_tier));
        id
    }

    /// Insert a widget as a child of `parent`. Returns the new widget's id.
    pub fn insert(&mut self, parent: WidgetId, widget: Widget) -> WidgetId {
        let padding = Self::default_padding(&widget);
        let id = self.arena.insert(WidgetNode {
            widget,
            parent: Some(parent),
            children: Vec::new(),
            position: Position::default(),
            width: Sizing::default(),
            height: Sizing::default(),
            padding,
            margin: Edges::ZERO,
            rect: Rect::default(),
            measured: Size::default(),
            tooltip: None,
            constraints: None,
            clip_rect: None,
            on_click: None,
        });
        if let Some(parent_node) = self.arena.get_mut(parent) {
            parent_node.children.push(id);
        }
        id
    }

    /// Default padding for widget types that have intrinsic internal spacing.
    /// Callers can override with `set_padding()`.
    fn default_padding(widget: &Widget) -> Edges {
        match widget {
            Widget::Button { .. } => Edges {
                top: 4.0,
                right: 8.0,
                bottom: 4.0,
                left: 8.0,
            },
            Widget::Dropdown { .. } => Edges {
                top: 4.0,
                right: 8.0,
                bottom: 4.0,
                left: 8.0,
            },
            Widget::TextInput { .. } => Edges {
                top: 4.0,
                right: 4.0,
                bottom: 4.0,
                left: 4.0,
            },
            _ => Edges::ZERO,
        }
    }

    /// Remove a widget and all its descendants.
    pub fn remove(&mut self, id: WidgetId) {
        // Collect descendants depth-first.
        let mut to_remove = Vec::new();
        Self::collect_subtree(&self.arena, id, &mut to_remove);

        // Unlink from parent.
        if let Some(node) = self.arena.get(id)
            && let Some(parent_id) = node.parent
            && let Some(parent) = self.arena.get_mut(parent_id)
        {
            parent.children.retain(|c| *c != id);
        }

        // Remove from roots if present.
        self.roots.retain(|(r, _)| *r != id);

        // Remove all nodes.
        for rid in to_remove {
            self.arena.remove(rid);
        }
    }

    pub(crate) fn collect_subtree(
        arena: &SlotMap<WidgetId, WidgetNode>,
        id: WidgetId,
        out: &mut Vec<WidgetId>,
    ) {
        out.push(id);
        if let Some(node) = arena.get(id) {
            for &child in &node.children {
                Self::collect_subtree(arena, child, out);
            }
        }
    }

    /// Check whether a widget ID exists in the tree.
    pub fn contains(&self, id: WidgetId) -> bool {
        self.arena.contains_key(id)
    }

    /// Get a reference to a widget node.
    pub(crate) fn get(&self, id: WidgetId) -> Option<&WidgetNode> {
        self.arena.get(id)
    }

    /// Get a mutable reference to a widget node.
    pub(crate) fn get_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode> {
        self.arena.get_mut(id)
    }

    /// Get the computed layout rect for a widget (set by the layout pass).
    pub fn node_rect(&self, id: WidgetId) -> Option<Rect> {
        self.arena.get(id).map(|n| n.rect)
    }

    /// Set position mode for a widget.
    pub fn set_position(&mut self, id: WidgetId, pos: Position) {
        if let Some(node) = self.arena.get_mut(id) {
            node.position = pos;
        }
    }

    /// Set sizing for a widget.
    pub fn set_sizing(&mut self, id: WidgetId, w: Sizing, h: Sizing) {
        if let Some(node) = self.arena.get_mut(id) {
            node.width = w;
            node.height = h;
        }
    }

    /// Set padding for a widget.
    pub fn set_padding(&mut self, id: WidgetId, padding: Edges) {
        if let Some(node) = self.arena.get_mut(id) {
            node.padding = padding;
        }
    }

    /// Set margin for a widget.
    pub fn set_margin(&mut self, id: WidgetId, margin: Edges) {
        if let Some(node) = self.arena.get_mut(id) {
            node.margin = margin;
        }
    }

    /// Set tooltip content for a widget.
    pub fn set_tooltip(&mut self, id: WidgetId, content: Option<widget::TooltipContent>) {
        if let Some(node) = self.arena.get_mut(id) {
            node.tooltip = content;
        }
    }

    /// Set min/max size constraints on a widget (UI-103).
    pub fn set_constraints(&mut self, id: WidgetId, constraints: Constraints) {
        if let Some(node) = self.arena.get_mut(id) {
            node.constraints = Some(constraints);
        }
    }

    /// Set a click callback action on a widget (UI-305).
    /// When this widget is clicked, `UiState::poll_click()` returns
    /// `Some((widget_id, action))`.
    pub fn set_on_click(&mut self, id: WidgetId, action: UiAction) {
        if let Some(node) = self.arena.get_mut(id) {
            node.on_click = Some(action);
        }
    }

    /// Set a scissor-rect clip region on a widget (UI-104).
    /// Children inherit the clip region during layout.
    pub fn set_clip_rect(&mut self, id: WidgetId, clip: Option<Rect>) {
        if let Some(node) = self.arena.get_mut(id) {
            node.clip_rect = clip;
        }
    }

    /// Root widget ids in draw order (sorted by Z-tier, stable within tier).
    pub fn roots(&self) -> Vec<WidgetId> {
        self.roots_draw_order()
    }

    /// Root ids in draw order: Panel -> Overlay -> Modal -> Tooltip.
    /// Within each tier, insertion order is preserved (stable sort).
    pub(crate) fn roots_draw_order(&self) -> Vec<WidgetId> {
        let mut sorted = self.roots.clone();
        sorted.sort_by_key(|(_, tier)| *tier);
        sorted.iter().map(|(id, _)| *id).collect()
    }

    /// Get the Z-tier of a root widget. Returns None if not a root.
    pub fn z_tier(&self, id: WidgetId) -> Option<ZTier> {
        self.roots.iter().find(|(r, _)| *r == id).map(|(_, t)| *t)
    }

    /// Get the Z-tier of the root that contains `id`.
    /// Walks up the parent chain to find the root, then returns its tier.
    pub fn z_tier_of_widget(&self, id: WidgetId) -> Option<ZTier> {
        // Walk to the root of this widget's subtree.
        let mut current = id;
        loop {
            let node = self.arena.get(current)?;
            match node.parent {
                Some(parent) => current = parent,
                None => return self.z_tier(current),
            }
        }
    }

    /// Change the Z-tier of an existing root widget (UI-307).
    pub fn set_z_tier(&mut self, id: WidgetId, tier: ZTier) {
        if let Some(entry) = self.roots.iter_mut().find(|(r, _)| *r == id) {
            entry.1 = tier;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::FontFamily;

    #[test]
    fn insert_root_and_child() {
        let mut tree = WidgetTree::new();
        let root = tree.insert_root(Widget::Panel {
            bg_color: [0.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        assert_eq!(tree.roots().len(), 1);

        let child = tree.insert(
            root,
            Widget::Label {
                text: "Hello".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let root_node = tree.get(root).expect("root exists");
        assert_eq!(root_node.children.len(), 1);
        assert_eq!(root_node.children[0], child);

        let child_node = tree.get(child).expect("child exists");
        assert_eq!(child_node.parent, Some(root));
    }

    #[test]
    fn remove_subtree() {
        let mut tree = WidgetTree::new();
        let root = tree.insert_root(Widget::Panel {
            bg_color: [0.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        let child = tree.insert(
            root,
            Widget::Label {
                text: "A".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let grandchild = tree.insert(
            child,
            Widget::Label {
                text: "B".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.remove(child);

        // Child and grandchild gone.
        assert!(tree.get(child).is_none());
        assert!(tree.get(grandchild).is_none());
        // Root still exists, no children.
        let root_node = tree.get(root).expect("root exists");
        assert!(root_node.children.is_empty());
    }

    #[test]
    fn z_tier_draw_order_panels_before_modals() {
        let mut tree = WidgetTree::new();
        // Insert a panel (default tier).
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.2, 0.2, 0.2, 1.0],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(panel, Sizing::Fixed(100.0), Sizing::Fixed(50.0));

        // Insert a modal (higher tier).
        let modal = tree.insert_root_with_tier(
            Widget::Panel {
                bg_color: [1.0, 0.0, 0.0, 1.0],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
            ZTier::Modal,
        );
        tree.set_position(modal, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(modal, Sizing::Fixed(80.0), Sizing::Fixed(40.0));

        let draw_order = tree.roots();
        assert_eq!(draw_order.len(), 2);
        assert_eq!(draw_order[0], panel, "panel drawn first (behind)");
        assert_eq!(draw_order[1], modal, "modal drawn last (on top)");
    }

    #[test]
    fn z_tier_modal_stays_above_raised_panel() {
        let mut tree = WidgetTree::new();
        // Panel A.
        let panel_a = tree.insert_root(Widget::Panel {
            bg_color: [0.1; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        // Modal.
        let modal = tree.insert_root_with_tier(
            Widget::Panel {
                bg_color: [1.0, 0.0, 0.0, 1.0],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
            ZTier::Modal,
        );
        // Panel B (inserted after modal — simulates "raising" by removing + re-inserting).
        let panel_b = tree.insert_root(Widget::Panel {
            bg_color: [0.3; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });

        let draw_order = tree.roots();
        assert_eq!(draw_order.len(), 3);
        // Both panels come before the modal regardless of insertion order.
        assert_eq!(draw_order[0], panel_a);
        assert_eq!(draw_order[1], panel_b);
        assert_eq!(draw_order[2], modal, "modal always draws last");
    }

    #[test]
    fn z_tier_set_z_tier_changes_draw_order() {
        let mut tree = WidgetTree::new();
        let a = tree.insert_root(Widget::Panel {
            bg_color: [0.1; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        let b = tree.insert_root(Widget::Panel {
            bg_color: [0.2; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });

        // Both at Panel tier — insertion order.
        let order = tree.roots();
        assert_eq!(order[0], a);
        assert_eq!(order[1], b);

        // Promote a to Overlay.
        tree.set_z_tier(a, ZTier::Overlay);
        let order = tree.roots();
        assert_eq!(order[0], b, "b stays at Panel tier");
        assert_eq!(order[1], a, "a promoted to Overlay tier");
    }

    #[test]
    fn z_tier_of_widget_walks_to_root() {
        let mut tree = WidgetTree::new();

        let modal_root = tree.insert_root_with_tier(
            Widget::Panel {
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
            ZTier::Modal,
        );
        let child = tree.insert(
            modal_root,
            Widget::Label {
                text: "Hello".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        assert_eq!(tree.z_tier_of_widget(modal_root), Some(ZTier::Modal));
        assert_eq!(tree.z_tier_of_widget(child), Some(ZTier::Modal));
    }
}
