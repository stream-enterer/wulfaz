//! Event popup screen (UI-401).
//!
//! Modal dialog for narrative events with choices.
//! CK3-style: parchment background, gold border, choice buttons.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::window::build_window_frame;
use super::{Edges, FontFamily, Sizing, Widget, WidgetId, WidgetTree};

/// A narrative event with title, body, and choices.
#[derive(Debug, Clone)]
pub struct NarrativeEvent {
    pub title: String,
    pub body: String,
    pub choices: Vec<EventChoice>,
}

/// A single choice in a narrative event.
#[derive(Debug, Clone)]
pub struct EventChoice {
    pub label: String,
    pub tooltip: Option<String>,
    pub callback: String,
}

/// Build the event popup (UI-401).
///
/// Returns the content root widget ID. The caller pushes it onto the ModalStack.
/// Choice buttons have `on_click` set to `"event_choice:<callback>"`.
pub fn build_event_popup(
    tree: &mut WidgetTree,
    theme: &Theme,
    event: &NarrativeEvent,
    screen_width: f32,
) -> WidgetId {
    // Panel width: 60% of screen, capped at 600px
    let panel_w = (screen_width * 0.6).min(600.0);

    let frame = build_window_frame(tree, theme, &event.title, panel_w, Sizing::Fit, false);

    // Apply event popup overrides:
    // - Gold border, +1 border_width
    // - 1.5x padding
    // - 1.25x title font
    // - 3x content gap
    if let Some(root_node) = tree.get_mut(frame.root) {
        root_node.widget = Widget::Panel {
            bg_color: theme.bg_parchment,
            border_color: theme.gold,
            border_width: theme.panel_border_width + 1.0,
            shadow_width: theme.panel_shadow_width,
        };
        root_node.padding = Edges::all(theme.panel_padding * 1.5);
    }

    // Override title font size to 1.25x
    if let Some(title_node) = tree.get_mut(frame.title)
        && let Widget::Label { font_size, .. } = &mut title_node.widget
    {
        *font_size = theme.font_header_size * 1.25;
    }

    // Override frame_col gap (first child of root)
    if let Some(frame_col_id) = tree.get(frame.root).map(|n| n.children[0])
        && let Some(col_node) = tree.get_mut(frame_col_id)
    {
        col_node.widget = Widget::Column {
            gap: theme.label_gap * 3.0,
            align: CrossAlign::Start,
        };
    }

    let content_w = panel_w - theme.panel_padding * 3.0;

    // Body text (wrapped)
    tree.insert(
        frame.content,
        Widget::Label {
            text: event.body.clone(),
            color: theme.text_medium,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: true,
        },
    );

    // Choice buttons row
    let button_row = tree.insert(
        frame.content,
        Widget::Row {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(button_row, Sizing::Fixed(content_w), Sizing::Fit);

    for choice in &event.choices {
        let btn = tree.insert(
            button_row,
            Widget::Button {
                text: choice.label.clone(),
                color: theme.text_medium,
                bg_color: theme.tab_inactive_color,
                border_color: theme.gold,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
            },
        );
        tree.set_on_click(btn, format!("event_choice:{}", choice.callback));
        tree.set_padding(
            btn,
            Edges {
                top: theme.button_pad_v,
                right: theme.button_pad_h,
                bottom: theme.button_pad_v,
                left: theme.button_pad_h,
            },
        );

        if let Some(ref tooltip) = choice.tooltip {
            tree.set_tooltip(
                btn,
                Some(super::widget::TooltipContent::Text(tooltip.clone())),
            );
        }
    }

    frame.root
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_event() -> NarrativeEvent {
        NarrativeEvent {
            title: "A Stranger Approaches".to_string(),
            body: "A hooded figure emerges from the shadows, offering a deal.".to_string(),
            choices: vec![
                EventChoice {
                    label: "Accept".to_string(),
                    tooltip: Some("Opinion +10 with Strangers".to_string()),
                    callback: "accept".to_string(),
                },
                EventChoice {
                    label: "Refuse".to_string(),
                    tooltip: None,
                    callback: "refuse".to_string(),
                },
                EventChoice {
                    label: "Attack".to_string(),
                    tooltip: Some("Opinion -20 with Vassals".to_string()),
                    callback: "attack".to_string(),
                },
            ],
        }
    }

    #[test]
    fn event_popup_has_3_choice_buttons() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let root = build_event_popup(&mut tree, &theme, &test_event(), 800.0);

        // Navigate: root -> frame_col -> content -> [body, button_row]
        let panel_node = tree.get(root).unwrap();
        let frame_col = panel_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();

        // Content children: body label, button_row
        assert!(content_node.children.len() >= 2);
        let button_row_id = content_node.children[1];
        let button_row_node = tree.get(button_row_id).unwrap();
        assert_eq!(
            button_row_node.children.len(),
            3,
            "Should have 3 choice buttons"
        );

        // Verify buttons have on_click callbacks
        for (i, &btn_id) in button_row_node.children.iter().enumerate() {
            let btn_node = tree.get(btn_id).unwrap();
            assert!(
                btn_node.on_click.is_some(),
                "Button {} should have on_click callback",
                i
            );
        }
    }

    #[test]
    fn event_popup_callback_keys() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let root = build_event_popup(&mut tree, &theme, &test_event(), 800.0);

        let panel_node = tree.get(root).unwrap();
        let frame_col = panel_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();
        let button_row_id = content_node.children[1];
        let button_row_node = tree.get(button_row_id).unwrap();

        let expected_callbacks = [
            "event_choice:accept",
            "event_choice:refuse",
            "event_choice:attack",
        ];
        for (i, &btn_id) in button_row_node.children.iter().enumerate() {
            let btn_node = tree.get(btn_id).unwrap();
            assert_eq!(btn_node.on_click.as_deref(), Some(expected_callbacks[i]),);
        }
    }

    #[test]
    fn event_popup_width_capped() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        // Large screen -- panel should be capped at 600px
        let root = build_event_popup(&mut tree, &theme, &test_event(), 2000.0);
        let panel_node = tree.get(root).unwrap();
        if let Sizing::Fixed(w) = panel_node.width {
            assert!(
                w <= 600.0,
                "Panel width should be capped at 600px, got {}",
                w
            );
        }
    }
}
