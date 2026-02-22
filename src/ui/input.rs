use super::widget::Widget;
use super::{WidgetId, WidgetTree};

/// Mouse button identifier (decoupled from winit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// UI events dispatched to widgets by the input system.
#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    /// Cursor entered this widget's rect.
    Hover,
    /// Mouse button clicked (press + release on same widget).
    Click(MouseButton),
    /// Drag started at (x, y) screen coords.
    DragStart { x: f32, y: f32 },
    /// Drag moved to (x, y) while captured.
    DragMove { x: f32, y: f32 },
    /// Drag ended (mouse released while captured).
    DragEnd,
    /// Scroll wheel delta (positive = scroll down).
    Scroll(f32),
}

/// Minimum pixel distance before a press becomes a drag.
const DRAG_THRESHOLD: f32 = 4.0;

/// Pixels scrolled per mouse wheel line.
const SCROLL_SPEED: f32 = 40.0;

/// Active scrollbar thumb drag state.
struct ScrollDrag {
    widget: WidgetId,
    start_mouse_y: f32,
    start_scroll_offset: f32,
    content_height: f32,
    viewport_height: f32,
}

/// Interaction state for the widget system. Lives on App, not World.
pub struct UiState {
    /// Widget currently under the cursor.
    pub hovered: Option<WidgetId>,
    /// Widget receiving keyboard events (Tab to cycle).
    pub focused: Option<WidgetId>,
    /// Widget being pressed (mouse down, not yet released).
    pressed: Option<WidgetId>,
    /// Mouse button that initiated the press.
    pressed_button: Option<MouseButton>,
    /// Widget with mouse capture (for drag operations).
    /// While captured, all mouse events route to this widget even if
    /// the cursor leaves its rect. Released on mouse-up.
    pub captured: Option<WidgetId>,
    /// Screen coords where the press started (for drag threshold).
    press_origin: Option<(f32, f32)>,
    /// Whether we've crossed the drag threshold for the current press.
    dragging: bool,
    /// Last known cursor position (screen coords).
    pub cursor: (f32, f32),
    /// Active scrollbar drag (if user is dragging a scrollbar thumb).
    scroll_drag: Option<ScrollDrag>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            hovered: None,
            focused: None,
            pressed: None,
            pressed_button: None,
            captured: None,
            press_origin: None,
            dragging: false,
            cursor: (0.0, 0.0),
            scroll_drag: None,
        }
    }

    /// Handle cursor movement. Returns true if the cursor is over a UI widget
    /// (event consumed — don't pass to game).
    pub fn handle_cursor_moved(&mut self, tree: &mut WidgetTree, x: f32, y: f32) -> bool {
        self.cursor = (x, y);

        // Active scrollbar drag — update scroll offset from mouse position.
        if let Some(ref drag) = self.scroll_drag {
            let delta_y = y - drag.start_mouse_y;
            let thumb_h = (drag.viewport_height * drag.viewport_height / drag.content_height)
                .max(WidgetTree::MIN_THUMB_HEIGHT);
            let available_track = drag.viewport_height - thumb_h;
            if available_track > 0.0 {
                let max_scroll = drag.content_height - drag.viewport_height;
                let new_offset = drag.start_scroll_offset + delta_y * max_scroll / available_track;
                let widget_id = drag.widget;
                tree.set_scroll_offset(widget_id, new_offset);
            }
            self.hovered = tree.hit_test(x, y);
            return true;
        }

        // If a widget has capture, route drag events to it.
        if self.captured.is_some() {
            if let Some(origin) = self.press_origin {
                let dx = x - origin.0;
                let dy = y - origin.1;
                if !self.dragging && (dx * dx + dy * dy).sqrt() >= DRAG_THRESHOLD {
                    self.dragging = true;
                    let _ = UiEvent::DragStart {
                        x: origin.0,
                        y: origin.1,
                    };
                }
                if self.dragging {
                    let _ = UiEvent::DragMove { x, y };
                }
            }
            self.hovered = tree.hit_test(x, y);
            return true;
        }

        let hit = tree.hit_test(x, y);
        self.hovered = hit;
        hit.is_some()
    }

    /// Handle mouse button press/release. Returns true if consumed by UI.
    pub fn handle_mouse_input(
        &mut self,
        tree: &mut WidgetTree,
        button: MouseButton,
        pressed: bool,
        x: f32,
        y: f32,
    ) -> bool {
        self.cursor = (x, y);

        if pressed {
            // Mouse down
            let hit = if let Some(cap) = self.captured {
                Some(cap)
            } else {
                tree.hit_test(x, y)
            };

            if let Some(widget_id) = hit {
                self.pressed = Some(widget_id);
                self.pressed_button = Some(button);
                self.press_origin = Some((x, y));
                self.dragging = false;

                // Check if clicking on a ScrollList's scrollbar area.
                if button == MouseButton::Left
                    && let Some(scroll_drag) = Self::try_start_scrollbar_drag(tree, widget_id, x, y)
                {
                    self.scroll_drag = Some(scroll_drag);
                    self.captured = Some(widget_id);
                    return true;
                }

                // Left click sets focus (Buttons and ScrollLists).
                if button == MouseButton::Left
                    && let Some(node) = tree.get(widget_id)
                    && matches!(
                        node.widget,
                        Widget::Button { .. } | Widget::ScrollList { .. }
                    )
                {
                    self.focused = Some(widget_id);
                }

                self.captured = Some(widget_id);
                return true;
            }

            // Clicked outside all widgets — clear focus.
            self.focused = None;
            return false;
        }

        // Mouse up
        let was_pressed = self.pressed.take();
        let was_button = self.pressed_button.take();
        let was_captured = self.captured.take();
        let was_dragging = self.dragging;
        let was_scrollbar_drag = self.scroll_drag.take().is_some();
        self.press_origin = None;
        self.dragging = false;

        if was_captured.is_some() {
            if was_scrollbar_drag {
                // Scrollbar drag ended — already handled.
            } else if was_dragging {
                let _ = UiEvent::DragEnd;
            } else if let Some(pressed_id) = was_pressed {
                let release_hit = tree.hit_test(x, y);
                if release_hit == Some(pressed_id)
                    && let Some(btn) = was_button
                {
                    let _ = UiEvent::Click(btn);
                }
            }
            return true;
        }

        false
    }

    /// Handle keyboard input. Returns true if consumed by a focused widget.
    pub fn handle_key_input(
        &mut self,
        tree: &mut WidgetTree,
        key: winit::keyboard::KeyCode,
        pressed: bool,
    ) -> bool {
        if !pressed {
            return self.focused.is_some();
        }

        use winit::keyboard::KeyCode;

        // Tab cycles focus through focusable widgets.
        if key == KeyCode::Tab {
            let focusable = tree.focusable_widgets();
            if focusable.is_empty() {
                self.focused = None;
                return false;
            }
            self.focused = match self.focused {
                None => Some(focusable[0]),
                Some(current) => {
                    if let Some(idx) = focusable.iter().position(|&id| id == current) {
                        Some(focusable[(idx + 1) % focusable.len()])
                    } else {
                        Some(focusable[0])
                    }
                }
            };
            return true;
        }

        // ScrollList keyboard navigation.
        if let Some(focused_id) = self.focused
            && let Some(node) = tree.get(focused_id)
            && let Widget::ScrollList { item_height, .. } = &node.widget
        {
            let ih = *item_height;
            let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);

            match key {
                KeyCode::ArrowUp => {
                    tree.scroll_by(focused_id, -ih);
                    return true;
                }
                KeyCode::ArrowDown => {
                    tree.scroll_by(focused_id, ih);
                    return true;
                }
                KeyCode::PageUp => {
                    tree.scroll_by(focused_id, -viewport_h);
                    return true;
                }
                KeyCode::PageDown => {
                    tree.scroll_by(focused_id, viewport_h);
                    return true;
                }
                KeyCode::Home => {
                    tree.set_scroll_offset(focused_id, 0.0);
                    return true;
                }
                KeyCode::End => {
                    let max = tree.max_scroll(focused_id);
                    tree.set_scroll_offset(focused_id, max);
                    return true;
                }
                _ => {}
            }
        }

        // Other keys go to focused widget (if any).
        self.focused.is_some()
    }

    /// Handle scroll wheel. Returns true if consumed by a widget under cursor.
    pub fn handle_scroll(&mut self, tree: &mut WidgetTree, delta: f32) -> bool {
        let hit = tree.hit_test(self.cursor.0, self.cursor.1);
        if let Some(widget_id) = hit {
            // Walk up to find nearest ScrollList ancestor (or self).
            if let Some(scroll_id) = Self::find_scroll_list_ancestor(tree, widget_id) {
                tree.scroll_by(scroll_id, delta * SCROLL_SPEED);
                return true;
            }
            let _ = UiEvent::Scroll(delta);
            return true;
        }
        false
    }

    /// Walk from `start` up the parent chain to find a ScrollList widget.
    fn find_scroll_list_ancestor(tree: &WidgetTree, start: WidgetId) -> Option<WidgetId> {
        let mut current = Some(start);
        while let Some(id) = current {
            let node = tree.get(id)?;
            if matches!(node.widget, Widget::ScrollList { .. }) {
                return Some(id);
            }
            current = node.parent;
        }
        None
    }

    /// Check if a mouse press at (x, y) is on a ScrollList's scrollbar area.
    /// If so, return a ScrollDrag to begin scrollbar dragging.
    fn try_start_scrollbar_drag(
        tree: &WidgetTree,
        widget_id: WidgetId,
        x: f32,
        y: f32,
    ) -> Option<ScrollDrag> {
        let node = tree.get(widget_id)?;
        let Widget::ScrollList {
            item_height,
            scroll_offset,
            scrollbar_width,
            ..
        } = &node.widget
        else {
            return None;
        };

        let viewport_h = (node.rect.height - node.padding.vertical()).max(0.0);
        let total_h = node.children.len() as f32 * item_height;

        // No scrollbar if content fits.
        if total_h <= viewport_h {
            return None;
        }

        // Check if click is in the scrollbar area (rightmost scrollbar_width pixels).
        let sb_x = node.rect.x + node.rect.width - scrollbar_width - node.padding.right;
        if x >= sb_x {
            return Some(ScrollDrag {
                widget: widget_id,
                start_mouse_y: y,
                start_scroll_offset: *scroll_offset,
                content_height: total_h,
                viewport_height: viewport_h,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::draw::FontFamily;
    use crate::ui::{Edges, Position, Rect, Size, Sizing, WidgetTree};

    /// Helper: build a tree with a panel containing a button.
    fn tree_with_button() -> (WidgetTree, WidgetId, WidgetId) {
        let mut tree = WidgetTree::new();
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 2.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 10.0, y: 10.0 });
        tree.set_sizing(panel, Sizing::Fixed(200.0), Sizing::Fixed(100.0));
        tree.set_padding(panel, Edges::all(8.0));

        let button = tree.insert(
            panel,
            Widget::Button {
                text: "Click me".into(),
                color: [1.0; 4],
                bg_color: [0.3; 4],
                border_color: [0.8; 4],
                font_size: 14.0,
                font_family: FontFamily::default(),
            },
        );
        tree.set_position(button, Position::Fixed { x: 0.0, y: 0.0 });

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            14.0,
        );
        (tree, panel, button)
    }

    #[test]
    fn rect_contains() {
        let r = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        assert!(r.contains(10.0, 20.0)); // top-left corner
        assert!(r.contains(50.0, 40.0)); // center
        assert!(r.contains(109.9, 69.9)); // near bottom-right
        assert!(!r.contains(110.0, 70.0)); // exactly at edge (exclusive)
        assert!(!r.contains(9.0, 20.0)); // just outside left
        assert!(!r.contains(10.0, 70.0)); // just outside bottom
    }

    #[test]
    fn hit_test_finds_topmost_widget() {
        let (tree, _panel, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;

        // Hitting inside button rect should return button (child is topmost).
        let hit = tree.hit_test(btn_rect.x + 1.0, btn_rect.y + 1.0);
        assert_eq!(hit, Some(button));
    }

    #[test]
    fn hit_test_falls_through_to_parent() {
        let (tree, panel, _button) = tree_with_button();
        let panel_rect = tree.get(panel).unwrap().rect;

        // Hit panel area outside the button — should return panel.
        let hit = tree.hit_test(
            panel_rect.x + panel_rect.width - 2.0,
            panel_rect.y + panel_rect.height - 2.0,
        );
        assert_eq!(hit, Some(panel));
    }

    #[test]
    fn hit_test_misses_empty_area() {
        let (tree, _, _) = tree_with_button();
        // Outside all widgets.
        let hit = tree.hit_test(0.0, 0.0);
        assert_eq!(hit, None);

        let hit = tree.hit_test(500.0, 500.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn focusable_widgets_returns_buttons() {
        let (tree, _panel, button) = tree_with_button();
        let focusable = tree.focusable_widgets();
        assert_eq!(focusable.len(), 1);
        assert_eq!(focusable[0], button);
    }

    #[test]
    fn hover_tracking() {
        let (mut tree, _panel, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();

        // Move cursor over button.
        let consumed = state.handle_cursor_moved(&mut tree, btn_rect.x + 1.0, btn_rect.y + 1.0);
        assert!(consumed);
        assert_eq!(state.hovered, Some(button));

        // Move cursor outside all widgets.
        let consumed = state.handle_cursor_moved(&mut tree, 0.0, 0.0);
        assert!(!consumed);
        assert_eq!(state.hovered, None);
    }

    #[test]
    fn click_sets_focus() {
        let (mut tree, _, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();
        let bx = btn_rect.x + 1.0;
        let by = btn_rect.y + 1.0;

        assert_eq!(state.focused, None);

        // Press on button.
        let consumed = state.handle_mouse_input(&mut tree, MouseButton::Left, true, bx, by);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));

        // Release on button.
        let consumed = state.handle_mouse_input(&mut tree, MouseButton::Left, false, bx, by);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));
    }

    #[test]
    fn click_outside_clears_focus() {
        let (mut tree, _, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();

        // Focus the button first.
        state.handle_mouse_input(
            &mut tree,
            MouseButton::Left,
            true,
            btn_rect.x + 1.0,
            btn_rect.y + 1.0,
        );
        state.handle_mouse_input(
            &mut tree,
            MouseButton::Left,
            false,
            btn_rect.x + 1.0,
            btn_rect.y + 1.0,
        );
        assert_eq!(state.focused, Some(button));

        // Click outside all widgets.
        let consumed = state.handle_mouse_input(&mut tree, MouseButton::Left, true, 0.0, 0.0);
        assert!(!consumed);
        assert_eq!(state.focused, None);
    }

    #[test]
    fn tab_cycles_focus() {
        let (mut tree, _, button) = tree_with_button();
        let mut state = UiState::new();

        // Tab with no focus — focuses first focusable.
        let consumed = state.handle_key_input(&mut tree, winit::keyboard::KeyCode::Tab, true);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));

        // Tab again wraps around (only 1 button, so stays on it).
        let consumed = state.handle_key_input(&mut tree, winit::keyboard::KeyCode::Tab, true);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));
    }

    #[test]
    fn mouse_capture_holds_during_drag() {
        let (mut tree, _, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();
        let bx = btn_rect.x + 1.0;
        let by = btn_rect.y + 1.0;

        // Press on button — starts capture.
        state.handle_mouse_input(&mut tree, MouseButton::Left, true, bx, by);
        assert_eq!(state.captured, Some(button));

        // Move far away — capture holds.
        state.handle_cursor_moved(&mut tree, 500.0, 500.0);
        assert_eq!(state.captured, Some(button));
        assert!(state.dragging); // crossed threshold

        // Release — capture ends.
        state.handle_mouse_input(&mut tree, MouseButton::Left, false, 500.0, 500.0);
        assert_eq!(state.captured, None);
        assert!(!state.dragging);
    }

    #[test]
    fn scroll_consumed_over_widget() {
        let (mut tree, panel, _) = tree_with_button();
        let panel_rect = tree.get(panel).unwrap().rect;
        let mut state = UiState::new();

        // Position cursor over panel.
        state.cursor = (panel_rect.x + 1.0, panel_rect.y + 1.0);
        let consumed = state.handle_scroll(&mut tree, 1.0);
        assert!(consumed);

        // Position cursor outside.
        state.cursor = (0.0, 0.0);
        let consumed = state.handle_scroll(&mut tree, 1.0);
        assert!(!consumed);
    }
}
