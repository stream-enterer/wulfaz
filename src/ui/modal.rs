use super::{Position, Sizing, Widget, WidgetId, WidgetTree, ZTier};

/// Modal dialog stack (UI-300).
///
/// Manages a stack of modal dialogs. Each modal is a root widget inserted
/// at `ZTier::Modal`, with a fullscreen dim layer behind it that blocks
/// clicks to widgets underneath.
///
/// The dim layer is a transparent Panel root at `ZTier::Modal` that covers
/// the entire screen. It is inserted just before the modal's content root.
pub struct ModalStack {
    /// Stack of (dim_layer_id, content_root_id) pairs.
    /// Last entry is the topmost modal.
    modals: Vec<(WidgetId, WidgetId)>,
}

impl ModalStack {
    pub fn new() -> Self {
        Self { modals: Vec::new() }
    }

    /// Push a modal onto the stack.
    ///
    /// `content_root` must already be inserted into `tree` as a root widget.
    /// This method:
    /// 1. Creates a fullscreen dim layer behind the modal.
    /// 2. Promotes the content root to `ZTier::Modal`.
    pub fn push(
        &mut self,
        tree: &mut WidgetTree,
        content_root: WidgetId,
        screen_w: f32,
        screen_h: f32,
    ) {
        // Dim layer: fullscreen semi-transparent panel.
        let dim = tree.insert_root_with_tier(
            Widget::Panel {
                bg_color: [0.0, 0.0, 0.0, 0.4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
            ZTier::Modal,
        );
        tree.set_position(dim, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(dim, Sizing::Fixed(screen_w), Sizing::Fixed(screen_h));

        // Promote the content root to Modal tier.
        tree.set_z_tier(content_root, ZTier::Modal);

        self.modals.push((dim, content_root));
    }

    /// Pop the topmost modal from the stack.
    /// Removes both the dim layer and the content root from the tree.
    /// Returns the content root id if a modal was popped.
    pub fn pop(&mut self, tree: &mut WidgetTree) -> Option<WidgetId> {
        let (dim, content) = self.modals.pop()?;
        tree.remove(dim);
        tree.remove(content);
        Some(content)
    }

    /// Number of modals on the stack.
    pub fn len(&self) -> usize {
        self.modals.len()
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.modals.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{Edges, HeuristicMeasurer, Size};

    fn make_modal_panel(tree: &mut WidgetTree) -> WidgetId {
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.5, 0.5, 0.5, 1.0],
            border_color: [1.0; 4],
            border_width: 2.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 100.0, y: 100.0 });
        tree.set_sizing(panel, Sizing::Fixed(200.0), Sizing::Fixed(150.0));
        tree.set_padding(panel, Edges::all(8.0));
        panel
    }

    #[test]
    fn push_two_pop_one_bottom_persists() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        let modal_a = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal_a, 800.0, 600.0);
        assert_eq!(stack.len(), 1);

        let modal_b = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal_b, 800.0, 600.0);
        assert_eq!(stack.len(), 2);

        // Pop top modal (B).
        let popped = stack.pop(&mut tree);
        assert_eq!(popped, Some(modal_b));
        assert_eq!(stack.len(), 1);

        // Modal A still in tree.
        assert!(tree.get(modal_a).is_some(), "bottom modal persists");
        // Modal B removed.
        assert!(tree.get(modal_b).is_none(), "top modal removed");
    }

    #[test]
    fn modal_roots_at_modal_z_tier() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        // Insert a regular panel first.
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.2; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(panel, Sizing::Fixed(800.0), Sizing::Fixed(600.0));

        let modal = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal, 800.0, 600.0);

        // Draw order: panel first, then dim + modal.
        let roots = tree.roots();
        assert_eq!(roots[0], panel, "panel drawn first");
        // Dim and modal are at Modal tier, after the panel.
        assert!(roots.len() >= 3, "panel + dim + modal");
        assert_eq!(tree.z_tier(roots[0]), Some(ZTier::Panel));
        assert_eq!(tree.z_tier(modal), Some(ZTier::Modal));
    }

    #[test]
    fn dim_layer_blocks_clicks_behind_modal() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        // Background panel covering full screen.
        let bg = tree.insert_root(Widget::Panel {
            bg_color: [0.2; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_position(bg, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(bg, Sizing::Fixed(800.0), Sizing::Fixed(600.0));

        // Modal in the center.
        let modal = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal, 800.0, 600.0);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        // Click on the dim layer area (outside modal but on screen).
        // Should NOT hit the background panel.
        let hit = tree.hit_test(10.0, 10.0);
        assert_ne!(hit, Some(bg), "dim layer blocks clicks to background");
        // Should hit the dim layer (a Modal-tier panel).
        assert!(hit.is_some());
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.pop(&mut tree), None);
    }
}
