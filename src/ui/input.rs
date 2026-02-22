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
        }
    }

    /// Handle cursor movement. Returns true if the cursor is over a UI widget
    /// (event consumed — don't pass to game).
    pub fn handle_cursor_moved(&mut self, tree: &WidgetTree, x: f32, y: f32) -> bool {
        self.cursor = (x, y);

        // If a widget has capture, route drag events to it.
        if self.captured.is_some() {
            if let Some(origin) = self.press_origin {
                let dx = x - origin.0;
                let dy = y - origin.1;
                if !self.dragging && (dx * dx + dy * dy).sqrt() >= DRAG_THRESHOLD {
                    self.dragging = true;
                    // DragStart event (widget can react when callbacks are added)
                    let _ = UiEvent::DragStart {
                        x: origin.0,
                        y: origin.1,
                    };
                }
                if self.dragging {
                    let _ = UiEvent::DragMove { x, y };
                }
            }
            // Cursor is captured — update hover to whatever is under cursor
            // but the capture still holds.
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
        tree: &WidgetTree,
        button: MouseButton,
        pressed: bool,
        x: f32,
        y: f32,
    ) -> bool {
        self.cursor = (x, y);

        if pressed {
            // Mouse down
            let hit = if let Some(cap) = self.captured {
                // Captured widget gets the event regardless of position.
                Some(cap)
            } else {
                tree.hit_test(x, y)
            };

            if let Some(widget_id) = hit {
                self.pressed = Some(widget_id);
                self.pressed_button = Some(button);
                self.press_origin = Some((x, y));
                self.dragging = false;

                // Left click sets focus to the clicked widget (if focusable).
                if button == MouseButton::Left
                    && let Some(node) = tree.get(widget_id)
                    && matches!(node.widget, Widget::Button { .. })
                {
                    self.focused = Some(widget_id);
                }

                // Start capture (for drag support).
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
        self.press_origin = None;
        self.dragging = false;

        if was_captured.is_some() {
            if was_dragging {
                // Drag ended.
                let _ = UiEvent::DragEnd;
            } else if let Some(pressed_id) = was_pressed {
                // Not a drag — check if release is on the same widget for a click.
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
        tree: &WidgetTree,
        key: winit::keyboard::KeyCode,
        pressed: bool,
    ) -> bool {
        if !pressed {
            return self.focused.is_some();
        }

        // Tab cycles focus through focusable widgets.
        if key == winit::keyboard::KeyCode::Tab {
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

        // Other keys go to focused widget (if any).
        self.focused.is_some()
    }

    /// Handle scroll wheel. Returns true if consumed by a widget under cursor.
    pub fn handle_scroll(&mut self, tree: &WidgetTree, delta: f32) -> bool {
        let hit = tree.hit_test(self.cursor.0, self.cursor.1);
        if hit.is_some() {
            let _ = UiEvent::Scroll(delta);
            return true;
        }
        false
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
        let (tree, _panel, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();

        // Move cursor over button.
        let consumed = state.handle_cursor_moved(&tree, btn_rect.x + 1.0, btn_rect.y + 1.0);
        assert!(consumed);
        assert_eq!(state.hovered, Some(button));

        // Move cursor outside all widgets.
        let consumed = state.handle_cursor_moved(&tree, 0.0, 0.0);
        assert!(!consumed);
        assert_eq!(state.hovered, None);
    }

    #[test]
    fn click_sets_focus() {
        let (tree, _, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();
        let bx = btn_rect.x + 1.0;
        let by = btn_rect.y + 1.0;

        assert_eq!(state.focused, None);

        // Press on button.
        let consumed = state.handle_mouse_input(&tree, MouseButton::Left, true, bx, by);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));

        // Release on button.
        let consumed = state.handle_mouse_input(&tree, MouseButton::Left, false, bx, by);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));
    }

    #[test]
    fn click_outside_clears_focus() {
        let (tree, _, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();

        // Focus the button first.
        state.handle_mouse_input(
            &tree,
            MouseButton::Left,
            true,
            btn_rect.x + 1.0,
            btn_rect.y + 1.0,
        );
        state.handle_mouse_input(
            &tree,
            MouseButton::Left,
            false,
            btn_rect.x + 1.0,
            btn_rect.y + 1.0,
        );
        assert_eq!(state.focused, Some(button));

        // Click outside all widgets.
        let consumed = state.handle_mouse_input(&tree, MouseButton::Left, true, 0.0, 0.0);
        assert!(!consumed);
        assert_eq!(state.focused, None);
    }

    #[test]
    fn tab_cycles_focus() {
        let (tree, _, button) = tree_with_button();
        let mut state = UiState::new();

        // Tab with no focus — focuses first focusable.
        let consumed = state.handle_key_input(&tree, winit::keyboard::KeyCode::Tab, true);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));

        // Tab again wraps around (only 1 button, so stays on it).
        let consumed = state.handle_key_input(&tree, winit::keyboard::KeyCode::Tab, true);
        assert!(consumed);
        assert_eq!(state.focused, Some(button));
    }

    #[test]
    fn mouse_capture_holds_during_drag() {
        let (tree, _, button) = tree_with_button();
        let btn_rect = tree.get(button).unwrap().rect;
        let mut state = UiState::new();
        let bx = btn_rect.x + 1.0;
        let by = btn_rect.y + 1.0;

        // Press on button — starts capture.
        state.handle_mouse_input(&tree, MouseButton::Left, true, bx, by);
        assert_eq!(state.captured, Some(button));

        // Move far away — capture holds.
        state.handle_cursor_moved(&tree, 500.0, 500.0);
        assert_eq!(state.captured, Some(button));
        assert!(state.dragging); // crossed threshold

        // Release — capture ends.
        state.handle_mouse_input(&tree, MouseButton::Left, false, 500.0, 500.0);
        assert_eq!(state.captured, None);
        assert!(!state.dragging);
    }

    #[test]
    fn scroll_consumed_over_widget() {
        let (tree, panel, _) = tree_with_button();
        let panel_rect = tree.get(panel).unwrap().rect;
        let mut state = UiState::new();

        // Position cursor over panel.
        state.cursor = (panel_rect.x + 1.0, panel_rect.y + 1.0);
        let consumed = state.handle_scroll(&tree, 1.0);
        assert!(consumed);

        // Position cursor outside.
        state.cursor = (0.0, 0.0);
        let consumed = state.handle_scroll(&tree, 1.0);
        assert!(!consumed);
    }
}
