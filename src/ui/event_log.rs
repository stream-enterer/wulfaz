use super::WidgetId;
use super::draw::{FontFamily, TextSpan};
use super::geometry::{Edges, Sizing};
use super::theme::Theme;
use super::tree::WidgetTree;
use super::widget::Widget;

/// Event data for the event log panel (UI-I01c).
/// Extracted from World in main.rs, consumed by `build_event_log`.
pub enum EventLogEntry {
    Spawned {
        name: String,
    },
    Died {
        name: String,
    },
    Ate {
        name: String,
        food_name: String,
    },
    Attacked {
        attacker: String,
        defender: String,
        damage: f32,
    },
}

/// Maximum significant events kept in the ScrollList.
const EVENT_LOG_MAX_ENTRIES: usize = 50;

/// Build the event log panel at the bottom of the screen (UI-I01c).
///
/// Chrome panel: permanent, rebuilt every frame with live event data.
/// Replaces the old string-based `render::render_recent_events()`.
///
/// Returns the root ScrollList's `WidgetId`. The caller should set its
/// position after computing available screen space, then call
/// `WidgetTree::layout` to finalize.
pub fn build_event_log(
    tree: &mut WidgetTree,
    theme: &Theme,
    entries: &[EventLogEntry],
    screen_width: f32,
    panel_height: f32,
) -> WidgetId {
    let pad_v = theme.status_bar_padding_v;
    let pad_h = theme.status_bar_padding_h;
    let viewport_h = (panel_height - pad_v * 2.0).max(0.0);
    let total_h = entries.len() as f32 * theme.scroll_item_height;
    let auto_scroll = (total_h - viewport_h).max(0.0);

    let list = tree.insert_root(Widget::ScrollList {
        bg_color: theme.status_bar_bg,
        border_color: theme.panel_border_color,
        border_width: theme.tooltip_border_width,
        item_height: theme.scroll_item_height,
        scroll_offset: auto_scroll,
        scrollbar_color: theme.scrollbar_color,
        scrollbar_width: theme.scrollbar_width,
        item_heights: Vec::new(),
        empty_text: None,
    });
    tree.set_sizing(
        list,
        Sizing::Fixed(screen_width),
        Sizing::Fixed(panel_height),
    );
    tree.set_padding(
        list,
        Edges {
            top: pad_v,
            right: pad_h,
            bottom: pad_v,
            left: pad_h,
        },
    );

    for entry in entries {
        let spans = match entry {
            EventLogEntry::Spawned { name } => vec![
                TextSpan {
                    text: name.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " spawned".to_string(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                },
            ],
            EventLogEntry::Died { name } => vec![
                TextSpan {
                    text: name.clone(),
                    color: theme.danger,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " died".to_string(),
                    color: theme.danger,
                    font_family: FontFamily::Mono,
                },
            ],
            EventLogEntry::Ate { name, food_name } => vec![
                TextSpan {
                    text: name.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " ate ".to_string(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: food_name.clone(),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
            ],
            EventLogEntry::Attacked {
                attacker,
                defender,
                damage,
            } => vec![
                TextSpan {
                    text: attacker.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " attacks ".to_string(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: defender.clone(),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: format!(" ({:.0} dmg)", damage),
                    color: theme.danger,
                    font_family: FontFamily::Mono,
                },
            ],
        };
        tree.insert(
            list,
            Widget::RichText {
                spans,
                font_size: theme.font_data_size,
            },
        );
    }

    list
}

/// Collect significant events from World into `EventLogEntry` structs.
///
/// Filters to Spawned/Died/Ate/Attacked (skips Moved, HungerChanged).
/// Returns up to `EVENT_LOG_MAX_ENTRIES` entries, newest last.
pub fn collect_event_entries(
    events: &crate::events::EventLog,
    names: &std::collections::HashMap<crate::components::Entity, crate::components::Name>,
) -> Vec<EventLogEntry> {
    use crate::events::Event;

    let resolve = |e: &crate::components::Entity| -> String {
        names
            .get(e)
            .map(|n| n.value.clone())
            .unwrap_or_else(|| format!("E{}", e.0))
    };

    let raw = events.recent(EVENT_LOG_MAX_ENTRIES * 10);
    let mut entries = Vec::new();

    for event in raw {
        let entry = match event {
            Event::Spawned { entity, .. } => EventLogEntry::Spawned {
                name: resolve(entity),
            },
            Event::Died { entity, .. } => EventLogEntry::Died {
                name: resolve(entity),
            },
            Event::Ate { entity, food, .. } => EventLogEntry::Ate {
                name: resolve(entity),
                food_name: resolve(food),
            },
            Event::Attacked {
                attacker,
                defender,
                damage,
                ..
            } => EventLogEntry::Attacked {
                attacker: resolve(attacker),
                defender: resolve(defender),
                damage: *damage,
            },
            Event::Moved { .. } | Event::HungerChanged { .. } => continue,
        };
        entries.push(entry);
        if entries.len() >= EVENT_LOG_MAX_ENTRIES {
            break;
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::draw::{DrawList, HeuristicMeasurer};
    use crate::ui::geometry::{Position, Size};
    use crate::ui::theme::Theme;

    fn screen() -> Size {
        Size {
            width: 800.0,
            height: 600.0,
        }
    }
    use crate::ui::tree::WidgetTree;
    use crate::ui::widget::Widget;

    #[test]
    fn event_log_empty() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let log = build_event_log(&mut tree, &theme, &[], 800.0, 108.0);

        assert_eq!(tree.roots().len(), 1);
        assert_eq!(tree.roots()[0], log);

        let node = tree.get(log).expect("log exists");
        assert!(node.children.is_empty());
        if let Widget::ScrollList { scroll_offset, .. } = &node.widget {
            assert!(scroll_offset.abs() < 0.01, "empty log should have 0 scroll");
        } else {
            panic!("event log root should be a ScrollList");
        }
    }

    #[test]
    fn event_log_spawned_entry() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Spawned {
            name: "Goblin".into(),
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let node = tree.get(log).expect("log");
        assert_eq!(node.children.len(), 1);

        let child = tree.get(node.children[0]).expect("child");
        if let Widget::RichText { spans, font_size } = &child.widget {
            assert!((font_size - theme.font_data_size).abs() < 0.01);
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0].text, "Goblin");
            assert_eq!(spans[0].color, theme.text_light);
            assert_eq!(spans[1].text, " spawned");
            assert_eq!(spans[1].color, theme.disabled);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_died_danger_color() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Died {
            name: "Wolf".into(),
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let child_id = tree.get(log).expect("log").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans[0].text, "Wolf");
            assert_eq!(spans[0].color, theme.danger);
            assert_eq!(spans[1].text, " died");
            assert_eq!(spans[1].color, theme.danger);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_ate_food_gold() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Ate {
            name: "Goblin".into(),
            food_name: "Bread".into(),
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let child_id = tree.get(log).expect("log").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans.len(), 3);
            assert_eq!(spans[0].text, "Goblin");
            assert_eq!(spans[0].color, theme.text_light);
            assert_eq!(spans[1].text, " ate ");
            assert_eq!(spans[2].text, "Bread");
            assert_eq!(spans[2].color, theme.gold);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_attacked_damage() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![EventLogEntry::Attacked {
            attacker: "Goblin".into(),
            defender: "Troll".into(),
            damage: 12.5,
        }];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let child_id = tree.get(log).expect("log").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans.len(), 4);
            assert_eq!(spans[0].text, "Goblin");
            assert_eq!(spans[1].text, " attacks ");
            assert_eq!(spans[2].text, "Troll");
            assert_eq!(spans[3].text, " (12 dmg)");
            assert_eq!(spans[3].color, theme.danger);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn event_log_auto_scrolls_to_bottom() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        // 20 entries at 20px each = 400px total; viewport ~100px -> should auto-scroll
        let entries: Vec<EventLogEntry> = (0..20)
            .map(|i| EventLogEntry::Spawned {
                name: format!("Entity{}", i),
            })
            .collect();
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);

        let node = tree.get(log).expect("log");
        if let Widget::ScrollList { scroll_offset, .. } = &node.widget {
            // total = 20*20 = 400, viewport = 108 - 8 = 100, max = 300
            let expected = (20.0 * theme.scroll_item_height
                - (108.0 - theme.status_bar_padding_v * 2.0))
                .max(0.0);
            assert!(
                (*scroll_offset - expected).abs() < 0.01,
                "scroll_offset={}, expected={}",
                scroll_offset,
                expected
            );
        } else {
            panic!("expected ScrollList");
        }
    }

    #[test]
    fn event_log_draw_output() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let entries = vec![
            EventLogEntry::Spawned {
                name: "Goblin".into(),
            },
            EventLogEntry::Died {
                name: "Wolf".into(),
            },
            EventLogEntry::Ate {
                name: "Elf".into(),
                food_name: "Apple".into(),
            },
        ];
        let log = build_event_log(&mut tree, &theme, &entries, 800.0, 108.0);
        tree.set_position(log, Position::Fixed { x: 0.0, y: 492.0 });

        tree.layout(screen(), &mut HeuristicMeasurer);
        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // 2 panels: ScrollList background + 1 alternating row tint (item index 1).
        assert_eq!(dl.panels.len(), 2);
        assert_eq!(dl.panels[0].bg_color, theme.status_bar_bg);
        assert!((dl.panels[0].width - 800.0).abs() < 0.01);

        // 3 rich text commands (one per event).
        assert_eq!(dl.rich_texts.len(), 3);
        assert_eq!(dl.rich_texts[0].spans[0].text, "Goblin");
        assert_eq!(dl.rich_texts[1].spans[0].text, "Wolf");
        assert_eq!(dl.rich_texts[2].spans[0].text, "Elf");
    }

    #[test]
    fn event_log_full_width_fixed_height() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let log = build_event_log(&mut tree, &theme, &[], 1024.0, 120.0);
        tree.set_position(log, Position::Fixed { x: 0.0, y: 500.0 });

        tree.layout(
            Size {
                width: 1024.0,
                height: 768.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(log).expect("rect");
        assert!((rect.width - 1024.0).abs() < 0.01);
        assert!((rect.height - 120.0).abs() < 0.01);
        assert!((rect.y - 500.0).abs() < 0.01);
    }
}
