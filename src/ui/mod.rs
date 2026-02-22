mod animation;
pub(crate) mod demo;
mod draw;
mod input;
mod keybindings;
mod theme;
mod widget;

#[allow(unused_imports)] // Public API: used by main.rs for animation (UI-W05).
pub use animation::{Animator, Easing};
#[allow(unused_imports)] // Public API: used by game panels constructing widgets.
pub use draw::{DrawList, FontFamily, PanelCommand, RichTextCommand, TextCommand, TextSpan};
#[allow(unused_imports)] // Public API: used by main.rs for input routing (UI-W02).
pub use input::{MapClick, MouseButton, UiEvent, UiState};
#[allow(unused_imports)] // Public API: used by main.rs for keyboard shortcuts (UI-I03).
pub use keybindings::{Action, KeyBindings, KeyCombo, ModifierFlags};
pub use theme::Theme;
#[allow(unused_imports)] // Public API: used by game panels setting tooltip content.
pub use widget::{CrossAlign, TooltipContent, Widget};

use slotmap::{SlotMap, new_key_type};

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
}

// ---------------------------------------------------------------------------
// WidgetTree
// ---------------------------------------------------------------------------

/// Arena-backed retained widget tree.
pub struct WidgetTree {
    arena: SlotMap<WidgetId, WidgetNode>,
    roots: Vec<WidgetId>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            arena: SlotMap::with_key(),
            roots: Vec::new(),
        }
    }

    /// Insert a widget as a root (no parent).
    pub fn insert_root(&mut self, widget: Widget) -> WidgetId {
        let id = self.arena.insert(WidgetNode {
            widget,
            parent: None,
            children: Vec::new(),
            position: Position::default(),
            width: Sizing::default(),
            height: Sizing::default(),
            padding: Edges::ZERO,
            margin: Edges::ZERO,
            dirty: true,
            rect: Rect::default(),
            measured: Size::default(),
            tooltip: None,
            constraints: None,
            clip_rect: None,
        });
        self.roots.push(id);
        id
    }

    /// Insert a widget as a child of `parent`. Returns the new widget's id.
    pub fn insert(&mut self, parent: WidgetId, widget: Widget) -> WidgetId {
        let id = self.arena.insert(WidgetNode {
            widget,
            parent: Some(parent),
            children: Vec::new(),
            position: Position::default(),
            width: Sizing::default(),
            height: Sizing::default(),
            padding: Edges::ZERO,
            margin: Edges::ZERO,
            dirty: true,
            rect: Rect::default(),
            measured: Size::default(),
            tooltip: None,
            constraints: None,
            clip_rect: None,
        });
        if let Some(parent_node) = self.arena.get_mut(parent) {
            parent_node.children.push(id);
            parent_node.dirty = true;
        }
        id
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
        self.roots.retain(|r| *r != id);

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

    /// Root widget ids.
    pub fn roots(&self) -> &[WidgetId] {
        &self.roots
    }

    // ------------------------------------------------------------------
    // Hit testing
    // ------------------------------------------------------------------

    /// Find the topmost widget whose rect contains the point (x, y).
    /// Walks back-to-front: last child / last root is topmost.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<WidgetId> {
        for &root in self.roots.iter().rev() {
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
        for &root in &self.roots {
            self.collect_focusable(root, &mut result);
        }
        result
    }

    fn collect_focusable(&self, id: WidgetId, out: &mut Vec<WidgetId>) {
        if let Some(node) = self.arena.get(id) {
            if matches!(
                node.widget,
                Widget::Button { .. } | Widget::ScrollList { .. }
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
    pub fn layout(&mut self, screen: Size, line_height: f32) {
        let root_ids: Vec<WidgetId> = self.roots.clone();
        for root in root_ids {
            self.layout_node(
                root,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: screen.width,
                    height: screen.height,
                },
                line_height,
            );
        }
    }

    fn layout_node(&mut self, id: WidgetId, parent_content: Rect, line_height: f32) {
        // Measure intrinsic size.
        let measured = self.measure_node(id, line_height);

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
        };

        // For wrapped labels with Fit height, recompute height based on resolved width (UI-102).
        let resolved_h = if let Widget::Label {
            text,
            font_size,
            wrap: true,
            ..
        } = &node.widget
        {
            if matches!(node.height, Sizing::Fit) && resolved_w > 0.0 {
                let scale = font_size / line_height;
                let char_w = line_height * 0.6 * scale;
                let line_h = line_height * scale;
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

            // First pass: measure all children, identify Percent-width children.
            let mut child_infos: Vec<(WidgetId, Size, Sizing, Edges, Edges)> = Vec::new();
            let mut fixed_total_w: f32 = 0.0;
            let mut percent_total: f32 = 0.0;
            for &child_id in &children {
                let child_measured = self.measure_node(child_id, line_height);
                let Some(child) = self.arena.get(child_id) else {
                    continue;
                };
                let cw = child.width;
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
                    child_node.dirty = false;
                }

                // Recurse into child's children.
                let child_content = Rect {
                    x: cursor_x + cm.left + cp.left,
                    y: child_y + cp.top,
                    width: (child_w - cp.horizontal()).max(0.0),
                    height: (stretched_h - cp.vertical()).max(0.0),
                };
                let grandchildren: Vec<WidgetId> = self
                    .arena
                    .get(*child_id)
                    .map(|n| n.children.clone())
                    .unwrap_or_default();
                for gc in grandchildren {
                    self.layout_node(gc, child_content, line_height);
                }

                cursor_x += child_total_w + gap;
            }
            return;
        }

        // Column: lay out children top-to-bottom with gap spacing (UI-101).
        if let Widget::Column { gap, align } = &node.widget {
            let gap = *gap;
            let align = *align;
            let children: Vec<WidgetId> = node.children.clone();

            // First pass: measure all children, identify Percent-height children.
            let mut child_infos: Vec<(WidgetId, Size, Sizing, Edges, Edges)> = Vec::new();
            let mut fixed_total_h: f32 = 0.0;
            let mut percent_total: f32 = 0.0;
            for &child_id in &children {
                let child_measured = self.measure_node(child_id, line_height);
                let Some(child) = self.arena.get(child_id) else {
                    continue;
                };
                let ch = child.height;
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
                    child_node.dirty = false;
                }

                // Recurse into child's children.
                let child_content = Rect {
                    x: child_x + cp.left,
                    y: cursor_y + cm.top + cp.top,
                    width: (stretched_w - cp.horizontal()).max(0.0),
                    height: (child_h - cp.vertical()).max(0.0),
                };
                let grandchildren: Vec<WidgetId> = self
                    .arena
                    .get(*child_id)
                    .map(|n| n.children.clone())
                    .unwrap_or_default();
                for gc in grandchildren {
                    self.layout_node(gc, child_content, line_height);
                }

                cursor_y += child_total_h + gap;
            }
            return;
        }

        // ScrollList positions children in a vertical stack with virtual scrolling.
        if let Widget::ScrollList {
            item_height,
            scroll_offset,
            scrollbar_width,
            ..
        } = &node.widget
        {
            let ih = *item_height;
            let so = *scroll_offset;
            let sbw = *scrollbar_width;
            let children: Vec<WidgetId> = node.children.clone();
            let viewport_h = content.height;
            let content_w = (content.width - sbw).max(0.0);

            for (i, child_id) in children.iter().enumerate() {
                let item_y = i as f32 * ih - so;

                // Virtual scrolling: skip items outside viewport.
                if item_y + ih < 0.0 || item_y >= viewport_h {
                    if let Some(child_node) = self.arena.get_mut(*child_id) {
                        child_node.rect = Rect::default();
                        child_node.dirty = false;
                    }
                    continue;
                }

                // Layout visible item: set rect directly, then recurse for children.
                self.layout_scroll_item(
                    *child_id,
                    content.x,
                    content.y + item_y,
                    content_w,
                    ih,
                    line_height,
                );
            }
            return;
        }

        // Propagate clip_rect from parent to children (UI-104).
        let parent_clip = node.clip_rect;
        let children: Vec<WidgetId> = node.children.clone();
        for child in &children {
            if let Some(child_node) = self.arena.get_mut(*child) {
                child_node.clip_rect = parent_clip;
            }
        }
        for child in children {
            self.layout_node(child, content, line_height);
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
        line_height: f32,
    ) {
        let measured = self.measure_node(id, line_height);

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

        // Content area for children (inside padding).
        let content = Rect {
            x: x + node.padding.left,
            y: y + node.padding.top,
            width: (width - node.padding.horizontal()).max(0.0),
            height: (height - node.padding.vertical()).max(0.0),
        };

        let children: Vec<WidgetId> = node.children.clone();
        for child in children {
            self.layout_node(child, content, line_height);
        }
    }

    /// Measure intrinsic size of a widget (content only, no padding).
    pub fn measure_node(&self, id: WidgetId, line_height: f32) -> Size {
        let Some(node) = self.arena.get(id) else {
            return Size::default();
        };

        match &node.widget {
            Widget::Label {
                text, font_size, ..
            } => {
                // Approximate: char count * estimated glyph width, one line height.
                // Font size ratio relative to base line_height.
                let scale = font_size / line_height;
                let char_w = line_height * 0.6 * scale; // rough estimate
                let h = line_height * scale;
                Size {
                    width: text.len() as f32 * char_w,
                    height: h,
                }
            }
            Widget::Button {
                text, font_size, ..
            } => {
                let scale = font_size / line_height;
                let char_w = line_height * 0.6 * scale;
                let h = line_height * scale;
                // Button adds internal padding (8px horizontal, 4px vertical).
                Size {
                    width: text.len() as f32 * char_w + 16.0,
                    height: h + 8.0,
                }
            }
            Widget::RichText { spans, font_size } => {
                // Approximate: sum of all span char counts * estimated glyph width.
                let scale = font_size / line_height;
                let char_w = line_height * 0.6 * scale;
                let h = line_height * scale;
                let total_chars: usize = spans.iter().map(|s| s.text.len()).sum();
                Size {
                    width: total_chars as f32 * char_w,
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
                        let child_measured = self.measure_node(child_id, line_height);
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
                        let child_measured = self.measure_node(child_id, line_height);
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
                        let child_measured = self.measure_node(child_id, line_height);
                        let (cx, cy) = match child.position {
                            Position::Fixed { x, y } => (x, y),
                            Position::Percent { .. } => (0.0, 0.0),
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
                ..
            } => {
                // Total content height = items * item_height.
                // Width = widest child + scrollbar.
                let mut max_w: f32 = 0.0;
                for &child_id in &node.children {
                    let child_measured = self.measure_node(child_id, line_height);
                    max_w = max_w.max(child_measured.width);
                }
                Size {
                    width: max_w + scrollbar_width,
                    height: node.children.len() as f32 * item_height,
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Draw
    // ------------------------------------------------------------------

    /// Walk the tree and emit draw commands into a `DrawList`.
    pub fn draw(&self, draw_list: &mut DrawList) {
        for &root in &self.roots {
            self.draw_node(root, draw_list);
        }
    }

    fn draw_node(&self, id: WidgetId, draw_list: &mut DrawList) {
        let Some(node) = self.arena.get(id) else {
            return;
        };

        let clip = node.clip_rect;

        match &node.widget {
            // Row and Column are transparent containers — no draw commands.
            Widget::Row { .. } | Widget::Column { .. } => {}
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
                    let scale = font_size / 14.0;
                    let char_w = 14.0 * 0.6 * scale;
                    let line_h = 14.0 * scale;
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
                    border_width: 1.0,
                    shadow_width: 0.0,
                    clip,
                });
                draw_list.texts.push(TextCommand {
                    text: text.clone(),
                    x: node.rect.x + 8.0,
                    y: node.rect.y + 4.0,
                    color: *color,
                    font_size: *font_size,
                    font_family: *font_family,
                    clip,
                });
            }
            Widget::RichText { spans, font_size } => {
                draw_list.rich_texts.push(RichTextCommand {
                    spans: spans.clone(),
                    x: node.rect.x,
                    y: node.rect.y,
                    font_size: *font_size,
                    clip,
                });
            }
            Widget::ScrollList {
                bg_color,
                border_color,
                border_width,
                item_height,
                scroll_offset,
                scrollbar_color,
                scrollbar_width,
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
                });

                let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
                let total_h = node.children.len() as f32 * item_height;
                let content_y = node.rect.y + node.padding.top;
                let sb_w = *scrollbar_width;
                let sb_color = *scrollbar_color;
                let so = *scroll_offset;
                let rect = node.rect;
                let padding = node.padding;
                let children: Vec<WidgetId> = node.children.clone();

                // Draw only visible children (those with non-zero rects from layout).
                for &child in &children {
                    if let Some(cn) = self.arena.get(child)
                        && cn.rect.width > 0.0
                        && cn.rect.height > 0.0
                    {
                        self.draw_node(child, draw_list);
                    }
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
                    });
                }

                return; // ScrollList handles its own children.
            }
        }

        // Draw children on top (non-ScrollList widgets).
        for &child in &node.children {
            self.draw_node(child, draw_list);
        }
    }

    // ------------------------------------------------------------------
    // ScrollList helpers
    // ------------------------------------------------------------------

    /// Minimum scrollbar thumb height in pixels.
    const MIN_THUMB_HEIGHT: f32 = 20.0;

    /// Compute maximum scroll offset for a ScrollList.
    /// Returns 0.0 if content fits in viewport.
    pub fn max_scroll(&self, id: WidgetId) -> f32 {
        let Some(node) = self.arena.get(id) else {
            return 0.0;
        };
        let Widget::ScrollList { item_height, .. } = &node.widget else {
            return 0.0;
        };
        let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
        let total_h = node.children.len() as f32 * item_height;
        (total_h - viewport_h).max(0.0)
    }

    /// Set scroll offset for a ScrollList, clamped to valid range.
    pub fn set_scroll_offset(&mut self, id: WidgetId, offset: f32) {
        let max = self.max_scroll(id);
        if let Some(node) = self.arena.get_mut(id)
            && let Widget::ScrollList { scroll_offset, .. } = &mut node.widget
        {
            *scroll_offset = offset.clamp(0.0, max);
        }
        self.mark_dirty(id);
    }

    /// Scroll a ScrollList by a delta (positive = down).
    pub fn scroll_by(&mut self, id: WidgetId, delta: f32) {
        let current = self
            .arena
            .get(id)
            .and_then(|n| {
                if let Widget::ScrollList { scroll_offset, .. } = &n.widget {
                    Some(*scroll_offset)
                } else {
                    None
                }
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
            ..
        } = &node.widget
        else {
            return;
        };
        let ih = *item_height;
        let so = *scroll_offset;
        let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
        if viewport_h <= 0.0 {
            return;
        }

        let item_top = child_index as f32 * ih;
        let item_bottom = item_top + ih;

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
            // Row and Column have no colors to fade.
            Widget::Row { .. } | Widget::Column { .. } => {}
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
    pub population: usize,
    pub is_turn_based: bool,
    pub player_name: Option<&'a str>,
    pub paused: bool,
    pub sim_speed: u32,
    pub keybindings: &'a KeyBindings,
    pub screen_width: f32,
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
            text: format!("Tick: {}", info.tick),
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
    line_height: f32,
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
    let measured = tree.measure_node(panel, line_height);
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
    tree.set_sizing(panel, Sizing::Fixed(INSPECTOR_WIDTH), Sizing::Fit);
    tree.set_padding(panel, Edges::all(theme.panel_padding));

    let mut y = 0.0_f32;
    let data_h = theme.font_data_size;
    let body_h = theme.font_body_size;
    let header_h = theme.font_header_size;
    let gap = theme.label_gap;

    // Close button — top-right corner (positioned relative to panel content).
    let close_btn = tree.insert(
        panel,
        Widget::Button {
            text: "X".to_string(),
            color: theme.danger,
            bg_color: [0.0, 0.0, 0.0, 0.0], // transparent
            border_color: theme.danger,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
        },
    );
    // Position at top-right of content area. Button measures ~25px wide.
    let close_x = INSPECTOR_WIDTH - theme.panel_padding * 2.0 - 25.0;
    tree.set_position(close_btn, Position::Fixed { x: close_x, y: 0.0 });

    // Header: icon (gold, mono) + name (text_light, serif)
    let header = tree.insert(
        panel,
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
    tree.set_position(header, Position::Fixed { x: 0.0, y });
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
            14.0,
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
            14.0,
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
            14.0,
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
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
        demo::build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, 14.0);

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
            14.0,
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
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
        demo::build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, 14.0);

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
            14.0,
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
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
        tree.draw(&mut dl);

        // Only 1 panel (background, no scrollbar thumb).
        assert_eq!(dl.panels.len(), 1);
    }

    #[test]
    fn scroll_list_scrollbar_when_content_overflows() {
        // 10 items * 20px = 200px > 100px viewport → scrollbar visible.
        let (tree, _list) = scroll_list_tree(10);

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

        // 2 panels: background + scrollbar thumb.
        assert_eq!(dl.panels.len(), 2);
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
        demo::build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, 14.0);

        // Demo is a single root panel.
        assert_eq!(tree.roots().len(), 1);

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
            population: 15,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
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
            assert_eq!(spans[0].text, "Tick: 42");
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
            population: 3,
            is_turn_based: true,
            player_name: Some("Goblin"),
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
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
            population: 0,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 1024.0,
        };
        let bar = build_status_bar(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 1024.0,
                height: 768.0,
            },
            14.0,
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
            population: 200,
            is_turn_based: true,
            player_name: Some("Wolf"),
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
        };
        build_status_bar(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
            population: 5,
            is_turn_based: false,
            player_name: None,
            paused: true,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
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
            population: 5,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 3,
            keybindings: &kb,
            screen_width: 800.0,
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
        let tip = build_hover_tooltip(&mut tree, &theme, &info, (100.0, 100.0), screen(), 14.0);

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
        let tip = build_hover_tooltip(&mut tree, &theme, &info, (200.0, 200.0), screen(), 14.0);

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
        let tip = build_hover_tooltip(&mut tree, &theme, &info, (50.0, 50.0), screen(), 14.0);

        let node = tree.get(tip).expect("panel");
        // Children: coords(1) + address(1) + 5 occupants(5) + "+3 more"(1) + year(1) = 9
        assert_eq!(node.children.len(), 9);

        // Verify "+3 more" label exists.
        let mut dl = DrawList::new();
        tree.layout(screen(), 14.0);
        tree.draw(&mut dl);
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
        let tip = build_hover_tooltip(&mut tree, &theme, &info, (100.0, 100.0), screen(), 14.0);

        let node = tree.get(tip).expect("panel");
        // coords(1) + 2 entities(2) = 3
        assert_eq!(node.children.len(), 3);

        tree.layout(screen(), 14.0);
        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
        build_hover_tooltip(&mut tree, &theme, &info, (100.0, 100.0), screen(), 14.0);

        tree.layout(screen(), 14.0);
        let mut dl = DrawList::new();
        tree.draw(&mut dl);

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
        let tip = build_hover_tooltip(&mut tree, &theme, &info, (750.0, 550.0), screen(), 14.0);

        tree.layout(screen(), 14.0);
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

        tree.layout(screen(), 14.0);
        let mut dl = DrawList::new();
        tree.draw(&mut dl);

        // One panel (ScrollList background), no scrollbar (3 items fit in viewport).
        assert_eq!(dl.panels.len(), 1);
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
            14.0,
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

        // Close button is a child of the panel.
        let panel_node = tree.get(panel_id).expect("panel");
        assert!(panel_node.children.contains(&close_id));

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
            14.0,
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
        let lh = 14.0; // line_height used in tests
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
            lh,
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
            14.0,
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
            14.0,
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

        let lh = 14.0;
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
            lh,
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
            14.0,
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
            14.0,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list);

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

        let lh = 14.0;
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            lh,
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
            14.0,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list);

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
            14.0,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list);

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
            14.0,
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
            14.0,
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
            14.0,
        );

        // Child should inherit parent's clip_rect.
        let child_node = tree.get(label).unwrap();
        assert!(
            child_node.clip_rect.is_some(),
            "child should inherit clip_rect"
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list);

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
            14.0,
        );

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list);

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
            14.0,
        );

        let rect = tree.node_rect(overlay).unwrap();
        assert!((rect.width - 800.0).abs() < 0.1);
        assert!((rect.height - 600.0).abs() < 0.1);

        let mut draw_list = DrawList::new();
        tree.draw(&mut draw_list);

        assert_eq!(draw_list.panels.len(), 1);
        let p = &draw_list.panels[0];
        assert!(
            (p.bg_color[3] - 0.15).abs() < 0.01,
            "overlay alpha should be 0.15"
        );
    }
}
