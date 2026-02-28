use super::WidgetId;
use super::draw;
use super::draw::{
    DrawList, FontFamily, PanelCommand, RichTextCommand, SpriteCommand, TextCommand, TextMeasurer,
};
use super::tree::WidgetTree;
use super::widget::Widget;

impl WidgetTree {
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
                    let text_size = tm.measure_text(text, *font_family, *font_size);
                    // Only wrap when the text's pixel width actually exceeds
                    // the laid-out rect width — matching the same check used
                    // by wrapped_content_height during layout.  Without this
                    // guard the character-count heuristic (M-width) can wrap
                    // narrow proportional text that layout sized as one line.
                    if text_size.width > node.rect.width {
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
}

pub(crate) fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::wrap_text;
    use crate::ui::draw::TextSpan;
    use crate::ui::widget::{self, CrossAlign};
    use crate::ui::*;

    // ------------------------------------------------------------------
    // DrawList output (basic)
    // ------------------------------------------------------------------

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
    fn showcase_tree_uses_theme() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = sidebar::SidebarInfo {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        sidebar::build_showcase_view(&mut tree, &theme, &kb, &live, screen, 0.0);
        tree.layout(screen, &mut HeuristicMeasurer);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Root panel uses theme parchment background.
        assert!(!dl.panels.is_empty());
        assert_eq!(dl.panels[0].bg_color, theme.bg_parchment);
        assert_eq!(dl.panels[0].border_color, theme.panel_border_color);
        assert!((dl.panels[0].border_width - theme.panel_border_width).abs() < 0.01);

        // Showcase uses all theme font families and sizes.
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
    fn showcase_tree_includes_rich_text() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = sidebar::SidebarInfo {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        sidebar::build_showcase_view(&mut tree, &theme, &kb, &live, screen, 0.0);
        tree.layout(screen, &mut HeuristicMeasurer);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Showcase has rich text blocks (title, population, live data, etc.).
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

    #[test]
    fn showcase_tree_includes_scroll_list() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = sidebar::SidebarInfo {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        sidebar::build_showcase_view(&mut tree, &theme, &kb, &live, screen, 0.0);
        tree.layout(screen, &mut HeuristicMeasurer);

        // Showcase view is a single root panel.
        assert_eq!(tree.roots().len(), 1);

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Should have scroll items and button texts.
        assert!(dl.texts.len() > 4, "scroll items + buttons should be drawn");

        // Verify scroll items exist.
        let all_texts: Vec<&str> = dl.texts.iter().map(|t| t.text.as_str()).collect();
        assert!(all_texts.contains(&"Item 1"));
    }

    // ------------------------------------------------------------------
    // Row / Column transparency
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Progress bar (UI-200)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Separator (UI-201)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Icon (UI-202c)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Dropdown (UI-202d)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Checkbox (UI-202e)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Slider (UI-202f)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // TextInput (UI-202g)
    // ------------------------------------------------------------------

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

    // ------------------------------------------------------------------
    // Collapsible (UI-202h)
    // ------------------------------------------------------------------

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
    // TabContainer (UI-301)
    // ------------------------------------------------------------------

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
}
