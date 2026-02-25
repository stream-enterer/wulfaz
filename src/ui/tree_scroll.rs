use super::WidgetId;
use super::tree::WidgetTree;
use super::widget::Widget;

impl WidgetTree {
    // ------------------------------------------------------------------
    // ScrollList helpers
    // ------------------------------------------------------------------

    /// Minimum scrollbar thumb height in pixels.
    pub(crate) const MIN_THUMB_HEIGHT: f32 = 20.0;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::draw::{DrawList, FontFamily, HeuristicMeasurer};
    use crate::ui::test_helpers::screen;
    use crate::ui::{Edges, Position, Size, Sizing};

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
    fn scroll_list_focusable() {
        let (tree, list) = scroll_list_tree(5);
        let focusable = tree.focusable_widgets();
        assert!(focusable.contains(&list));
    }
}
