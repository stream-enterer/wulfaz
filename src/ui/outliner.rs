//! Outliner panel (UI-405).
//!
//! Persistent side panel (right edge, CK3 style) showing pinned items,
//! active events, and alerts. Uses Collapsible sections.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::window::build_window_frame;
use super::{FontFamily, Sizing, Widget, WidgetId, WidgetTree};

/// A pinned character entry in the outliner.
#[derive(Debug, Clone)]
pub struct PinnedCharacter {
    pub entity_id: u64,
    pub icon: char,
    pub name: String,
}

/// An active event entry in the outliner.
#[derive(Debug, Clone)]
pub struct ActiveEvent {
    pub title: String,
    pub callback: String,
}

/// An alert entry in the outliner.
#[derive(Debug, Clone)]
pub struct AlertEntry {
    pub message: String,
    pub priority: AlertPriority,
}

/// Alert priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertPriority {
    Info,
    Warning,
    Critical,
}

/// Info needed to build the outliner.
pub struct OutlinerInfo {
    pub pinned_characters: Vec<PinnedCharacter>,
    pub active_events: Vec<ActiveEvent>,
    pub alerts: Vec<AlertEntry>,
    pub screen_height: f32,
}

/// Outliner panel width.
const OUTLINER_WIDTH: f32 = 200.0;

/// Build the outliner panel (UI-405).
///
/// Returns `(panel_root_id, close_button_id)`. Register with PanelManager as `"outliner"`.
pub fn build_outliner(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &OutlinerInfo,
) -> (WidgetId, WidgetId) {
    let w = theme.s(OUTLINER_WIDTH);
    let panel_h = (info.screen_height * 0.5).min(500.0);

    let frame = build_window_frame(tree, theme, "Outliner", w, Sizing::Fixed(panel_h), true);
    let content_w = frame.content_width;

    // === Pinned Characters section ===
    let pinned = tree.insert(
        frame.content,
        Widget::Collapsible {
            header: format!("Pinned Characters ({})", info.pinned_characters.len()),
            expanded: true,
            color: theme.text_medium,
            font_size: theme.font_body_size,
        },
    );
    tree.set_sizing(pinned, Sizing::Fixed(content_w), Sizing::Fit);

    for ch in &info.pinned_characters {
        let row = tree.insert(
            pinned,
            Widget::Row {
                gap: theme.label_gap,
                align: CrossAlign::Center,
            },
        );
        tree.set_sizing(row, Sizing::Fixed(content_w - 16.0), Sizing::Fit);
        tree.set_on_click(row, format!("outliner::character:{}", ch.entity_id));

        tree.insert(
            row,
            Widget::Label {
                text: ch.icon.to_string(),
                color: theme.gold,
                font_size: theme.font_body_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        tree.insert(
            row,
            Widget::Label {
                text: ch.name.clone(),
                color: theme.text_medium,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
    }

    if info.pinned_characters.is_empty() {
        let empty = tree.insert(
            pinned,
            Widget::Label {
                text: "No pinned characters.".to_string(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
        tree.set_sizing(empty, Sizing::Fixed(content_w - 16.0), Sizing::Fit);
    }

    // === Active Events section ===
    let events_section = tree.insert(
        frame.content,
        Widget::Collapsible {
            header: format!("Active Events ({})", info.active_events.len()),
            expanded: true,
            color: theme.text_medium,
            font_size: theme.font_body_size,
        },
    );
    tree.set_sizing(events_section, Sizing::Fixed(content_w), Sizing::Fit);

    for evt in &info.active_events {
        let evt_label = tree.insert(
            events_section,
            Widget::Label {
                text: evt.title.clone(),
                color: theme.text_medium,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
        tree.set_sizing(evt_label, Sizing::Fixed(content_w - 16.0), Sizing::Fit);
        tree.set_on_click(evt_label, format!("outliner::event:{}", evt.callback));
    }

    if info.active_events.is_empty() {
        let empty = tree.insert(
            events_section,
            Widget::Label {
                text: "No active events.".to_string(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
        tree.set_sizing(empty, Sizing::Fixed(content_w - 16.0), Sizing::Fit);
    }

    // === Alerts section ===
    let alerts_section = tree.insert(
        frame.content,
        Widget::Collapsible {
            header: format!("Alerts ({})", info.alerts.len()),
            expanded: !info.alerts.is_empty(),
            color: theme.text_medium,
            font_size: theme.font_body_size,
        },
    );
    tree.set_sizing(alerts_section, Sizing::Fixed(content_w), Sizing::Fit);

    for alert in &info.alerts {
        let color = match alert.priority {
            AlertPriority::Info => theme.text_medium,
            AlertPriority::Warning => theme.text_warning,
            AlertPriority::Critical => theme.text_negative,
        };
        let alert_label = tree.insert(
            alerts_section,
            Widget::Label {
                text: alert.message.clone(),
                color,
                font_size: theme.font_data_size,
                font_family: FontFamily::Serif,
                wrap: true,
            },
        );
        tree.set_sizing(alert_label, Sizing::Fixed(content_w - 16.0), Sizing::Fit);
    }

    if info.alerts.is_empty() {
        let empty = tree.insert(
            alerts_section,
            Widget::Label {
                text: "No alerts.".to_string(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
        tree.set_sizing(empty, Sizing::Fixed(content_w - 16.0), Sizing::Fit);
    }

    // close_btn is always Some here since closeable=true
    (frame.root, frame.close_btn.expect("closeable frame"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outliner_with_pinned_characters() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = OutlinerInfo {
            pinned_characters: vec![
                PinnedCharacter {
                    entity_id: 1,
                    icon: 'g',
                    name: "Goblin".to_string(),
                },
                PinnedCharacter {
                    entity_id: 2,
                    icon: 'w',
                    name: "Wolf".to_string(),
                },
                PinnedCharacter {
                    entity_id: 3,
                    icon: 'g',
                    name: "Goblin Chief".to_string(),
                },
            ],
            active_events: vec![],
            alerts: vec![],
            screen_height: 600.0,
        };
        let (root, _close) = build_outliner(&mut tree, &theme, &info);

        // Navigate: root -> frame_col -> content -> Pinned Characters
        let root_node = tree.get(root).unwrap();
        let frame_col = root_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();

        // First child of content is the Pinned Characters Collapsible
        let pinned_id = content_node.children[0];
        let pinned_node = tree.get(pinned_id).unwrap();
        if let Widget::Collapsible { header, .. } = &pinned_node.widget {
            assert!(header.contains("Pinned Characters"));
            assert_eq!(
                pinned_node.children.len(),
                3,
                "Should have 3 pinned characters"
            );
        } else {
            panic!("Expected Collapsible widget");
        }
    }

    #[test]
    fn outliner_empty_sections() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = OutlinerInfo {
            pinned_characters: vec![],
            active_events: vec![],
            alerts: vec![],
            screen_height: 600.0,
        };
        let (root, _close) = build_outliner(&mut tree, &theme, &info);
        assert!(tree.get(root).is_some());
    }

    #[test]
    fn outliner_alerts_section_has_critical() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = OutlinerInfo {
            pinned_characters: vec![],
            active_events: vec![],
            alerts: vec![AlertEntry {
                message: "Fire!".to_string(),
                priority: AlertPriority::Critical,
            }],
            screen_height: 600.0,
        };
        let (root, _close) = build_outliner(&mut tree, &theme, &info);

        // Navigate: root -> frame_col -> content -> alerts collapsible
        let root_node = tree.get(root).unwrap();
        let frame_col = root_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();

        // Alerts is the 3rd child of content
        let alerts_id = content_node.children[2];
        let alerts_node = tree.get(alerts_id).unwrap();
        if let Widget::Collapsible {
            header, expanded, ..
        } = &alerts_node.widget
        {
            assert!(header.contains("Alerts"));
            assert!(
                *expanded,
                "Alerts section should be expanded when there are alerts"
            );
        } else {
            panic!("Expected Collapsible widget");
        }
    }

    #[test]
    fn outliner_has_close_button() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = OutlinerInfo {
            pinned_characters: vec![],
            active_events: vec![],
            alerts: vec![],
            screen_height: 600.0,
        };
        let (_root, close) = build_outliner(&mut tree, &theme, &info);
        let close_node = tree.get(close).unwrap();
        if let Widget::Button { text, .. } = &close_node.widget {
            assert_eq!(text, "X");
        } else {
            panic!("Expected Button widget for close");
        }
    }
}
