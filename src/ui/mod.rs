mod draw;
mod widget;

pub use draw::{DrawList, FontFamily, PanelCommand, TextCommand};
pub use widget::Widget;

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

        let children: Vec<WidgetId> = node.children.clone();
        for child in children {
            self.layout_node(child, content, line_height);
        }
    }

    /// Measure intrinsic size of a widget (content only, no padding).
    fn measure_node(&self, id: WidgetId, line_height: f32) -> Size {
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
        }

        // Draw children on top.
        for &child in &node.children {
            self.draw_node(child, draw_list);
        }
    }
}

// ---------------------------------------------------------------------------
// Tier 1 UI-DEMO: parchment panel with 3 colored labels
// ---------------------------------------------------------------------------

/// Build the Tier 1 demo widget tree: a parchment panel containing three
/// colored labels (gold header, white body, red warning).
pub fn demo_tree(line_height: f32) -> WidgetTree {
    let mut tree = WidgetTree::new();

    // CK3 parchment panel
    let panel = tree.insert_root(Widget::Panel {
        bg_color: [0.87, 0.81, 0.70, 0.95],
        border_color: [0.83, 0.68, 0.21, 1.0],
        border_width: 2.0,
        shadow_width: 6.0,
    });
    tree.set_position(panel, Position::Fixed { x: 20.0, y: 20.0 });
    tree.set_sizing(panel, Sizing::Fixed(260.0), Sizing::Fixed(120.0));
    tree.set_padding(panel, Edges::all(12.0));

    // Gold header — Serif 16pt style
    let header = tree.insert(
        panel,
        Widget::Label {
            text: "Header".into(),
            color: [0.78, 0.66, 0.31, 1.0], // CK3 gold
            font_size: line_height,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(header, Position::Fixed { x: 0.0, y: 0.0 });

    // White body text — Serif
    let body = tree.insert(
        panel,
        Widget::Label {
            text: "Body text".into(),
            color: [0.94, 0.90, 0.82, 1.0], // warm white
            font_size: line_height,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(
        body,
        Position::Fixed {
            x: 0.0,
            y: line_height + 4.0,
        },
    );

    // Red warning — Mono for data/terminal style
    let warning = tree.insert(
        panel,
        Widget::Label {
            text: "Warning".into(),
            color: [0.75, 0.25, 0.25, 1.0], // danger red
            font_size: line_height,
            font_family: FontFamily::Mono,
        },
    );
    tree.set_position(
        warning,
        Position::Fixed {
            x: 0.0,
            y: (line_height + 4.0) * 2.0,
        },
    );

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
}
