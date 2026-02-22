//! Outliner panel (UI-405).
//!
//! Persistent side panel (right edge, CK3 style) showing pinned items,
//! active events, and alerts. Uses Collapsible sections.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{Edges, FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

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
/// Returns the panel root ID. Register with PanelManager as `"outliner"`.
pub fn build_outliner(tree: &mut WidgetTree, theme: &Theme, info: &OutlinerInfo) -> WidgetId {
    let panel_h = (info.screen_height * 0.5).min(500.0);

    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    tree.set_sizing(panel, Sizing::Fixed(OUTLINER_WIDTH), Sizing::Fixed(panel_h));
    tree.set_padding(panel, Edges::all(theme.panel_padding));

    let content_w = OUTLINER_WIDTH - theme.panel_padding * 2.0;
    let mut y = 0.0_f32;

    // Title
    let title = tree.insert(
        panel,
        Widget::Label {
            text: "Outliner".to_string(),
            color: theme.gold,
            font_size: theme.font_header_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );
    tree.set_position(title, Position::Fixed { x: 0.0, y });
    y += theme.font_header_size + theme.label_gap;

    let sep = tree.insert(
        panel,
        Widget::Separator {
            color: theme.gold,
            thickness: theme.separator_thickness,
            horizontal: true,
        },
    );
    tree.set_position(sep, Position::Fixed { x: 0.0, y });
    tree.set_sizing(sep, Sizing::Fixed(content_w), Sizing::Fit);
    y += theme.separator_thickness + theme.label_gap * 2.0;

    // === Pinned Characters section ===
    let pinned = tree.insert(
        panel,
        Widget::Collapsible {
            header: format!("Pinned Characters ({})", info.pinned_characters.len()),
            expanded: true,
            color: theme.text_dark,
            font_size: theme.font_body_size,
        },
    );
    tree.set_position(pinned, Position::Fixed { x: 0.0, y });
    tree.set_sizing(pinned, Sizing::Fixed(content_w), Sizing::Fit);
    y += theme.font_body_size + theme.label_gap;

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
                color: theme.text_dark,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
        y += theme.font_body_size + theme.label_gap;
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
        y += theme.font_data_size + theme.label_gap;
    }

    y += theme.label_gap;

    // === Active Events section ===
    let events_section = tree.insert(
        panel,
        Widget::Collapsible {
            header: format!("Active Events ({})", info.active_events.len()),
            expanded: true,
            color: theme.text_dark,
            font_size: theme.font_body_size,
        },
    );
    tree.set_position(events_section, Position::Fixed { x: 0.0, y });
    tree.set_sizing(events_section, Sizing::Fixed(content_w), Sizing::Fit);
    y += theme.font_body_size + theme.label_gap;

    for evt in &info.active_events {
        let evt_label = tree.insert(
            events_section,
            Widget::Label {
                text: evt.title.clone(),
                color: theme.text_dark,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
        tree.set_sizing(evt_label, Sizing::Fixed(content_w - 16.0), Sizing::Fit);
        tree.set_on_click(evt_label, format!("outliner::event:{}", evt.callback));
        y += theme.font_body_size + theme.label_gap;
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

    y += theme.label_gap;

    // === Alerts section ===
    let alerts_section = tree.insert(
        panel,
        Widget::Collapsible {
            header: format!("Alerts ({})", info.alerts.len()),
            expanded: !info.alerts.is_empty(),
            color: theme.text_dark,
            font_size: theme.font_body_size,
        },
    );
    tree.set_position(alerts_section, Position::Fixed { x: 0.0, y });
    tree.set_sizing(alerts_section, Sizing::Fixed(content_w), Sizing::Fit);

    for alert in &info.alerts {
        let color = match alert.priority {
            AlertPriority::Info => theme.text_dark,
            AlertPriority::Warning => theme.gold,
            AlertPriority::Critical => theme.danger,
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

    panel
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
        let root = build_outliner(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();

        // Find the Pinned Characters Collapsible
        let mut found_pinned = false;
        for &child_id in &panel_node.children {
            if let Some(node) = tree.get(child_id) {
                if let Widget::Collapsible { header, .. } = &node.widget {
                    if header.contains("Pinned Characters") {
                        assert_eq!(node.children.len(), 3, "Should have 3 pinned characters");
                        found_pinned = true;
                    }
                }
            }
        }
        assert!(found_pinned, "Should have a Pinned Characters section");
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
        let root = build_outliner(&mut tree, &theme, &info);
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
        let root = build_outliner(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();

        // Find the Alerts Collapsible and verify it's expanded
        for &child_id in &panel_node.children {
            if let Some(node) = tree.get(child_id) {
                if let Widget::Collapsible {
                    header, expanded, ..
                } = &node.widget
                {
                    if header.contains("Alerts") {
                        assert!(
                            *expanded,
                            "Alerts section should be expanded when there are alerts"
                        );
                    }
                }
            }
        }
    }
}
