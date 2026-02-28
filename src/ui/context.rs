use std::collections::HashMap;
use std::time::Instant;

use super::WidgetId;
use super::action::PanelKind;
use super::animation::Animator;
use super::input::UiState;
use super::modal::{ModalPop, ModalStack};
use super::panel_manager::PanelManager;
use super::tree::WidgetTree;
use crate::components::Entity;

/// All persistent UI state. Mirrors World's role for the simulation layer.
/// Pub fields enable Rust's field-level split borrowing.
pub struct UiContext {
    /// Input state: hover, focus, press, captured, scroll drag, tooltips.
    pub input: UiState,
    /// Active animations keyed by string name.
    pub animator: Animator,
    /// Modal dialog stack with dim layers and focus scoping.
    pub modals: ModalStack,
    /// Open panel tracking, draw order, animated close.
    pub panels: PanelManager,
    /// Scroll offsets for panels, keyed by PanelKind.
    /// Decoupled from PanelManager — survives close/reopen cycles.
    pub scroll: HashMap<PanelKind, f32>,
    /// Sidebar-specific persistent state.
    pub sidebar: SidebarState,
}

/// Sidebar persistent state (was ad-hoc fields on App).
pub struct SidebarState {
    pub active_tab: Option<usize>,
    pub scroll_offset: f32,
    pub scroll_view_id: Option<WidgetId>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            active_tab: None,
            scroll_offset: 0.0,
            scroll_view_id: None,
        }
    }
}

/// Result of the ESC / CloseTopmost priority chain.
#[derive(Debug)]
pub enum DismissResult {
    /// Dismissed all active tooltips (level 1).
    Tooltips,
    /// Popped the topmost modal (level 2). Carries the `ModalPop`.
    Modal(ModalPop),
    /// Closed the topmost panel (level 3).
    Panel(PanelKind),
    /// Cleared the entity inspector selection (level 4).
    Inspector,
    /// Closed the active sidebar tab (level 5). Carries the tab index.
    Sidebar(usize),
    /// Nothing left to dismiss — caller should save state and exit (level 6).
    Exit,
}

impl UiContext {
    /// Walk the 6-level ESC dismiss chain and perform the first applicable
    /// dismissal. Returns which layer was dismissed so the caller can handle
    /// side effects (animations, window close, etc.).
    ///
    /// `selected_entity` is passed by mutable reference because the inspector
    /// selection lives on App, not on UiContext.
    pub fn close_topmost_layer(
        &mut self,
        tree: &mut WidgetTree,
        selected_entity: &mut Option<Entity>,
        now: Instant,
    ) -> DismissResult {
        // Level 1: Tooltips.
        if self.input.tooltip_count() > 0 {
            self.input.dismiss_all_tooltips(tree, now);
            return DismissResult::Tooltips;
        }

        // Level 2: Modals.
        if let Some(pop) = self.modals.pop(tree) {
            return DismissResult::Modal(pop);
        }

        // Level 3: Panels.
        if let Some((kind, _root)) = self.panels.close_topmost(tree) {
            return DismissResult::Panel(kind);
        }

        // Level 4: Inspector (entity selection).
        if selected_entity.is_some() {
            *selected_entity = None;
            return DismissResult::Inspector;
        }

        // Level 5: Sidebar tab.
        if self.sidebar.active_tab.is_some() && self.animator.target("sidebar_slide") != Some(1.0) {
            let tab = self.sidebar.active_tab.unwrap_or(0);
            return DismissResult::Sidebar(tab);
        }

        // Level 6: Nothing left — exit.
        DismissResult::Exit
    }
}

#[cfg(test)]
mod tests {
    use super::super::modal::ModalOptions;
    use super::super::widget::Widget;
    use super::*;

    fn make_ui() -> (UiContext, WidgetTree) {
        let ui = UiContext {
            input: UiState::new(),
            animator: Animator::new(),
            modals: ModalStack::new(),
            panels: PanelManager::new(),
            scroll: HashMap::new(),
            sidebar: SidebarState::new(),
        };
        let tree = WidgetTree::new();
        (ui, tree)
    }

    fn dummy_panel(tree: &mut WidgetTree) -> WidgetId {
        tree.insert_root(Widget::Panel {
            bg_color: [0.2, 0.2, 0.2, 1.0],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        })
    }

    /// Set up all 6 dismissable layers then press ESC repeatedly.
    /// Each press must dismiss exactly one layer in priority order.
    #[test]
    fn esc_chain_dismisses_in_priority_order() {
        let (mut ui, mut tree) = make_ui();
        let now = Instant::now();

        // --- Populate all 6 layers (in reverse order for clarity) ---

        // Level 5: Sidebar — set an active tab.
        ui.sidebar.active_tab = Some(0);

        // Level 4: Inspector — select an entity.
        let mut selected = Some(Entity(42));

        // Level 3: Panel — open a closeable panel.
        let panel_root = dummy_panel(&mut tree);
        ui.panels.open(PanelKind::CharacterPanel, panel_root, true);

        // Level 2: Modal — push a modal.
        let modal_content = dummy_panel(&mut tree);
        ui.modals.push(&mut tree, modal_content, ModalOptions::NONE);

        // Level 1: Tooltip — push a fake tooltip.
        let tooltip_source = dummy_panel(&mut tree);
        let tooltip_root = dummy_panel(&mut tree);
        ui.input.push_fake_tooltip(tooltip_source, tooltip_root);

        // --- ESC #1: should dismiss tooltips ---
        let r = ui.close_topmost_layer(&mut tree, &mut selected, now);
        assert!(matches!(r, DismissResult::Tooltips));
        assert_eq!(ui.input.tooltip_count(), 0);

        // --- ESC #2: should pop the modal ---
        let r = ui.close_topmost_layer(&mut tree, &mut selected, now);
        assert!(matches!(r, DismissResult::Modal(_)));
        assert!(ui.modals.is_empty());

        // --- ESC #3: should close the panel ---
        let r = ui.close_topmost_layer(&mut tree, &mut selected, now);
        assert!(matches!(r, DismissResult::Panel(PanelKind::CharacterPanel)));

        // --- ESC #4: should clear inspector selection ---
        let r = ui.close_topmost_layer(&mut tree, &mut selected, now);
        assert!(matches!(r, DismissResult::Inspector));
        assert!(selected.is_none());

        // --- ESC #5: should close the sidebar tab ---
        let r = ui.close_topmost_layer(&mut tree, &mut selected, now);
        assert!(matches!(r, DismissResult::Sidebar(0)));

        // Simulate caller starting the close animation (as handle_tab_click does).
        ui.animator.start(
            "sidebar_slide",
            super::super::animation::Anim {
                from: 0.0,
                to: 1.0,
                duration: std::time::Duration::from_millis(200),
                easing: super::super::animation::Easing::EaseIn,
                ..super::super::animation::Anim::DEFAULT
            },
            now,
        );

        // --- ESC #6: sidebar animating closed, nothing else — should signal exit ---
        let r = ui.close_topmost_layer(&mut tree, &mut selected, now);
        assert!(matches!(r, DismissResult::Exit));
    }

    /// When no layers are open, ESC immediately returns Exit.
    #[test]
    fn esc_with_nothing_open_returns_exit() {
        let (mut ui, mut tree) = make_ui();
        let mut selected: Option<Entity> = None;
        let r = ui.close_topmost_layer(&mut tree, &mut selected, Instant::now());
        assert!(matches!(r, DismissResult::Exit));
    }

    /// Sidebar with animator target == 1.0 (fully closed) is skipped.
    #[test]
    fn esc_skips_sidebar_when_closing_animation_active() {
        let (mut ui, mut tree) = make_ui();
        let mut selected: Option<Entity> = None;
        let now = Instant::now();

        ui.sidebar.active_tab = Some(0);
        // Simulate a sidebar that's animating closed (target = 1.0).
        ui.animator.start(
            "sidebar_slide",
            super::super::animation::Anim {
                from: 0.0,
                to: 1.0,
                duration: std::time::Duration::from_millis(200),
                ..super::super::animation::Anim::DEFAULT
            },
            now,
        );

        let r = ui.close_topmost_layer(&mut tree, &mut selected, now);
        assert!(matches!(r, DismissResult::Exit));
    }
}
