//! Full character panel (UI-400).
//!
//! CK3-style character sheet with tabs: Overview, Family, Relations, Traits.
//! Registered with PanelManager as `"character:<entity_id>"`.

use super::draw::TextSpan;
use super::theme::Theme;
use super::widget::CrossAlign;
use super::window::build_window_frame;
use super::{FontFamily, Sizing, Widget, WidgetId, WidgetTree};

/// Entity data needed to build the character panel.
pub struct CharacterPanelInfo {
    pub entity_id: u64,
    pub name: String,
    pub icon: char,
    pub health: Option<(f32, f32)>,
    pub hunger: Option<(f32, f32)>,
    pub fatigue: Option<f32>,
    pub combat: Option<(f32, f32, f32)>, // attack, defense, aggression
    pub position: (i32, i32),
    pub gait: Option<String>,
    pub action: Option<String>,
}

/// Character panel width in pixels.
const PANEL_WIDTH: f32 = 280.0;

/// Build the full character panel (UI-400).
///
/// Returns `(panel_root_id, close_button_id)`.
/// The caller registers it with PanelManager as `"character:<entity_id>"`.
pub fn build_character_panel(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &CharacterPanelInfo,
) -> (WidgetId, WidgetId) {
    let title_text = format!("{} {}", info.icon, info.name);
    let frame = build_window_frame(tree, theme, &title_text, PANEL_WIDTH, Sizing::Fit, true);

    // Replace the plain Label title with a RichText title (icon in gold Mono + name in dark Serif)
    if let Some(node) = tree.get_mut(frame.title) {
        node.widget = Widget::RichText {
            spans: vec![
                TextSpan {
                    text: format!("{} ", info.icon),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: info.name.clone(),
                    color: theme.text_dark,
                    font_family: FontFamily::Serif,
                },
            ],
            font_size: theme.font_header_size,
        };
    }

    let content_w = frame.content_width;

    // Tab container with 4 tabs
    let tabs = tree.insert(
        frame.content,
        Widget::TabContainer {
            tabs: vec![
                "Overview".to_string(),
                "Family".to_string(),
                "Relations".to_string(),
                "Traits".to_string(),
            ],
            active: 0,
            tab_color: theme.tab_inactive_color,
            active_color: theme.tab_active_color,
            font_size: theme.font_body_size,
        },
    );
    tree.set_sizing(tabs, Sizing::Fixed(content_w), Sizing::Fit);

    // === Overview tab content (tab child 0) ===
    let overview_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(overview_col, Sizing::Fixed(content_w), Sizing::Fit);

    // Position label
    tree.insert(
        overview_col,
        Widget::Label {
            text: format!("Position: ({}, {})", info.position.0, info.position.1),
            color: theme.disabled,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );

    // Health bar
    if let Some((cur, max)) = info.health {
        let ratio = if max > 0.0 { cur / max } else { 0.0 };
        tree.insert(
            overview_col,
            Widget::Label {
                text: format!("Health: {:.0}/{:.0}", cur, max),
                color: severity_color(theme, ratio),
                font_size: theme.font_body_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        let bar = tree.insert(
            overview_col,
            Widget::ProgressBar {
                fraction: ratio,
                fg_color: theme.progress_bar_health_fg,
                bg_color: theme.progress_bar_health_bg,
                border_color: theme.panel_border_color,
                border_width: theme.progress_bar_border_width,
                height: theme.progress_bar_height,
            },
        );
        tree.set_sizing(bar, Sizing::Fixed(content_w - 20.0), Sizing::Fit);
    }

    // Hunger bar
    if let Some((cur, max)) = info.hunger {
        let ratio = if max > 0.0 { cur / max } else { 0.0 };
        tree.insert(
            overview_col,
            Widget::Label {
                text: format!("Hunger: {:.0}/{:.0}", cur, max),
                color: severity_color(theme, 1.0 - ratio), // invert: high hunger is bad
                font_size: theme.font_body_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        let bar = tree.insert(
            overview_col,
            Widget::ProgressBar {
                fraction: ratio,
                fg_color: theme.gold,
                bg_color: theme.progress_bar_health_bg,
                border_color: theme.panel_border_color,
                border_width: theme.progress_bar_border_width,
                height: theme.progress_bar_height,
            },
        );
        tree.set_sizing(bar, Sizing::Fixed(content_w - 20.0), Sizing::Fit);
    }

    // Fatigue
    if let Some(fat) = info.fatigue {
        tree.insert(
            overview_col,
            Widget::Label {
                text: format!("Fatigue: {:.0}", fat),
                color: theme.text_dark,
                font_size: theme.font_body_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
    }

    // Combat stats
    if let Some((atk, def, agg)) = info.combat {
        tree.insert(
            overview_col,
            Widget::Label {
                text: format!("ATK {:.0}  DEF {:.0}  AGG {:.1}", atk, def, agg),
                color: theme.text_dark,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
    }

    // Gait
    if let Some(ref gait) = info.gait {
        tree.insert(
            overview_col,
            Widget::Label {
                text: format!("Gait: {}", gait),
                color: theme.text_dark,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
    }

    // Current action
    if let Some(ref action) = info.action {
        tree.insert(
            overview_col,
            Widget::Label {
                text: format!("Action: {}", action),
                color: theme.text_dark,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
    }

    // === Family tab content (tab child 1) -- placeholder ===
    let family_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(family_col, Sizing::Fixed(content_w), Sizing::Fit);
    tree.insert(
        family_col,
        Widget::Label {
            text: format!("Entity #{}", info.entity_id),
            color: theme.disabled,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );
    tree.insert(
        family_col,
        Widget::Label {
            text: "No family data yet.".to_string(),
            color: theme.disabled,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: true,
        },
    );

    // === Relations tab content (tab child 2) -- placeholder ===
    let relations_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(relations_col, Sizing::Fixed(content_w), Sizing::Fit);
    tree.insert(
        relations_col,
        Widget::Label {
            text: "No relationships yet.".to_string(),
            color: theme.disabled,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: true,
        },
    );

    // === Traits tab content (tab child 3) -- placeholder ===
    let traits_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(traits_col, Sizing::Fixed(content_w), Sizing::Fit);
    tree.insert(
        traits_col,
        Widget::Label {
            text: "No traits data yet.".to_string(),
            color: theme.disabled,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: true,
        },
    );

    // close_btn is always Some here since closeable=true
    (frame.root, frame.close_btn.expect("closeable frame"))
}

/// Collect character panel info from the world.
pub fn collect_character_info(
    entity: crate::components::Entity,
    world: &crate::world::World,
) -> Option<CharacterPanelInfo> {
    if !world.alive.contains(&entity) {
        return None;
    }
    world.body.positions.get(&entity)?;

    let name = world
        .body
        .names
        .get(&entity)
        .map(|n| n.value.clone())
        .unwrap_or_else(|| format!("Entity #{}", entity.0));
    let icon = world.body.icons.get(&entity).map(|i| i.ch).unwrap_or('?');
    let pos = world
        .body
        .positions
        .get(&entity)
        .map(|p| (p.x, p.y))
        .unwrap_or((0, 0));
    let health = world.body.healths.get(&entity).map(|h| (h.current, h.max));
    let hunger = world.mind.hungers.get(&entity).map(|h| (h.current, h.max));
    let fatigue = world.body.fatigues.get(&entity).map(|f| f.current);
    let combat = world
        .body
        .combat_stats
        .get(&entity)
        .map(|c| (c.attack, c.defense, c.aggression));
    let gait = world
        .body
        .current_gaits
        .get(&entity)
        .map(|g| format!("{:?}", g));
    let action = world
        .mind
        .action_states
        .get(&entity)
        .and_then(|a| a.current_action.as_ref().map(|id| format!("{:?}", id)));

    Some(CharacterPanelInfo {
        entity_id: entity.0,
        name,
        icon,
        health,
        hunger,
        fatigue,
        combat,
        position: pos,
        gait,
        action,
    })
}

/// Pick color by severity ratio (current/max).
fn severity_color(theme: &Theme, ratio: f32) -> [f32; 4] {
    if ratio > 0.5 {
        theme.text_dark
    } else if ratio > 0.25 {
        theme.gold
    } else {
        theme.danger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_info() -> CharacterPanelInfo {
        CharacterPanelInfo {
            entity_id: 42,
            name: "Goblin Warrior".to_string(),
            icon: 'g',
            health: Some((80.0, 100.0)),
            hunger: Some((30.0, 100.0)),
            fatigue: Some(15.0),
            combat: Some((10.0, 5.0, 0.8)),
            position: (12, 34),
            gait: Some("Walk".to_string()),
            action: Some("Wander".to_string()),
        }
    }

    #[test]
    fn character_panel_has_4_tabs() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let (root, _close) = build_character_panel(&mut tree, &theme, &test_info());
        assert!(tree.get(root).is_some());

        // Navigate: root -> frame_col -> content -> TabContainer
        let panel_node = tree.get(root).unwrap();
        let frame_col = panel_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();
        let tab_id = content_node.children[0];
        let tab_node = tree.get(tab_id).unwrap();
        if let Widget::TabContainer { tabs, .. } = &tab_node.widget {
            assert_eq!(tabs.len(), 4);
            assert_eq!(tabs[0], "Overview");
            assert_eq!(tabs[1], "Family");
            assert_eq!(tabs[2], "Relations");
            assert_eq!(tabs[3], "Traits");
        } else {
            panic!("Expected TabContainer widget");
        }
    }

    #[test]
    fn overview_tab_has_health_progress_bar() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let (root, _close) = build_character_panel(&mut tree, &theme, &test_info());

        // Navigate: root -> frame_col -> content -> tabs -> overview_col
        let panel_node = tree.get(root).unwrap();
        let frame_col = panel_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();
        let tab_id = content_node.children[0];
        let tab_node = tree.get(tab_id).unwrap();

        // Tab child 0 = overview column
        let overview_id = tab_node.children[0];
        let overview_node = tree.get(overview_id).unwrap();

        // Find a ProgressBar in the overview children
        let has_progress = overview_node.children.iter().any(|&child_id| {
            if let Some(node) = tree.get(child_id) {
                matches!(&node.widget, Widget::ProgressBar { .. })
            } else {
                false
            }
        });
        assert!(
            has_progress,
            "Overview tab should contain a ProgressBar for health"
        );
    }

    #[test]
    fn close_button_exists() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let (_root, close) = build_character_panel(&mut tree, &theme, &test_info());
        let close_node = tree.get(close).unwrap();
        if let Widget::Button { text, .. } = &close_node.widget {
            assert_eq!(text, "X");
        } else {
            panic!("Expected Button widget for close");
        }
    }
}
