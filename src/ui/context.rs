use std::collections::HashMap;

use super::WidgetId;
use super::action::PanelKind;
use super::animation::Animator;
use super::input::UiState;
use super::modal::ModalStack;
use super::panel_manager::PanelManager;

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

impl SidebarState {
    pub fn new() -> Self {
        Self {
            active_tab: None,
            scroll_offset: 0.0,
            scroll_view_id: None,
        }
    }
}
