use super::{Position, Sizing, Widget, WidgetId, WidgetTree, ZTier};

/// Options for pushing a modal onto the stack.
pub struct ModalOptions {
    /// Callback key fired when the modal is dismissed (ESC, click-outside).
    pub on_dismiss: Option<String>,
    /// Callback key fired when the modal is confirmed (Enter).
    pub on_confirm: Option<String>,
}

impl ModalOptions {
    pub const NONE: Self = Self {
        on_dismiss: None,
        on_confirm: None,
    };
}

/// Result of popping a modal.
pub struct ModalPop {
    /// The content root that was removed.
    pub content: WidgetId,
    /// Dismiss callback from the modal's options.
    pub on_dismiss: Option<String>,
    /// Confirm callback from the modal's options.
    pub on_confirm: Option<String>,
}

struct ModalEntry {
    dim: WidgetId,
    content: WidgetId,
    on_dismiss: Option<String>,
    on_confirm: Option<String>,
}

/// Callback action for dismissing the topmost modal (click-outside-to-dismiss).
pub const MODAL_DISMISS: &str = "modal::dismiss";

/// Modal dialog stack (UI-300).
///
/// Manages a stack of modal dialogs. Each modal is a root widget inserted
/// at `ZTier::Modal`, with a fullscreen dim layer behind it that blocks
/// clicks to widgets underneath. Clicking the dim layer dismisses the
/// topmost modal.
///
/// The dim layer is a transparent Panel root at `ZTier::Modal` that covers
/// the entire screen. It is inserted just before the modal's content root.
pub struct ModalStack {
    /// Stack of modal entries. Last entry is the topmost modal.
    modals: Vec<ModalEntry>,
}

impl ModalStack {
    pub fn new() -> Self {
        Self { modals: Vec::new() }
    }

    /// Push a modal onto the stack.
    ///
    /// `content_root` must already be inserted into `tree` as a root widget.
    /// This method:
    /// 1. Creates a fullscreen dim layer behind the modal (click-outside-to-dismiss).
    /// 2. Promotes the content root to `ZTier::Modal`.
    /// 3. Centers the content root in the screen.
    pub fn push(&mut self, tree: &mut WidgetTree, content_root: WidgetId, opts: ModalOptions) {
        // Dim layer: fullscreen semi-transparent panel with click-to-dismiss.
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
        tree.set_sizing(dim, Sizing::Percent(1.0), Sizing::Percent(1.0));
        tree.set_on_click(dim, MODAL_DISMISS);

        // Promote the content root to Modal tier and center it.
        tree.set_z_tier(content_root, ZTier::Modal);
        tree.set_position(content_root, Position::Center);

        self.modals.push(ModalEntry {
            dim,
            content: content_root,
            on_dismiss: opts.on_dismiss,
            on_confirm: opts.on_confirm,
        });
    }

    /// Pop the topmost modal from the stack.
    /// Removes both the dim layer and the content root from the tree.
    /// Returns the pop result with callbacks for the caller to dispatch.
    pub fn pop(&mut self, tree: &mut WidgetTree) -> Option<ModalPop> {
        let entry = self.modals.pop()?;
        tree.remove(entry.dim);
        tree.remove(entry.content);
        Some(ModalPop {
            content: entry.content,
            on_dismiss: entry.on_dismiss,
            on_confirm: entry.on_confirm,
        })
    }

    /// Get the confirm callback of the topmost modal (for Enter key dispatch).
    pub fn confirm_callback(&self) -> Option<&str> {
        self.modals.last().and_then(|e| e.on_confirm.as_deref())
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
        stack.push(&mut tree, modal_a, ModalOptions::NONE);
        assert_eq!(stack.len(), 1);

        let modal_b = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal_b, ModalOptions::NONE);
        assert_eq!(stack.len(), 2);

        // Pop top modal (B).
        let popped = stack.pop(&mut tree);
        assert_eq!(popped.as_ref().map(|p| p.content), Some(modal_b));
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
        stack.push(&mut tree, modal, ModalOptions::NONE);

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
        stack.push(&mut tree, modal, ModalOptions::NONE);

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
        assert!(stack.pop(&mut tree).is_none());
    }

    #[test]
    fn dim_layer_has_click_callback() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        let modal = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal, ModalOptions::NONE);

        // Find the dim layer (first Modal-tier root that isn't the modal content).
        let roots = tree.roots();
        let dim_id = roots
            .iter()
            .find(|&&r| tree.z_tier(r) == Some(ZTier::Modal) && r != modal)
            .unwrap();

        let dim_node = tree.get(*dim_id).unwrap();
        assert_eq!(
            dim_node.on_click.as_deref(),
            Some(MODAL_DISMISS),
            "dim layer has click-outside-to-dismiss callback"
        );
    }

    #[test]
    fn dim_layer_uses_percent_sizing() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        let modal = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal, ModalOptions::NONE);

        let roots = tree.roots();
        let dim_id = roots
            .iter()
            .find(|&&r| tree.z_tier(r) == Some(ZTier::Modal) && r != modal)
            .unwrap();

        let dim_node = tree.get(*dim_id).unwrap();
        assert!(
            matches!(dim_node.width, Sizing::Percent(f) if (f - 1.0).abs() < 0.001),
            "dim uses Percent width"
        );
        assert!(
            matches!(dim_node.height, Sizing::Percent(f) if (f - 1.0).abs() < 0.001),
            "dim uses Percent height"
        );
    }

    #[test]
    fn content_root_centered() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        let modal = make_modal_panel(&mut tree);
        stack.push(&mut tree, modal, ModalOptions::NONE);

        let node = tree.get(modal).unwrap();
        assert!(
            matches!(node.position, Position::Center),
            "modal content is centered"
        );
    }

    #[test]
    fn pop_returns_callbacks() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        let modal = make_modal_panel(&mut tree);
        stack.push(
            &mut tree,
            modal,
            ModalOptions {
                on_dismiss: Some("event_choice:refuse".into()),
                on_confirm: Some("event_choice:accept".into()),
            },
        );

        let pop = stack.pop(&mut tree).unwrap();
        assert_eq!(pop.on_dismiss.as_deref(), Some("event_choice:refuse"));
        assert_eq!(pop.on_confirm.as_deref(), Some("event_choice:accept"));
    }

    #[test]
    fn confirm_callback_returns_topmost() {
        let mut tree = WidgetTree::new();
        let mut stack = ModalStack::new();

        assert!(stack.confirm_callback().is_none());

        let modal_a = make_modal_panel(&mut tree);
        stack.push(
            &mut tree,
            modal_a,
            ModalOptions {
                on_dismiss: None,
                on_confirm: Some("first".into()),
            },
        );
        assert_eq!(stack.confirm_callback(), Some("first"));

        let modal_b = make_modal_panel(&mut tree);
        stack.push(
            &mut tree,
            modal_b,
            ModalOptions {
                on_dismiss: None,
                on_confirm: Some("second".into()),
            },
        );
        assert_eq!(stack.confirm_callback(), Some("second"));

        stack.pop(&mut tree);
        assert_eq!(stack.confirm_callback(), Some("first"));
    }
}
