mod draw;
mod input;
mod theme;
mod widget;

#[allow(unused_imports)] // Public API: used by game panels constructing widgets.
pub use draw::{DrawList, FontFamily, PanelCommand, RichTextCommand, TextCommand, TextSpan};
#[allow(unused_imports)] // Public API: used by main.rs for input routing (UI-W02).
pub use input::{MouseButton, UiEvent, UiState};
pub use theme::Theme;
#[allow(unused_imports)] // Public API: used by game panels setting tooltip content.
pub use widget::{TooltipContent, Widget};

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

        let children: Vec<WidgetId> = node.children.clone();
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

        match &node.widget {
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
                });
            }
            Widget::Label {
                text,
                color,
                font_size,
                font_family,
            } => {
                draw_list.texts.push(TextCommand {
                    text: text.clone(),
                    x: node.rect.x,
                    y: node.rect.y,
                    color: *color,
                    font_size: *font_size,
                    font_family: *font_family,
                });
            }
            Widget::Button {
                text,
                color,
                bg_color,
                border_color,
                font_size,
                font_family,
            } => {
                // Button = panel background + centered text.
                draw_list.panels.push(PanelCommand {
                    x: node.rect.x,
                    y: node.rect.y,
                    width: node.rect.width,
                    height: node.rect.height,
                    bg_color: *bg_color,
                    border_color: *border_color,
                    border_width: 1.0,
                    shadow_width: 0.0,
                });
                // Text offset by internal button padding.
                draw_list.texts.push(TextCommand {
                    text: text.clone(),
                    x: node.rect.x + 8.0,
                    y: node.rect.y + 4.0,
                    color: *color,
                    font_size: *font_size,
                    font_family: *font_family,
                });
            }
            Widget::RichText { spans, font_size } => {
                draw_list.rich_texts.push(RichTextCommand {
                    spans: spans.clone(),
                    x: node.rect.x,
                    y: node.rect.y,
                    font_size: *font_size,
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
}

// ---------------------------------------------------------------------------
// Tier 3 UI-DEMO: themed panel with multi-font labels + rich text
// ---------------------------------------------------------------------------

/// Build the demo widget tree using Theme constants.
/// Tier 3: parchment panel with themed colors, Serif header (16pt),
/// Serif body (12pt), Mono warning (9pt), and a rich text block
/// mixing serif body with mono inline data and gold highlights.
pub fn demo_tree(theme: &Theme) -> WidgetTree {
    let mut tree = WidgetTree::new();

    // Themed parchment panel
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    tree.set_position(panel, Position::Fixed { x: 20.0, y: 20.0 });
    tree.set_sizing(panel, Sizing::Fixed(320.0), Sizing::Fixed(160.0));
    tree.set_padding(panel, Edges::all(theme.panel_padding));

    // Gold header — Serif, header size
    let header = tree.insert(
        panel,
        Widget::Label {
            text: "Header".into(),
            color: theme.gold,
            font_size: theme.font_header_size,
            font_family: theme.font_header_family,
        },
    );
    tree.set_position(header, Position::Fixed { x: 0.0, y: 0.0 });

    // Light body text — Serif, body size
    let body = tree.insert(
        panel,
        Widget::Label {
            text: "Body text".into(),
            color: theme.text_light,
            font_size: theme.font_body_size,
            font_family: theme.font_body_family,
        },
    );
    tree.set_position(
        body,
        Position::Fixed {
            x: 0.0,
            y: theme.font_header_size + theme.label_gap,
        },
    );

    // Red warning — Mono, data size
    let warning = tree.insert(
        panel,
        Widget::Label {
            text: "Warning".into(),
            color: theme.danger,
            font_size: theme.font_data_size,
            font_family: theme.font_data_family,
        },
    );
    let warning_y =
        theme.font_header_size + theme.label_gap + theme.font_body_size + theme.label_gap;
    tree.set_position(
        warning,
        Position::Fixed {
            x: 0.0,
            y: warning_y,
        },
    );

    // Rich text — mixed serif body + mono data + gold highlight (UI-R01)
    let rich = tree.insert(
        panel,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: "Population: ".into(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
                TextSpan {
                    text: "1,034,196".into(),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " souls".into(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
            ],
            font_size: theme.font_body_size,
        },
    );
    tree.set_position(
        rich,
        Position::Fixed {
            x: 0.0,
            y: warning_y + theme.font_data_size + theme.label_gap,
        },
    );

    // ScrollList with 100 items (UI-W03 demo).
    let scroll_list = tree.insert_root(Widget::ScrollList {
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        border_width: theme.panel_border_width,
        item_height: theme.scroll_item_height,
        scroll_offset: 0.0,
        scrollbar_color: theme.scrollbar_color,
        scrollbar_width: theme.scrollbar_width,
    });
    tree.set_position(scroll_list, Position::Fixed { x: 360.0, y: 20.0 });
    tree.set_sizing(scroll_list, Sizing::Fixed(200.0), Sizing::Fixed(160.0));
    tree.set_padding(scroll_list, Edges::all(4.0));

    for i in 0..100 {
        tree.insert(
            scroll_list,
            Widget::Label {
                text: format!("Item {}", i + 1),
                color: theme.text_dark,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
            },
        );
    }

    // Tooltip demo: button with 3-level nested tooltip chain (UI-W04 demo).
    let tooltip_btn = tree.insert_root(Widget::Button {
        text: "Hover for tooltip".into(),
        color: theme.text_light,
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        font_size: theme.font_body_size,
        font_family: theme.font_body_family,
    });
    tree.set_position(tooltip_btn, Position::Fixed { x: 580.0, y: 20.0 });

    // Level 3 (deepest): simple text.
    let level3 = widget::TooltipContent::Text("Level 3: deepest tooltip".into());
    // Level 2: custom with a hoverable label that shows level 3.
    let level2 = widget::TooltipContent::Custom(vec![
        (
            Widget::Label {
                text: "Level 2 tooltip".into(),
                color: theme.text_light,
                font_size: theme.font_body_size,
                font_family: theme.font_body_family,
            },
            None,
        ),
        (
            Widget::Label {
                text: "[hover for level 3]".into(),
                color: theme.gold,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
            },
            Some(level3),
        ),
    ]);
    // Level 1: custom with a hoverable label that shows level 2.
    let level1 = widget::TooltipContent::Custom(vec![
        (
            Widget::Label {
                text: "Level 1 tooltip".into(),
                color: theme.text_light,
                font_size: theme.font_body_size,
                font_family: theme.font_body_family,
            },
            None,
        ),
        (
            Widget::Label {
                text: "[hover for level 2]".into(),
                color: theme.gold,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
            },
            Some(level2),
        ),
    ]);
    tree.set_tooltip(tooltip_btn, Some(level1));

    tree
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
            },
        );
        let grandchild = tree.insert(
            child,
            Widget::Label {
                text: "B".into(),
                color: [1.0; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
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
        let mut tree = demo_tree(&theme);
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

        // First panel is the themed parchment panel, second is the ScrollList bg.
        assert!(dl.panels.len() >= 2);
        assert_eq!(dl.panels[0].bg_color, theme.bg_parchment);
        assert_eq!(dl.panels[0].border_color, theme.panel_border_color);
        assert!((dl.panels[0].border_width - theme.panel_border_width).abs() < 0.01);

        // First 3 texts are labels: gold header, light body, red warning.
        // Remaining texts are visible ScrollList items.
        assert!(dl.texts.len() >= 3);
        assert_eq!(dl.texts[0].color, theme.gold);
        assert_eq!(dl.texts[0].font_family, theme.font_header_family);
        assert!((dl.texts[0].font_size - theme.font_header_size).abs() < 0.01);

        assert_eq!(dl.texts[1].color, theme.text_light);
        assert_eq!(dl.texts[1].font_family, theme.font_body_family);
        assert!((dl.texts[1].font_size - theme.font_body_size).abs() < 0.01);

        assert_eq!(dl.texts[2].color, theme.danger);
        assert_eq!(dl.texts[2].font_family, theme.font_data_family);
        assert!((dl.texts[2].font_size - theme.font_data_size).abs() < 0.01);
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
        let mut tree = demo_tree(&theme);
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            14.0,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

        // Panels: original panel + ScrollList bg + scrollbar thumb
        assert!(dl.panels.len() >= 2);
        // Labels: 3 original + visible scroll items
        assert!(dl.texts.len() >= 3);
        assert_eq!(dl.rich_texts.len(), 1);

        // Rich text has 3 spans: "Population: " + "1,034,196" + " souls"
        let rt = &dl.rich_texts[0];
        assert_eq!(rt.spans.len(), 3);
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
        let mut tree = demo_tree(&theme);
        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            14.0,
        );

        // Should have 3 roots: panel + ScrollList + tooltip button.
        assert_eq!(tree.roots().len(), 3);

        let mut dl = DrawList::new();
        tree.draw(&mut dl);

        // Original panel + ScrollList bg + scrollbar thumb + tooltip button bg = 4 panels.
        assert_eq!(dl.panels.len(), 4);

        // 3 original labels + visible scroll items + tooltip button text.
        assert!(dl.texts.len() > 4, "scroll items + button should be drawn");

        // Verify scroll items are from the list (skip first 3 labels).
        let after_labels: Vec<&str> = dl.texts[3..].iter().map(|t| t.text.as_str()).collect();
        assert!(after_labels.contains(&"Item 1"));
        assert!(after_labels.contains(&"Hover for tooltip"));
    }

    #[test]
    fn scroll_list_focusable() {
        let (tree, list) = scroll_list_tree(5);
        let focusable = tree.focusable_widgets();
        assert!(focusable.contains(&list));
    }
}
