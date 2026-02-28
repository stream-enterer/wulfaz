use super::WidgetId;
use super::action::UiAction;
use super::geometry::{Constraints, Edges, Position, Rect, Size, Sizing};
use super::widget::{self, Widget};

// ---------------------------------------------------------------------------
// Performance metrics (UI-505)
// ---------------------------------------------------------------------------

/// Per-frame UI performance metrics. Stored on App, displayed one frame late.
#[derive(Debug, Clone, Copy, Default)]
pub struct UiPerfMetrics {
    /// Time spent in simulation ticks this frame (microseconds).
    pub sim_us: u64,
    /// Number of simulation ticks run this frame.
    pub sim_ticks: u32,
    /// Time spent in build phase (microseconds).
    pub build_us: u64,
    /// Time spent in layout phase (microseconds).
    pub layout_us: u64,
    /// Time spent in draw phase (microseconds).
    pub draw_us: u64,
    /// Time spent in render phase (microseconds).
    pub render_us: u64,
    /// Number of widgets in the tree.
    pub widget_count: usize,
    /// Number of panel draw commands.
    pub panel_cmds: usize,
    /// Number of text draw commands.
    pub text_cmds: usize,
    /// Number of sprite draw commands.
    pub sprite_cmds: usize,
}

// ---------------------------------------------------------------------------
// Widget node (arena entry)
// ---------------------------------------------------------------------------

/// Internal arena entry pairing a widget with tree/layout metadata.
pub(crate) struct WidgetNode {
    pub widget: Widget,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
    pub position: Position,
    pub width: Sizing,
    pub height: Sizing,
    pub padding: Edges,
    pub margin: Edges,
    /// Computed layout rect (set by layout pass).
    pub rect: Rect,
    /// Measured intrinsic size (set by measure pass).
    pub measured: Size,
    /// Optional tooltip content shown on hover (UI-W04).
    pub tooltip: Option<widget::TooltipContent>,
    /// Optional min/max size constraints applied after Sizing resolution (UI-103).
    pub constraints: Option<Constraints>,
    /// Scissor-rect clip region inherited from parent (UI-104).
    pub clip_rect: Option<Rect>,
    /// Callback action dispatched on click (UI-305). Builder sets this.
    pub on_click: Option<UiAction>,
}

// ---------------------------------------------------------------------------
// Z-order tiers (UI-307)
// ---------------------------------------------------------------------------

/// Z-order tier for root widgets (UI-307).
/// Roots are drawn in tier order: Panel first (bottom), Tooltip last (top).
/// Within a tier, roots draw in insertion order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[repr(u8)]
pub enum ZTier {
    /// Regular panels (character sheets, outliner, status bar).
    #[default]
    Panel = 0,
    /// Dropdowns, context menus — above panels.
    Overlay = 1,
    /// Modal dialogs — above overlays.
    Modal = 2,
    /// Tooltips — always on top.
    Tooltip = 3,
}
