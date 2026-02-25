use super::WidgetId;
use super::draw::{FontFamily, TextSpan};
use super::geometry::{Edges, Position, Sizing};
use super::theme::Theme;
use super::tree::WidgetTree;
use super::widget::{self, Widget};

/// Data for the entity inspector panel (UI-I01d).
/// Extracted from World by `collect_inspector_info`, consumed by
/// `build_entity_inspector`. Plain struct — no references to World.
pub struct EntityInspectorInfo {
    pub name: String,
    pub icon: char,
    pub position: (i32, i32),
    pub health: Option<(f32, f32)>, // (current, max)
    pub hunger: Option<(f32, f32)>, // (current, max)
    pub fatigue: Option<f32>,
    pub combat: Option<(f32, f32, f32)>, // (atk, def, aggression)
    pub action: Option<String>,          // "Idle", "Wandering", etc.
    pub gait: Option<String>,            // "Walk", "Run", etc.
}

/// Collect inspector data for an entity. Returns `None` if the entity is
/// not alive or has no position (same pattern as `collect_event_entries`).
pub fn collect_inspector_info(
    entity: crate::components::Entity,
    world: &crate::world::World,
) -> Option<EntityInspectorInfo> {
    if !world.alive.contains(&entity) {
        return None;
    }
    let pos = world.body.positions.get(&entity)?;

    let name = world
        .body
        .names
        .get(&entity)
        .map(|n| n.value.clone())
        .unwrap_or_else(|| format!("E{}", entity.0));
    let icon = world.body.icons.get(&entity).map(|i| i.ch).unwrap_or('?');

    let health = world.body.healths.get(&entity).map(|h| (h.current, h.max));
    let hunger = world.mind.hungers.get(&entity).map(|h| (h.current, h.max));
    let fatigue = world.body.fatigues.get(&entity).map(|f| f.current);
    let combat = world
        .body
        .combat_stats
        .get(&entity)
        .map(|c| (c.attack, c.defense, c.aggression));

    let action = world
        .mind
        .action_states
        .get(&entity)
        .and_then(|a| a.current_action.as_ref().map(|id| format!("{:?}", id)));

    let gait = world
        .body
        .current_gaits
        .get(&entity)
        .map(|g| format!("{:?}", g));

    Some(EntityInspectorInfo {
        name,
        icon,
        position: (pos.x, pos.y),
        health,
        hunger,
        fatigue,
        combat,
        action,
        gait,
    })
}

/// Inspector panel width in pixels.
const INSPECTOR_WIDTH: f32 = 220.0;

/// Build the entity inspector panel (UI-I01d).
///
/// Right-aligned panel showing entity stats. Returns `(panel_id, close_button_id)`.
/// The caller positions the panel and uses `close_button_id` to detect close clicks.
pub fn build_entity_inspector(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &EntityInspectorInfo,
) -> (WidgetId, WidgetId) {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.tooltip_bg_color,
        border_color: theme.tooltip_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    let inspector_w = theme.s(INSPECTOR_WIDTH);
    tree.set_sizing(panel, Sizing::Fixed(inspector_w), Sizing::Fit);
    tree.set_padding(panel, Edges::all(theme.panel_padding));

    let content_w = inspector_w - theme.panel_padding * 2.0;
    let mut y = 0.0_f32;
    let data_h = theme.font_data_size;
    let body_h = theme.font_body_size;
    let header_h = theme.font_header_size;
    let gap = theme.label_gap;

    // Header row: [icon+name, Expand, close button] (UI-601).
    let header_row = tree.insert(
        panel,
        Widget::Row {
            gap: theme.label_gap,
            align: widget::CrossAlign::Center,
        },
    );
    tree.set_sizing(header_row, Sizing::Fixed(content_w), Sizing::Fit);
    tree.set_position(header_row, Position::Fixed { x: 0.0, y: 0.0 });

    tree.insert(
        header_row,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: format!("{} ", info.icon),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: info.name.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
            ],
            font_size: theme.font_header_size,
        },
    );
    tree.insert(header_row, Widget::Expand);
    let close_btn = tree.insert(
        header_row,
        Widget::Button {
            text: "X".to_string(),
            color: theme.danger,
            bg_color: [0.0, 0.0, 0.0, 0.0], // transparent
            border_color: theme.danger,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
        },
    );
    y += header_h + gap;

    // Position: "(x, y)" in disabled/mono at data size
    let pos_label = tree.insert(
        panel,
        Widget::Label {
            text: format!("({}, {})", info.position.0, info.position.1),
            color: theme.disabled,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );
    tree.set_position(pos_label, Position::Fixed { x: 0.0, y });
    y += data_h + gap;

    // Separator gap
    y += gap;

    // Helper: pick color by severity ratio (current/max).
    let severity_color = |ratio: f32| -> [f32; 4] {
        if ratio > 0.5 {
            theme.text_light
        } else if ratio > 0.25 {
            theme.gold
        } else {
            theme.danger
        }
    };

    // Health
    if let Some((cur, max)) = info.health {
        let ratio = if max > 0.0 { cur / max } else { 0.0 };
        let hp = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "HP ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}/{:.0}", cur, max),
                        color: severity_color(ratio),
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(hp, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Hunger
    if let Some((cur, max)) = info.hunger {
        let ratio = if max > 0.0 { cur / max } else { 0.0 };
        let hunger = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "Hunger ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}/{:.0}", cur, max),
                        color: severity_color(ratio),
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(hunger, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Fatigue
    if let Some(fat) = info.fatigue {
        let fat_label = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "Fatigue ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.1}", fat),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(fat_label, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Combat stats
    if let Some((atk, def, agg)) = info.combat {
        let combat = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "ATK ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}", atk),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: "  DEF ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.0}", def),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: "  AGG ".to_string(),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: format!("{:.1}", agg),
                        color: theme.gold,
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_data_size,
            },
        );
        tree.set_position(combat, Position::Fixed { x: 0.0, y });
        y += data_h + gap;
    }

    // Action
    if let Some(ref action) = info.action {
        let act = tree.insert(
            panel,
            Widget::RichText {
                spans: vec![
                    TextSpan {
                        text: "Action ".to_string(),
                        color: theme.gold,
                        font_family: FontFamily::Mono,
                    },
                    TextSpan {
                        text: action.clone(),
                        color: theme.text_light,
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(act, Position::Fixed { x: 0.0, y });
        y += body_h + gap;
    }

    // Gait
    if let Some(ref gait) = info.gait {
        let gait_label = tree.insert(
            panel,
            Widget::Label {
                text: gait.clone(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        tree.set_position(gait_label, Position::Fixed { x: 0.0, y });
        // y += data_h + gap; // last line, no trailing gap needed
    }

    (panel, close_btn)
}

#[cfg(test)]
mod tests {
    use super::super::draw::HeuristicMeasurer;
    use super::super::geometry::Size;
    use super::*;

    fn spawn_full_entity(world: &mut crate::world::World) -> crate::components::Entity {
        let e = world.spawn();
        world
            .body
            .positions
            .insert(e, crate::components::Position { x: 10, y: 20 });
        world.body.names.insert(
            e,
            crate::components::Name {
                value: "Goblin".into(),
            },
        );
        world
            .body
            .icons
            .insert(e, crate::components::Icon { ch: 'g' });
        world.body.healths.insert(
            e,
            crate::components::Health {
                current: 80.0,
                max: 100.0,
            },
        );
        world.mind.hungers.insert(
            e,
            crate::components::Hunger {
                current: 30.0,
                max: 100.0,
            },
        );
        world
            .body
            .fatigues
            .insert(e, crate::components::Fatigue { current: 5.0 });
        world.body.combat_stats.insert(
            e,
            crate::components::CombatStats {
                attack: 12.0,
                defense: 8.0,
                aggression: 0.7,
            },
        );
        world
            .body
            .current_gaits
            .insert(e, crate::components::Gait::Walk);
        e
    }

    #[test]
    fn collect_inspector_info_alive() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = spawn_full_entity(&mut world);

        let info = collect_inspector_info(e, &world).expect("alive entity should return Some");
        assert_eq!(info.name, "Goblin");
        assert_eq!(info.icon, 'g');
        assert_eq!(info.position, (10, 20));
        assert_eq!(info.health, Some((80.0, 100.0)));
        assert_eq!(info.hunger, Some((30.0, 100.0)));
        assert!((info.fatigue.unwrap() - 5.0).abs() < 0.01);
        let (atk, def, agg) = info.combat.unwrap();
        assert!((atk - 12.0).abs() < 0.01);
        assert!((def - 8.0).abs() < 0.01);
        assert!((agg - 0.7).abs() < 0.01);
        assert_eq!(info.gait.as_deref(), Some("Walk"));
    }

    #[test]
    fn collect_inspector_info_dead() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = spawn_full_entity(&mut world);
        world.alive.remove(&e);

        assert!(collect_inspector_info(e, &world).is_none());
    }

    #[test]
    fn collect_inspector_info_no_position() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = world.spawn();
        world.body.names.insert(
            e,
            crate::components::Name {
                value: "Ghost".into(),
            },
        );
        // No position inserted.

        assert!(collect_inspector_info(e, &world).is_none());
    }

    #[test]
    fn collect_inspector_info_minimal() {
        let mut world = crate::world::World::new_with_seed(42);
        let e = world.spawn();
        world
            .body
            .positions
            .insert(e, crate::components::Position { x: 5, y: 5 });
        // Only position, no other components.

        let info = collect_inspector_info(e, &world).expect("alive with position");
        assert_eq!(info.position, (5, 5));
        // Name falls back to "E{id}".
        assert!(info.name.starts_with('E'));
        assert_eq!(info.icon, '?');
        assert!(info.health.is_none());
        assert!(info.hunger.is_none());
        assert!(info.fatigue.is_none());
        assert!(info.combat.is_none());
        assert!(info.action.is_none());
        assert!(info.gait.is_none());
    }

    #[test]
    fn build_inspector_creates_panel() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = EntityInspectorInfo {
            name: "Goblin".into(),
            icon: 'g',
            position: (10, 20),
            health: Some((80.0, 100.0)),
            hunger: Some((30.0, 100.0)),
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id, _close_id) = build_entity_inspector(&mut tree, &theme, &info);

        let node = tree.get(panel_id).expect("panel exists");
        assert!(matches!(node.widget, Widget::Panel { .. }));
    }

    #[test]
    fn build_inspector_has_close_button() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = EntityInspectorInfo {
            name: "Wolf".into(),
            icon: 'w',
            position: (0, 0),
            health: None,
            hunger: None,
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id, close_id) = build_entity_inspector(&mut tree, &theme, &info);

        // Close button is inside the header row (grandchild of panel).
        let panel_node = tree.get(panel_id).expect("panel");
        let header_row_id = panel_node.children[0];
        let header_node = tree.get(header_row_id).expect("header row");
        assert!(header_node.children.contains(&close_id));

        let close_node = tree.get(close_id).expect("close button");
        assert!(matches!(close_node.widget, Widget::Button { .. }));
    }

    #[test]
    fn build_inspector_health_colors() {
        let theme = Theme::default();

        // Low HP (<=25%) -> danger color
        let mut tree = WidgetTree::new();
        let info_low = EntityInspectorInfo {
            name: "Dying".into(),
            icon: 'd',
            position: (0, 0),
            health: Some((10.0, 100.0)), // 10% = danger
            hunger: None,
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id, _) = build_entity_inspector(&mut tree, &theme, &info_low);

        // Find the HP RichText child (has "HP " span).
        let panel_node = tree.get(panel_id).expect("panel");
        let mut found_danger = false;
        for &child_id in &panel_node.children {
            if let Some(child) = tree.get(child_id) {
                if let Widget::RichText { spans, .. } = &child.widget {
                    if spans.len() >= 2 && spans[0].text == "HP " {
                        assert_eq!(spans[1].color, theme.danger);
                        found_danger = true;
                    }
                }
            }
        }
        assert!(found_danger, "should find HP span with danger color");

        // High HP (>50%) -> text_light color
        let mut tree2 = WidgetTree::new();
        let info_high = EntityInspectorInfo {
            name: "Healthy".into(),
            icon: 'h',
            position: (0, 0),
            health: Some((90.0, 100.0)), // 90% = text_light
            hunger: None,
            fatigue: None,
            combat: None,
            action: None,
            gait: None,
        };
        let (panel_id2, _) = build_entity_inspector(&mut tree2, &theme, &info_high);
        let panel_node2 = tree2.get(panel_id2).expect("panel");
        let mut found_light = false;
        for &child_id in &panel_node2.children {
            if let Some(child) = tree2.get(child_id) {
                if let Widget::RichText { spans, .. } = &child.widget {
                    if spans.len() >= 2 && spans[0].text == "HP " {
                        assert_eq!(spans[1].color, theme.text_light);
                        found_light = true;
                    }
                }
            }
        }
        assert!(found_light, "should find HP span with text_light color");
    }

    #[test]
    fn build_inspector_sizing() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = EntityInspectorInfo {
            name: "Goblin".into(),
            icon: 'g',
            position: (10, 20),
            health: Some((80.0, 100.0)),
            hunger: Some((30.0, 100.0)),
            fatigue: Some(5.0),
            combat: Some((12.0, 8.0, 0.7)),
            action: Some("Idle".into()),
            gait: Some("Walk".into()),
        };
        let (panel_id, _) = build_entity_inspector(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(panel_id).expect("rect after layout");
        // Width is fixed at INSPECTOR_WIDTH (220px).
        assert!((rect.width - 220.0).abs() < 0.01);
        // Height is Fit -- should be > 0 (content exists).
        assert!(rect.height > 0.0);
    }
}
