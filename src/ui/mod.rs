mod animation;
pub(crate) mod character_finder;
pub(crate) mod character_panel;
mod context_menu;
pub(crate) mod demo;
mod draw;
pub(crate) mod event_popup;
mod input;
mod keybindings;
pub(crate) mod loading_screen;
pub(crate) mod main_menu;
pub(crate) mod map_mode;
pub(crate) mod minimap;
pub(crate) mod modal;
mod notification;
pub(crate) mod opinion_view;
pub(crate) mod outliner;
mod panel_manager;
pub(crate) mod save_load;
pub(crate) mod settings;
pub(crate) mod sprite;
mod theme;
mod widget;
pub(crate) mod window;

#[allow(unused_imports)] // Public API: used by main.rs for animation (UI-W05).
pub use animation::{Anim, Animator, Easing};
#[allow(unused_imports)] // Public API: used by main.rs for character finder (UI-402).
pub use character_finder::{
    CharacterFinderInfo, FinderEntry, FinderSort, build_character_finder, collect_finder_entries,
};
#[allow(unused_imports)] // Public API: used by main.rs for character panel (UI-400).
pub use character_panel::{CharacterPanelInfo, build_character_panel, collect_character_info};
#[allow(unused_imports)] // Public API: used by main.rs for context menus (UI-303).
pub use context_menu::{ContextMenu, MenuItem};
#[allow(unused_imports)] // Public API: used by game panels constructing widgets.
pub use draw::{
    DrawList, FontFamily, HeuristicMeasurer, PanelCommand, RichTextCommand, SpriteCommand,
    TextCommand, TextMeasurer, TextSpan,
};
#[allow(unused_imports)] // Public API: used by main.rs for event popups (UI-401).
pub use event_popup::{EventChoice, NarrativeEvent, build_event_popup};
#[allow(unused_imports)] // Public API: used by main.rs for input routing (UI-W02).
pub use input::{MapClick, MouseButton, UiEvent, UiState};
#[allow(unused_imports)] // Public API: used by main.rs for keyboard shortcuts (UI-I03).
pub use keybindings::{Action, KeyBindings, KeyCombo, ModifierFlags};
#[allow(unused_imports)] // Public API: used by main.rs for loading screen (UI-414).
pub use loading_screen::{LoadingScreenInfo, LoadingStage, build_loading_screen};
#[allow(unused_imports)] // Public API: used by main.rs for main menu (UI-415).
pub use main_menu::{AppState, MainMenuInfo, build_main_menu};
#[allow(unused_imports)] // Public API: used by main.rs for map mode selector (UI-403).
pub use map_mode::{MapMode, MapModeInfo, build_map_mode_selector};
#[allow(unused_imports)] // Public API: used by main.rs for minimap (UI-407).
pub use minimap::{MinimapInfo, build_minimap, minimap_click_to_world};
#[allow(unused_imports)] // Public API: used by main.rs for modal management (UI-300).
pub use modal::{ModalOptions, ModalPop, ModalStack};
#[allow(unused_imports)] // Public API: used by main.rs for notification system (UI-302).
pub use notification::{NotificationManager, NotificationPriority};
#[allow(unused_imports)] // Public API: used by main.rs for opinion view (UI-406).
pub use opinion_view::{OpinionModifier, OpinionViewInfo, Sentiment, build_opinion_view};
#[allow(unused_imports)] // Public API: used by main.rs for outliner (UI-405).
pub use outliner::{
    ActiveEvent, AlertEntry, AlertPriority, OutlinerInfo, PinnedCharacter, build_outliner,
};
#[allow(unused_imports)] // Public API: used by main.rs for panel management (UI-306).
pub use panel_manager::PanelManager;
#[allow(unused_imports)] // Public API: used by main.rs for save/load screen (UI-412).
pub use save_load::{SaveFileEntry, SaveLoadInfo, build_save_load_screen};
#[allow(unused_imports)] // Public API: used by main.rs for settings screen (UI-413).
pub use settings::{SettingsInfo, build_settings_screen};
#[allow(unused_imports)] // Public API: used by sprite renderer (UI-202b).
pub use sprite::{SpriteAtlas, SpriteRect};
pub use theme::Theme;
#[allow(unused_imports)] // Public API: used by game panels setting tooltip content.
pub use widget::{CrossAlign, TooltipContent, Widget};
#[allow(unused_imports)]
// Public API: used by screen builders for shared window frame (UI-600).
pub use window::{ConfirmationDialog, WindowFrame, build_confirmation_dialog, build_window_frame};

use slotmap::{SlotMap, new_key_type};

// ---------------------------------------------------------------------------
// Performance metrics (UI-505)
// ---------------------------------------------------------------------------

/// Per-frame UI performance metrics. Stored on App, displayed one frame late.
#[derive(Debug, Clone, Copy, Default)]
pub struct UiPerfMetrics {
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

new_key_type! {
    /// Handle into the widget arena. Stable across insertions/removals.
    pub struct WidgetId;
}

// ---------------------------------------------------------------------------
// Geometry primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Returns true if the point (px, py) is inside this rectangle.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    /// Compute the intersection of two rectangles. Returns None if they don't overlap.
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        if x2 > x1 && y2 > y1 {
            Some(Rect {
                x: x1,
                y: y1,
                width: x2 - x1,
                height: y2 - y1,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Constraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Constraints {
    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            min_height: height,
            max_width: width,
            max_height: height,
        }
    }

    pub fn loose(max_width: f32, max_height: f32) -> Self {
        Self {
            min_width: 0.0,
            min_height: 0.0,
            max_width,
            max_height,
        }
    }

    pub fn clamp(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }
}

/// Padding / margin edges (top, right, bottom, left — CSS order).
#[derive(Debug, Clone, Copy, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn all(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

// ---------------------------------------------------------------------------
// Positioning mode
// ---------------------------------------------------------------------------

/// How a widget is positioned within its parent.
#[derive(Debug, Clone, Copy)]
pub enum Position {
    /// Fixed pixel offset from parent's content origin.
    Fixed { x: f32, y: f32 },
    /// Percentage of parent's content area (0.0–1.0).
    Percent { x: f32, y: f32 },
    /// Centered in parent's content area.
    /// Computed as `(parent_w - widget_w) / 2` after size resolution.
    Center,
}

impl Default for Position {
    fn default() -> Self {
        Position::Fixed { x: 0.0, y: 0.0 }
    }
}

/// How a widget's width/height is determined.
#[derive(Debug, Clone, Copy, Default)]
pub enum Sizing {
    /// Fixed pixel size.
    Fixed(f32),
    /// Percentage of parent's content dimension (0.0–1.0).
    Percent(f32),
    /// Fit to content (intrinsic size from measure).
    #[default]
    Fit,
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
    pub dirty: bool,
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
    /// Callback key dispatched on click (UI-305). Builder sets this.
    pub on_click: Option<String>,
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

// ---------------------------------------------------------------------------
// WidgetTree
// ---------------------------------------------------------------------------

/// Arena-backed retained widget tree.
pub struct WidgetTree {
    arena: SlotMap<WidgetId, WidgetNode>,
    roots: Vec<(WidgetId, ZTier)>,
    /// Alpha for alternating ScrollList row tint (from Theme).
    scroll_row_alt_alpha: f32,
    /// Border width for control widgets (buttons, checkboxes, etc.).
    /// Set from `Theme::control_border()`.
    control_border_width: f32,
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
            dirty: true,
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
            dirty: true,
            rect: Rect::default(),
            measured: Size::default(),
            tooltip: None,
            constraints: None,
            clip_rect: None,
            on_click: None,
        });
        if let Some(parent_node) = self.arena.get_mut(parent) {
            parent_node.children.push(id);
            parent_node.dirty = true;
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
            parent.dirty = true;
        }

        // Remove from roots if present.
        self.roots.retain(|(r, _)| *r != id);

        // Remove all nodes.
        for rid in to_remove {
            self.arena.remove(rid);
        }
    }

    fn collect_subtree(
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

    /// Get a reference to a widget node.
    pub fn get(&self, id: WidgetId) -> Option<&WidgetNode> {
        self.arena.get(id)
    }

    /// Get a mutable reference to a widget node.
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode> {
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
            node.dirty = true;
        }
    }

    /// Set sizing for a widget.
    pub fn set_sizing(&mut self, id: WidgetId, w: Sizing, h: Sizing) {
        if let Some(node) = self.arena.get_mut(id) {
            node.width = w;
            node.height = h;
            node.dirty = true;
        }
    }

    /// Set padding for a widget.
    pub fn set_padding(&mut self, id: WidgetId, padding: Edges) {
        if let Some(node) = self.arena.get_mut(id) {
            node.padding = padding;
            node.dirty = true;
        }
    }

    /// Set margin for a widget.
    pub fn set_margin(&mut self, id: WidgetId, margin: Edges) {
        if let Some(node) = self.arena.get_mut(id) {
            node.margin = margin;
            node.dirty = true;
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
            node.dirty = true;
        }
    }

    /// Set a click callback key on a widget (UI-305).
    /// When this widget is clicked, `UiState::poll_click()` returns
    /// `Some((widget_id, key))`.
    pub fn set_on_click(&mut self, id: WidgetId, key: impl Into<String>) {
        if let Some(node) = self.arena.get_mut(id) {
            node.on_click = Some(key.into());
        }
    }

    /// Set a scissor-rect clip region on a widget (UI-104).
    /// Children inherit the clip region during layout.
    pub fn set_clip_rect(&mut self, id: WidgetId, clip: Option<Rect>) {
        if let Some(node) = self.arena.get_mut(id) {
            node.clip_rect = clip;
        }
    }

    /// Mark a widget and its ancestors as dirty.
    pub fn mark_dirty(&mut self, id: WidgetId) {
        let mut current = Some(id);
        while let Some(cid) = current {
            if let Some(node) = self.arena.get_mut(cid) {
                if node.dirty {
                    break; // already dirty up from here
                }
                node.dirty = true;
                current = node.parent;
            } else {
                break;
            }
        }
    }

    /// Root widget ids in draw order (sorted by Z-tier, stable within tier).
    pub fn roots(&self) -> Vec<WidgetId> {
        self.roots_draw_order()
    }

    /// Root ids in draw order: Panel → Overlay → Modal → Tooltip.
    /// Within each tier, insertion order is preserved (stable sort).
    fn roots_draw_order(&self) -> Vec<WidgetId> {
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

    // ------------------------------------------------------------------
    // Hit testing
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Layout
    // ------------------------------------------------------------------

    /// Run the full layout pass over the tree. `screen` is the available area.
    pub fn layout(&mut self, screen: Size, tm: &mut dyn TextMeasurer) {
        let root_ids = self.roots_draw_order();
        for root in root_ids {
            self.layout_node(
                root,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: screen.width,
                    height: screen.height,
                },
                tm,
            );
        }
    }

    fn layout_node(&mut self, id: WidgetId, parent_content: Rect, tm: &mut dyn TextMeasurer) {
        // Measure intrinsic size.
        let measured = self.measure_node(id, tm);

        let Some(node) = self.arena.get_mut(id) else {
            return;
        };
        node.measured = measured;

        // Resolve width/height from Sizing.
        let resolved_w = match node.width {
            Sizing::Fixed(px) => px,
            Sizing::Percent(frac) => parent_content.width * frac,
            Sizing::Fit => measured.width + node.padding.horizontal(),
        };
        let resolved_h = match node.height {
            Sizing::Fixed(px) => px,
            Sizing::Percent(frac) => parent_content.height * frac,
            Sizing::Fit => measured.height + node.padding.vertical(),
        };

        // Resolve position.
        let (ox, oy) = match node.position {
            Position::Fixed { x, y } => (x, y),
            Position::Percent { x, y } => (parent_content.width * x, parent_content.height * y),
            Position::Center => (
                (parent_content.width - resolved_w) * 0.5,
                (parent_content.height - resolved_h) * 0.5,
            ),
        };

        // For wrapped labels with Fit height, recompute height based on resolved width (UI-102).
        let resolved_h = if let Widget::Label {
            text,
            font_size,
            font_family,
            wrap: true,
            ..
        } = &node.widget
        {
            if matches!(node.height, Sizing::Fit) && resolved_w > 0.0 {
                let ts = tm.measure_text("M", *font_family, *font_size);
                let char_w = ts.width;
                let line_h = ts.height;
                let content_w = (resolved_w - node.padding.horizontal()).max(0.0);
                let n_lines = wrapped_line_count(text, content_w, char_w);
                n_lines as f32 * line_h + node.padding.vertical()
            } else {
                resolved_h
            }
        } else {
            resolved_h
        };

        // Apply min/max constraints if set (UI-103).
        let (resolved_w, resolved_h) = if let Some(c) = &node.constraints {
            let clamped = c.clamp(Size {
                width: resolved_w,
                height: resolved_h,
            });
            (clamped.width, clamped.height)
        } else {
            (resolved_w, resolved_h)
        };

        node.rect = Rect {
            x: parent_content.x + node.margin.left + ox,
            y: parent_content.y + node.margin.top + oy,
            width: resolved_w,
            height: resolved_h,
        };
        node.dirty = false;

        self.layout_node_children(id, tm);
    }

    /// Lay out the children of a node according to its container type.
    ///
    /// Separated from `layout_node` so that container children (Row-in-Column,
    /// Column-in-Collapsible, etc.) dispatch through their own layout logic
    /// instead of being treated as flat grandchildren.
    fn layout_node_children(&mut self, id: WidgetId, tm: &mut dyn TextMeasurer) {
        let Some(node) = self.arena.get(id) else {
            return;
        };

        // Content area for children (inside padding).
        let content = Rect {
            x: node.rect.x + node.padding.left,
            y: node.rect.y + node.padding.top,
            width: (node.rect.width - node.padding.horizontal()).max(0.0),
            height: (node.rect.height - node.padding.vertical()).max(0.0),
        };

        // Row: lay out children left-to-right with gap spacing (UI-100).
        if let Widget::Row { gap, align } = &node.widget {
            let gap = *gap;
            let align = *align;
            let children: Vec<WidgetId> = node.children.clone();
            let parent_clip = node.clip_rect;

            // First pass: measure all children, identify Percent-width children.
            let mut child_infos: Vec<(WidgetId, Size, Sizing, Edges, Edges)> = Vec::new();
            let mut fixed_total_w: f32 = 0.0;
            let mut percent_total: f32 = 0.0;
            for &child_id in &children {
                let child_measured = self.measure_node(child_id, tm);
                let Some(child) = self.arena.get(child_id) else {
                    continue;
                };
                // Expand children auto-fill remaining width (UI-601).
                let cw = if matches!(child.widget, Widget::Expand) {
                    Sizing::Percent(1.0)
                } else {
                    child.width
                };
                let cp = child.padding;
                let cm = child.margin;
                let child_w = match cw {
                    Sizing::Fixed(px) => px + cm.horizontal(),
                    Sizing::Fit => child_measured.width + cp.horizontal() + cm.horizontal(),
                    Sizing::Percent(frac) => {
                        percent_total += frac;
                        0.0
                    }
                };
                fixed_total_w += child_w;
                child_infos.push((child_id, child_measured, cw, cp, cm));
            }
            let n = children.len();
            let gap_total = if n > 1 { gap * (n - 1) as f32 } else { 0.0 };
            let remaining = (content.width - fixed_total_w - gap_total).max(0.0);

            // Second pass: position children.
            let mut cursor_x = content.x;
            for (child_id, child_measured, cw, cp, cm) in &child_infos {
                let child_w = match cw {
                    Sizing::Fixed(px) => *px,
                    Sizing::Fit => child_measured.width + cp.horizontal(),
                    Sizing::Percent(frac) => {
                        if percent_total > 0.0 {
                            remaining * frac / percent_total
                        } else {
                            0.0
                        }
                    }
                };
                let child_total_w = child_w + cm.horizontal();

                // Resolve child height.
                let child_h = match self.arena.get(*child_id).map(|n| n.height) {
                    Some(Sizing::Fixed(px)) => px,
                    Some(Sizing::Percent(frac)) => content.height * frac,
                    Some(Sizing::Fit) | None => child_measured.height + cp.vertical(),
                };

                // Cross-axis alignment: vertical position within row.
                let child_y = match align {
                    widget::CrossAlign::Start => content.y + cm.top,
                    widget::CrossAlign::Center => content.y + (content.height - child_h) / 2.0,
                    widget::CrossAlign::End => content.y + content.height - child_h - cm.bottom,
                    widget::CrossAlign::Stretch => content.y + cm.top,
                };
                let stretched_h = if align == widget::CrossAlign::Stretch {
                    content.height - cm.vertical()
                } else {
                    child_h
                };

                if let Some(child_node) = self.arena.get_mut(*child_id) {
                    child_node.measured = *child_measured;
                    child_node.rect = Rect {
                        x: cursor_x + cm.left,
                        y: child_y,
                        width: child_w,
                        height: stretched_h,
                    };
                    child_node.clip_rect = Self::merge_clips(parent_clip, child_node.clip_rect);
                    child_node.dirty = false;
                }

                // Recurse into child's own layout (Row, Column, etc.).
                self.layout_node_children(*child_id, tm);

                cursor_x += child_total_w + gap;
            }
            return;
        }

        // Column: lay out children top-to-bottom with gap spacing (UI-101).
        if let Widget::Column { gap, align } = &node.widget {
            let gap = *gap;
            let align = *align;
            let children: Vec<WidgetId> = node.children.clone();
            let parent_clip = node.clip_rect;

            // First pass: measure all children, identify Percent-height children.
            let mut child_infos: Vec<(WidgetId, Size, Sizing, Edges, Edges)> = Vec::new();
            let mut fixed_total_h: f32 = 0.0;
            let mut percent_total: f32 = 0.0;
            for &child_id in &children {
                let child_measured = self.measure_node(child_id, tm);
                let Some(child) = self.arena.get(child_id) else {
                    continue;
                };
                // Expand children auto-fill remaining height (UI-601).
                let ch = if matches!(child.widget, Widget::Expand) {
                    Sizing::Percent(1.0)
                } else {
                    child.height
                };
                let cp = child.padding;
                let cm = child.margin;
                let child_h = match ch {
                    Sizing::Fixed(px) => px + cm.vertical(),
                    Sizing::Fit => child_measured.height + cp.vertical() + cm.vertical(),
                    Sizing::Percent(frac) => {
                        percent_total += frac;
                        0.0
                    }
                };
                fixed_total_h += child_h;
                child_infos.push((child_id, child_measured, ch, cp, cm));
            }
            let n = children.len();
            let gap_total = if n > 1 { gap * (n - 1) as f32 } else { 0.0 };
            let remaining = (content.height - fixed_total_h - gap_total).max(0.0);

            // Second pass: position children.
            let mut cursor_y = content.y;
            for (child_id, child_measured, ch, cp, cm) in &child_infos {
                let child_h = match ch {
                    Sizing::Fixed(px) => *px,
                    Sizing::Fit => child_measured.height + cp.vertical(),
                    Sizing::Percent(frac) => {
                        if percent_total > 0.0 {
                            remaining * frac / percent_total
                        } else {
                            0.0
                        }
                    }
                };
                let child_total_h = child_h + cm.vertical();

                // Resolve child width.
                let child_w = match self.arena.get(*child_id).map(|n| n.width) {
                    Some(Sizing::Fixed(px)) => px,
                    Some(Sizing::Percent(frac)) => content.width * frac,
                    Some(Sizing::Fit) | None => child_measured.width + cp.horizontal(),
                };

                // Cross-axis alignment: horizontal position within column.
                let child_x = match align {
                    widget::CrossAlign::Start => content.x + cm.left,
                    widget::CrossAlign::Center => content.x + (content.width - child_w) / 2.0,
                    widget::CrossAlign::End => content.x + content.width - child_w - cm.right,
                    widget::CrossAlign::Stretch => content.x + cm.left,
                };
                let stretched_w = if align == widget::CrossAlign::Stretch {
                    content.width - cm.horizontal()
                } else {
                    child_w
                };

                if let Some(child_node) = self.arena.get_mut(*child_id) {
                    child_node.measured = *child_measured;
                    child_node.rect = Rect {
                        x: child_x,
                        y: cursor_y + cm.top,
                        width: stretched_w,
                        height: child_h,
                    };
                    child_node.clip_rect = Self::merge_clips(parent_clip, child_node.clip_rect);
                    child_node.dirty = false;
                }

                // Recurse into child's own layout (Row, Column, etc.).
                self.layout_node_children(*child_id, tm);

                cursor_y += child_total_h + gap;
            }

            // Update Fit-sized Column height to actual laid-out extent.
            // measure_node reports intrinsic sizes (e.g. ScrollList returns
            // total content height of all items), but layout resolves
            // Fixed/Percent sizing on children, so the actual content extent
            // may be smaller than measured.
            let is_fit = self
                .arena
                .get(id)
                .is_some_and(|n| matches!(n.height, Sizing::Fit));
            if is_fit {
                let actual_content_h = if n > 0 {
                    (cursor_y - content.y - gap).max(0.0)
                } else {
                    0.0
                };
                if let Some(node) = self.arena.get_mut(id) {
                    node.rect.height = actual_content_h + node.padding.vertical();
                }
            }
            return;
        }

        // ScrollList positions children in a vertical stack with virtual scrolling.
        if let Widget::ScrollList {
            item_height,
            scroll_offset,
            scrollbar_width,
            item_heights,
            ..
        } = &node.widget
        {
            let ih = *item_height;
            let so = *scroll_offset;
            let sbw = *scrollbar_width;
            let ihs = item_heights.clone();
            let children: Vec<WidgetId> = node.children.clone();
            let parent_clip = node.clip_rect;
            let viewport_h = content.height;
            let content_w = (content.width - sbw).max(0.0);
            let n = children.len();

            let first = Self::scroll_first_visible(&ihs, ih, n, so);
            for (i, child_id) in children.iter().enumerate() {
                // Skip items before first visible.
                if i < first {
                    if let Some(child_node) = self.arena.get_mut(*child_id) {
                        child_node.rect = Rect::default();
                        child_node.dirty = false;
                    }
                    continue;
                }

                let item_y_abs = Self::scroll_item_y(&ihs, ih, i);
                let item_h = Self::scroll_item_h(&ihs, ih, i);
                let item_y = item_y_abs - so;

                // Virtual scrolling: break once past viewport.
                if item_y >= viewport_h {
                    // Zero-rect remaining items.
                    for remaining_id in &children[i..] {
                        if let Some(child_node) = self.arena.get_mut(*remaining_id) {
                            child_node.rect = Rect::default();
                            child_node.dirty = false;
                        }
                    }
                    break;
                }

                // Propagate clip from parent (UI-104).
                if let Some(child_node) = self.arena.get_mut(*child_id) {
                    child_node.clip_rect = Self::merge_clips(parent_clip, child_node.clip_rect);
                }

                // Layout visible item: set rect directly, then recurse for children.
                self.layout_scroll_item(
                    *child_id,
                    content.x,
                    content.y + item_y,
                    content_w,
                    item_h,
                    tm,
                );
            }
            return;
        }

        // ScrollView: offset children by scroll_offset, clip to viewport (UI-W06).
        if let Widget::ScrollView {
            scroll_offset,
            scrollbar_width,
            ..
        } = &node.widget
        {
            let so = *scroll_offset;
            let sbw = *scrollbar_width;
            let children: Vec<WidgetId> = node.children.clone();
            let parent_clip = node.clip_rect;
            let viewport_h = content.height;
            let content_w = (content.width - sbw).max(0.0);

            // Viewport clip rect — children outside this are GPU-clipped.
            let viewport_clip = Some(Rect {
                x: content.x,
                y: content.y,
                width: content.width,
                height: viewport_h,
            });

            let mut cursor_y = content.y - so;
            for &child_id in &children {
                let child_measured = self.measure_node(child_id, tm);
                let Some(child) = self.arena.get(child_id) else {
                    continue;
                };
                let cp = child.padding;
                let cm = child.margin;
                let child_h = match child.height {
                    Sizing::Fixed(px) => px,
                    Sizing::Percent(frac) => viewport_h * frac,
                    Sizing::Fit => child_measured.height + cp.vertical(),
                };
                let child_w = match child.width {
                    Sizing::Fixed(px) => px,
                    Sizing::Percent(frac) => content_w * frac,
                    Sizing::Fit => child_measured.width + cp.horizontal(),
                };

                if let Some(child_node) = self.arena.get_mut(child_id) {
                    child_node.measured = child_measured;
                    child_node.rect = Rect {
                        x: content.x + cm.left,
                        y: cursor_y + cm.top,
                        width: child_w,
                        height: child_h,
                    };
                    child_node.clip_rect = Self::merge_clips(
                        Self::merge_clips(parent_clip, viewport_clip),
                        child_node.clip_rect,
                    );
                    child_node.dirty = false;
                }

                self.layout_node_children(child_id, tm);

                cursor_y += child_h + cm.vertical();
            }
            return;
        }

        // Collapsible: header row + vertical children when expanded (UI-304).
        if let Widget::Collapsible {
            expanded,
            font_size,
            ..
        } = &node.widget
        {
            let expanded = *expanded;
            let header_h = tm
                .measure_text("M", FontFamily::default(), *font_size)
                .height
                + 4.0;
            let children: Vec<WidgetId> = node.children.clone();
            let parent_clip = node.clip_rect;

            if expanded {
                // Lay out children below the header, Column-style.
                let mut cursor_y = content.y + header_h;
                for &child_id in &children {
                    let child_measured = self.measure_node(child_id, tm);
                    let Some(child) = self.arena.get(child_id) else {
                        continue;
                    };
                    let cp = child.padding;
                    let cm = child.margin;
                    let child_h = match child.height {
                        Sizing::Fixed(px) => px,
                        Sizing::Percent(frac) => content.height * frac,
                        Sizing::Fit => child_measured.height + cp.vertical(),
                    };

                    if let Some(child_node) = self.arena.get_mut(child_id) {
                        child_node.measured = child_measured;
                        child_node.rect = Rect {
                            x: content.x + cm.left,
                            y: cursor_y + cm.top,
                            width: content.width - cm.horizontal(),
                            height: child_h,
                        };
                        child_node.clip_rect = Self::merge_clips(parent_clip, child_node.clip_rect);
                        child_node.dirty = false;
                    }

                    // Recurse into child's own layout (Row, Column, etc.).
                    self.layout_node_children(child_id, tm);

                    cursor_y += child_h + cm.vertical();
                }
            }
            // When collapsed, skip children entirely.
            return;
        }

        // TabContainer: tab bar + Column-style content children (UI-301).
        if let Widget::TabContainer { font_size, .. } = &node.widget {
            let tab_bar_h = tm
                .measure_text("M", FontFamily::default(), *font_size)
                .height
                + 6.0;
            let children: Vec<WidgetId> = node.children.clone();
            let parent_clip = node.clip_rect;

            // Lay out children Column-style below the tab bar.
            let mut cursor_y = content.y + tab_bar_h;
            for &child_id in &children {
                let child_measured = self.measure_node(child_id, tm);
                let Some(child) = self.arena.get(child_id) else {
                    continue;
                };
                let cp = child.padding;
                let cm = child.margin;
                let child_h = match child.height {
                    Sizing::Fixed(px) => px,
                    Sizing::Percent(frac) => content.height * frac,
                    Sizing::Fit => child_measured.height + cp.vertical(),
                };
                let child_w = match child.width {
                    Sizing::Fixed(px) => px,
                    Sizing::Percent(frac) => content.width * frac,
                    Sizing::Fit => child_measured.width + cp.horizontal(),
                };

                if let Some(child_node) = self.arena.get_mut(child_id) {
                    child_node.measured = child_measured;
                    child_node.rect = Rect {
                        x: content.x + cm.left,
                        y: cursor_y + cm.top,
                        width: child_w,
                        height: child_h,
                    };
                    child_node.clip_rect = Self::merge_clips(parent_clip, child_node.clip_rect);
                }

                // Recurse into child's own layout (Row, Column, etc.).
                self.layout_node_children(child_id, tm);

                cursor_y += child_h + cm.vertical();
            }
            return;
        }

        // Propagate clip_rect from parent to children (UI-104).
        let parent_clip = node.clip_rect;
        let children: Vec<WidgetId> = node.children.clone();
        for child in &children {
            if let Some(child_node) = self.arena.get_mut(*child) {
                child_node.clip_rect = Self::merge_clips(parent_clip, child_node.clip_rect);
            }
        }
        for child in children {
            self.layout_node(child, content, tm);
        }
    }

    /// Merge parent and child clip rects (UI-104). If both exist, intersect them.
    fn merge_clips(parent: Option<Rect>, child: Option<Rect>) -> Option<Rect> {
        match (parent, child) {
            (Some(p), Some(c)) => p.intersect(&c),
            (Some(p), None) => Some(p),
            (None, c) => c,
        }
    }

    /// Layout a scroll list item: set its rect directly and recurse into its children.
    fn layout_scroll_item(
        &mut self,
        id: WidgetId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        tm: &mut dyn TextMeasurer,
    ) {
        let measured = self.measure_node(id, tm);

        let Some(node) = self.arena.get_mut(id) else {
            return;
        };
        node.measured = measured;
        node.rect = Rect {
            x,
            y,
            width,
            height,
        };
        node.dirty = false;

        self.layout_node_children(id, tm);
    }

    /// Measure intrinsic size of a widget (content only, no padding).
    pub fn measure_node(&self, id: WidgetId, tm: &mut dyn TextMeasurer) -> Size {
        let Some(node) = self.arena.get(id) else {
            return Size::default();
        };

        match &node.widget {
            Widget::Label {
                text,
                font_size,
                font_family,
                ..
            } => {
                let ts = tm.measure_text(text, *font_family, *font_size);
                let h = tm.measure_text("M", *font_family, *font_size).height;
                Size {
                    width: ts.width,
                    height: h,
                }
            }
            Widget::Button {
                text,
                font_size,
                font_family,
                ..
            } => {
                let ts = tm.measure_text(text, *font_family, *font_size);
                let h = tm.measure_text("M", *font_family, *font_size).height;
                // Intrinsic content size only; padding added by layout_node.
                Size {
                    width: ts.width,
                    height: h,
                }
            }
            Widget::RichText { spans, font_size } => {
                let total_w: f32 = spans
                    .iter()
                    .map(|s| tm.measure_text(&s.text, s.font_family, *font_size).width)
                    .sum();
                let h = tm
                    .measure_text("M", FontFamily::default(), *font_size)
                    .height;
                Size {
                    width: total_w,
                    height: h,
                }
            }
            Widget::Row { gap, .. } => {
                // Row: width = sum of child widths + gaps, height = max child height.
                let n = node.children.len();
                let mut total_w: f32 = 0.0;
                let mut max_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_measured = self.measure_node(child_id, tm);
                        let child_w = child_measured.width
                            + child.padding.horizontal()
                            + child.margin.horizontal();
                        let child_h = child_measured.height
                            + child.padding.vertical()
                            + child.margin.vertical();
                        total_w += child_w;
                        max_h = max_h.max(child_h);
                    }
                }
                if n > 1 {
                    total_w += *gap * (n - 1) as f32;
                }
                Size {
                    width: total_w,
                    height: max_h,
                }
            }
            Widget::Column { gap, .. } => {
                // Column: width = max child width, height = sum of child heights + gaps.
                let n = node.children.len();
                let mut max_w: f32 = 0.0;
                let mut total_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_measured = self.measure_node(child_id, tm);
                        let child_w = child_measured.width
                            + child.padding.horizontal()
                            + child.margin.horizontal();
                        let child_h = child_measured.height
                            + child.padding.vertical()
                            + child.margin.vertical();
                        max_w = max_w.max(child_w);
                        total_h += child_h;
                    }
                }
                if n > 1 {
                    total_h += *gap * (n - 1) as f32;
                }
                Size {
                    width: max_w,
                    height: total_h,
                }
            }
            Widget::Panel { .. } => {
                // Panel measures from children bounding box.
                let mut max_w: f32 = 0.0;
                let mut max_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_measured = self.measure_node(child_id, tm);
                        let (cx, cy) = match child.position {
                            Position::Fixed { x, y } => (x, y),
                            Position::Percent { .. } | Position::Center => (0.0, 0.0),
                        };
                        max_w = max_w.max(
                            cx + child_measured.width
                                + child.padding.horizontal()
                                + child.margin.horizontal(),
                        );
                        max_h = max_h.max(
                            cy + child_measured.height
                                + child.padding.vertical()
                                + child.margin.vertical(),
                        );
                    }
                }
                Size {
                    width: max_w,
                    height: max_h,
                }
            }
            Widget::ScrollList {
                item_height,
                scrollbar_width,
                item_heights,
                ..
            } => {
                // Total content height (variable or fixed).
                // Width = widest child + scrollbar.
                let mut max_w: f32 = 0.0;
                for &child_id in &node.children {
                    let child_measured = self.measure_node(child_id, tm);
                    max_w = max_w.max(child_measured.width);
                }
                let n = node.children.len();
                let total_h = Self::scroll_total_height(item_heights, *item_height, n);
                Size {
                    width: max_w + scrollbar_width,
                    height: total_h,
                }
            }
            Widget::ScrollView {
                scrollbar_width, ..
            } => {
                // Width = max child width + scrollbar, height = sum of child heights.
                let mut max_w: f32 = 0.0;
                let mut total_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_measured = self.measure_node(child_id, tm);
                        let child_w = child_measured.width
                            + child.padding.horizontal()
                            + child.margin.horizontal();
                        let child_h = child_measured.height
                            + child.padding.vertical()
                            + child.margin.vertical();
                        max_w = max_w.max(child_w);
                        total_h += child_h;
                    }
                }
                Size {
                    width: max_w + scrollbar_width,
                    height: total_h,
                }
            }
            Widget::ProgressBar { height, .. } => {
                // Width = parent-provided (stretch-width), intrinsic width 0.
                // Height from field.
                Size {
                    width: 0.0,
                    height: *height,
                }
            }
            Widget::Separator {
                thickness,
                horizontal,
                ..
            } => {
                // Horizontal: width = parent, height = thickness.
                // Vertical: width = thickness, height = parent.
                if *horizontal {
                    Size {
                        width: 0.0,
                        height: *thickness,
                    }
                } else {
                    Size {
                        width: *thickness,
                        height: 0.0,
                    }
                }
            }
            Widget::Icon { size, .. } => {
                // Square icon.
                Size {
                    width: *size,
                    height: *size,
                }
            }
            Widget::Checkbox {
                label, font_size, ..
            } => {
                let ts = tm.measure_text(label, FontFamily::default(), *font_size);
                let text_h = tm
                    .measure_text("M", FontFamily::default(), *font_size)
                    .height;
                let box_size = 16.0;
                let gap = 6.0;
                Size {
                    width: box_size + gap + ts.width,
                    height: box_size.max(text_h),
                }
            }
            Widget::Dropdown {
                options, font_size, ..
            } => {
                // Width = widest option text + arrow "▼" + padding.
                let widest_w: f32 = options
                    .iter()
                    .map(|o| tm.measure_text(o, FontFamily::default(), *font_size).width)
                    .fold(0.0_f32, f32::max);
                let h = tm
                    .measure_text("M", FontFamily::default(), *font_size)
                    .height;
                let arrow_w = tm
                    .measure_text("\u{25BC}\u{25BC}", FontFamily::default(), *font_size)
                    .width;
                Size {
                    width: widest_w + arrow_w,
                    height: h,
                }
            }
            Widget::Slider { width, .. } => {
                let thumb_size = 16.0;
                Size {
                    width: *width,
                    height: thumb_size,
                }
            }
            Widget::TextInput { font_size, .. } => {
                let h = tm
                    .measure_text("M", FontFamily::default(), *font_size)
                    .height;
                // Stretch-width (intrinsic 0), height = text only (padding added by layout_node).
                Size {
                    width: 0.0,
                    height: h,
                }
            }
            Widget::Collapsible {
                header,
                expanded,
                font_size,
                ..
            } => {
                let m_size = tm.measure_text("M", FontFamily::default(), *font_size);
                let header_h = m_size.height + 4.0;
                // Triangle indicator (~2 chars) + header text.
                let indicator_w = tm
                    .measure_text("\u{25BC}\u{25BC}", FontFamily::default(), *font_size)
                    .width;
                let header_text_w = tm
                    .measure_text(header, FontFamily::default(), *font_size)
                    .width;
                let header_w = indicator_w + header_text_w;
                if *expanded {
                    // Header + sum of children heights.
                    let mut max_w = header_w;
                    let mut total_h = header_h;
                    for &child_id in &node.children {
                        if let Some(child) = self.arena.get(child_id) {
                            let child_measured = self.measure_node(child_id, tm);
                            let child_w = child_measured.width
                                + child.padding.horizontal()
                                + child.margin.horizontal();
                            let child_h = child_measured.height
                                + child.padding.vertical()
                                + child.margin.vertical();
                            max_w = max_w.max(child_w);
                            total_h += child_h;
                        }
                    }
                    Size {
                        width: max_w,
                        height: total_h,
                    }
                } else {
                    // Collapsed: header only.
                    Size {
                        width: header_w,
                        height: header_h,
                    }
                }
            }
            Widget::TabContainer {
                tabs, font_size, ..
            } => {
                let m_size = tm.measure_text("M", FontFamily::default(), *font_size);
                let tab_pad = 8.0; // horizontal padding per tab
                let tab_bar_h = m_size.height + 6.0;
                let tab_bar_w: f32 = tabs
                    .iter()
                    .map(|t| {
                        tm.measure_text(t, FontFamily::default(), *font_size).width + tab_pad * 2.0
                    })
                    .sum();
                // Content children measured Column-style.
                let mut content_w = 0.0_f32;
                let mut content_h = 0.0_f32;
                for &child_id in &node.children {
                    let child_measured = self.measure_node(child_id, tm);
                    if let Some(child) = self.arena.get(child_id) {
                        content_w = content_w.max(
                            child_measured.width
                                + child.padding.horizontal()
                                + child.margin.horizontal(),
                        );
                        content_h += child_measured.height
                            + child.padding.vertical()
                            + child.margin.vertical();
                    }
                }
                Size {
                    width: tab_bar_w.max(content_w),
                    height: tab_bar_h + content_h,
                }
            }
            // Expand is an invisible spacer — zero intrinsic size.
            Widget::Expand => Size::default(),
        }
    }

    // ------------------------------------------------------------------
    // Draw
    // ------------------------------------------------------------------

    /// Walk the tree and emit draw commands into a `DrawList`.
    pub fn draw(&self, draw_list: &mut DrawList, tm: &mut dyn TextMeasurer) {
        self.draw_with_measurer(draw_list, tm);
    }

    /// Walk the tree and emit draw commands, using a `TextMeasurer` for metrics.
    pub fn draw_with_measurer(&self, draw_list: &mut DrawList, tm: &mut dyn TextMeasurer) {
        let mut sorted = self.roots.clone();
        sorted.sort_by_key(|(_, tier)| *tier);
        for (id, tier) in sorted {
            let p0 = draw_list.panels.len();
            let t0 = draw_list.texts.len();
            let r0 = draw_list.rich_texts.len();
            self.draw_node(id, draw_list, tm, tier as u8);
            draw_list.root_slices.push(draw::RootSlice {
                panels: p0..draw_list.panels.len(),
                texts: t0..draw_list.texts.len(),
                rich_texts: r0..draw_list.rich_texts.len(),
            });
        }
    }

    fn draw_node(
        &self,
        id: WidgetId,
        draw_list: &mut DrawList,
        tm: &mut dyn TextMeasurer,
        tier: u8,
    ) {
        let Some(node) = self.arena.get(id) else {
            return;
        };

        let clip = node.clip_rect;

        match &node.widget {
            // Row, Column, and Expand are transparent — no draw commands.
            Widget::Row { .. } | Widget::Column { .. } | Widget::Expand => {}
            Widget::Panel {
                bg_color,
                border_color,
                border_width,
                shadow_width,
            } => {
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    bg_color: *bg_color,
                    border_color: *border_color,
                    border_width: *border_width,
                    shadow_width: *shadow_width,
                    clip,
                    tier,
                });
            }
            Widget::Label {
                text,
                color,
                font_size,
                font_family,
                wrap,
            } => {
                if *wrap && node.rect.width > 0.0 {
                    let ts = tm.measure_text("M", *font_family, *font_size);
                    let char_w = ts.width;
                    let line_h = ts.height;
                    let max_chars = (node.rect.width / char_w).max(1.0) as usize;
                    let lines = wrap_text(text, max_chars);
                    for (i, line) in lines.iter().enumerate() {
                        draw_list.texts.push(TextCommand {
                            text: line.clone(),
                            x: node.rect.x,
                            y: node.rect.y + i as f32 * line_h,
                            color: *color,
                            font_size: *font_size,
                            font_family: *font_family,
                            clip,
                            tier,
                        });
                    }
                } else {
                    draw_list.texts.push(TextCommand {
                        text: text.clone(),
                        x: node.rect.x,
                        y: node.rect.y,
                        color: *color,
                        font_size: *font_size,
                        font_family: *font_family,
                        clip,
                        tier,
                    });
                }
            }
            Widget::Button {
                text,
                color,
                bg_color,
                border_color,
                font_size,
                font_family,
            } => {
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    bg_color: *bg_color,
                    border_color: *border_color,
                    border_width: self.control_border_width,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
                draw_list.texts.push(TextCommand {
                    text: text.clone(),
                    x: node.rect.x + node.padding.left,
                    y: node.rect.y + node.padding.top,
                    color: *color,
                    font_size: *font_size,
                    font_family: *font_family,
                    clip,
                    tier,
                });
            }
            Widget::RichText { spans, font_size } => {
                draw_list.rich_texts.push(RichTextCommand {
                    spans: spans.clone(),
                    x: node.rect.x,
                    y: node.rect.y,
                    font_size: *font_size,
                    clip,
                    tier,
                });
            }
            Widget::ProgressBar {
                fraction,
                fg_color,
                bg_color,
                border_color,
                border_width,
                ..
            } => {
                let f = fraction.clamp(0.0, 1.0);
                // Background rect (full width).
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    bg_color: *bg_color,
                    border_color: *border_color,
                    border_width: *border_width,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
                // Foreground rect (fraction of width).
                if f > 0.0 {
                    let inner_x = node.rect.x + *border_width;
                    let inner_y = node.rect.y + *border_width;
                    let inner_w = (node.rect.width - 2.0 * border_width).max(0.0);
                    let inner_h = (node.rect.height - 2.0 * border_width).max(0.0);
                    draw_list.panels.push(PanelCommand {
                        x: inner_x,
                        y: inner_y,
                        width: inner_w * f,
                        height: inner_h,
                        bg_color: *fg_color,
                        border_color: [0.0; 4],
                        border_width: 0.0,
                        shadow_width: 0.0,
                        clip,
                        tier,
                    });
                }
            }
            Widget::Separator { color, .. } => {
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    bg_color: *color,
                    border_color: [0.0; 4],
                    border_width: 0.0,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
            }
            Widget::Icon { sprite, tint, .. } => {
                draw_list.sprites.push(SpriteCommand {
                    sprite: sprite.clone(),
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    tint: tint.unwrap_or([1.0, 1.0, 1.0, 1.0]),
                    clip,
                    tier,
                });
            }
            Widget::Checkbox {
                checked,
                label,
                color,
                font_size,
            } => {
                let box_size = 16.0;
                let gap = 6.0;
                let box_y = node.rect.y + (node.rect.height - box_size).max(0.0) / 2.0;
                // Box border.
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: box_y,
                    width: box_size,
                    height: box_size,
                    bg_color: [0.0, 0.0, 0.0, 0.0],
                    border_color: *color,
                    border_width: self.control_border_width,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
                // Checkmark when checked.
                if *checked {
                    draw_list.texts.push(TextCommand {
                        text: "\u{2713}".to_string(),
                        x: node.rect.x + 2.0,
                        y: box_y + 1.0,
                        color: *color,
                        font_size: box_size - 4.0,
                        font_family: FontFamily::default(),
                        clip,
                        tier,
                    });
                }
                // Label text.
                let text_h = tm
                    .measure_text("M", FontFamily::default(), *font_size)
                    .height;
                let label_y = node.rect.y + (node.rect.height - text_h).max(0.0) / 2.0;
                draw_list.texts.push(TextCommand {
                    text: label.clone(),
                    x: node.rect.x + box_size + gap,
                    y: label_y,
                    color: *color,
                    font_size: *font_size,
                    font_family: FontFamily::default(),
                    clip,
                    tier,
                });
            }
            Widget::Dropdown {
                selected,
                options,
                open,
                color,
                bg_color,
                font_size,
            } => {
                let row_h = tm
                    .measure_text("M", FontFamily::default(), *font_size)
                    .height
                    + node.padding.top
                    + node.padding.bottom;
                // Trigger button background.
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: row_h,
                    bg_color: *bg_color,
                    border_color: *color,
                    border_width: self.control_border_width,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
                // Selected option text.
                let label = options.get(*selected).map(|s| s.as_str()).unwrap_or("");
                draw_list.texts.push(TextCommand {
                    text: label.to_string(),
                    x: node.rect.x + node.padding.left,
                    y: node.rect.y + node.padding.top,
                    color: *color,
                    font_size: *font_size,
                    font_family: FontFamily::default(),
                    clip,
                    tier,
                });
                // Down-arrow indicator.
                let arrow_w = tm
                    .measure_text("\u{25BC}", FontFamily::default(), *font_size)
                    .width;
                draw_list.texts.push(TextCommand {
                    text: "\u{25BC}".to_string(),
                    x: node.rect.x + node.rect.width - node.padding.right - arrow_w,
                    y: node.rect.y + node.padding.top,
                    color: *color,
                    font_size: *font_size,
                    font_family: FontFamily::default(),
                    clip,
                    tier,
                });
                // Open state: option list overlay below trigger.
                if *open {
                    let list_y = node.rect.y + row_h;
                    // Options background.
                    draw_list.panels.push(PanelCommand {
                        x: node.rect.x,
                        y: list_y,
                        width: node.rect.width,
                        height: row_h * options.len() as f32,
                        bg_color: *bg_color,
                        border_color: *color,
                        border_width: self.control_border_width,
                        shadow_width: 0.0,
                        clip: None, // overlay not clipped by parent
                        tier,
                    });
                    // Option labels.
                    for (i, opt) in options.iter().enumerate() {
                        draw_list.texts.push(TextCommand {
                            text: opt.clone(),
                            x: node.rect.x + node.padding.left,
                            y: list_y + i as f32 * row_h + node.padding.top,
                            color: *color,
                            font_size: *font_size,
                            font_family: FontFamily::default(),
                            clip: None,
                            tier,
                        });
                    }
                }
            }
            Widget::Slider {
                value,
                min,
                max,
                track_color,
                thumb_color,
                ..
            } => {
                let thumb_size = 16.0;
                let track_h = 4.0;
                let track_y = node.rect.y + (node.rect.height - track_h) / 2.0;
                // Track bar.
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: track_y,
                    width: node.rect.width,
                    height: track_h,
                    bg_color: *track_color,
                    border_color: [0.0; 4],
                    border_width: 0.0,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
                // Thumb.
                let range = (max - min).max(f32::EPSILON);
                let t = ((value - min) / range).clamp(0.0, 1.0);
                let thumb_x = node.rect.x + t * (node.rect.width - thumb_size);
                draw_list.panels.push(PanelCommand {
                    x: thumb_x,
                    y: node.rect.y,
                    width: thumb_size,
                    height: thumb_size,
                    bg_color: *thumb_color,
                    border_color: *track_color,
                    border_width: self.control_border_width,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
            }
            Widget::TextInput {
                text,
                cursor_pos,
                color,
                bg_color,
                font_size,
                placeholder,
                focused,
            } => {
                // Background.
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    bg_color: *bg_color,
                    border_color: *color,
                    border_width: self.control_border_width,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });
                // Text content or placeholder.
                let display_text = if text.is_empty() {
                    placeholder.clone()
                } else {
                    text.clone()
                };
                let text_color = if text.is_empty() {
                    // Placeholder: dimmed color.
                    [color[0], color[1], color[2], color[3] * 0.5]
                } else {
                    *color
                };
                draw_list.texts.push(TextCommand {
                    text: display_text,
                    x: node.rect.x + node.padding.left,
                    y: node.rect.y + node.padding.top,
                    color: text_color,
                    font_size: *font_size,
                    font_family: FontFamily::default(),
                    clip,
                    tier,
                });
                // Cursor line when focused.
                if *focused {
                    let text_before_cursor = &text[..(*cursor_pos).min(text.len())];
                    let cursor_offset = tm
                        .measure_text(text_before_cursor, FontFamily::default(), *font_size)
                        .width;
                    let cursor_x = node.rect.x + node.padding.left + cursor_offset;
                    let cursor_h = tm
                        .measure_text("M", FontFamily::default(), *font_size)
                        .height;
                    draw_list.panels.push(PanelCommand {
                        x: cursor_x,
                        y: node.rect.y + node.padding.top,
                        width: 1.0,
                        height: cursor_h,
                        bg_color: *color,
                        border_color: [0.0; 4],
                        border_width: 0.0,
                        shadow_width: 0.0,
                        clip,
                        tier,
                    });
                }
            }
            Widget::Collapsible {
                header,
                expanded,
                color,
                font_size,
            } => {
                let indicator_w = tm
                    .measure_text("\u{25BC}\u{25BC}", FontFamily::default(), *font_size)
                    .width;
                // Triangle indicator: ▶ collapsed, ▼ expanded.
                let indicator = if *expanded { "\u{25BC}" } else { "\u{25B6}" };
                draw_list.texts.push(TextCommand {
                    text: indicator.to_string(),
                    x: node.rect.x,
                    y: node.rect.y + 2.0,
                    color: *color,
                    font_size: *font_size,
                    font_family: FontFamily::default(),
                    clip,
                    tier,
                });
                // Header label, offset past the triangle.
                draw_list.texts.push(TextCommand {
                    text: header.clone(),
                    x: node.rect.x + indicator_w,
                    y: node.rect.y + 2.0,
                    color: *color,
                    font_size: *font_size,
                    font_family: FontFamily::default(),
                    clip,
                    tier,
                });
                if !*expanded {
                    return; // Skip children when collapsed.
                }
                // When expanded, fall through to the default child draw loop.
            }
            Widget::TabContainer {
                tabs,
                active,
                tab_color,
                active_color,
                font_size,
            } => {
                let m_size = tm.measure_text("M", FontFamily::default(), *font_size);
                let tab_pad = 8.0;
                let tab_bar_h = m_size.height + 6.0;

                // Draw tab buttons.
                let mut tab_x = node.rect.x;
                for (i, label) in tabs.iter().enumerate() {
                    let tab_w = tm
                        .measure_text(label, FontFamily::default(), *font_size)
                        .width
                        + tab_pad * 2.0;
                    let is_active = i == *active;
                    let bg = if is_active { *active_color } else { *tab_color };

                    // Tab background.
                    draw_list.panels.push(PanelCommand {
                        x: tab_x,
                        y: node.rect.y,
                        width: tab_w,
                        height: tab_bar_h,
                        bg_color: bg,
                        border_color: [0.0; 4],
                        border_width: 0.0,
                        shadow_width: 0.0,
                        clip,
                        tier,
                    });

                    // Tab label — dimmed for inactive tabs.
                    let text_color = if is_active {
                        [1.0, 1.0, 1.0, 1.0]
                    } else {
                        [1.0, 1.0, 1.0, 0.6]
                    };
                    draw_list.texts.push(TextCommand {
                        text: label.clone(),
                        x: tab_x + tab_pad,
                        y: node.rect.y + 3.0,
                        color: text_color,
                        font_size: *font_size,
                        font_family: FontFamily::default(),
                        clip,
                        tier,
                    });

                    tab_x += tab_w;
                }
                // Fall through to draw content children.
            }
            Widget::ScrollList {
                bg_color,
                border_color,
                border_width,
                item_height,
                scroll_offset,
                scrollbar_color,
                scrollbar_width,
                item_heights,
                empty_text,
            } => {
                // Background panel.
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    bg_color: *bg_color,
                    border_color: *border_color,
                    border_width: *border_width,
                    shadow_width: 0.0,
                    clip,
                    tier,
                });

                let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
                let n = node.children.len();
                let total_h = Self::scroll_total_height(item_heights, *item_height, n);
                let content_x = node.rect.x + node.padding.left;
                let content_y = node.rect.y + node.padding.top;
                let sb_w = *scrollbar_width;
                let sb_color = *scrollbar_color;
                let so = *scroll_offset;
                let ih = *item_height;
                let ihs = item_heights.clone();
                let empty_msg = empty_text.clone();
                let rect = node.rect;
                let padding = node.padding;
                let children: Vec<WidgetId> = node.children.clone();
                let content_w = (rect.width - padding.horizontal() - sb_w).max(0.0);
                let alt_alpha = self.scroll_row_alt_alpha;

                if children.is_empty() {
                    // Empty state: draw centered placeholder text.
                    if let Some(msg) = empty_msg {
                        draw_list.texts.push(TextCommand {
                            text: msg,
                            x: content_x + content_w * 0.5,
                            y: content_y + viewport_h * 0.5,
                            color: [0.5, 0.5, 0.5, 0.7],
                            font_size: 12.0,
                            font_family: FontFamily::Serif,
                            clip,
                            tier,
                        });
                    }
                } else {
                    // Alternating row tint + draw visible children.
                    let first = Self::scroll_first_visible(&ihs, ih, n, so);
                    for (idx, &child) in children.iter().enumerate() {
                        if let Some(cn) = self.arena.get(child)
                            && cn.rect.width > 0.0
                            && cn.rect.height > 0.0
                        {
                            // Alternating row background on odd items.
                            if idx % 2 == 1 && alt_alpha > 0.0 {
                                let item_y_abs = Self::scroll_item_y(&ihs, ih, idx);
                                let item_h = Self::scroll_item_h(&ihs, ih, idx);
                                let item_y = item_y_abs - so;
                                draw_list.panels.push(PanelCommand {
                                    x: content_x,
                                    y: content_y + item_y,
                                    width: content_w,
                                    height: item_h,
                                    bg_color: [0.0, 0.0, 0.0, alt_alpha],
                                    border_color: [0.0; 4],
                                    border_width: 0.0,
                                    shadow_width: 0.0,
                                    clip,
                                    tier,
                                });
                            }
                            self.draw_node(child, draw_list, tm, tier);
                        }
                    }
                    let _ = first; // used for skip logic in layout; draw uses rect check
                }

                // Scrollbar thumb (auto-hides when content fits).
                if total_h > viewport_h && viewport_h > 0.0 {
                    let thumb_ratio = viewport_h / total_h;
                    let thumb_h = (viewport_h * thumb_ratio).max(20.0); // min 20px
                    let track_range = viewport_h - thumb_h;
                    let max_scroll = total_h - viewport_h;
                    let thumb_y = if max_scroll > 0.0 {
                        content_y + (so / max_scroll) * track_range
                    } else {
                        content_y
                    };
                    let sb_x = rect.x + rect.width - sb_w - padding.right;

                    draw_list.panels.push(PanelCommand {
                        x: sb_x,
                        y: thumb_y,
                        width: sb_w,
                        height: thumb_h,
                        bg_color: sb_color,
                        border_color: [0.0; 4],
                        border_width: 0.0,
                        shadow_width: 0.0,
                        clip,
                        tier,
                    });
                }

                return; // ScrollList handles its own children.
            }
            Widget::ScrollView {
                scroll_offset,
                scrollbar_color,
                scrollbar_width,
            } => {
                // No background panel (transparent viewport).
                let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
                let rect = node.rect;
                let padding = node.padding;
                let sb_w = *scrollbar_width;
                let sb_color = *scrollbar_color;
                let so = *scroll_offset;
                let children: Vec<WidgetId> = node.children.clone();

                // Compute total content height from laid-out rects.
                let mut total_h: f32 = 0.0;
                for &child in &children {
                    if let Some(cn) = self.arena.get(child) {
                        total_h += cn.rect.height + cn.margin.vertical();
                    }
                }

                // Draw all children (GPU clipping hides overflow).
                for &child in &children {
                    self.draw_node(child, draw_list, tm, tier);
                }

                // Scrollbar thumb (auto-hides when content fits).
                // Track spans the full rect height (ignoring padding) so
                // the scrollbar can sit flush against the parent border.
                if total_h > viewport_h && viewport_h > 0.0 {
                    let track_h = rect.height;
                    let thumb_ratio = viewport_h / total_h;
                    let thumb_h = (track_h * thumb_ratio).max(Self::MIN_THUMB_HEIGHT);
                    let track_range = track_h - thumb_h;
                    let max_scroll = total_h - viewport_h;
                    let thumb_y = if max_scroll > 0.0 {
                        rect.y + (so / max_scroll) * track_range
                    } else {
                        rect.y
                    };
                    let sb_x = rect.x + rect.width - sb_w - padding.right;

                    // Scrollbar uses the ScrollView's own clip_rect, not the
                    // viewport clip, so it remains visible at all times.
                    draw_list.panels.push(PanelCommand {
                        x: sb_x,
                        y: thumb_y,
                        width: sb_w,
                        height: thumb_h,
                        bg_color: sb_color,
                        border_color: [0.0; 4],
                        border_width: 0.0,
                        shadow_width: 0.0,
                        clip,
                        tier,
                    });
                }

                return; // ScrollView handles its own children.
            }
        }

        // Draw children on top (non-ScrollList/ScrollView widgets).
        for &child in &node.children {
            self.draw_node(child, draw_list, tm, tier);
        }
    }

    // ------------------------------------------------------------------
    // ScrollList helpers
    // ------------------------------------------------------------------

    /// Minimum scrollbar thumb height in pixels.
    const MIN_THUMB_HEIGHT: f32 = 20.0;

    // Variable-height helpers (UI-501).
    // When `item_heights` is empty or shorter than the queried index,
    // these fall back to the fixed `item_height` value.
    // Public for use by input.rs.

    /// Cumulative Y offset for the item at `index`.
    pub fn scroll_item_y(item_heights: &[f32], item_height: f32, index: usize) -> f32 {
        let mut y = 0.0;
        for i in 0..index {
            y += if i < item_heights.len() {
                item_heights[i]
            } else {
                item_height
            };
        }
        y
    }

    /// Height of the item at `index`.
    pub fn scroll_item_h(item_heights: &[f32], item_height: f32, index: usize) -> f32 {
        if index < item_heights.len() {
            item_heights[index]
        } else {
            item_height
        }
    }

    /// Total content height for `count` items.
    pub fn scroll_total_height(item_heights: &[f32], item_height: f32, count: usize) -> f32 {
        let mut total = 0.0;
        for i in 0..count {
            total += if i < item_heights.len() {
                item_heights[i]
            } else {
                item_height
            };
        }
        total
    }

    /// Index of the first visible item given scroll offset.
    pub fn scroll_first_visible(
        item_heights: &[f32],
        item_height: f32,
        count: usize,
        offset: f32,
    ) -> usize {
        let mut y = 0.0;
        for i in 0..count {
            let h = if i < item_heights.len() {
                item_heights[i]
            } else {
                item_height
            };
            if y + h > offset {
                return i;
            }
            y += h;
        }
        count
    }

    /// Compute maximum scroll offset for a ScrollList or ScrollView.
    /// Returns 0.0 if content fits in viewport.
    pub fn max_scroll(&self, id: WidgetId) -> f32 {
        let Some(node) = self.arena.get(id) else {
            return 0.0;
        };
        let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
        match &node.widget {
            Widget::ScrollList {
                item_height,
                item_heights,
                ..
            } => {
                let n = node.children.len();
                let total_h = Self::scroll_total_height(item_heights, *item_height, n);
                (total_h - viewport_h).max(0.0)
            }
            Widget::ScrollView { .. } => {
                // Use laid-out rect heights (not measured intrinsic heights) so
                // fixed-size children like ScrollList contribute their actual
                // display height, not their full content height.
                let mut total_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        total_h += child.rect.height + child.margin.vertical();
                    }
                }
                (total_h - viewport_h).max(0.0)
            }
            _ => 0.0,
        }
    }

    /// Set scroll offset for a ScrollList or ScrollView, clamped to valid range.
    pub fn set_scroll_offset(&mut self, id: WidgetId, offset: f32) {
        let max = self.max_scroll(id);
        if let Some(node) = self.arena.get_mut(id) {
            match &mut node.widget {
                Widget::ScrollList { scroll_offset, .. }
                | Widget::ScrollView { scroll_offset, .. } => {
                    *scroll_offset = offset.clamp(0.0, max);
                }
                _ => {}
            }
        }
        self.mark_dirty(id);
    }

    /// Read current scroll offset for a ScrollList or ScrollView.
    pub fn scroll_offset(&self, id: WidgetId) -> f32 {
        self.arena
            .get(id)
            .and_then(|n| match &n.widget {
                Widget::ScrollList { scroll_offset, .. }
                | Widget::ScrollView { scroll_offset, .. } => Some(*scroll_offset),
                _ => None,
            })
            .unwrap_or(0.0)
    }

    /// Scroll a ScrollList or ScrollView by a delta (positive = down).
    pub fn scroll_by(&mut self, id: WidgetId, delta: f32) {
        let current = self
            .arena
            .get(id)
            .and_then(|n| match &n.widget {
                Widget::ScrollList { scroll_offset, .. }
                | Widget::ScrollView { scroll_offset, .. } => Some(*scroll_offset),
                _ => None,
            })
            .unwrap_or(0.0);
        self.set_scroll_offset(id, current + delta);
    }

    /// Scroll to make a specific child visible by index.
    pub fn ensure_visible(&mut self, id: WidgetId, child_index: usize) {
        let Some(node) = self.arena.get(id) else {
            return;
        };
        let Widget::ScrollList {
            item_height,
            scroll_offset,
            item_heights,
            ..
        } = &node.widget
        else {
            return;
        };
        let ih = *item_height;
        let ihs = item_heights.clone();
        let so = *scroll_offset;
        let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
        if viewport_h <= 0.0 {
            return;
        }

        let item_top = Self::scroll_item_y(&ihs, ih, child_index);
        let item_h = Self::scroll_item_h(&ihs, ih, child_index);
        let item_bottom = item_top + item_h;

        let new_offset = if item_top < so {
            item_top
        } else if item_bottom > so + viewport_h {
            item_bottom - viewport_h
        } else {
            return; // already visible
        };

        self.set_scroll_offset(id, new_offset);
    }

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

// ---------------------------------------------------------------------------
// Text wrapping helper (UI-102)
// ---------------------------------------------------------------------------

/// Break `text` into lines of at most `max_chars` characters, splitting at word boundaries.
/// If a single word exceeds `max_chars`, it is placed on its own line (no mid-word break).
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current_line));
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Compute the number of wrapped lines for a label, given resolved width and font metrics.
fn wrapped_line_count(text: &str, width: f32, char_w: f32) -> usize {
    let max_chars = (width / char_w).max(1.0) as usize;
    wrap_text(text, max_chars).len()
}

/// Build the status bar panel at the top of the screen (UI-I01a).
///
/// Chrome panel: permanent, rebuilt every frame with live simulation data.
/// Replaces the old string-based `render::render_status()`.
///
/// Returns the root panel's `WidgetId` so the caller can read its
/// computed height after layout (via `WidgetTree::node_rect`).
/// Status bar configuration for pause/speed display (UI-I03).
pub struct StatusBarInfo<'a> {
    pub tick: u64,
    pub date: String,
    pub population: usize,
    pub is_turn_based: bool,
    pub player_name: Option<&'a str>,
    pub paused: bool,
    pub sim_speed: u32,
    pub keybindings: &'a KeyBindings,
    pub screen_width: f32,
    /// Previous frame's perf metrics (UI-505). None = don't display.
    pub perf: Option<UiPerfMetrics>,
}

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

pub fn build_status_bar(tree: &mut WidgetTree, theme: &Theme, info: &StatusBarInfo) -> WidgetId {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.status_bar_bg,
        border_color: theme.panel_border_color,
        border_width: theme.tooltip_border_width, // thin 1px border
        shadow_width: 0.0,                        // no shadow for a flat bar
    });
    tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
    tree.set_sizing(panel, Sizing::Fixed(info.screen_width), Sizing::Fit);
    tree.set_padding(
        panel,
        Edges {
            top: theme.status_bar_padding_v,
            right: theme.status_bar_padding_h,
            bottom: theme.status_bar_padding_v,
            left: theme.status_bar_padding_h,
        },
    );

    let sep = || TextSpan {
        text: "  |  ".to_string(),
        color: theme.disabled,
        font_family: FontFamily::Mono,
    };

    let mut spans = vec![
        TextSpan {
            text: info.date.clone(),
            color: theme.gold,
            font_family: FontFamily::Mono,
        },
        sep(),
        TextSpan {
            text: format!("Pop: {}", info.population),
            color: theme.text_light,
            font_family: FontFamily::Mono,
        },
        sep(),
    ];

    if info.is_turn_based {
        spans.push(TextSpan {
            text: "TURN-BASED".to_string(),
            color: theme.gold,
            font_family: FontFamily::Mono,
        });
    } else if info.paused {
        // Show "PAUSED (Space)" with keybinding label.
        let pause_label = info
            .keybindings
            .label_for(keybindings::Action::PauseSim)
            .map(|k| format!("PAUSED ({k})"))
            .unwrap_or_else(|| "PAUSED".to_string());
        spans.push(TextSpan {
            text: pause_label,
            color: theme.danger,
            font_family: FontFamily::Mono,
        });
    } else {
        // Show speed indicator: "Speed: 1x (1)" through "Speed: 5x (5)"
        let speed_label = info
            .keybindings
            .label_for(keybindings::Action::SpeedSet(info.sim_speed))
            .map(|k| format!("Speed: {}x ({k})", info.sim_speed))
            .unwrap_or_else(|| format!("Speed: {}x", info.sim_speed));
        let color = if info.sim_speed > 1 {
            theme.gold
        } else {
            theme.text_light
        };
        spans.push(TextSpan {
            text: speed_label,
            color,
            font_family: FontFamily::Mono,
        });
    }

    if let Some(name) = info.player_name {
        spans.push(sep());
        spans.push(TextSpan {
            text: format!("@{name}"),
            color: theme.gold,
            font_family: FontFamily::Mono,
        });
    }

    // Perf metrics (UI-505): right-aligned debug info from previous frame.
    if let Some(perf) = &info.perf {
        spans.push(sep());
        spans.push(TextSpan {
            text: format!(
                "UI: build {:.1}ms | layout {:.1}ms | draw {:.1}ms | render {:.1}ms | {}w",
                perf.build_us as f64 / 1000.0,
                perf.layout_us as f64 / 1000.0,
                perf.draw_us as f64 / 1000.0,
                perf.render_us as f64 / 1000.0,
                perf.widget_count,
            ),
            color: theme.disabled,
            font_family: FontFamily::Mono,
        });
    }

    tree.insert(
        panel,
        Widget::RichText {
            spans,
            font_size: theme.font_data_size,
        },
    );

    panel
}

// ---------------------------------------------------------------------------
// Hover tooltip (UI-I01b)
// ---------------------------------------------------------------------------

/// Data for the map hover tooltip (UI-I01b).
/// Extracted from World in main.rs, consumed by `build_hover_tooltip`.
pub struct HoverInfo {
    pub tile_x: i32,
    pub tile_y: i32,
    pub terrain: String,
    pub quartier: Option<String>,
    pub address: Option<String>,
    pub building_name: Option<String>,
    /// (name, activity) pairs for building occupants.
    pub occupants: Vec<(String, String)>,
    /// Year suffix like "[1842]" if data is from a fallback year.
    pub occupant_year_suffix: Option<String>,
    /// (icon_char, name) pairs for alive entities on this tile.
    pub entities: Vec<(char, String)>,
}

/// Maximum number of occupants shown in the hover tooltip.
const HOVER_MAX_OCCUPANTS: usize = 5;

/// Build a hover tooltip panel for the hovered map tile (UI-I01b).
///
/// Created on demand when cursor is over a map tile, destroyed when
/// cursor leaves (per DD-5). Styled like a W04 tooltip panel.
/// Replaces the old string-based `render::render_hover_info()`.
///
/// Returns the root panel's `WidgetId`.
pub fn build_hover_tooltip(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &HoverInfo,
    cursor: (f32, f32),
    screen: Size,
    tm: &mut dyn TextMeasurer,
) -> WidgetId {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.tooltip_bg_color,
        border_color: theme.tooltip_border_color,
        border_width: theme.tooltip_border_width,
        shadow_width: theme.tooltip_shadow_width,
    });
    tree.set_sizing(panel, Sizing::Fit, Sizing::Fit);
    tree.set_padding(panel, Edges::all(theme.tooltip_padding));

    let mut y = 0.0_f32;
    let data_h = theme.font_data_size;
    let body_h = theme.font_body_size;
    let gap = theme.label_gap;

    // Line 1: coordinates + terrain type
    let coord_line = tree.insert(
        panel,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: format!("({}, {})", info.tile_x, info.tile_y),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: format!("  {}", info.terrain),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
            ],
            font_size: theme.font_data_size,
        },
    );
    tree.set_position(coord_line, Position::Fixed { x: 0.0, y });
    y += data_h + gap;

    // Line 2: quartier name (optional)
    if let Some(ref quartier) = info.quartier {
        let q = tree.insert(
            panel,
            Widget::Label {
                text: quartier.clone(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
                wrap: false,
            },
        );
        tree.set_position(q, Position::Fixed { x: 0.0, y });
        y += data_h + gap;
    }

    // Line 3: address + building name (optional)
    if let Some(ref address) = info.address {
        let mut spans = vec![TextSpan {
            text: address.clone(),
            color: theme.text_light,
            font_family: FontFamily::Serif,
        }];
        if let Some(ref name) = info.building_name {
            spans.push(TextSpan {
                text: " \u{2014} ".to_string(),
                color: theme.disabled,
                font_family: FontFamily::Serif,
            });
            spans.push(TextSpan {
                text: name.clone(),
                color: theme.gold,
                font_family: FontFamily::Serif,
            });
        }
        let addr_line = tree.insert(
            panel,
            Widget::RichText {
                spans,
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(addr_line, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Occupants section
    if !info.occupants.is_empty() {
        let show_count = info.occupants.len().min(HOVER_MAX_OCCUPANTS);
        for (name, activity) in &info.occupants[..show_count] {
            let occ = tree.insert(
                panel,
                Widget::RichText {
                    spans: vec![
                        TextSpan {
                            text: name.clone(),
                            color: theme.text_light,
                            font_family: FontFamily::Mono,
                        },
                        TextSpan {
                            text: format!(" ({})", activity),
                            color: theme.disabled,
                            font_family: FontFamily::Mono,
                        },
                    ],
                    font_size: theme.font_data_size,
                },
            );
            tree.set_position(occ, Position::Fixed { x: 0.0, y });
            y += data_h + gap;
        }
        if info.occupants.len() > HOVER_MAX_OCCUPANTS {
            let more = tree.insert(
                panel,
                Widget::Label {
                    text: format!("+{} more", info.occupants.len() - HOVER_MAX_OCCUPANTS),
                    color: theme.disabled,
                    font_size: theme.font_data_size,
                    font_family: theme.font_data_family,
                    wrap: false,
                },
            );
            tree.set_position(more, Position::Fixed { x: 0.0, y });
            y += data_h + gap;
        }
        if let Some(ref suffix) = info.occupant_year_suffix {
            let yr = tree.insert(
                panel,
                Widget::Label {
                    text: suffix.clone(),
                    color: theme.disabled,
                    font_size: theme.font_data_size,
                    font_family: theme.font_data_family,
                    wrap: false,
                },
            );
            tree.set_position(yr, Position::Fixed { x: 0.0, y });
            // y += data_h + gap; // last line, no trailing gap needed
        }
    }

    // Entities section
    if !info.entities.is_empty() {
        for (icon, name) in &info.entities {
            let ent = tree.insert(
                panel,
                Widget::RichText {
                    spans: vec![
                        TextSpan {
                            text: format!("{} ", icon),
                            color: theme.gold,
                            font_family: FontFamily::Mono,
                        },
                        TextSpan {
                            text: name.clone(),
                            color: theme.text_light,
                            font_family: FontFamily::Mono,
                        },
                    ],
                    font_size: theme.font_data_size,
                },
            );
            tree.set_position(ent, Position::Fixed { x: 0.0, y });
            y += data_h + gap;
        }
    }

    // Position: below-right of cursor, edge-flip if clipping screen.
    let measured = tree.measure_node(panel, tm);
    let tooltip_w = measured.width + theme.tooltip_padding * 2.0;
    let tooltip_h = measured.height + theme.tooltip_padding * 2.0;

    let (tx, ty) = UiState::compute_tooltip_position(
        cursor,
        Size {
            width: tooltip_w,
            height: tooltip_h,
        },
        screen,
        0,
        theme,
    );
    tree.set_position(panel, Position::Fixed { x: tx, y: ty });

    panel
}

// ---------------------------------------------------------------------------
// Event log (UI-I01c)
// ---------------------------------------------------------------------------

/// Event data for the event log panel (UI-I01c).
/// Extracted from World in main.rs, consumed by `build_event_log`.
pub enum EventLogEntry {
    Spawned {
        name: String,
    },
    Died {
        name: String,
    },
    Ate {
        name: String,
        food_name: String,
    },
    Attacked {
        attacker: String,
        defender: String,
        damage: f32,
    },
}

/// Maximum significant events kept in the ScrollList.
const EVENT_LOG_MAX_ENTRIES: usize = 50;

/// Build the event log panel at the bottom of the screen (UI-I01c).
///
/// Chrome panel: permanent, rebuilt every frame with live event data.
/// Replaces the old string-based `render::render_recent_events()`.
///
/// Returns the root ScrollList's `WidgetId`. The caller should set its
/// position after computing available screen space, then call
/// `WidgetTree::layout` to finalize.
pub fn build_event_log(
    tree: &mut WidgetTree,
    theme: &Theme,
    entries: &[EventLogEntry],
    screen_width: f32,
    panel_height: f32,
) -> WidgetId {
    let pad_v = theme.status_bar_padding_v;
    let pad_h = theme.status_bar_padding_h;
    let viewport_h = (panel_height - pad_v * 2.0).max(0.0);
    let total_h = entries.len() as f32 * theme.scroll_item_height;
    let auto_scroll = (total_h - viewport_h).max(0.0);

    let list = tree.insert_root(Widget::ScrollList {
        bg_color: theme.status_bar_bg,
        border_color: theme.panel_border_color,
        border_width: theme.tooltip_border_width,
        item_height: theme.scroll_item_height,
        scroll_offset: auto_scroll,
        scrollbar_color: theme.scrollbar_color,
        scrollbar_width: theme.scrollbar_width,
        item_heights: Vec::new(),
        empty_text: None,
    });
    tree.set_sizing(
        list,
        Sizing::Fixed(screen_width),
        Sizing::Fixed(panel_height),
    );
    tree.set_padding(
        list,
        Edges {
            top: pad_v,
            right: pad_h,
            bottom: pad_v,
            left: pad_h,
        },
    );

    for entry in entries {
        let spans = match entry {
            EventLogEntry::Spawned { name } => vec![
                TextSpan {
                    text: name.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " spawned".to_string(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                },
            ],
            EventLogEntry::Died { name } => vec![
                TextSpan {
                    text: name.clone(),
                    color: theme.danger,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " died".to_string(),
                    color: theme.danger,
                    font_family: FontFamily::Mono,
                },
            ],
            EventLogEntry::Ate { name, food_name } => vec![
                TextSpan {
                    text: name.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " ate ".to_string(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: food_name.clone(),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
            ],
            EventLogEntry::Attacked {
                attacker,
                defender,
                damage,
            } => vec![
                TextSpan {
                    text: attacker.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " attacks ".to_string(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: defender.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: format!(" ({:.0} dmg)", damage),
                    color: theme.danger,
                    font_family: FontFamily::Mono,
                },
            ],
        };
        tree.insert(
            list,
            Widget::RichText {
                spans,
                font_size: theme.font_data_size,
            },
        );
    }

    list
}

/// Collect significant events from World into `EventLogEntry` structs.
///
/// Filters to Spawned/Died/Ate/Attacked (skips Moved, HungerChanged).
/// Returns up to `EVENT_LOG_MAX_ENTRIES` entries, newest last.
pub fn collect_event_entries(
    events: &crate::events::EventLog,
    names: &std::collections::HashMap<crate::components::Entity, crate::components::Name>,
) -> Vec<EventLogEntry> {
    use crate::events::Event;

    let resolve = |e: &crate::components::Entity| -> String {
        names
            .get(e)
            .map(|n| n.value.clone())
            .unwrap_or_else(|| format!("E{}", e.0))
    };

    let raw = events.recent(EVENT_LOG_MAX_ENTRIES * 10);
    let mut entries = Vec::new();

    for event in raw {
        let entry = match event {
            Event::Spawned { entity, .. } => EventLogEntry::Spawned {
                name: resolve(entity),
            },
            Event::Died { entity, .. } => EventLogEntry::Died {
                name: resolve(entity),
            },
            Event::Ate { entity, food, .. } => EventLogEntry::Ate {
                name: resolve(entity),
                food_name: resolve(food),
            },
            Event::Attacked {
                attacker,
                defender,
                damage,
                ..
            } => EventLogEntry::Attacked {
                attacker: resolve(attacker),
                defender: resolve(defender),
                damage: *damage,
            },
            Event::Moved { .. } | Event::HungerChanged { .. } => continue,
        };
        entries.push(entry);
        if entries.len() >= EVENT_LOG_MAX_ENTRIES {
            break;
        }
    }

    entries
}

// ---------------------------------------------------------------------------
// Entity inspector (UI-I01d)
// ---------------------------------------------------------------------------

/// Data for the entity inspector panel (UI-I01d).
/// Extracted from World by `collect_inspector_info`, consumed by
/// `build_entity_inspector`. Plain struct — no references to World.
pub struct EntityInspectorInfo {
    pub name: String,
    pub icon: char,
    pub position: (i32, i32),
    pub health: Option<(f32, f32)>, // (current, max)
    pub hunger: Option<(f32, f32)>, // (current, max)
    pub fatigue: Option<f32>,
    pub combat: Option<(f32, f32, f32)>, // (atk, def, aggression)
    pub action: Option<String>,          // "Idle", "Wandering", etc.
    pub gait: Option<String>,            // "Walk", "Run", etc.
}

/// Collect inspector data for an entity. Returns `None` if the entity is
/// not alive or has no position (same pattern as `collect_event_entries`).
pub fn collect_inspector_info(
    entity: crate::components::Entity,
    world: &crate::world::World,
) -> Option<EntityInspectorInfo> {
    if !world.alive.contains(&entity) {
        return None;
    }
    let pos = world.body.positions.get(&entity)?;

    let name = world
        .body
        .names
        .get(&entity)
        .map(|n| n.value.clone())
        .unwrap_or_else(|| format!("E{}", entity.0));
    let icon = world.body.icons.get(&entity).map(|i| i.ch).unwrap_or('?');

    let health = world.body.healths.get(&entity).map(|h| (h.current, h.max));
    let hunger = world.mind.hungers.get(&entity).map(|h| (h.current, h.max));
    let fatigue = world.body.fatigues.get(&entity).map(|f| f.current);
    let combat = world
        .body
        .combat_stats
        .get(&entity)
        .map(|c| (c.attack, c.defense, c.aggression));

    let action = world
        .mind
        .action_states
        .get(&entity)
        .and_then(|a| a.current_action.as_ref().map(|id| format!("{:?}", id)));

    let gait = world
        .body
        .current_gaits
        .get(&entity)
        .map(|g| format!("{:?}", g));

    Some(EntityInspectorInfo {
        name,
        icon,
        position: (pos.x, pos.y),
        health,
        hunger,
        fatigue,
        combat,
        action,
        gait,
    })
}

/// Inspector panel width in pixels.
const INSPECTOR_WIDTH: f32 = 220.0;

/// Build the entity inspector panel (UI-I01d).
///
/// Right-aligned panel showing entity stats. Returns `(panel_id, close_button_id)`.
/// The caller positions the panel and uses `close_button_id` to detect close clicks.
pub fn build_entity_inspector(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &EntityInspectorInfo,
) -> (WidgetId, WidgetId) {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.tooltip_bg_color,
        border_color: theme.tooltip_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    let inspector_w = theme.s(INSPECTOR_WIDTH);
    tree.set_sizing(panel, Sizing::Fixed(inspector_w), Sizing::Fit);
    tree.set_padding(panel, Edges::all(theme.panel_padding));

    let content_w = inspector_w - theme.panel_padding * 2.0;
    let mut y = 0.0_f32;
    let data_h = theme.font_data_size;
    let body_h = theme.font_body_size;
    let header_h = theme.font_header_size;
    let gap = theme.label_gap;

    // Header row: [icon+name, Expand, close button] (UI-601).
    let header_row = tree.insert(
        panel,
        Widget::Row {
            gap: theme.label_gap,
            align: widget::CrossAlign::Center,
        },
    );
    tree.set_sizing(header_row, Sizing::Fixed(content_w), Sizing::Fit);
    tree.set_position(header_row, Position::Fixed { x: 0.0, y: 0.0 });

    tree.insert(
        header_row,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: format!("{} ", info.icon),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: info.name.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
            ],
            font_size: theme.font_header_size,
        },
    );
    tree.insert(header_row, Widget::Expand);
    let close_btn = tree.insert(
        header_row,
        Widget::Button {
            text: "X".to_string(),
            color: theme.danger,
            bg_color: [0.0, 0.0, 0.0, 0.0], // transparent
            border_color: theme.danger,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
        },
    );
    y += header_h + gap;

    // Position: "(x, y)" in disabled/mono at data size
    let pos_label = tree.insert(
        panel,
        Widget::Label {
            text: format!("({}, {})", info.position.0, info.position.1),
            color: theme.disabled,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );
    tree.set_position(pos_label, Position::Fixed { x: 0.0, y });
    y += data_h + gap;

    // Separator gap
    y += gap;

    // Helper: pick color by severity ratio (current/max).
    let severity_color = |ratio: f32| -> [f32; 4] {
        if ratio > 0.5 {
            theme.text_light
        } else if ratio > 0.25 {
            theme.gold
        } else {
            theme.danger
        }
    };

    // Health
    if let Some((cur, max)) = info.health {
        let ratio = if max > 0.0 { cur / max } else { 0.0 };
        let hp = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "HP ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}/{:.0}", cur, max),
                        color: severity_color(ratio),
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(hp, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Hunger
    if let Some((cur, max)) = info.hunger {
        let ratio = if max > 0.0 { cur / max } else { 0.0 };
        let hunger = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "Hunger ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}/{:.0}", cur, max),
                        color: severity_color(ratio),
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(hunger, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Fatigue
    if let Some(fat) = info.fatigue {
        let fat_label = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "Fatigue ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.1}", fat),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(fat_label, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Combat stats
    if let Some((atk, def, agg)) = info.combat {
        let combat = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "ATK ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}", atk),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: "  DEF ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}", def),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: "  AGG ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.1}", agg),
                        color: theme.gold,
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_data_size,
            },
        );
        tree.set_position(combat, Position::Fixed { x: 0.0, y });
        y += data_h + gap;
    }

    // Action
    if let Some(ref action) = info.action {
        let act = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "Action ".to_string(),
                        color: theme.gold,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: action.clone(),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(act, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Gait
    if let Some(ref gait) = info.gait {
        let gait_label = tree.insert(
            panel,
            Widget::Label {
                text: gait.clone(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        tree.set_position(gait_label, Position::Fixed { x: 0.0, y });
        // y += data_h + gap; // last line, no trailing gap needed
    }

    (panel, close_btn)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
    fn dirty_propagation() {
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
                text: "X".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        // Clear dirty flags via layout.
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );
        assert!(!tree.get(root).expect("root").dirty);
        assert!(!tree.get(child).expect("child").dirty);

        // Mark child dirty — should propagate to root.
        tree.mark_dirty(child);
        assert!(tree.get(child).expect("child").dirty);
        assert!(tree.get(root).expect("root").dirty);
    }

    #[test]
    fn layout_fixed_position() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.5, 0.5, 0.5, 1.0],
            border_color: [1.0, 1.0, 0.0, 1.0],
            border_width: 2.0,
            shadow_width: 4.0,
        });
        tree.set_position(panel, Position::Fixed { x: 20.0, y: 30.0 });
        tree.set_sizing(panel, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        tree.set_padding(panel, Edges::all(10.0));

        let label = tree.insert(
            panel,
            Widget::Label {
                text: "Hello".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        tree.set_position(label, Position::Fixed { x: 0.0, y: 0.0 });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let panel_rect = tree.get(panel).expect("panel").rect;
        assert!((panel_rect.x - 20.0).abs() < 0.01);
        assert!((panel_rect.y - 30.0).abs() < 0.01);
        assert!((panel_rect.width - 200.0).abs() < 0.01);
        assert!((panel_rect.height - 100.0).abs() < 0.01);

        // Label inside panel's content area (offset by padding).
        let label_rect = tree.get(label).expect("label").rect;
        assert!((label_rect.x - 30.0).abs() < 0.01); // 20 + 10 padding
        assert!((label_rect.y - 40.0).abs() < 0.01); // 30 + 10 padding
    }

    #[test]
    fn layout_percent_sizing() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_sizing(panel, Sizing::Percent(0.5), Sizing::Percent(0.25));

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.get(panel).expect("panel").rect;
        assert!((rect.width - 400.0).abs() < 0.01);
        assert!((rect.height - 150.0).abs() < 0.01);
    }

    #[test]
    fn draw_list_output() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.5, 0.5, 0.5, 0.9],
            border_color: [1.0, 0.8, 0.2, 1.0],
            border_width: 2.0,
            shadow_width: 6.0,
        });
        tree.set_position(panel, Position::Fixed { x: 10.0, y: 10.0 });
        tree.set_sizing(panel, Sizing::Fixed(260.0), Sizing::Fixed(120.0));
        tree.set_padding(panel, Edges::all(12.0));

        let _label = tree.insert(
            panel,
            Widget::Label {
                text: "Gold Header".into(),
                color: [0.78, 0.66, 0.31, 1.0],
                font_size: 16.0,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.texts.len(), 1);
        assert!((dl.panels[0].border_width - 2.0).abs() < 0.01);
        assert_eq!(dl.texts[0].text, "Gold Header");
        assert_eq!(dl.texts[0].font_family, FontFamily::Serif);
    }

    #[test]
    fn demo_tree_uses_theme() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = demo::DemoLiveData {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        demo::build_demo(&mut tree, &theme, &kb, &live, screen, 0.0);
        tree.layout(screen, &mut HeuristicMeasurer);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Root panel uses theme parchment background.
        assert!(!dl.panels.is_empty());
        assert_eq!(dl.panels[0].bg_color, theme.bg_parchment);
        assert_eq!(dl.panels[0].border_color, theme.panel_border_color);
        assert!((dl.panels[0].border_width - theme.panel_border_width).abs() < 0.01);

        // Demo uses all theme font families and sizes.
        assert!(
            dl.texts
                .iter()
                .any(|t| t.font_family == theme.font_header_family
                    && (t.font_size - theme.font_header_size).abs() < 0.01)
        );
        assert!(
            dl.texts
                .iter()
                .any(|t| t.font_family == theme.font_body_family
                    && (t.font_size - theme.font_body_size).abs() < 0.01)
        );
        assert!(dl.texts.iter().any(|t| t.color == theme.danger));
    }

    #[test]
    fn draw_list_multi_font() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.5, 0.5, 0.5, 0.9],
            border_color: [1.0, 0.8, 0.2, 1.0],
            border_width: 2.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(panel, Sizing::Fixed(400.0), Sizing::Fixed(200.0));
        tree.set_padding(panel, Edges::all(8.0));

        // Serif label
        let _serif = tree.insert(
            panel,
            Widget::Label {
                text: "Serif Text".into(),
                color: [1.0; 4],
                font_size: 16.0,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );

        // Mono label
        let mono = tree.insert(
            panel,
            Widget::Label {
                text: "Mono Data".into(),
                color: [0.8, 0.8, 0.8, 1.0],
                font_size: 9.0,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        tree.set_position(mono, Position::Fixed { x: 0.0, y: 20.0 });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Panel + two text commands
        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.texts.len(), 2);

        // Verify font families are preserved in draw commands
        assert_eq!(dl.texts[0].font_family, FontFamily::Serif);
        assert_eq!(dl.texts[0].text, "Serif Text");
        assert!((dl.texts[0].font_size - 16.0).abs() < 0.01);

        assert_eq!(dl.texts[1].font_family, FontFamily::Mono);
        assert_eq!(dl.texts[1].text, "Mono Data");
        assert!((dl.texts[1].font_size - 9.0).abs() < 0.01);
    }

    #[test]
    fn rich_text_draw_command() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 2.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 10.0, y: 10.0 });
        tree.set_sizing(panel, Sizing::Fixed(400.0), Sizing::Fixed(100.0));
        tree.set_padding(panel, Edges::all(8.0));

        let gold = [0.78, 0.66, 0.31, 1.0];
        let white = [1.0, 1.0, 1.0, 1.0];

        let rich = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "Name: ".into(),
                        color: white,
                        font_family: FontFamily::Serif,
                    },
                    TextSpan {
                        text: "Jean Valjean".into(),
                        color: gold,
                        font_family: FontFamily::Serif,
                    },
                ],
                font_size: 12.0,
            },
        );
        tree.set_position(rich, Position::Fixed { x: 0.0, y: 0.0 });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.texts.len(), 0);
        assert_eq!(dl.rich_texts.len(), 1);

        let cmd = &dl.rich_texts[0];
        assert_eq!(cmd.spans.len(), 2);
        assert_eq!(cmd.spans[0].text, "Name: ");
        assert_eq!(cmd.spans[0].color, white);
        assert_eq!(cmd.spans[0].font_family, FontFamily::Serif);
        assert_eq!(cmd.spans[1].text, "Jean Valjean");
        assert_eq!(cmd.spans[1].color, gold);
        assert!((cmd.font_size - 12.0).abs() < 0.01);
        // Position = panel (10,10) + padding (8,8)
        assert!((cmd.x - 18.0).abs() < 0.01);
        assert!((cmd.y - 18.0).abs() < 0.01);
    }

    #[test]
    fn rich_text_measure() {
        let mut tree = WidgetTree::new();
        let rich = tree.insert_root(Widget::RichText {
            spans: vec![
                TextSpan {
                    text: "Hello ".into(),
                    color: [1.0; 4],
                    font_family: FontFamily::Serif,
                },
                TextSpan {
                    text: "World".into(),
                    color: [0.8; 4],
                    font_family: FontFamily::Mono,
                },
            ],
            font_size: 14.0,
        });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let node = tree.get(rich).expect("rich text exists");
        // 11 total chars ("Hello " + "World"), intrinsic width > 0
        assert!(node.measured.width > 0.0);
        assert!(node.measured.height > 0.0);
    }

    #[test]
    fn rich_text_empty_spans() {
        let mut tree = WidgetTree::new();
        let rich = tree.insert_root(Widget::RichText {
            spans: vec![],
            font_size: 12.0,
        });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        assert_eq!(dl.rich_texts.len(), 1);
        assert!(dl.rich_texts[0].spans.is_empty());

        let node = tree.get(rich).expect("exists");
        assert!((node.measured.width - 0.0).abs() < 0.01);
    }

    #[test]
    fn demo_tree_includes_rich_text() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = demo::DemoLiveData {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        demo::build_demo(&mut tree, &theme, &kb, &live, screen, 0.0);
        tree.layout(screen, &mut HeuristicMeasurer);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Demo has rich text blocks (title, population, live data, etc.).
        assert!(!dl.rich_texts.is_empty());

        // Find the "Population:" rich text.
        let pop_rt = dl
            .rich_texts
            .iter()
            .find(|rt| rt.spans.iter().any(|s| s.text == "Population: "));
        assert!(pop_rt.is_some(), "should have Population rich text");
        let rt = pop_rt.unwrap();
        assert_eq!(rt.spans[0].text, "Population: ");
        assert_eq!(rt.spans[0].font_family, FontFamily::Serif);
        assert_eq!(rt.spans[1].text, "1,034,196");
        assert_eq!(rt.spans[1].font_family, FontFamily::Mono);
        assert_eq!(rt.spans[1].color, theme.gold);
        assert_eq!(rt.spans[2].text, " souls");
        assert!((rt.font_size - theme.font_body_size).abs() < 0.01);
    }

    // ------------------------------------------------------------------
    // ScrollList tests (UI-W03)
    // ------------------------------------------------------------------

    /// Helper: build a ScrollList with N items.
    fn scroll_list_tree(n: usize) -> (WidgetTree, WidgetId) {
        let mut tree = WidgetTree::new();
        let list = tree.insert_root(Widget::ScrollList {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 1.0,
            item_height: 20.0,
            scroll_offset: 0.0,
            scrollbar_color: [0.8, 0.6, 0.3, 0.5],
            scrollbar_width: 6.0,
            item_heights: Vec::new(),
            empty_text: None,
        });
        tree.set_position(list, Position::Fixed { x: 0.0, y: 0.0 });
        // 100px tall viewport = 5 visible items at 20px each.
        tree.set_sizing(list, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        tree.set_padding(list, Edges::all(0.0));

        for i in 0..n {
            tree.insert(
                list,
                Widget::Label {
                    text: format!("Item {}", i),
                    color: [1.0; 4],
                    font_size: 12.0,
                    font_family: FontFamily::Mono,
                    wrap: false,
                },
            );
        }

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );
        (tree, list)
    }

    #[test]
    fn scroll_list_layout_vertical_stack() {
        let (tree, list) = scroll_list_tree(10);
        let node = tree.get(list).unwrap();
        let children = &node.children;

        // First 5 items are visible (viewport 100px / item_height 20px).
        for i in 0..5 {
            let child = tree.get(children[i]).unwrap();
            assert!(child.rect.width > 0.0, "item {} should be visible", i);
            assert!(
                (child.rect.y - (i as f32 * 20.0)).abs() < 0.01,
                "item {} y = {}, expected {}",
                i,
                child.rect.y,
                i as f32 * 20.0
            );
        }

        // Items 5-9 are outside viewport — should have zero rects.
        for i in 5..10 {
            let child = tree.get(children[i]).unwrap();
            assert!(
                child.rect.width == 0.0 && child.rect.height == 0.0,
                "item {} should be invisible (rect {:?})",
                i,
                child.rect
            );
        }
    }

    #[test]
    fn scroll_list_virtual_scrolling() {
        let (mut tree, list) = scroll_list_tree(20);

        // Scroll down by 60px (3 items).
        tree.set_scroll_offset(list, 60.0);
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Only visible items (3-7ish) should produce text commands.
        // Background panel + scrollbar thumb = 2 panel commands.
        assert!(dl.panels.len() >= 1);

        // Count visible text commands — should be around 5-8 (viewport 100px / 20px items,
        // plus partially visible items at edges).
        let visible_texts = dl.texts.len();
        assert!(
            visible_texts <= 8,
            "expected <=8 visible items, got {}",
            visible_texts
        );
        assert!(
            visible_texts >= 4,
            "expected >=4 visible items, got {}",
            visible_texts
        );
    }

    #[test]
    fn scroll_offset_clamping() {
        let (mut tree, list) = scroll_list_tree(10);

        // Max scroll = total_height - viewport = 10*20 - 100 = 100.
        assert!((tree.max_scroll(list) - 100.0).abs() < 0.01);

        // Scroll beyond max clamps.
        tree.set_scroll_offset(list, 999.0);
        let offset = match &tree.get(list).unwrap().widget {
            Widget::ScrollList { scroll_offset, .. } => *scroll_offset,
            _ => panic!(),
        };
        assert!((offset - 100.0).abs() < 0.01);

        // Negative scroll clamps to 0.
        tree.set_scroll_offset(list, -50.0);
        let offset = match &tree.get(list).unwrap().widget {
            Widget::ScrollList { scroll_offset, .. } => *scroll_offset,
            _ => panic!(),
        };
        assert!(offset.abs() < 0.01);
    }

    #[test]
    fn scroll_list_no_scrollbar_when_content_fits() {
        // 3 items * 20px = 60px < 100px viewport → no scrollbar.
        let (tree, _list) = scroll_list_tree(3);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // 2 panels: background + 1 alternating row tint (item index 1).
        assert_eq!(dl.panels.len(), 2);
    }

    #[test]
    fn scroll_list_scrollbar_when_content_overflows() {
        // 10 items * 20px = 200px > 100px viewport → scrollbar visible.
        let (tree, _list) = scroll_list_tree(10);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // 4 panels: background + 2 alternating row tints (items 1,3) + scrollbar thumb.
        assert_eq!(dl.panels.len(), 4);
    }

    #[test]
    fn ensure_visible_scrolls_to_item() {
        let (mut tree, list) = scroll_list_tree(20);

        // Item 15 is at y=300, well below viewport (0-100). Ensure visible.
        tree.ensure_visible(list, 15);
        let offset = match &tree.get(list).unwrap().widget {
            Widget::ScrollList { scroll_offset, .. } => *scroll_offset,
            _ => panic!(),
        };
        // Item 15 bottom = 16*20 = 320. Scroll to 320 - 100 = 220.
        assert!((offset - 220.0).abs() < 0.01);

        // Ensure visible on an already-visible item doesn't change offset.
        let before = offset;
        tree.ensure_visible(list, 15); // 15 is at 300, viewport 220..320 → visible.
        let after = match &tree.get(list).unwrap().widget {
            Widget::ScrollList { scroll_offset, .. } => *scroll_offset,
            _ => panic!(),
        };
        assert!((after - before).abs() < 0.01);

        // Ensure visible scrolls up when item is above viewport.
        tree.ensure_visible(list, 0);
        let offset = match &tree.get(list).unwrap().widget {
            Widget::ScrollList { scroll_offset, .. } => *scroll_offset,
            _ => panic!(),
        };
        assert!(offset.abs() < 0.01); // scrolled to top
    }

    // ------------------------------------------------------------------
    // Scroll offset getter + empty text + alternating rows (Area 8)
    // ------------------------------------------------------------------

    #[test]
    fn scroll_offset_getter() {
        let (mut tree, list) = scroll_list_tree(20);
        assert!(tree.scroll_offset(list).abs() < 0.01);
        tree.set_scroll_offset(list, 42.0);
        assert!((tree.scroll_offset(list) - 42.0).abs() < 0.01);
    }

    #[test]
    fn scroll_list_empty_text_drawn() {
        let mut tree = WidgetTree::new();
        let list = tree.insert_root(Widget::ScrollList {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 1.0,
            item_height: 20.0,
            scroll_offset: 0.0,
            scrollbar_color: [0.8, 0.6, 0.3, 0.5],
            scrollbar_width: 6.0,
            item_heights: Vec::new(),
            empty_text: Some("No items.".to_string()),
        });
        tree.set_position(list, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(list, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        tree.set_padding(list, Edges::all(0.0));
        tree.layout(screen(), &mut HeuristicMeasurer);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // 1 panel (background) + 1 text (empty message).
        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.texts.len(), 1);
        assert_eq!(dl.texts[0].text, "No items.");
    }

    #[test]
    fn scroll_list_empty_no_text_without_message() {
        let mut tree = WidgetTree::new();
        let list = tree.insert_root(Widget::ScrollList {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 1.0,
            item_height: 20.0,
            scroll_offset: 0.0,
            scrollbar_color: [0.8, 0.6, 0.3, 0.5],
            scrollbar_width: 6.0,
            item_heights: Vec::new(),
            empty_text: None,
        });
        tree.set_position(list, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(list, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        tree.set_padding(list, Edges::all(0.0));
        tree.layout(screen(), &mut HeuristicMeasurer);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // 1 panel (background), no text.
        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.texts.len(), 0);
    }

    #[test]
    fn scroll_list_alternating_row_tint() {
        // 5 items, viewport fits all. Odd items: 1, 3. → 2 alt tint panels.
        let (tree, _list) = scroll_list_tree(5);
        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // 1 background + 2 alternating tints = 3 panels (no scrollbar, content fits).
        assert_eq!(dl.panels.len(), 3);
        // Alternating tint panels have near-zero alpha black.
        assert!(dl.panels[1].bg_color[3] > 0.0 && dl.panels[1].bg_color[3] < 0.1);
        assert!(dl.panels[2].bg_color[3] > 0.0 && dl.panels[2].bg_color[3] < 0.1);
    }

    // ------------------------------------------------------------------
    // Performance metrics tests (UI-505)
    // ------------------------------------------------------------------

    #[test]
    fn widget_count_on_demo() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = demo::DemoLiveData {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        demo::build_demo(&mut tree, &theme, &kb, &live, screen, 0.0);
        assert!(tree.widget_count() > 0, "demo tree should have widgets");
    }

    #[test]
    fn perf_metrics_default() {
        let m = UiPerfMetrics::default();
        assert_eq!(m.build_us, 0);
        assert_eq!(m.widget_count, 0);
    }

    #[test]
    fn status_bar_with_perf_metrics() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let perf = UiPerfMetrics {
            build_us: 300,
            layout_us: 100,
            draw_us: 200,
            render_us: 400,
            widget_count: 42,
            panel_cmds: 10,
            text_cmds: 20,
            sprite_cmds: 0,
        };
        let info = StatusBarInfo {
            tick: 0,
            date: "1 January 1845, 00:00".to_string(),
            population: 0,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: Some(perf),
        };
        let bar = build_status_bar(&mut tree, &theme, &info);
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            // With perf: 5 normal + sep + perf = 7 spans.
            assert_eq!(spans.len(), 7);
            let perf_span = &spans[6];
            assert!(perf_span.text.contains("build 0.3ms"));
            assert!(perf_span.text.contains("layout 0.1ms"));
            assert!(perf_span.text.contains("42w"));
        } else {
            panic!("expected RichText");
        }
    }

    // ------------------------------------------------------------------
    // Variable-height ScrollList tests (UI-501)
    // ------------------------------------------------------------------

    #[test]
    fn variable_height_scroll_list_layout() {
        // 4 items with heights [20, 40, 20, 60]. Total = 140.
        let mut tree = WidgetTree::new();
        let list = tree.insert_root(Widget::ScrollList {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 1.0,
            item_height: 20.0, // fallback, unused here
            scroll_offset: 0.0,
            scrollbar_color: [0.8, 0.6, 0.3, 0.5],
            scrollbar_width: 6.0,
            item_heights: vec![20.0, 40.0, 20.0, 60.0],
            empty_text: None,
        });
        tree.set_position(list, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(list, Sizing::Fixed(200.0), Sizing::Fixed(200.0));
        tree.set_padding(list, Edges::all(0.0));

        for i in 0..4 {
            tree.insert(
                list,
                Widget::Label {
                    text: format!("Item {}", i),
                    color: [1.0; 4],
                    font_size: 12.0,
                    font_family: FontFamily::Mono,
                    wrap: false,
                },
            );
        }

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        // Total content height.
        let total = WidgetTree::scroll_total_height(&[20.0, 40.0, 20.0, 60.0], 20.0, 4);
        assert!((total - 140.0).abs() < 0.01);

        // Item 2 starts at y=60 (20+40).
        let item2_y = WidgetTree::scroll_item_y(&[20.0, 40.0, 20.0, 60.0], 20.0, 2);
        assert!((item2_y - 60.0).abs() < 0.01);

        // Verify layout rects.
        let node = tree.get(list).unwrap();
        let c0 = tree.get(node.children[0]).unwrap();
        assert!((c0.rect.y - 0.0).abs() < 0.01);
        assert!((c0.rect.height - 20.0).abs() < 0.01);

        let c1 = tree.get(node.children[1]).unwrap();
        assert!((c1.rect.y - 20.0).abs() < 0.01);
        assert!((c1.rect.height - 40.0).abs() < 0.01);

        let c2 = tree.get(node.children[2]).unwrap();
        assert!((c2.rect.y - 60.0).abs() < 0.01);
        assert!((c2.rect.height - 20.0).abs() < 0.01);

        let c3 = tree.get(node.children[3]).unwrap();
        assert!((c3.rect.y - 80.0).abs() < 0.01);
        assert!((c3.rect.height - 60.0).abs() < 0.01);
    }

    #[test]
    fn variable_height_backward_compat() {
        // Empty item_heights should behave identically to fixed-height.
        let (tree_fixed, list_fixed) = scroll_list_tree(10);
        let max_fixed = tree_fixed.max_scroll(list_fixed);

        // Same tree but with explicit empty item_heights.
        let mut tree = WidgetTree::new();
        let list = tree.insert_root(Widget::ScrollList {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 1.0,
            item_height: 20.0,
            scroll_offset: 0.0,
            scrollbar_color: [0.8, 0.6, 0.3, 0.5],
            scrollbar_width: 6.0,
            item_heights: Vec::new(),
            empty_text: None,
        });
        tree.set_position(list, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(list, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        tree.set_padding(list, Edges::all(0.0));
        for i in 0..10 {
            tree.insert(
                list,
                Widget::Label {
                    text: format!("Item {}", i),
                    color: [1.0; 4],
                    font_size: 12.0,
                    font_family: FontFamily::Mono,
                    wrap: false,
                },
            );
        }
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let max_var = tree.max_scroll(list);
        assert!((max_fixed - max_var).abs() < 0.01);
    }

    #[test]
    fn variable_height_first_visible() {
        // Items: [20, 40, 20, 60], total = 140.
        let ihs = [20.0, 40.0, 20.0, 60.0];
        // offset 0 → first visible = 0.
        assert_eq!(WidgetTree::scroll_first_visible(&ihs, 20.0, 4, 0.0), 0);
        // offset 20 → item 0 ends at 20, so first visible = 1.
        assert_eq!(WidgetTree::scroll_first_visible(&ihs, 20.0, 4, 20.0), 1);
        // offset 50 → item 0 ends at 20, item 1 ends at 60 → 50 is within item 1.
        assert_eq!(WidgetTree::scroll_first_visible(&ihs, 20.0, 4, 50.0), 1);
        // offset 60 → item 2 starts at 60, first visible = 2.
        assert_eq!(WidgetTree::scroll_first_visible(&ihs, 20.0, 4, 60.0), 2);
        // offset 80 → item 3 starts at 80, first visible = 3.
        assert_eq!(WidgetTree::scroll_first_visible(&ihs, 20.0, 4, 80.0), 3);
    }

    #[test]
    fn variable_height_scrollbar_proportional() {
        // Viewport 100px, items [20, 40, 20, 60] = 140. max_scroll = 40.
        let mut tree = WidgetTree::new();
        let list = tree.insert_root(Widget::ScrollList {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 1.0,
            item_height: 20.0,
            scroll_offset: 0.0,
            scrollbar_color: [0.8, 0.6, 0.3, 0.5],
            scrollbar_width: 6.0,
            item_heights: vec![20.0, 40.0, 20.0, 60.0],
            empty_text: None,
        });
        tree.set_position(list, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(list, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        tree.set_padding(list, Edges::all(0.0));
        for i in 0..4 {
            tree.insert(
                list,
                Widget::Label {
                    text: format!("Item {}", i),
                    color: [1.0; 4],
                    font_size: 12.0,
                    font_family: FontFamily::Mono,
                    wrap: false,
                },
            );
        }
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let max = tree.max_scroll(list);
        assert!((max - 40.0).abs() < 0.01); // 140 - 100 = 40
    }

    #[test]
    fn demo_tree_includes_scroll_list() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = demo::DemoLiveData {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        demo::build_demo(&mut tree, &theme, &kb, &live, screen, 0.0);
        tree.layout(screen, &mut HeuristicMeasurer);

        // Demo is a single root panel.
        assert_eq!(tree.roots().len(), 1);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Should have scroll items and button texts.
        assert!(dl.texts.len() > 4, "scroll items + buttons should be drawn");

        // Verify scroll items exist.
        let all_texts: Vec<&str> = dl.texts.iter().map(|t| t.text.as_str()).collect();
        assert!(all_texts.contains(&"Item 1"));
    }

    #[test]
    fn scroll_list_focusable() {
        let (tree, list) = scroll_list_tree(5);
        let focusable = tree.focusable_widgets();
        assert!(focusable.contains(&list));
    }

    // ------------------------------------------------------------------
    // Status bar tests (UI-I01a)
    // ------------------------------------------------------------------

    #[test]
    fn status_bar_structure() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 42,
            date: "1 January 1845, 00:42".to_string(),
            population: 15,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        let bar = build_status_bar(&mut tree, &theme, &info);

        // One root: the status bar panel.
        assert_eq!(tree.roots().len(), 1);
        assert_eq!(tree.roots()[0], bar);

        // Panel has one child: the RichText.
        let node = tree.get(bar).expect("bar exists");
        assert_eq!(node.children.len(), 1);
        if let Widget::Panel { bg_color, .. } = &node.widget {
            assert_eq!(*bg_color, theme.status_bar_bg);
        } else {
            panic!("status bar root should be a Panel");
        }

        // Child is RichText with data font size.
        let child = tree.get(node.children[0]).expect("child exists");
        if let Widget::RichText { spans, font_size } = &child.widget {
            assert!((font_size - theme.font_data_size).abs() < 0.01);
            // Real-time mode, speed 1, no player: 5 spans (tick, sep, pop, sep, speed).
            assert_eq!(spans.len(), 5);
            assert_eq!(spans[0].text, "1 January 1845, 00:42");
            assert_eq!(spans[0].color, theme.gold);
            assert_eq!(spans[2].text, "Pop: 15");
            assert_eq!(spans[2].color, theme.text_light);
            assert!(spans[4].text.starts_with("Speed: 1x"));
        } else {
            panic!("status bar child should be RichText");
        }
    }

    #[test]
    fn status_bar_turn_based_with_player() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 100,
            date: "1 January 1845, 01:40".to_string(),
            population: 3,
            is_turn_based: true,
            player_name: Some("Goblin"),
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        let bar = tree.roots()[0];
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            // Turn-based + player: 7 spans (tick, sep, pop, sep, mode, sep, @name).
            assert_eq!(spans.len(), 7);
            assert_eq!(spans[4].text, "TURN-BASED");
            assert_eq!(spans[4].color, theme.gold);
            assert_eq!(spans[6].text, "@Goblin");
            assert_eq!(spans[6].color, theme.gold);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn status_bar_layout_full_width() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 0,
            date: "1 January 1845, 00:00".to_string(),
            population: 0,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 1024.0,
            perf: None,
        };
        let bar = build_status_bar(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 1024.0,
                height: 768.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(bar).expect("rect after layout");
        assert!((rect.x - 0.0).abs() < 0.01);
        assert!((rect.y - 0.0).abs() < 0.01);
        assert!((rect.width - 1024.0).abs() < 0.01);
        // Height = padding_v*2 + content (Fit sizing).
        assert!(rect.height > 0.0);
        assert!(rect.height < 100.0); // sanity: a single-line bar shouldn't be huge
    }

    #[test]
    fn status_bar_draw_output() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 7,
            date: "1 January 1845, 00:07".to_string(),
            population: 200,
            is_turn_based: true,
            player_name: Some("Wolf"),
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // One panel (the status bar background).
        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.panels[0].bg_color, theme.status_bar_bg);
        assert!((dl.panels[0].width - 800.0).abs() < 0.01);

        // One rich text command with 7 spans.
        assert_eq!(dl.rich_texts.len(), 1);
        assert_eq!(dl.rich_texts[0].spans.len(), 7);
        assert!(dl.rich_texts[0].spans[0].text.contains("7"));
        assert!(dl.rich_texts[0].spans[2].text.contains("200"));
        assert_eq!(dl.rich_texts[0].spans[6].text, "@Wolf");

        // No plain text commands (only rich text).
        assert_eq!(dl.texts.len(), 0);
    }

    #[test]
    fn status_bar_paused_display() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 10,
            date: "1 January 1845, 00:10".to_string(),
            population: 5,
            is_turn_based: false,
            player_name: None,
            paused: true,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        let bar = tree.roots()[0];
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            // Paused: 5 spans (tick, sep, pop, sep, "PAUSED (Space)").
            assert_eq!(spans.len(), 5);
            assert!(spans[4].text.contains("PAUSED"));
            assert!(spans[4].text.contains("Space"));
            assert_eq!(spans[4].color, theme.danger);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn status_bar_speed_display() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 10,
            date: "1 January 1845, 00:10".to_string(),
            population: 5,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 3,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        let bar = tree.roots()[0];
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans.len(), 5);
            assert!(spans[4].text.contains("3x"));
            assert!(spans[4].text.contains("(3)"));
            // Speed > 1 gets gold highlight.
            assert_eq!(spans[4].color, theme.gold);
        } else {
            panic!("expected RichText");
        }
    }

    // ------------------------------------------------------------------
    // Hover tooltip tests (UI-I01b)
    // ------------------------------------------------------------------

    fn screen() -> Size {
        Size {
            width: 800.0,
            height: 600.0,
        }
    }

    /// Helper: build a minimal HoverInfo with just terrain.
    fn hover_terrain_only() -> HoverInfo {
        HoverInfo {
            tile_x: 100,
            tile_y: 200,
            terrain: "Road".into(),
            quartier: None,
            address: None,
            building_name: None,
            occupants: Vec::new(),
            occupant_year_suffix: None,
            entities: Vec::new(),
        }
    }

    #[test]
    fn hover_tooltip_terrain_only() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = hover_terrain_only();
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (100.0, 100.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        // One root: the tooltip panel.
        assert_eq!(tree.roots().len(), 1);
        assert_eq!(tree.roots()[0], tip);

        // Panel has one child: the coordinates + terrain RichText.
        let node = tree.get(tip).expect("tooltip panel");
        assert_eq!(node.children.len(), 1);
        if let Widget::Panel { bg_color, .. } = &node.widget {
            assert_eq!(*bg_color, theme.tooltip_bg_color);
        } else {
            panic!("tooltip root should be a Panel");
        }

        // Child is RichText with coords and terrain.
        let child = tree.get(node.children[0]).expect("child");
        if let Widget::RichText { spans, font_size } = &child.widget {
            assert!((font_size - theme.font_data_size).abs() < 0.01);
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0].text, "(100, 200)");
            assert_eq!(spans[0].color, theme.gold);
            assert!(spans[1].text.contains("Road"));
            assert_eq!(spans[1].color, theme.text_light);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn hover_tooltip_full_building() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = HoverInfo {
            tile_x: 42,
            tile_y: 99,
            terrain: "Floor".into(),
            quartier: Some("Marais".into()),
            address: Some("42 Rue de Rivoli".into()),
            building_name: Some("Boulangerie".into()),
            occupants: vec![
                ("Jean Dupont".into(), "flour merchant".into()),
                ("Marie".into(), "baker".into()),
            ],
            occupant_year_suffix: None,
            entities: vec![('g', "Goblin".into())],
        };
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (200.0, 200.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        let node = tree.get(tip).expect("panel");
        // Children: coords(1) + quartier(1) + address(1) + 2 occupants(2) + 1 entity(1) = 6
        assert_eq!(node.children.len(), 6);
    }

    #[test]
    fn hover_tooltip_occupant_truncation() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = HoverInfo {
            tile_x: 0,
            tile_y: 0,
            terrain: "Floor".into(),
            quartier: None,
            address: Some("1 Rue X".into()),
            building_name: None,
            occupants: (0..8)
                .map(|i| (format!("Person {}", i), "trade".into()))
                .collect(),
            occupant_year_suffix: Some("[1842]".into()),
            entities: Vec::new(),
        };
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (50.0, 50.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        let node = tree.get(tip).expect("panel");
        // Children: coords(1) + address(1) + 5 occupants(5) + "+3 more"(1) + year(1) = 9
        assert_eq!(node.children.len(), 9);

        // Verify "+3 more" label exists.
        let mut dl = DrawList::new();
        tree.layout(screen(), &mut HeuristicMeasurer);
        tree.draw(&mut dl, &mut HeuristicMeasurer);
        let has_more = dl.texts.iter().any(|t| t.text == "+3 more");
        assert!(has_more, "should show +3 more for 8 occupants (max 5)");
    }

    #[test]
    fn hover_tooltip_entities_shown() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = HoverInfo {
            tile_x: 10,
            tile_y: 20,
            terrain: "Road".into(),
            quartier: None,
            address: None,
            building_name: None,
            occupants: Vec::new(),
            occupant_year_suffix: None,
            entities: vec![('g', "Goblin".into()), ('w', "Wolf".into())],
        };
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (100.0, 100.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        let node = tree.get(tip).expect("panel");
        // coords(1) + 2 entities(2) = 3
        assert_eq!(node.children.len(), 3);

        tree.layout(screen(), &mut HeuristicMeasurer);
        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Entity entries are RichText with icon + name spans.
        let entity_rts: Vec<_> = dl
            .rich_texts
            .iter()
            .filter(|rt| rt.spans.len() == 2 && rt.spans[0].text.starts_with('g'))
            .collect();
        assert_eq!(entity_rts.len(), 1);
        assert_eq!(entity_rts[0].spans[1].text, "Goblin");
    }

    #[test]
    fn hover_tooltip_draw_output() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = hover_terrain_only();
        build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (100.0, 100.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        tree.layout(screen(), &mut HeuristicMeasurer);
        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // One panel (tooltip background).
        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.panels[0].bg_color, theme.tooltip_bg_color);
        assert_eq!(dl.panels[0].border_color, theme.tooltip_border_color);

        // One rich text (coords + terrain).
        assert_eq!(dl.rich_texts.len(), 1);
        assert_eq!(dl.rich_texts[0].spans.len(), 2);
        assert!(dl.rich_texts[0].spans[0].text.contains("100"));
    }

    #[test]
    fn hover_tooltip_positioned_on_screen() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = hover_terrain_only();
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (750.0, 550.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        tree.layout(screen(), &mut HeuristicMeasurer);
        let rect = tree.node_rect(tip).expect("rect");

        // Tooltip should be fully on screen (edge-flipped if necessary).
        assert!(rect.x >= 0.0, "x={} should be >= 0", rect.x);
        assert!(rect.y >= 0.0, "y={} should be >= 0", rect.y);
        assert!(
            rect.x + rect.width <= 800.0,
            "right edge {} should be <= 800",
            rect.x + rect.width
        );
        assert!(
            rect.y + rect.height <= 600.0,
            "bottom edge {} should be <= 600",
            rect.y + rect.height
        );
    }

    // ------------------------------------------------------------------
    // Event log tests (UI-I01c)
    // ------------------------------------------------------------------

    #[test]
    fn event_log_empty() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let log = build_event_log(&mut tree, &theme, &[], 800.0, 108.0);

        assert_eq!(tree.roots().len(), 1);
        assert_eq!(tree.roots()[0], log);

        let node = tree.get(log).expect("log exists");
        assert!(node.children.is_empty());
        if let Widget::ScrollList { scroll_offset, .. } = &node.widget {
            assert!(scroll_offset.abs() < 0.01, "empty log should have 0 scroll");
        } else {
            panic!("event log root should be a ScrollList");
        }
    }

    #[test]
    fn event_log_spawned_entry() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Spawned {
            name: "Goblin".into(),
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let node = tree.get(log).expect("log");
        assert_eq!(node.children.len(), 1);

        let child = tree.get(node.children[0]).expect("child");
        if let Widget::RichText { spans, font_size } = &child.widget {
            assert!((font_size - theme.font_data_size).abs() < 0.01);
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0].text, "Goblin");
            assert_eq!(spans[0].color, theme.text_light);
            assert_eq!(spans[1].text, " spawned");
            assert_eq!(spans[1].color, theme.disabled);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_died_danger_color() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Died {
            name: "Wolf".into(),
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let child_id = tree.get(log).expect("log").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans[0].text, "Wolf");
            assert_eq!(spans[0].color, theme.danger);
            assert_eq!(spans[1].text, " died");
            assert_eq!(spans[1].color, theme.danger);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_ate_food_gold() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Ate {
            name: "Goblin".into(),
            food_name: "Bread".into(),
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let child_id = tree.get(log).expect("log").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans.len(), 3);
            assert_eq!(spans[0].text, "Goblin");
            assert_eq!(spans[0].color, theme.text_light);
            assert_eq!(spans[1].text, " ate ");
            assert_eq!(spans[2].text, "Bread");
            assert_eq!(spans[2].color, theme.gold);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_attacked_damage() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Attacked {
            attacker: "Goblin".into(),
            defender: "Troll".into(),
            damage: 12.5,
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let child_id = tree.get(log).expect("log").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans.len(), 4);
            assert_eq!(spans[0].text, "Goblin");
            assert_eq!(spans[1].text, " attacks ");
            assert_eq!(spans[2].text, "Troll");
            assert_eq!(spans[3].text, " (12 dmg)");
            assert_eq!(spans[3].color, theme.danger);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_auto_scrolls_to_bottom() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        // 20 entries at 20px each = 400px total; viewport ~100px → should auto-scroll
        let entries: Vec<EventLogEntry> = (0..20)
            .map(|i| EventLogEntry::Spawned {
                name: format!("Entity{}", i),
            })
            .collect();
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let node = tree.get(log).expect("log");
        if let Widget::ScrollList { scroll_offset, .. } = &node.widget {
            // total = 20*20 = 400, viewport = 108 - 8 = 100, max = 300
            let expected = (20.0 * theme.scroll_item_height
                - (108.0 - theme.status_bar_padding_v * 2.0))
                .max(0.0);
            assert!(
                (*scroll_offset - expected).abs() < 0.01,
                "scroll_offset={}, expected={}",
                scroll_offset,
                expected
            );
        } else {
            panic!("expected ScrollList");
        }
    }

    #[test]
    fn event_log_draw_output() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![
            EventLogEntry::Spawned {
                name: "Goblin".into(),
            },
            EventLogEntry::Died {
                name: "Wolf".into(),
            },
            EventLogEntry::Ate {
                name: "Elf".into(),
                food_name: "Apple".into(),
            },
        ];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);
        tree.set_position(log, Position::Fixed { x: 0.0, y: 492.0 });

        tree.layout(screen(), &mut HeuristicMeasurer);
        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // 2 panels: ScrollList background + 1 alternating row tint (item index 1).
        assert_eq!(dl.panels.len(), 2);
        assert_eq!(dl.panels[0].bg_color, theme.status_bar_bg);
        assert!((dl.panels[0].width - 800.0).abs() < 0.01);

        // 3 rich text commands (one per event).
        assert_eq!(dl.rich_texts.len(), 3);
        assert_eq!(dl.rich_texts[0].spans[0].text, "Goblin");
        assert_eq!(dl.rich_texts[1].spans[0].text, "Wolf");
        assert_eq!(dl.rich_texts[2].spans[0].text, "Elf");
    }

    #[test]
    fn event_log_full_width_fixed_height() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let log = build_event_log(&mut tree, &theme, &[], 1024.0, 120.0);
        tree.set_position(log, Position::Fixed { x: 0.0, y: 500.0 });

        tree.layout(
            Size {
                width: 1024.0,
                height: 768.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(log).expect("rect");
        assert!((rect.width - 1024.0).abs() < 0.01);
        assert!((rect.height - 120.0).abs() < 0.01);
        assert!((rect.y - 500.0).abs() < 0.01);
    }

    // ------------------------------------------------------------------
    // Entity inspector tests (UI-I01d)
    // ------------------------------------------------------------------

    /// Helper: create a minimal World and spawn an entity with all components.
    fn spawn_full_entity(world: &mut crate::world::World) -> crate::components::Entity {
        let e = world.spawn();
        world
            .body
            .positions
            .insert(e, crate::components::Position { x: 10, y: 20 });
        world.body.names.insert(
            e,
            crate::components::Name {
                value: "Goblin".into(),
            },
        );
        world
            .body
            .icons
            .insert(e, crate::components::Icon { ch: 'g' });
        world.body.healths.insert(
            e,
            crate::components::Health {
                current: 80.0,
                max: 100.0,
            },
        );
        world.mind.hungers.insert(
            e,
            crate::components::Hunger {
                current: 30.0,
                max: 100.0,
            },
        );
        world
            .body
            .fatigues
            .insert(e, crate::components::Fatigue { current: 5.0 });
        world.body.combat_stats.insert(
            e,
            crate::components::CombatStats {
                attack: 12.0,
                defense: 8.0,
                aggression: 0.7,
            },
        );
        world
            .body
            .current_gaits
            .insert(e, crate::components::Gait::Walk);
        e
    }

    #[test]
    fn collect_inspector_info_alive() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = spawn_full_entity(&mut world);

        let info = collect_inspector_info(e, &world).expect("alive entity should return Some");
        assert_eq!(info.name, "Goblin");
        assert_eq!(info.icon, 'g');
        assert_eq!(info.position, (10, 20));
        assert_eq!(info.health, Some((80.0, 100.0)));
        assert_eq!(info.hunger, Some((30.0, 100.0)));
        assert!((info.fatigue.unwrap() - 5.0).abs() < 0.01);
        let (atk, def, agg) = info.combat.unwrap();
        assert!((atk - 12.0).abs() < 0.01);
        assert!((def - 8.0).abs() < 0.01);
        assert!((agg - 0.7).abs() < 0.01);
        assert_eq!(info.gait.as_deref(), Some("Walk"));
    }

    #[test]
    fn collect_inspector_info_dead() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = spawn_full_entity(&mut world);
        world.alive.remove(&e);

        assert!(collect_inspector_info(e, &world).is_none());
    }

    #[test]
    fn collect_inspector_info_no_position() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = world.spawn();
        world.body.names.insert(
            e,
            crate::components::Name {
                value: "Ghost".into(),
            },
        );
        // No position inserted.

        assert!(collect_inspector_info(e, &world).is_none());
    }

    #[test]
    fn collect_inspector_info_minimal() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = world.spawn();
        world
            .body
            .positions
            .insert(e, crate::components::Position { x: 5, y: 5 });
        // Only position, no other components.

        let info = collect_inspector_info(e, &world).expect("alive with position");
        assert_eq!(info.position, (5, 5));
        // Name falls back to "E{id}".
        assert!(info.name.starts_with('E'));
        assert_eq!(info.icon, '?');
        assert!(info.health.is_none());
        assert!(info.hunger.is_none());
        assert!(info.fatigue.is_none());
        assert!(info.combat.is_none());
        assert!(info.action.is_none());
        assert!(info.gait.is_none());
    }

    #[test]
    fn build_inspector_creates_panel() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = EntityInspectorInfo {
            name: "Goblin".into(),
            icon: 'g',
            position: (10, 20),
            health: Some((80.0, 100.0)),
            hunger: Some((30.0, 100.0)),
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id, _close_id) = build_entity_inspector(&mut tree, &theme, &info);

        let node = tree.get(panel_id).expect("panel exists");
        assert!(matches!(node.widget, Widget::Panel { .. }));
    }

    #[test]
    fn build_inspector_has_close_button() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = EntityInspectorInfo {
            name: "Wolf".into(),
            icon: 'w',
            position: (0, 0),
            health: None,
            hunger: None,
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id, close_id) = build_entity_inspector(&mut tree, &theme, &info);

        // Close button is inside the header row (grandchild of panel).
        let panel_node = tree.get(panel_id).expect("panel");
        let header_row_id = panel_node.children[0];
        let header_node = tree.get(header_row_id).expect("header row");
        assert!(header_node.children.contains(&close_id));

        let close_node = tree.get(close_id).expect("close button");
        assert!(matches!(close_node.widget, Widget::Button { .. }));
    }

    #[test]
    fn build_inspector_health_colors() {
        let theme = Theme::default();

        // Low HP (≤25%) → danger color
        let mut tree = WidgetTree::new();
        let info_low = EntityInspectorInfo {
            name: "Dying".into(),
            icon: 'd',
            position: (0, 0),
            health: Some((10.0, 100.0)), // 10% = danger
            hunger: None,
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id, _) = build_entity_inspector(&mut tree, &theme, &info_low);

        // Find the HP RichText child (has "HP " span).
        let panel_node = tree.get(panel_id).expect("panel");
        let mut found_danger = false;
        for &child_id in &panel_node.children {
            if let Some(child) = tree.get(child_id) {
                if let Widget::RichText { spans, .. } = &child.widget {
                    if spans.len() >= 2 && spans[0].text == "HP " {
                        assert_eq!(spans[1].color, theme.danger);
                        found_danger = true;
                    }
                }
            }
        }
        assert!(found_danger, "should find HP span with danger color");

        // High HP (>50%) → text_light color
        let mut tree2 = WidgetTree::new();
        let info_high = EntityInspectorInfo {
            name: "Healthy".into(),
            icon: 'h',
            position: (0, 0),
            health: Some((90.0, 100.0)), // 90% = text_light
            hunger: None,
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id2, _) = build_entity_inspector(&mut tree2, &theme, &info_high);
        let panel_node2 = tree2.get(panel_id2).expect("panel");
        let mut found_light = false;
        for &child_id in &panel_node2.children {
            if let Some(child) = tree2.get(child_id) {
                if let Widget::RichText { spans, .. } = &child.widget {
                    if spans.len() >= 2 && spans[0].text == "HP " {
                        assert_eq!(spans[1].color, theme.text_light);
                        found_light = true;
                    }
                }
            }
        }
        assert!(found_light, "should find HP span with text_light color");
    }

    #[test]
    fn build_inspector_sizing() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = EntityInspectorInfo {
            name: "Goblin".into(),
            icon: 'g',
            position: (10, 20),
            health: Some((80.0, 100.0)),
            hunger: Some((30.0, 100.0)),
            fatigue: Some(5.0),
            combat: Some((12.0, 8.0, 0.7)),
            action: Some("Idle".into()),
            gait: Some("Walk".into()),
        };
        let (panel_id, _) = build_entity_inspector(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(panel_id).expect("rect after layout");
        // Width is fixed at INSPECTOR_WIDTH (220px).
        assert!((rect.width - 220.0).abs() < 0.01);
        // Height is Fit — should be > 0 (content exists).
        assert!(rect.height > 0.0);
    }

    // ------------------------------------------------------------------
    // Animation helper tests (UI-W05)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Row auto-layout (UI-100)
    // ------------------------------------------------------------------

    #[test]
    fn row_children_contiguous_with_gap() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 4.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(400.0), Sizing::Fixed(50.0));

        // 3 labels with known approximate widths.
        let label_a = tree.insert(
            row,
            Widget::Label {
                text: "AAA".into(), // 3 chars
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let label_b = tree.insert(
            row,
            Widget::Label {
                text: "BBBB".into(), // 4 chars
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let label_c = tree.insert(
            row,
            Widget::Label {
                text: "CC".into(), // 2 chars
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let ra = tree.node_rect(label_a).unwrap();
        let rb = tree.node_rect(label_b).unwrap();
        let rc = tree.node_rect(label_c).unwrap();

        // Children are contiguous left-to-right with 4px gap.
        let expected_b_x = ra.x + ra.width + 4.0;
        let expected_c_x = rb.x + rb.width + 4.0;
        assert!(
            (rb.x - expected_b_x).abs() < 0.1,
            "label_b should start at {expected_b_x}, got {}",
            rb.x
        );
        assert!(
            (rc.x - expected_c_x).abs() < 0.1,
            "label_c should start at {expected_c_x}, got {}",
            rc.x
        );
    }

    #[test]
    fn row_cross_align_center() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Center,
        });
        tree.set_sizing(row, Sizing::Fixed(400.0), Sizing::Fixed(100.0));

        let label = tree.insert(
            row,
            Widget::Label {
                text: "Hi".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rl = tree.node_rect(label).unwrap();
        // Label height is 14.0 (scale=1.0), row is 100.0 tall.
        // Centered: y = (100 - 14) / 2 = 43.
        assert!(
            (rl.y - 43.0).abs() < 1.0,
            "label should be vertically centered, y = {}",
            rl.y
        );
    }

    #[test]
    fn row_percent_children_split_remaining() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(300.0), Sizing::Fixed(50.0));

        // One fixed-width label (approx 3 chars * 8.4 = 25.2), two percent children.
        let _fixed = tree.insert(
            row,
            Widget::Label {
                text: "AAA".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let pct_a = tree.insert(
            row,
            Widget::Panel {
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
        );
        tree.set_sizing(pct_a, Sizing::Percent(0.5), Sizing::Fit);

        let pct_b = tree.insert(
            row,
            Widget::Panel {
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
        );
        tree.set_sizing(pct_b, Sizing::Percent(0.5), Sizing::Fit);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rpa = tree.node_rect(pct_a).unwrap();
        let rpb = tree.node_rect(pct_b).unwrap();

        // Both percent children should share remaining space equally.
        assert!(
            (rpa.width - rpb.width).abs() < 1.0,
            "percent children should be equal width: {} vs {}",
            rpa.width,
            rpb.width
        );
        assert!(
            rpa.width > 100.0,
            "percent child should be > 100px, got {}",
            rpa.width
        );
    }

    // ------------------------------------------------------------------
    // Expand spacer (UI-601)
    // ------------------------------------------------------------------

    #[test]
    fn expand_pushes_last_child_to_end_in_row() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(400.0), Sizing::Fixed(50.0));

        // [Label, Expand, Button] — button should be at right edge.
        let label = tree.insert(
            row,
            Widget::Label {
                text: "Title".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        tree.insert(row, Widget::Expand);
        let btn = tree.insert(
            row,
            Widget::Button {
                text: "X".into(),
                color: [1.0; 4],
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rl = tree.node_rect(label).unwrap();
        let rb = tree.node_rect(btn).unwrap();

        // Button should be near the right edge (400 - button_width).
        assert!(
            (rb.x + rb.width - 400.0).abs() < 1.0,
            "button should be at right edge: x={}, w={}, row_w=400",
            rb.x,
            rb.width
        );
        // Label should be at the left edge.
        assert!(rl.x < 1.0, "label should be at left edge: x={}", rl.x);
    }

    #[test]
    fn expand_fills_remaining_in_column() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(col, Sizing::Fixed(200.0), Sizing::Fixed(400.0));

        // [Label, Expand, Label] — bottom label should be at bottom.
        tree.insert(
            col,
            Widget::Label {
                text: "Top".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let spacer = tree.insert(col, Widget::Expand);
        let bottom = tree.insert(
            col,
            Widget::Label {
                text: "Bottom".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rs = tree.node_rect(spacer).unwrap();
        let rb = tree.node_rect(bottom).unwrap();

        // Spacer should have consumed most of the column height.
        assert!(
            rs.height > 300.0,
            "expand spacer should fill remaining height: got {}",
            rs.height
        );
        // Bottom label should be near column bottom.
        assert!(
            (rb.y + rb.height - 400.0).abs() < 1.0,
            "bottom label should be at column bottom: y={}, h={}, col_h=400",
            rb.y,
            rb.height
        );
    }

    #[test]
    fn two_expands_split_remaining_equally() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(300.0), Sizing::Fixed(50.0));

        // [Expand, Label, Expand] — label should be centered.
        let exp_a = tree.insert(row, Widget::Expand);
        let label = tree.insert(
            row,
            Widget::Label {
                text: "Center".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let exp_b = tree.insert(row, Widget::Expand);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let ra = tree.node_rect(exp_a).unwrap();
        let rb = tree.node_rect(exp_b).unwrap();
        let rl = tree.node_rect(label).unwrap();

        // Both expands should be equal width.
        assert!(
            (ra.width - rb.width).abs() < 1.0,
            "two expands should be equal: {} vs {}",
            ra.width,
            rb.width
        );
        // Label should be roughly centered.
        let label_center = rl.x + rl.width / 2.0;
        assert!(
            (label_center - 150.0).abs() < 5.0,
            "label should be centered: center={}, expected ~150",
            label_center
        );
    }

    #[test]
    fn expand_measures_as_zero() {
        let tree = WidgetTree::new();
        // Expand should report zero intrinsic size via the public API.
        // We test indirectly: insert into a Fit-sized row and verify it doesn't add width.
        let mut tree = tree;
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fit, Sizing::Fit);
        tree.insert(row, Widget::Expand);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rr = tree.node_rect(row).unwrap();
        // Fit-sized row with only an Expand child should have ~0 width.
        assert!(
            rr.width < 1.0,
            "Fit row with only Expand should be ~0 width: got {}",
            rr.width
        );
    }

    // ------------------------------------------------------------------
    // Column auto-layout (UI-101)
    // ------------------------------------------------------------------

    #[test]
    fn column_children_stacked_with_gap() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 4.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(col, Sizing::Fixed(200.0), Sizing::Fixed(400.0));

        let mut labels = Vec::new();
        for i in 0..5 {
            let l = tree.insert(
                col,
                Widget::Label {
                    text: format!("Line {i}"),
                    color: [1.0; 4],
                    font_size: 14.0,
                    font_family: FontFamily::default(),
                    wrap: false,
                },
            );
            labels.push(l);
        }

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        // Each label should start where the previous one ended + gap.
        for i in 1..labels.len() {
            let prev = tree.node_rect(labels[i - 1]).unwrap();
            let curr = tree.node_rect(labels[i]).unwrap();
            let expected_y = prev.y + prev.height + 4.0;
            assert!(
                (curr.y - expected_y).abs() < 0.1,
                "label {i} should start at y={expected_y}, got y={}",
                curr.y
            );
        }
    }

    #[test]
    fn column_cross_align_center() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: CrossAlign::Center,
        });
        tree.set_sizing(col, Sizing::Fixed(400.0), Sizing::Fixed(200.0));

        let label = tree.insert(
            col,
            Widget::Label {
                text: "Hi".into(), // 2 chars → narrow
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rl = tree.node_rect(label).unwrap();
        // Label width ≈ 2 * 8.4 = 16.8. Column is 400 wide.
        // Centered: x ≈ (400 - 16.8) / 2 ≈ 191.6
        let expected_center = (400.0 - rl.width) / 2.0;
        assert!(
            (rl.x - expected_center).abs() < 1.0,
            "label should be horizontally centered, x = {}, expected ≈ {}",
            rl.x,
            expected_center
        );
    }

    #[test]
    fn row_emits_no_draw_commands() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(200.0), Sizing::Fixed(50.0));

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // Row with no children should produce no draw commands.
        assert!(draw_list.panels.is_empty());
        assert!(draw_list.texts.is_empty());
    }

    // ------------------------------------------------------------------
    // Text wrapping (UI-102)
    // ------------------------------------------------------------------

    #[test]
    fn wrap_text_basic() {
        let lines = wrap_text("hello world foo bar", 11);
        assert_eq!(lines, vec!["hello world", "foo bar"]);
    }

    #[test]
    fn wrap_text_long_word() {
        let lines = wrap_text("supercalifragilistic", 10);
        // Single word exceeds max_chars — placed on its own line.
        assert_eq!(lines, vec!["supercalifragilistic"]);
    }

    #[test]
    fn wrap_text_empty() {
        let lines = wrap_text("", 10);
        assert_eq!(lines, vec![""]);
    }

    #[test]
    fn wrapped_label_height_exceeds_single_line() {
        let mut tree = WidgetTree::new();
        // A long text that should wrap within 100px.
        let long_text = "The quick brown fox jumps over the lazy dog and then some more words";
        let label = tree.insert_root(Widget::Label {
            text: long_text.into(),
            color: [1.0; 4],
            font_size: 14.0,
            font_family: FontFamily::default(),
            wrap: true,
        });
        tree.set_sizing(label, Sizing::Fixed(100.0), Sizing::Fit);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(label).unwrap();
        // Single line height would be 14.0. Wrapped should be taller.
        assert!(
            rect.height > 14.0,
            "wrapped label height should exceed single line: {}",
            rect.height
        );
    }

    #[test]
    fn wrapped_label_emits_multiple_text_commands() {
        let mut tree = WidgetTree::new();
        let long_text = "aaa bbb ccc ddd eee fff ggg hhh iii jjj";
        let label = tree.insert_root(Widget::Label {
            text: long_text.into(),
            color: [1.0; 4],
            font_size: 14.0,
            font_family: FontFamily::default(),
            wrap: true,
        });
        tree.set_sizing(label, Sizing::Fixed(80.0), Sizing::Fit);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // Should produce multiple text commands (one per wrapped line).
        assert!(
            draw_list.texts.len() > 1,
            "wrapped label should emit multiple TextCommands, got {}",
            draw_list.texts.len()
        );
    }

    #[test]
    fn unwrapped_label_single_line() {
        let mut tree = WidgetTree::new();
        let _label = tree.insert_root(Widget::Label {
            text: "short text".into(),
            color: [1.0; 4],
            font_size: 14.0,
            font_family: FontFamily::default(),
            wrap: false,
        });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        assert_eq!(draw_list.texts.len(), 1);
    }

    // ------------------------------------------------------------------
    // Min/Max constraints (UI-103)
    // ------------------------------------------------------------------

    #[test]
    fn constraints_min_width_enforced() {
        let mut tree = WidgetTree::new();
        let label = tree.insert_root(Widget::Label {
            text: "Hi".into(), // ~2 chars ≈ 16.8px wide
            color: [1.0; 4],
            font_size: 14.0,
            font_family: FontFamily::default(),
            wrap: false,
        });
        tree.set_constraints(
            label,
            Constraints {
                min_width: 200.0,
                min_height: 0.0,
                max_width: f32::MAX,
                max_height: f32::MAX,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(label).unwrap();
        assert!(
            rect.width >= 200.0,
            "min_width constraint should enforce width >= 200, got {}",
            rect.width
        );
    }

    #[test]
    fn constraints_max_width_enforced() {
        let mut tree = WidgetTree::new();
        let label = tree.insert_root(Widget::Label {
            text: "A very long label that should be wider than 50 pixels normally".into(),
            color: [1.0; 4],
            font_size: 14.0,
            font_family: FontFamily::default(),
            wrap: false,
        });
        tree.set_constraints(
            label,
            Constraints {
                min_width: 0.0,
                min_height: 0.0,
                max_width: 50.0,
                max_height: f32::MAX,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(label).unwrap();
        assert!(
            rect.width <= 50.0,
            "max_width constraint should enforce width <= 50, got {}",
            rect.width
        );
    }

    // ------------------------------------------------------------------
    // Scissor-rect clipping (UI-104)
    // ------------------------------------------------------------------

    #[test]
    fn clip_rect_propagates_to_draw_commands() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [1.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_sizing(panel, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        let clip = Rect {
            x: 10.0,
            y: 10.0,
            width: 180.0,
            height: 80.0,
        };
        tree.set_clip_rect(panel, Some(clip));

        let label = tree.insert(
            panel,
            Widget::Label {
                text: "Clipped text".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        // Child should inherit parent's clip_rect.
        let child_node = tree.get(label).unwrap();
        assert!(
            child_node.clip_rect.is_some(),
            "child should inherit clip_rect"
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // Panel command should carry the clip rect.
        assert!(
            draw_list.panels[0].clip.is_some(),
            "panel command should have clip"
        );
        let pc = draw_list.panels[0].clip.unwrap();
        assert!((pc.x - 10.0).abs() < 0.1);
        assert!((pc.width - 180.0).abs() < 0.1);

        // Text command should also carry the inherited clip rect.
        assert!(
            draw_list.texts[0].clip.is_some(),
            "text command should have clip"
        );
    }

    #[test]
    fn no_clip_by_default() {
        let mut tree = WidgetTree::new();
        let _label = tree.insert_root(Widget::Label {
            text: "No clip".into(),
            color: [1.0; 4],
            font_size: 14.0,
            font_family: FontFamily::default(),
            wrap: false,
        });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        assert!(
            draw_list.texts[0].clip.is_none(),
            "default should have no clip"
        );
    }

    // ------------------------------------------------------------------
    // Pause overlay (UI-105)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Edge-case tests (quality pass)
    // ------------------------------------------------------------------

    #[test]
    fn row_zero_children_measures_zero() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 4.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fit, Sizing::Fit);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let node = tree.get(row).unwrap();
        assert!((node.measured.width).abs() < 0.01);
        assert!((node.measured.height).abs() < 0.01);
    }

    #[test]
    fn column_zero_children_measures_zero() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 4.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(col, Sizing::Fit, Sizing::Fit);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let node = tree.get(col).unwrap();
        assert!((node.measured.width).abs() < 0.01);
        assert!((node.measured.height).abs() < 0.01);
    }

    #[test]
    fn row_single_child() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 4.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(400.0), Sizing::Fixed(50.0));

        let label = tree.insert(
            row,
            Widget::Label {
                text: "Only".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rl = tree.node_rect(label).unwrap();
        assert!(rl.width > 0.0, "single child should have width");
        // No gap applied since there's only one child.
    }

    #[test]
    fn row_all_percent_children() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(300.0), Sizing::Fixed(50.0));

        let a = tree.insert(
            row,
            Widget::Panel {
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
        );
        tree.set_sizing(a, Sizing::Percent(0.25), Sizing::Fit);

        let b = tree.insert(
            row,
            Widget::Panel {
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
        );
        tree.set_sizing(b, Sizing::Percent(0.75), Sizing::Fit);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let ra = tree.node_rect(a).unwrap();
        let rb = tree.node_rect(b).unwrap();
        // No fixed children, so all 300px goes to percent children.
        // 25% of 300 = 75, 75% of 300 = 225.
        assert!(
            (ra.width - 75.0).abs() < 1.0,
            "25% of 300 should be ~75, got {}",
            ra.width
        );
        assert!(
            (rb.width - 225.0).abs() < 1.0,
            "75% of 300 should be ~225, got {}",
            rb.width
        );
    }

    #[test]
    fn column_single_child() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 4.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(col, Sizing::Fixed(200.0), Sizing::Fixed(400.0));

        let label = tree.insert(
            col,
            Widget::Label {
                text: "Only".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rl = tree.node_rect(label).unwrap();
        assert!(rl.height > 0.0, "single child should have height");
    }

    #[test]
    fn wrap_text_short_no_wrap() {
        // Text shorter than max_chars should remain on one line.
        let lines = wrap_text("short", 20);
        assert_eq!(lines, vec!["short"]);
    }

    #[test]
    fn wrap_text_zero_max_chars() {
        // max_chars == 0 returns text unchanged.
        let lines = wrap_text("hello world", 0);
        assert_eq!(lines, vec!["hello world"]);
    }

    #[test]
    fn clip_rect_intersection() {
        let a = Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let b = Rect {
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
        };
        let result = a.intersect(&b).expect("should intersect");
        assert!((result.x - 50.0).abs() < 0.01);
        assert!((result.y - 50.0).abs() < 0.01);
        assert!((result.width - 50.0).abs() < 0.01);
        assert!((result.height - 50.0).abs() < 0.01);
    }

    #[test]
    fn clip_rect_no_intersection() {
        let a = Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let b = Rect {
            x: 20.0,
            y: 20.0,
            width: 10.0,
            height: 10.0,
        };
        assert!(a.intersect(&b).is_none());
    }

    #[test]
    fn clip_propagates_through_row() {
        let mut tree = WidgetTree::new();
        let row = tree.insert_root(Widget::Row {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(row, Sizing::Fixed(200.0), Sizing::Fixed(50.0));
        let clip = Rect {
            x: 10.0,
            y: 10.0,
            width: 180.0,
            height: 30.0,
        };
        tree.set_clip_rect(row, Some(clip));

        let label = tree.insert(
            row,
            Widget::Label {
                text: "Clipped".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let child_node = tree.get(label).unwrap();
        assert!(
            child_node.clip_rect.is_some(),
            "Row child should inherit clip_rect"
        );
        let c = child_node.clip_rect.unwrap();
        assert!((c.x - 10.0).abs() < 0.1);
        assert!((c.width - 180.0).abs() < 0.1);
    }

    #[test]
    fn clip_propagates_through_column() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: CrossAlign::Start,
        });
        tree.set_sizing(col, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        let clip = Rect {
            x: 5.0,
            y: 5.0,
            width: 190.0,
            height: 90.0,
        };
        tree.set_clip_rect(col, Some(clip));

        let label = tree.insert(
            col,
            Widget::Label {
                text: "Clipped".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let child_node = tree.get(label).unwrap();
        assert!(
            child_node.clip_rect.is_some(),
            "Column child should inherit clip_rect"
        );
    }

    // --- UI-200: Progress bar tests ---

    #[test]
    fn progress_bar_half_fraction_emits_half_width_foreground() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: widget::CrossAlign::Stretch,
        });
        tree.set_sizing(col, Sizing::Fixed(200.0), Sizing::Fixed(100.0));

        tree.insert(
            col,
            Widget::ProgressBar {
                fraction: 0.5,
                fg_color: [0.0, 1.0, 0.0, 1.0],
                bg_color: [0.2, 0.2, 0.2, 1.0],
                border_color: [1.0, 1.0, 1.0, 1.0],
                border_width: 1.0,
                height: 8.0,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // Should have at least 2 panels: background + foreground.
        assert!(draw_list.panels.len() >= 2);

        // Background panel = full 200px width.
        let bg = &draw_list.panels[0];
        assert!((bg.width - 200.0).abs() < 0.1);
        assert!((bg.height - 8.0).abs() < 0.1);

        // Foreground panel = inner width (200 - 2*border) * 0.5 = 99.
        let fg = &draw_list.panels[1];
        let inner_w = 200.0 - 2.0; // 1px border on each side
        assert!(
            (fg.width - inner_w * 0.5).abs() < 0.1,
            "foreground width should be {} but got {}",
            inner_w * 0.5,
            fg.width
        );
    }

    #[test]
    fn progress_bar_fraction_clamped() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: widget::CrossAlign::Stretch,
        });
        tree.set_sizing(col, Sizing::Fixed(100.0), Sizing::Fixed(100.0));

        let _bar = tree.insert(
            col,
            Widget::ProgressBar {
                fraction: 1.5, // over 1.0
                fg_color: [0.0, 1.0, 0.0, 1.0],
                bg_color: [0.2, 0.2, 0.2, 1.0],
                border_color: [0.0; 4],
                border_width: 0.0,
                height: 10.0,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );
        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // Foreground width should be clamped to full inner width, not 150%.
        let fg = &draw_list.panels[1];
        assert!(
            (fg.width - 100.0).abs() < 0.1,
            "clamped fraction should produce 100px foreground, got {}",
            fg.width
        );
    }

    #[test]
    fn progress_bar_zero_fraction_no_foreground() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: widget::CrossAlign::Stretch,
        });
        tree.set_sizing(col, Sizing::Fixed(100.0), Sizing::Fixed(100.0));

        let _bar = tree.insert(
            col,
            Widget::ProgressBar {
                fraction: 0.0,
                fg_color: [0.0, 1.0, 0.0, 1.0],
                bg_color: [0.2, 0.2, 0.2, 1.0],
                border_color: [0.0; 4],
                border_width: 0.0,
                height: 10.0,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );
        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // Only 1 panel: the background. No foreground for fraction 0.
        assert_eq!(
            draw_list.panels.len(),
            1,
            "fraction 0 should emit only background"
        );
    }

    #[test]
    fn progress_bar_intrinsic_height() {
        let tree = {
            let mut t = WidgetTree::new();
            t.insert_root(Widget::ProgressBar {
                fraction: 0.5,
                fg_color: [0.0, 1.0, 0.0, 1.0],
                bg_color: [0.2, 0.2, 0.2, 1.0],
                border_color: [0.0; 4],
                border_width: 0.0,
                height: 12.0,
            });
            t
        };
        let root = tree.roots()[0];
        let measured = tree.measure_node(root, &mut HeuristicMeasurer);
        assert!(
            (measured.width - 0.0).abs() < 0.01,
            "intrinsic width should be 0 (stretch)"
        );
        assert!(
            (measured.height - 12.0).abs() < 0.01,
            "intrinsic height should match field"
        );
    }

    // --- UI-201: Separator tests ---

    #[test]
    fn horizontal_separator_stretches_to_column_width() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: widget::CrossAlign::Stretch,
        });
        tree.set_sizing(col, Sizing::Fixed(300.0), Sizing::Fixed(100.0));

        let sep = tree.insert(
            col,
            Widget::Separator {
                color: [1.0, 0.8, 0.3, 0.3],
                thickness: 1.0,
                horizontal: true,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let sep_node = tree.get(sep).unwrap();
        assert!(
            (sep_node.rect.width - 300.0).abs() < 0.1,
            "horizontal separator width should match column: got {}",
            sep_node.rect.width
        );
        assert!(
            (sep_node.rect.height - 1.0).abs() < 0.1,
            "horizontal separator height should be thickness: got {}",
            sep_node.rect.height
        );
    }

    // --- UI-202c: Icon widget tests ---

    #[test]
    fn icon_measures_square() {
        let mut tree = WidgetTree::new();
        let icon = tree.insert_root(Widget::Icon {
            sprite: "heart".into(),
            size: 16.0,
            tint: None,
        });
        let measured = tree.measure_node(icon, &mut HeuristicMeasurer);
        assert!((measured.width - 16.0).abs() < 0.01);
        assert!((measured.height - 16.0).abs() < 0.01);
    }

    #[test]
    fn icon_emits_sprite_command() {
        let mut tree = WidgetTree::new();
        let icon = tree.insert_root(Widget::Icon {
            sprite: "sword".into(),
            size: 24.0,
            tint: Some([1.0, 0.0, 0.0, 1.0]),
        });
        tree.set_sizing(icon, Sizing::Fixed(24.0), Sizing::Fixed(24.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        assert_eq!(
            draw_list.sprites.len(),
            1,
            "Icon should emit exactly 1 SpriteCommand"
        );
        assert_eq!(draw_list.sprites[0].sprite, "sword");
        assert!((draw_list.sprites[0].width - 24.0).abs() < 0.1);
        assert!(
            (draw_list.sprites[0].tint[0] - 1.0).abs() < 0.01,
            "red tint"
        );
        assert!(
            (draw_list.sprites[0].tint[1] - 0.0).abs() < 0.01,
            "no green"
        );
    }

    #[test]
    fn icon_default_tint_is_white() {
        let mut tree = WidgetTree::new();
        let icon = tree.insert_root(Widget::Icon {
            sprite: "shield".into(),
            size: 16.0,
            tint: None,
        });
        tree.set_sizing(icon, Sizing::Fixed(16.0), Sizing::Fixed(16.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        assert_eq!(draw_list.sprites.len(), 1);
        let tint = draw_list.sprites[0].tint;
        assert!((tint[0] - 1.0).abs() < 0.01);
        assert!((tint[1] - 1.0).abs() < 0.01);
        assert!((tint[2] - 1.0).abs() < 0.01);
        assert!((tint[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn separator_emits_single_panel_no_border() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: widget::CrossAlign::Stretch,
        });
        tree.set_sizing(col, Sizing::Fixed(200.0), Sizing::Fixed(100.0));

        tree.insert(
            col,
            Widget::Separator {
                color: [1.0, 0.8, 0.3, 0.3],
                thickness: 2.0,
                horizontal: true,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );
        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        assert_eq!(
            draw_list.panels.len(),
            1,
            "separator should emit exactly 1 panel"
        );
        let panel = &draw_list.panels[0];
        assert!(
            (panel.border_width - 0.0).abs() < 0.01,
            "separator should have no border"
        );
        assert!(
            (panel.shadow_width - 0.0).abs() < 0.01,
            "separator should have no shadow"
        );
    }

    #[test]
    fn dropdown_measure_uses_widest_option() {
        let mut tree = WidgetTree::new();
        let dd = tree.insert_root(Widget::Dropdown {
            selected: 0,
            options: vec!["A".into(), "Long Option".into(), "B".into()],
            open: false,
            color: [1.0; 4],
            bg_color: [0.2; 4],
            font_size: 14.0,
        });
        let size = tree.measure_node(dd, &mut HeuristicMeasurer);
        // Width should be based on "Long Option" (11 chars), not "A" (1 char).
        let char_w = 14.0 * 0.6; // scale = 1.0
        let expected_min = 11.0 * char_w; // widest option text
        assert!(
            size.width > expected_min,
            "dropdown width {:.1} should exceed widest option text width {:.1}",
            size.width,
            expected_min
        );
    }

    #[test]
    fn dropdown_closed_emits_panel_and_two_texts() {
        let mut tree = WidgetTree::new();
        let dd = tree.insert_root(Widget::Dropdown {
            selected: 1,
            options: vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
            open: false,
            color: [1.0; 4],
            bg_color: [0.2, 0.2, 0.2, 1.0],
            font_size: 14.0,
        });
        tree.set_sizing(dd, Sizing::Fixed(200.0), Sizing::Fixed(30.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 1 panel (trigger bg) + 2 texts (selected label + arrow).
        assert_eq!(draw_list.panels.len(), 1, "closed dropdown: 1 panel");
        assert_eq!(draw_list.texts.len(), 2, "closed dropdown: 2 texts");
        assert_eq!(draw_list.texts[0].text, "Beta");
    }

    #[test]
    fn dropdown_open_emits_overlay_panels_and_option_texts() {
        let mut tree = WidgetTree::new();
        let dd = tree.insert_root(Widget::Dropdown {
            selected: 0,
            options: vec!["One".into(), "Two".into(), "Three".into()],
            open: true,
            color: [1.0; 4],
            bg_color: [0.2; 4],
            font_size: 14.0,
        });
        tree.set_sizing(dd, Sizing::Fixed(200.0), Sizing::Fixed(30.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 2 panels: trigger bg + options overlay bg.
        assert_eq!(draw_list.panels.len(), 2, "open dropdown: 2 panels");
        // 5 texts: selected label + arrow + 3 option labels.
        assert_eq!(
            draw_list.texts.len(),
            5,
            "open dropdown: 2 + 3 option texts"
        );
        // Options overlay panel should be below trigger.
        assert!(
            draw_list.panels[1].y > draw_list.panels[0].y,
            "overlay should be below trigger"
        );
    }

    #[test]
    fn dropdown_apply_opacity() {
        let mut tree = WidgetTree::new();
        let root = tree.insert_root(Widget::Column {
            gap: 0.0,
            align: widget::CrossAlign::Start,
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
    fn checkbox_measure_includes_box_and_label() {
        let mut tree = WidgetTree::new();
        let cb = tree.insert_root(Widget::Checkbox {
            checked: false,
            label: "Enable".into(),
            color: [1.0; 4],
            font_size: 14.0,
        });
        let size = tree.measure_node(cb, &mut HeuristicMeasurer);
        // box_size=16 + gap=6 + label "Enable" (6 chars) * char_w(8.4) ≈ 72.4
        assert!(
            size.width > 60.0,
            "width {:.1} should include box + gap + label",
            size.width
        );
        assert!(
            size.height >= 16.0,
            "height {:.1} should be at least box_size",
            size.height
        );
    }

    #[test]
    fn checkbox_unchecked_emits_box_and_label_only() {
        let mut tree = WidgetTree::new();
        let cb = tree.insert_root(Widget::Checkbox {
            checked: false,
            label: "Option".into(),
            color: [1.0; 4],
            font_size: 14.0,
        });
        tree.set_sizing(cb, Sizing::Fixed(200.0), Sizing::Fixed(20.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 1 panel (box border), 1 text (label). No checkmark text.
        assert_eq!(draw_list.panels.len(), 1, "unchecked: 1 panel for box");
        assert_eq!(draw_list.texts.len(), 1, "unchecked: 1 text for label");
        assert_eq!(draw_list.texts[0].text, "Option");
    }

    #[test]
    fn checkbox_checked_emits_checkmark() {
        let mut tree = WidgetTree::new();
        let cb = tree.insert_root(Widget::Checkbox {
            checked: true,
            label: "Toggle".into(),
            color: [1.0; 4],
            font_size: 14.0,
        });
        tree.set_sizing(cb, Sizing::Fixed(200.0), Sizing::Fixed(20.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 1 panel (box border), 2 texts (checkmark + label).
        assert_eq!(draw_list.panels.len(), 1, "checked: 1 panel for box");
        assert_eq!(
            draw_list.texts.len(),
            2,
            "checked: 2 texts (checkmark + label)"
        );
        assert_eq!(draw_list.texts[0].text, "\u{2713}");
        assert_eq!(draw_list.texts[1].text, "Toggle");
    }

    #[test]
    fn slider_measure_uses_width_field() {
        let mut tree = WidgetTree::new();
        let sl = tree.insert_root(Widget::Slider {
            value: 0.5,
            min: 0.0,
            max: 1.0,
            track_color: [0.3; 4],
            thumb_color: [0.8; 4],
            width: 200.0,
        });
        let size = tree.measure_node(sl, &mut HeuristicMeasurer);
        assert!((size.width - 200.0).abs() < 0.01);
        assert!(
            (size.height - 16.0).abs() < 0.01,
            "height should be thumb_size 16"
        );
    }

    #[test]
    fn slider_thumb_at_midpoint() {
        let mut tree = WidgetTree::new();
        let sl = tree.insert_root(Widget::Slider {
            value: 0.5,
            min: 0.0,
            max: 1.0,
            track_color: [0.3, 0.3, 0.3, 1.0],
            thumb_color: [0.8, 0.8, 0.8, 1.0],
            width: 200.0,
        });
        tree.set_sizing(sl, Sizing::Fixed(200.0), Sizing::Fixed(16.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 2 panels: track + thumb.
        assert_eq!(draw_list.panels.len(), 2, "slider: track + thumb");
        let thumb = &draw_list.panels[1];
        // thumb_x = 0 + 0.5 * (200 - 16) = 92
        let expected_x = 0.5 * (200.0 - 16.0);
        assert!(
            (thumb.x - expected_x).abs() < 0.5,
            "thumb x {:.1} should be near {:.1}",
            thumb.x,
            expected_x
        );
    }

    #[test]
    fn slider_thumb_clamped_at_extremes() {
        let mut tree = WidgetTree::new();
        // Value beyond max.
        let sl = tree.insert_root(Widget::Slider {
            value: 2.0,
            min: 0.0,
            max: 1.0,
            track_color: [0.3; 4],
            thumb_color: [0.8; 4],
            width: 100.0,
        });
        tree.set_sizing(sl, Sizing::Fixed(100.0), Sizing::Fixed(16.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        let thumb = &draw_list.panels[1];
        // t clamped to 1.0, thumb_x = 0 + 1.0 * (100 - 16) = 84
        let expected_x = 100.0 - 16.0;
        assert!(
            (thumb.x - expected_x).abs() < 0.5,
            "thumb at max: x {:.1} should be near {:.1}",
            thumb.x,
            expected_x
        );
    }

    #[test]
    fn text_input_stretch_width_intrinsic_zero() {
        let mut tree = WidgetTree::new();
        let ti = tree.insert_root(Widget::TextInput {
            text: "hello".into(),
            cursor_pos: 5,
            color: [1.0; 4],
            bg_color: [0.1; 4],
            font_size: 14.0,
            placeholder: "Type here".into(),
            focused: false,
        });
        let size = tree.measure_node(ti, &mut HeuristicMeasurer);
        assert!(
            (size.width - 0.0).abs() < 0.01,
            "intrinsic width should be 0 (stretch)"
        );
        assert!(
            (size.height - 14.0).abs() < 0.01,
            "intrinsic height should be text only (padding added by layout)"
        );
    }

    #[test]
    fn text_input_shows_text_when_non_empty() {
        let mut tree = WidgetTree::new();
        let ti = tree.insert_root(Widget::TextInput {
            text: "hello".into(),
            cursor_pos: 5,
            color: [1.0; 4],
            bg_color: [0.1; 4],
            font_size: 14.0,
            placeholder: "Type here".into(),
            focused: false,
        });
        tree.set_sizing(ti, Sizing::Fixed(200.0), Sizing::Fixed(22.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 1 panel (bg), 1 text (content). No cursor when not focused.
        assert_eq!(draw_list.panels.len(), 1, "unfocused: 1 panel (bg)");
        assert_eq!(draw_list.texts.len(), 1, "unfocused: 1 text");
        assert_eq!(draw_list.texts[0].text, "hello");
    }

    #[test]
    fn text_input_shows_placeholder_when_empty() {
        let mut tree = WidgetTree::new();
        let ti = tree.insert_root(Widget::TextInput {
            text: String::new(),
            cursor_pos: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            bg_color: [0.1; 4],
            font_size: 14.0,
            placeholder: "Search...".into(),
            focused: false,
        });
        tree.set_sizing(ti, Sizing::Fixed(200.0), Sizing::Fixed(22.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        assert_eq!(draw_list.texts[0].text, "Search...");
        // Placeholder should be dimmed (alpha * 0.5).
        assert!(
            (draw_list.texts[0].color[3] - 0.5).abs() < 0.01,
            "placeholder alpha should be dimmed"
        );
    }

    #[test]
    fn text_input_focused_shows_cursor() {
        let mut tree = WidgetTree::new();
        let ti = tree.insert_root(Widget::TextInput {
            text: "abc".into(),
            cursor_pos: 2,
            color: [1.0; 4],
            bg_color: [0.1; 4],
            font_size: 14.0,
            placeholder: String::new(),
            focused: true,
        });
        tree.set_sizing(ti, Sizing::Fixed(200.0), Sizing::Fixed(22.0));
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 2 panels: bg + cursor line.
        assert_eq!(draw_list.panels.len(), 2, "focused: bg + cursor");
        let cursor = &draw_list.panels[1];
        assert!(
            (cursor.width - 1.0).abs() < 0.01,
            "cursor should be 1px wide"
        );
    }

    #[test]
    fn collapsible_collapsed_measures_header_only() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Collapsible {
            header: "Section".into(),
            expanded: false,
            color: [1.0; 4],
            font_size: 14.0,
        });
        // Add a child that should NOT contribute to height.
        tree.insert(
            col,
            Widget::Label {
                text: "Hidden content".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let size = tree.measure_node(col, &mut HeuristicMeasurer);
        // Header only: line_height * scale + 4.0 = 14 + 4 = 18
        assert!(
            (size.height - 18.0).abs() < 0.5,
            "collapsed height {:.1} should be header-only (~18)",
            size.height
        );
    }

    #[test]
    fn collapsible_expanded_includes_children() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Collapsible {
            header: "Section".into(),
            expanded: true,
            color: [1.0; 4],
            font_size: 14.0,
        });
        tree.insert(
            col,
            Widget::Label {
                text: "Content".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let size = tree.measure_node(col, &mut HeuristicMeasurer);
        // Header (18) + child label (14) = 32
        assert!(
            size.height > 25.0,
            "expanded height {:.1} should include header + children",
            size.height
        );
    }

    #[test]
    fn collapsible_collapsed_skips_child_draw() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Collapsible {
            header: "Items".into(),
            expanded: false,
            color: [1.0; 4],
            font_size: 14.0,
        });
        tree.set_sizing(col, Sizing::Fixed(300.0), Sizing::Fixed(200.0));
        tree.insert(
            col,
            Widget::Label {
                text: "Should not appear".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 2 texts: triangle indicator + header label. No child labels.
        assert_eq!(
            draw_list.texts.len(),
            2,
            "collapsed: indicator + header only"
        );
        assert_eq!(draw_list.texts[1].text, "Items");
    }

    #[test]
    fn collapsible_expanded_draws_children() {
        let mut tree = WidgetTree::new();
        let col = tree.insert_root(Widget::Collapsible {
            header: "Items".into(),
            expanded: true,
            color: [1.0; 4],
            font_size: 14.0,
        });
        tree.set_sizing(col, Sizing::Fixed(300.0), Sizing::Fixed(200.0));
        tree.insert(
            col,
            Widget::Label {
                text: "Child A".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        tree.insert(
            col,
            Widget::Label {
                text: "Child B".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 4 texts: triangle + header + 2 children.
        assert_eq!(
            draw_list.texts.len(),
            4,
            "expanded: indicator + header + 2 children"
        );
        assert_eq!(draw_list.texts[0].text, "\u{25BC}"); // down triangle
        assert_eq!(draw_list.texts[1].text, "Items");
        assert_eq!(draw_list.texts[2].text, "Child A");
        assert_eq!(draw_list.texts[3].text, "Child B");
    }

    // ------------------------------------------------------------------
    // Z-order tier tests (UI-307)
    // ------------------------------------------------------------------

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

        // Hit at overlay position → overlay wins (higher tier).
        assert_eq!(tree.hit_test(20.0, 20.0), Some(overlay));
        // Hit outside overlay but inside panel → panel wins.
        assert_eq!(tree.hit_test(100.0, 100.0), Some(panel));
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

    // ------------------------------------------------------------------
    // TabContainer tests (UI-301)
    // ------------------------------------------------------------------

    #[test]
    fn tab_container_measure_includes_tab_bar_and_content() {
        let mut tree = WidgetTree::new();
        let tc = tree.insert_root(Widget::TabContainer {
            tabs: vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
            active: 0,
            tab_color: [0.5; 4],
            active_color: [0.8; 4],
            font_size: 14.0,
        });
        // Add a content child (active tab's content).
        let label = tree.insert(
            tc,
            Widget::Label {
                text: "Tab content".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        tree.set_sizing(label, Sizing::Fixed(120.0), Sizing::Fixed(30.0));

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let node = tree.get(tc).unwrap();
        // Tab bar height = line_height * scale + 6 = 14 + 6 = 20.
        // Label intrinsic height = font_size = 14. Plus child padding (none) = 14.
        // Fixed sizing on child is used by layout_node, not measure_node.
        // Total measured height = 20 + 14 = 34. Layout may differ from Fit.
        let tab_bar_h = 14.0 + 6.0;
        assert!(
            node.rect.height >= tab_bar_h + 14.0,
            "height {} should be >= tab_bar(20) + label(14)",
            node.rect.height
        );
    }

    #[test]
    fn tab_container_draws_tab_buttons_and_content() {
        let mut tree = WidgetTree::new();
        let tc = tree.insert_root(Widget::TabContainer {
            tabs: vec!["Tab A".into(), "Tab B".into()],
            active: 1,
            tab_color: [0.4; 4],
            active_color: [0.9, 0.9, 0.9, 1.0],
            font_size: 14.0,
        });
        tree.set_position(tc, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(tc, Sizing::Fixed(300.0), Sizing::Fixed(200.0));

        tree.insert(
            tc,
            Widget::Label {
                text: "Content B".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // 2 tab button panels.
        assert_eq!(draw_list.panels.len(), 2, "2 tab bg panels");
        // 3 texts: "Tab A" + "Tab B" + "Content B".
        assert_eq!(draw_list.texts.len(), 3, "2 tab labels + 1 content label");
        assert_eq!(draw_list.texts[0].text, "Tab A");
        assert_eq!(draw_list.texts[1].text, "Tab B");
        assert_eq!(draw_list.texts[2].text, "Content B");
    }

    #[test]
    fn tab_container_active_tab_gets_active_color() {
        let mut tree = WidgetTree::new();
        let tc = tree.insert_root(Widget::TabContainer {
            tabs: vec!["One".into(), "Two".into()],
            active: 0,
            tab_color: [0.3, 0.3, 0.3, 1.0],
            active_color: [0.9, 0.9, 0.9, 1.0],
            font_size: 14.0,
        });
        tree.set_position(tc, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(tc, Sizing::Fixed(200.0), Sizing::Fixed(100.0));

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list, &mut HeuristicMeasurer);

        // First tab (active=0) gets active_color.
        assert!(
            (draw_list.panels[0].bg_color[0] - 0.9).abs() < 0.01,
            "active tab has active_color"
        );
        // Second tab gets inactive tab_color.
        assert!(
            (draw_list.panels[1].bg_color[0] - 0.3).abs() < 0.01,
            "inactive tab has tab_color"
        );
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

    /// Regression: Row-in-Column must dispatch Row layout for its children,
    /// giving them distinct x positions instead of overlapping at x=0.
    #[test]
    fn row_in_column_children_have_distinct_x() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(panel, Sizing::Fixed(400.0), Sizing::Fixed(300.0));

        let col = tree.insert(
            panel,
            Widget::Column {
                gap: 4.0,
                align: widget::CrossAlign::Start,
            },
        );
        tree.set_sizing(col, Sizing::Fixed(400.0), Sizing::Fit);

        let row = tree.insert(
            col,
            Widget::Row {
                gap: 8.0,
                align: widget::CrossAlign::Center,
            },
        );
        tree.set_sizing(row, Sizing::Fixed(400.0), Sizing::Fit);

        let label_a = tree.insert(
            row,
            Widget::Label {
                text: "AAA".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let label_b = tree.insert(
            row,
            Widget::Label {
                text: "BBB".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let ra = tree.get(label_a).unwrap().rect;
        let rb = tree.get(label_b).unwrap().rect;
        assert!(
            rb.x > ra.x,
            "Row children must have distinct x: a.x={}, b.x={}",
            ra.x,
            rb.x
        );
    }

    /// Regression: Column-in-Collapsible must dispatch Column layout,
    /// giving children distinct y positions instead of overlapping at y=0.
    #[test]
    fn column_in_collapsible_children_have_distinct_y() {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
        tree.set_sizing(panel, Sizing::Fixed(400.0), Sizing::Fixed(400.0));

        let collapsible = tree.insert(
            panel,
            Widget::Collapsible {
                header: "Section".into(),
                expanded: true,
                color: [1.0; 4],
                font_size: 14.0,
            },
        );
        tree.set_sizing(collapsible, Sizing::Fixed(400.0), Sizing::Fit);

        let inner_col = tree.insert(
            collapsible,
            Widget::Column {
                gap: 4.0,
                align: widget::CrossAlign::Start,
            },
        );
        tree.set_sizing(inner_col, Sizing::Fixed(400.0), Sizing::Fit);

        let label_a = tree.insert(
            inner_col,
            Widget::Label {
                text: "First".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );
        let label_b = tree.insert(
            inner_col,
            Widget::Label {
                text: "Second".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let ra = tree.get(label_a).unwrap().rect;
        let rb = tree.get(label_b).unwrap().rect;
        assert!(
            rb.y > ra.y,
            "Column children must have distinct y: a.y={}, b.y={}",
            ra.y,
            rb.y
        );
    }
}
