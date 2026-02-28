use super::WidgetId;
use super::draw::{FontFamily, TextMeasurer};
use super::geometry::{Edges, Position, Rect, Size, Sizing};
use super::node::WidgetNode;
use super::tree::WidgetTree;
use super::widget::{self, Widget};

impl WidgetTree {
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
        // Fit caps at parent_content width: available width flows down the tree,
        // matching the standard "width-in, height-out" layout model (UI-102).
        let resolved_w = match node.width {
            Sizing::Fixed(px) => px,
            Sizing::Percent(frac) => parent_content.width * frac,
            Sizing::Fit => (measured.width + node.padding.horizontal()).min(parent_content.width),
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
            // For Fit-width children, cap at content.width so wrapped labels
            // compute correct multi-line heights (UI-102).
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
                    Sizing::Fit => {
                        // Cap child width at available space (cross-axis).
                        let effective_w =
                            (child_measured.width + cp.horizontal()).min(content.width);
                        let h = wrapped_content_height(
                            &child.widget,
                            &child_measured,
                            effective_w,
                            &cp,
                            tm,
                        );
                        h + cp.vertical() + cm.vertical()
                    }
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
                // Resolve child width — cap Fit at content.width (cross-axis).
                let child_w = match self.arena.get(*child_id).map(|n| n.width) {
                    Some(Sizing::Fixed(px)) => px,
                    Some(Sizing::Percent(frac)) => content.width * frac,
                    Some(Sizing::Fit) | None => {
                        (child_measured.width + cp.horizontal()).min(content.width)
                    }
                };

                let child_h = match ch {
                    Sizing::Fixed(px) => *px,
                    Sizing::Fit => {
                        // Recompute height for wrapped labels at capped width.
                        if let Some(n) = self.arena.get(*child_id) {
                            let h =
                                wrapped_content_height(&n.widget, child_measured, child_w, cp, tm);
                            h + cp.vertical()
                        } else {
                            child_measured.height + cp.vertical()
                        }
                    }
                    Sizing::Percent(frac) => {
                        if percent_total > 0.0 {
                            remaining * frac / percent_total
                        } else {
                            0.0
                        }
                    }
                };
                let child_total_h = child_h + cm.vertical();

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

        self.layout_node_children(id, tm);
    }

    /// Resolve a child's outer extent for parent measurement.
    ///
    /// `Sizing::Fixed(px)` already includes padding (layout resolves it to
    /// exactly `px`), so only margin is added.  For `Fit`/`Percent` the
    /// intrinsic measured size plus padding and margin is used.
    fn child_extent(child_measured: &Size, child: &WidgetNode) -> (f32, f32) {
        let w = match child.width {
            Sizing::Fixed(px) => px + child.margin.horizontal(),
            _ => child_measured.width + child.padding.horizontal() + child.margin.horizontal(),
        };
        let h = match child.height {
            Sizing::Fixed(px) => px + child.margin.vertical(),
            _ => child_measured.height + child.padding.vertical() + child.margin.vertical(),
        };
        (w, h)
    }

    /// Measure intrinsic size of a widget (content only, no padding).
    ///
    /// For nodes with constraints, respects max_width and propagates it to
    /// children so that wrapped labels compute correct multi-line heights.
    pub fn measure_node(&self, id: WidgetId, tm: &mut dyn TextMeasurer) -> Size {
        self.measure_node_constrained(id, tm, None)
    }

    /// Measure content size with an optional max-content-width constraint.
    ///
    /// `max_content_w`: if Some, the maximum content width this node should
    /// report. The node's own constraints (if any) further narrow this bound.
    /// Containers propagate the effective bound to children so that wrapped
    /// labels compute correct multi-line heights.
    fn measure_node_constrained(
        &self,
        id: WidgetId,
        tm: &mut dyn TextMeasurer,
        max_content_w: Option<f32>,
    ) -> Size {
        let Some(node) = self.arena.get(id) else {
            return Size::default();
        };

        // Merge passed-in max with this node's own constraint (which is total
        // width, so subtract padding to get content max).
        let own_content_max = node
            .constraints
            .as_ref()
            .filter(|c| c.max_width < f32::MAX)
            .map(|c| (c.max_width - node.padding.horizontal()).max(0.0));

        let effective_max_w = match (max_content_w, own_content_max) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a @ Some(_), None) => a,
            (None, b @ Some(_)) => b,
            (None, None) => None,
        };

        match &node.widget {
            Widget::Label {
                text,
                font_size,
                font_family,
                wrap,
                ..
            } => {
                let ts = tm.measure_text(text, *font_family, *font_size);
                let line_h = tm.measure_text("M", *font_family, *font_size).height;

                // When wrapping and constrained, compute multi-line dimensions.
                if *wrap
                    && let Some(max_w) = effective_max_w
                    && ts.width > max_w
                    && max_w > 0.0
                {
                    let char_w = tm.measure_text("M", *font_family, *font_size).width;
                    let n_lines = wrapped_line_count(text, max_w, char_w);
                    return Size {
                        width: max_w,
                        height: n_lines as f32 * line_h,
                    };
                }

                Size {
                    width: ts.width,
                    height: line_h,
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
                // Row is main-axis: can't divide max_w among children, pass None.
                let n = node.children.len();
                let mut total_w: f32 = 0.0;
                let mut max_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_measured = self.measure_node_constrained(child_id, tm, None);
                        let (child_w, child_h) = Self::child_extent(&child_measured, child);
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
                // Cross-axis is width: propagate effective_max_w to children.
                let n = node.children.len();
                let mut max_w: f32 = 0.0;
                let mut total_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_max = effective_max_w.map(|w| {
                            (w - child.padding.horizontal() - child.margin.horizontal()).max(0.0)
                        });
                        let child_measured = self.measure_node_constrained(child_id, tm, child_max);
                        let (child_w, child_h) = Self::child_extent(&child_measured, child);
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
                // Propagate effective_max_w to children.
                let mut max_w: f32 = 0.0;
                let mut max_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_max = effective_max_w.map(|w| {
                            (w - child.padding.horizontal() - child.margin.horizontal()).max(0.0)
                        });
                        let child_measured = self.measure_node_constrained(child_id, tm, child_max);
                        let (cx, cy) = match child.position {
                            Position::Fixed { x, y } => (x, y),
                            Position::Percent { .. } | Position::Center => (0.0, 0.0),
                        };
                        let (child_w, child_h) = Self::child_extent(&child_measured, child);
                        max_w = max_w.max(cx + child_w);
                        max_h = max_h.max(cy + child_h);
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
                // Scrollable: don't propagate width constraint to children.
                let mut max_w: f32 = 0.0;
                for &child_id in &node.children {
                    let child_measured = self.measure_node_constrained(child_id, tm, None);
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
                // Scrollable: don't propagate width constraint to children.
                let mut max_w: f32 = 0.0;
                let mut total_h: f32 = 0.0;
                for &child_id in &node.children {
                    if let Some(child) = self.arena.get(child_id) {
                        let child_measured = self.measure_node_constrained(child_id, tm, None);
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
                            let child_max = effective_max_w.map(|w| {
                                (w - child.padding.horizontal() - child.margin.horizontal())
                                    .max(0.0)
                            });
                            let child_measured =
                                self.measure_node_constrained(child_id, tm, child_max);
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
                // Content children measured Column-style; propagate constraint.
                let mut content_w = 0.0_f32;
                let mut content_h = 0.0_f32;
                for &child_id in &node.children {
                    let child_max = if let Some(child) = self.arena.get(child_id) {
                        effective_max_w.map(|w| {
                            (w - child.padding.horizontal() - child.margin.horizontal()).max(0.0)
                        })
                    } else {
                        None
                    };
                    let child_measured = self.measure_node_constrained(child_id, tm, child_max);
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
}

/// Compute the number of wrapped lines for a label, given resolved width and font metrics.
fn wrapped_line_count(text: &str, width: f32, char_w: f32) -> usize {
    let max_chars = (width / char_w).max(1.0) as usize;
    super::tree_draw::wrap_text(text, max_chars).len()
}

/// For a wrapping Label, compute content height given an effective widget width.
/// For non-wrapping or non-Label widgets, returns `measured.height` unchanged.
///
/// Used by Column layout to produce correct multi-line heights when a Fit-width
/// label is capped at the available cross-axis width (UI-102).
fn wrapped_content_height(
    widget: &widget::Widget,
    measured: &Size,
    effective_w: f32,
    padding: &Edges,
    tm: &mut dyn TextMeasurer,
) -> f32 {
    if let widget::Widget::Label {
        text,
        font_size,
        font_family,
        wrap: true,
        ..
    } = widget
    {
        let content_w = (effective_w - padding.horizontal()).max(0.0);
        if measured.width > content_w && content_w > 0.0 {
            let ts = tm.measure_text("M", *font_family, *font_size);
            let n_lines = wrapped_line_count(text, content_w, ts.width);
            return n_lines as f32 * ts.height;
        }
    }
    measured.height
}

#[cfg(test)]
mod tests {
    use super::super::draw::{DrawList, HeuristicMeasurer, TextSpan};
    use super::super::geometry::Constraints;
    use super::super::widget::CrossAlign;
    use super::*;

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
    fn fit_parent_propagates_fixed_child_through_column() {
        // Regression: measure_node for Column must include Fixed-sized
        // children so that a Fit-sized parent Panel gets the correct size.
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.0; 4],
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        });
        tree.set_sizing(panel, Sizing::Fit, Sizing::Fit);
        tree.set_padding(panel, Edges::all(4.0));

        let col = tree.insert(
            panel,
            Widget::Column {
                gap: 2.0,
                align: widget::CrossAlign::Center,
            },
        );

        tree.insert(
            col,
            Widget::Label {
                text: "Title".into(),
                color: [1.0; 4],
                font_size: 12.0,
                font_family: FontFamily::default(),
                wrap: false,
            },
        );

        let inner = tree.insert(
            col,
            Widget::Panel {
                bg_color: [0.0; 4],
                border_color: [0.0; 4],
                border_width: 0.0,
                shadow_width: 0.0,
            },
        );
        tree.set_sizing(inner, Sizing::Fixed(128.0), Sizing::Fixed(96.0));

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let panel_rect = tree.get(panel).expect("panel").rect;
        let inner_rect = tree.get(inner).expect("inner").rect;

        // Inner panel must fit within the outer panel.
        assert!(
            inner_rect.y + inner_rect.height <= panel_rect.y + panel_rect.height,
            "Fixed child overflows Fit parent: inner bottom {} > panel bottom {}",
            inner_rect.y + inner_rect.height,
            panel_rect.y + panel_rect.height,
        );

        // Panel width must accommodate the 128px child + 8px padding.
        assert!(
            panel_rect.width >= 128.0 + 8.0,
            "Panel width {} too narrow for 128px child",
            panel_rect.width,
        );
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
                text: "Hi".into(), // 2 chars -> narrow
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
        // Label width ~ 2 * 8.4 = 16.8. Column is 400 wide.
        // Centered: x ~ (400 - 16.8) / 2 ~ 191.6
        let expected_center = (400.0 - rl.width) / 2.0;
        assert!(
            (rl.x - expected_center).abs() < 1.0,
            "label should be horizontally centered, x = {}, expected ~ {}",
            rl.x,
            expected_center
        );
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

    // ------------------------------------------------------------------
    // Min/Max constraints (UI-103)
    // ------------------------------------------------------------------

    #[test]
    fn constraints_min_width_enforced() {
        let mut tree = WidgetTree::new();
        let label = tree.insert_root(Widget::Label {
            text: "Hi".into(), // ~2 chars ~ 16.8px wide
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
    fn checkbox_measure_includes_box_and_label() {
        let mut tree = WidgetTree::new();
        let cb = tree.insert_root(Widget::Checkbox {
            checked: false,
            label: "Enable".into(),
            color: [1.0; 4],
            font_size: 14.0,
        });
        let size = tree.measure_node(cb, &mut HeuristicMeasurer);
        // box_size=16 + gap=6 + label "Enable" (6 chars) * char_w(8.4) ~ 72.4
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
