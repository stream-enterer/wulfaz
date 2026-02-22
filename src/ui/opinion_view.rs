//! Relationship/opinion view (UI-406).
//!
//! Sub-panel within the character panel's Relations tab.
//! Stubbed with mock data until SIM-008 (relationships) is implemented.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

/// An opinion modifier contributing to the total opinion score.
#[derive(Debug, Clone)]
pub struct OpinionModifier {
    pub label: String,
    pub value: i32,               // -100 to +100
    pub icon: Option<String>,     // sprite atlas key
    pub duration: Option<String>, // e.g., "3y remaining"
}

/// Info needed to build the opinion view.
pub struct OpinionViewInfo {
    pub target_name: String,
    pub target_id: u64,
    pub modifiers: Vec<OpinionModifier>,
    pub sentiment: Option<Sentiment>,
}

/// Relationship sentiment indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sentiment {
    Ally,
    Rival,
    Friend,
    Lover,
}

impl Sentiment {
    pub fn label(self) -> &'static str {
        match self {
            Sentiment::Ally => "Ally",
            Sentiment::Rival => "Rival",
            Sentiment::Friend => "Friend",
            Sentiment::Lover => "Lover",
        }
    }

    pub fn color(self, theme: &Theme) -> [f32; 4] {
        match self {
            Sentiment::Ally | Sentiment::Friend | Sentiment::Lover => theme.progress_bar_health_fg,
            Sentiment::Rival => theme.danger,
        }
    }
}

/// Opinion panel width.
const OPINION_WIDTH: f32 = 250.0;

/// Build the opinion view sub-panel (UI-406).
///
/// Returns the panel root ID. Designed to be inserted into the character panel's Relations tab.
pub fn build_opinion_view(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &OpinionViewInfo,
) -> WidgetId {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: [0.0, 0.0, 0.0, 0.0], // transparent — embedded in parent panel
        border_color: [0.0; 4],
        border_width: 0.0,
        shadow_width: 0.0,
    });
    tree.set_sizing(panel, Sizing::Fixed(OPINION_WIDTH), Sizing::Fit);

    let content_w = OPINION_WIDTH;
    let mut y = 0.0_f32;

    // Target name header
    let header = tree.insert(
        panel,
        Widget::Label {
            text: info.target_name.clone(),
            color: theme.text_dark,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );
    tree.set_position(header, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    // Sentiment indicator
    if let Some(sentiment) = info.sentiment {
        let sent = tree.insert(
            panel,
            Widget::Label {
                text: sentiment.label().to_string(),
                color: sentiment.color(theme),
                font_size: theme.font_data_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
        tree.set_position(sent, Position::Fixed { x: 0.0, y });
        y += theme.font_data_size + theme.label_gap;
    }

    // Opinion bar: -100 (red) to +100 (green), centered at 0
    let total_opinion: i32 = info.modifiers.iter().map(|m| m.value).sum();
    let clamped = total_opinion.clamp(-100, 100);
    // Map -100..+100 to 0.0..1.0
    let fraction = (clamped as f32 + 100.0) / 200.0;

    let opinion_label = tree.insert(
        panel,
        Widget::Label {
            text: format!(
                "Opinion: {}{}",
                if total_opinion >= 0 { "+" } else { "" },
                total_opinion
            ),
            color: if total_opinion >= 0 {
                theme.progress_bar_health_fg
            } else {
                theme.danger
            },
            font_size: theme.font_body_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );
    tree.set_position(opinion_label, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    // Opinion bar — green fill for positive, red fill for negative
    let bar_fg = if total_opinion >= 0 {
        theme.progress_bar_health_fg
    } else {
        theme.danger
    };
    let bar = tree.insert(
        panel,
        Widget::ProgressBar {
            fraction,
            fg_color: bar_fg,
            bg_color: theme.progress_bar_health_bg,
            border_color: theme.panel_border_color,
            border_width: theme.progress_bar_border_width,
            height: theme.progress_bar_height + 2.0,
        },
    );
    tree.set_position(bar, Position::Fixed { x: 0.0, y });
    tree.set_sizing(bar, Sizing::Fixed(content_w), Sizing::Fit);
    y += theme.progress_bar_height + 4.0 + theme.label_gap * 2.0;

    // Modifiers breakdown (Collapsible)
    let modifiers_section = tree.insert(
        panel,
        Widget::Collapsible {
            header: "Opinion Modifiers".to_string(),
            expanded: true,
            color: theme.text_dark,
            font_size: theme.font_body_size,
        },
    );
    tree.set_position(modifiers_section, Position::Fixed { x: 0.0, y });
    tree.set_sizing(modifiers_section, Sizing::Fixed(content_w), Sizing::Fit);

    for modifier in &info.modifiers {
        let row = tree.insert(
            modifiers_section,
            Widget::Row {
                gap: theme.label_gap * 2.0,
                align: CrossAlign::Center,
            },
        );
        tree.set_sizing(row, Sizing::Fixed(content_w - 16.0), Sizing::Fit);

        // Modifier value (colored)
        let value_color = if modifier.value >= 0 {
            theme.progress_bar_health_fg
        } else {
            theme.danger
        };
        tree.insert(
            row,
            Widget::Label {
                text: format!(
                    "{}{}",
                    if modifier.value >= 0 { "+" } else { "" },
                    modifier.value
                ),
                color: value_color,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );

        // Modifier label
        tree.insert(
            row,
            Widget::Label {
                text: modifier.label.clone(),
                color: theme.text_dark,
                font_size: theme.font_data_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );

        // Duration (if temporary)
        if let Some(ref duration) = modifier.duration {
            tree.insert(
                row,
                Widget::Label {
                    text: duration.clone(),
                    color: theme.disabled,
                    font_size: theme.font_data_size,
                    font_family: FontFamily::Mono,
                    wrap: false,
                },
            );
        }
    }

    panel
}

/// Create mock opinion data for testing.
pub fn mock_opinion_info() -> OpinionViewInfo {
    OpinionViewInfo {
        target_name: "Baron Guillaume".to_string(),
        target_id: 42,
        modifiers: vec![
            OpinionModifier {
                label: "Same culture".to_string(),
                value: 15,
                icon: None,
                duration: None,
            },
            OpinionModifier {
                label: "Rival".to_string(),
                value: -50,
                icon: None,
                duration: None,
            },
            OpinionModifier {
                label: "Recent gift".to_string(),
                value: 10,
                icon: None,
                duration: Some("3y remaining".to_string()),
            },
            OpinionModifier {
                label: "Alliance".to_string(),
                value: 50,
                icon: None,
                duration: None,
            },
        ],
        sentiment: Some(Sentiment::Ally),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opinion_bar_fraction_positive() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = OpinionViewInfo {
            target_name: "Test".to_string(),
            target_id: 1,
            modifiers: vec![
                OpinionModifier {
                    label: "A".to_string(),
                    value: 15,
                    icon: None,
                    duration: None,
                },
                OpinionModifier {
                    label: "B".to_string(),
                    value: 10,
                    icon: None,
                    duration: None,
                },
            ],
            sentiment: None,
        };
        let root = build_opinion_view(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();

        // Find ProgressBar and verify fraction
        // Total = +25, fraction = (25 + 100) / 200 = 0.625
        let mut found_bar = false;
        for &child_id in &panel_node.children {
            if let Some(node) = tree.get(child_id) {
                if let Widget::ProgressBar { fraction, .. } = &node.widget {
                    assert!(
                        (*fraction - 0.625).abs() < 0.01,
                        "Expected 0.625, got {}",
                        fraction
                    );
                    found_bar = true;
                }
            }
        }
        assert!(found_bar, "Should have a ProgressBar for opinion");
    }

    #[test]
    fn opinion_bar_fraction_negative() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = OpinionViewInfo {
            target_name: "Enemy".to_string(),
            target_id: 2,
            modifiers: vec![OpinionModifier {
                label: "Rival".to_string(),
                value: -80,
                icon: None,
                duration: None,
            }],
            sentiment: Some(Sentiment::Rival),
        };
        let root = build_opinion_view(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();

        // Total = -80, fraction = (-80 + 100) / 200 = 0.1
        for &child_id in &panel_node.children {
            if let Some(node) = tree.get(child_id) {
                if let Widget::ProgressBar { fraction, .. } = &node.widget {
                    assert!(
                        (*fraction - 0.1).abs() < 0.01,
                        "Expected 0.1, got {}",
                        fraction
                    );
                }
            }
        }
    }

    #[test]
    fn mock_opinion_data_sums_to_25() {
        let info = mock_opinion_info();
        let total: i32 = info.modifiers.iter().map(|m| m.value).sum();
        assert_eq!(total, 25);
    }

    #[test]
    fn modifiers_section_has_entries() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = mock_opinion_info();
        let root = build_opinion_view(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();

        // Find the Collapsible section
        let mut found_modifiers = false;
        for &child_id in &panel_node.children {
            if let Some(node) = tree.get(child_id) {
                if let Widget::Collapsible { header, .. } = &node.widget {
                    if header == "Opinion Modifiers" {
                        assert_eq!(node.children.len(), 4, "Should have 4 modifier rows");
                        found_modifiers = true;
                    }
                }
            }
        }
        assert!(found_modifiers, "Should have a modifiers section");
    }
}
