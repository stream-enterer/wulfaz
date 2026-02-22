//! UI-DEMO: Widget showcase panel.
//!
//! A persistent developer reference panel (toggled with F11 or `--ui-demo`)
//! that renders every available widget and style. Verifies all 5 UI tiers
//! work together: typography, rich text, buttons, scroll lists, tooltips,
//! animations, keybinding labels, and live entity data.

use super::draw::{FontFamily, TextSpan};
use super::keybindings::{Action, KeyBindings};
use super::theme::Theme;
use super::widget::{TooltipContent, Widget};
use super::{Edges, EntityInspectorInfo, Position, Size, Sizing, WidgetId, WidgetTree};

/// Live simulation data for the demo's Tier 4 section.
pub struct DemoLiveData<'a> {
    pub entity_info: Option<&'a EntityInspectorInfo>,
    pub tick: u64,
    pub population: usize,
}

/// Build the demo widget showcase into an existing tree.
///
/// Returns the root panel `WidgetId` so the caller can apply slide-in
/// animation. The demo occupies a 400px-wide panel on the left side.
pub fn build_demo(
    tree: &mut WidgetTree,
    theme: &Theme,
    keybindings: &KeyBindings,
    live: &DemoLiveData,
    screen: Size,
) -> WidgetId {
    let panel_w = 400.0_f32;
    let panel_h = screen.height - 8.0; // 4px margin top+bottom

    // Root panel — parchment background with gold border.
    let root = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    tree.set_position(root, Position::Fixed { x: 4.0, y: 4.0 });
    tree.set_sizing(root, Sizing::Fixed(panel_w), Sizing::Fixed(panel_h));
    tree.set_padding(root, Edges::all(theme.panel_padding));

    let content_w = panel_w - theme.panel_padding * 2.0;
    let mut y = 0.0_f32;

    // -----------------------------------------------------------------------
    // Title
    // -----------------------------------------------------------------------
    let title = tree.insert(
        root,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: "Widget Showcase".into(),
                    color: theme.gold,
                    font_family: FontFamily::Serif,
                },
                TextSpan {
                    text: "  (F11)".into(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                },
            ],
            font_size: theme.font_header_size,
        },
    );
    tree.set_position(title, Position::Fixed { x: 0.0, y });
    y += theme.font_header_size + theme.label_gap * 2.0;

    // -----------------------------------------------------------------------
    // Tier 1 — Typography: header, body, warning, data
    // -----------------------------------------------------------------------
    let sec_label = tree.insert(
        root,
        Widget::Label {
            text: "Typography".into(),
            color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(sec_label, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    // Separator line using a thin panel
    let sep = tree.insert(
        root,
        Widget::Panel {
            bg_color: theme.gold,
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        },
    );
    tree.set_position(sep, Position::Fixed { x: 0.0, y });
    tree.set_sizing(sep, Sizing::Fixed(content_w), Sizing::Fixed(1.0));
    y += 1.0 + theme.label_gap;

    let header_sample = tree.insert(
        root,
        Widget::Label {
            text: "Serif Header 16pt".into(),
            color: theme.text_light,
            font_size: theme.font_header_size,
            font_family: theme.font_header_family,
        },
    );
    tree.set_position(header_sample, Position::Fixed { x: 0.0, y });
    y += theme.font_header_size + theme.label_gap;

    let body_sample = tree.insert(
        root,
        Widget::Label {
            text: "Serif Body 12pt".into(),
            color: theme.text_light,
            font_size: theme.font_body_size,
            font_family: theme.font_body_family,
        },
    );
    tree.set_position(body_sample, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    let data_sample = tree.insert(
        root,
        Widget::Label {
            text: "Mono Data 9pt".into(),
            color: theme.text_light,
            font_size: theme.font_data_size,
            font_family: theme.font_data_family,
        },
    );
    tree.set_position(data_sample, Position::Fixed { x: 0.0, y });
    y += theme.font_data_size + theme.label_gap;

    let warning_sample = tree.insert(
        root,
        Widget::Label {
            text: "Danger Red".into(),
            color: theme.danger,
            font_size: theme.font_data_size,
            font_family: theme.font_data_family,
        },
    );
    tree.set_position(warning_sample, Position::Fixed { x: 0.0, y });
    y += theme.font_data_size + theme.label_gap;

    let disabled_sample = tree.insert(
        root,
        Widget::Label {
            text: "Disabled Grey".into(),
            color: theme.disabled,
            font_size: theme.font_data_size,
            font_family: theme.font_data_family,
        },
    );
    tree.set_position(disabled_sample, Position::Fixed { x: 0.0, y });
    y += theme.font_data_size + theme.label_gap * 2.0;

    // -----------------------------------------------------------------------
    // Tier 3 — Rich text: mixed fonts, colors, families in one block
    // -----------------------------------------------------------------------
    let rich_label = tree.insert(
        root,
        Widget::Label {
            text: "Rich Text".into(),
            color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(rich_label, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    let sep2 = tree.insert(
        root,
        Widget::Panel {
            bg_color: theme.gold,
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        },
    );
    tree.set_position(sep2, Position::Fixed { x: 0.0, y });
    tree.set_sizing(sep2, Sizing::Fixed(content_w), Sizing::Fixed(1.0));
    y += 1.0 + theme.label_gap;

    let rich = tree.insert(
        root,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: "Population: ".into(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
                TextSpan {
                    text: "1,034,196".into(),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: " souls".into(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
            ],
            font_size: theme.font_body_size,
        },
    );
    tree.set_position(rich, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap * 2.0;

    // -----------------------------------------------------------------------
    // Tier 5 — Buttons with keybinding labels (UI-I03 verification)
    // -----------------------------------------------------------------------
    let btn_label = tree.insert(
        root,
        Widget::Label {
            text: "Buttons + Keybindings".into(),
            color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(btn_label, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    let sep3 = tree.insert(
        root,
        Widget::Panel {
            bg_color: theme.gold,
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        },
    );
    tree.set_position(sep3, Position::Fixed { x: 0.0, y });
    tree.set_sizing(sep3, Sizing::Fixed(content_w), Sizing::Fixed(1.0));
    y += 1.0 + theme.label_gap;

    // Pause button with keybinding label.
    let pause_label = keybindings
        .label_for(Action::PauseSim)
        .unwrap_or_else(|| "?".into());
    let pause_btn = tree.insert(
        root,
        Widget::Button {
            text: format!("Pause ({})", pause_label),
            color: theme.text_light,
            bg_color: [0.0, 0.0, 0.0, 0.0], // transparent, hover highlight via animation
            border_color: theme.panel_border_color,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(pause_btn, Position::Fixed { x: 0.0, y });
    tree.set_tooltip(
        pause_btn,
        Some(TooltipContent::Text("Toggle simulation pause".into())),
    );
    y += theme.font_body_size + theme.button_pad_v * 2.0 + theme.label_gap;

    // Speed buttons.
    let mut btn_x = 0.0_f32;
    for speed in 1..=5 {
        let speed_label = keybindings
            .label_for(Action::SpeedSet(speed))
            .unwrap_or_else(|| format!("{}", speed));
        let btn = tree.insert(
            root,
            Widget::Button {
                text: format!("{}x ({})", speed, speed_label),
                color: theme.text_light,
                bg_color: [0.0, 0.0, 0.0, 0.0],
                border_color: theme.panel_border_color,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
            },
        );
        tree.set_position(btn, Position::Fixed { x: btn_x, y });
        btn_x += 65.0;
    }
    y += theme.font_data_size + theme.button_pad_v * 2.0 + theme.label_gap;

    // Close button (Esc) — demonstrates hover highlight animation.
    let close_label = keybindings
        .label_for(Action::CloseTopmost)
        .unwrap_or_else(|| "?".into());
    let close_btn = tree.insert(
        root,
        Widget::Button {
            text: format!("Close ({})", close_label),
            color: theme.danger,
            bg_color: [0.0, 0.0, 0.0, 0.0],
            border_color: theme.danger,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(close_btn, Position::Fixed { x: 0.0, y });
    tree.set_tooltip(
        close_btn,
        Some(TooltipContent::Text("Close topmost overlay".into())),
    );
    y += theme.font_body_size + theme.button_pad_v * 2.0 + theme.label_gap * 2.0;

    // -----------------------------------------------------------------------
    // Tier 3 — ScrollList with virtual scrolling
    // -----------------------------------------------------------------------
    let scroll_label = tree.insert(
        root,
        Widget::Label {
            text: "Scroll List".into(),
            color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(scroll_label, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    let sep4 = tree.insert(
        root,
        Widget::Panel {
            bg_color: theme.gold,
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        },
    );
    tree.set_position(sep4, Position::Fixed { x: 0.0, y });
    tree.set_sizing(sep4, Sizing::Fixed(content_w), Sizing::Fixed(1.0));
    y += 1.0 + theme.label_gap;

    let scroll_h = 100.0_f32;
    let scroll_list = tree.insert(
        root,
        Widget::ScrollList {
            bg_color: [
                theme.bg_parchment[0] * 0.9,
                theme.bg_parchment[1] * 0.9,
                theme.bg_parchment[2] * 0.9,
                theme.bg_parchment[3],
            ],
            border_color: theme.panel_border_color,
            border_width: 1.0,
            item_height: theme.scroll_item_height,
            scroll_offset: 0.0,
            scrollbar_color: theme.scrollbar_color,
            scrollbar_width: theme.scrollbar_width,
        },
    );
    tree.set_position(scroll_list, Position::Fixed { x: 0.0, y });
    tree.set_sizing(
        scroll_list,
        Sizing::Fixed(content_w),
        Sizing::Fixed(scroll_h),
    );
    tree.set_padding(scroll_list, Edges::all(4.0));

    for i in 0..50 {
        tree.insert(
            scroll_list,
            Widget::Label {
                text: format!("Item {}", i + 1),
                color: theme.text_dark,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
            },
        );
    }
    y += scroll_h + theme.label_gap * 2.0;

    // -----------------------------------------------------------------------
    // Tier 4 — Live entity data (data binding verification)
    // -----------------------------------------------------------------------
    let live_label = tree.insert(
        root,
        Widget::Label {
            text: "Live Data".into(),
            color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(live_label, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    let sep5 = tree.insert(
        root,
        Widget::Panel {
            bg_color: theme.gold,
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        },
    );
    tree.set_position(sep5, Position::Fixed { x: 0.0, y });
    tree.set_sizing(sep5, Sizing::Fixed(content_w), Sizing::Fixed(1.0));
    y += 1.0 + theme.label_gap;

    // Tick + population.
    let tick_rich = tree.insert(
        root,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: "Tick: ".into(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
                TextSpan {
                    text: format!("{}", live.tick),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: "  Pop: ".into(),
                    color: theme.text_light,
                    font_family: FontFamily::Serif,
                },
                TextSpan {
                    text: format!("{}", live.population),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
            ],
            font_size: theme.font_body_size,
        },
    );
    tree.set_position(tick_rich, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    // First alive entity details.
    if let Some(info) = live.entity_info {
        // Name + icon.
        let name_rich = tree.insert(
            root,
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
                    TextSpan {
                        text: format!("  ({}, {})", info.position.0, info.position.1),
                        color: theme.disabled,
                        font_family: FontFamily::Mono,
                    },
                ],
                font_size: theme.font_body_size,
            },
        );
        tree.set_position(name_rich, Position::Fixed { x: 0.0, y });
        y += theme.font_body_size + theme.label_gap;

        // Health + hunger.
        let mut stat_spans = Vec::new();
        if let Some((cur, max)) = info.health {
            let ratio = if max > 0.0 { cur / max } else { 0.0 };
            let color = if ratio > 0.5 {
                theme.text_light
            } else if ratio > 0.25 {
                theme.gold
            } else {
                theme.danger
            };
            stat_spans.push(TextSpan {
                text: "HP ".into(),
                color: theme.disabled,
                font_family: FontFamily::Mono,
            });
            stat_spans.push(TextSpan {
                text: format!("{:.0}/{:.0}", cur, max),
                color,
                font_family: FontFamily::Mono,
            });
        }
        if let Some((cur, max)) = info.hunger {
            if !stat_spans.is_empty() {
                stat_spans.push(TextSpan {
                    text: "  ".into(),
                    color: theme.disabled,
                    font_family: FontFamily::Mono,
                });
            }
            let ratio = if max > 0.0 { cur / max } else { 0.0 };
            let color = if ratio > 0.5 {
                theme.text_light
            } else if ratio > 0.25 {
                theme.gold
            } else {
                theme.danger
            };
            stat_spans.push(TextSpan {
                text: "Hunger ".into(),
                color: theme.disabled,
                font_family: FontFamily::Mono,
            });
            stat_spans.push(TextSpan {
                text: format!("{:.0}/{:.0}", cur, max),
                color,
                font_family: FontFamily::Mono,
            });
        }
        if !stat_spans.is_empty() {
            let stats = tree.insert(
                root,
                Widget::RichText {
                    spans: stat_spans,
                    font_size: theme.font_data_size,
                },
            );
            tree.set_position(stats, Position::Fixed { x: 0.0, y });
            y += theme.font_data_size + theme.label_gap;
        }

        // Action + gait.
        if let Some(ref action) = info.action {
            let action_rich = tree.insert(
                root,
                Widget::RichText {
                    spans: vec![
                        TextSpan {
                            text: "Action: ".into(),
                            color: theme.disabled,
                            font_family: FontFamily::Mono,
                        },
                        TextSpan {
                            text: action.clone(),
                            color: theme.text_light,
                            font_family: FontFamily::Mono,
                        },
                    ],
                    font_size: theme.font_data_size,
                },
            );
            tree.set_position(action_rich, Position::Fixed { x: 0.0, y });
            y += theme.font_data_size + theme.label_gap;
        }
    } else {
        let no_entity = tree.insert(
            root,
            Widget::Label {
                text: "No entities alive".into(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
            },
        );
        tree.set_position(no_entity, Position::Fixed { x: 0.0, y });
        y += theme.font_data_size + theme.label_gap;
    }
    y += theme.label_gap;

    // -----------------------------------------------------------------------
    // Tier 3 — Tooltip chain (3-level nested tooltips)
    // -----------------------------------------------------------------------
    let tooltip_label = tree.insert(
        root,
        Widget::Label {
            text: "Tooltips".into(),
            color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(tooltip_label, Position::Fixed { x: 0.0, y });
    y += theme.font_body_size + theme.label_gap;

    let sep6 = tree.insert(
        root,
        Widget::Panel {
            bg_color: theme.gold,
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        },
    );
    tree.set_position(sep6, Position::Fixed { x: 0.0, y });
    tree.set_sizing(sep6, Sizing::Fixed(content_w), Sizing::Fixed(1.0));
    y += 1.0 + theme.label_gap;

    let tooltip_btn = tree.insert(
        root,
        Widget::Button {
            text: "Hover for nested tooltips".into(),
            color: theme.text_light,
            bg_color: [0.0, 0.0, 0.0, 0.0],
            border_color: theme.panel_border_color,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_position(tooltip_btn, Position::Fixed { x: 0.0, y });

    // Level 3 (deepest).
    let level3 = TooltipContent::Text("Level 3: deepest tooltip".into());
    // Level 2.
    let level2 = TooltipContent::Custom(vec![
        (
            Widget::Label {
                text: "Level 2 tooltip".into(),
                color: theme.text_light,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
            },
            None,
        ),
        (
            Widget::Label {
                text: "[hover for level 3]".into(),
                color: theme.gold,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
            },
            Some(level3),
        ),
    ]);
    // Level 1.
    let level1 = TooltipContent::Custom(vec![
        (
            Widget::Label {
                text: "Level 1 tooltip".into(),
                color: theme.text_light,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
            },
            None,
        ),
        (
            Widget::Label {
                text: "[hover for level 2]".into(),
                color: theme.gold,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
            },
            Some(level2),
        ),
    ]);
    tree.set_tooltip(tooltip_btn, Some(level1));

    // Use `y` to suppress unused variable warning.
    let _ = y;

    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_builds_without_entity() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let live = DemoLiveData {
            entity_info: None,
            tick: 42,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        let root = build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, 16.0);
        let rect = tree.node_rect(root);
        assert!(rect.is_some());
        let r = rect.unwrap();
        assert!(r.width > 0.0);
        assert!(r.height > 0.0);
    }

    #[test]
    fn demo_builds_with_entity() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let info = EntityInspectorInfo {
            name: "Goblin".into(),
            icon: 'g',
            position: (10, 20),
            health: Some((75.0, 100.0)),
            hunger: Some((30.0, 80.0)),
            fatigue: None,
            combat: Some((5.0, 3.0, 0.7)),
            action: Some("Wandering".into()),
            gait: Some("Walk".into()),
        };
        let live = DemoLiveData {
            entity_info: Some(&info),
            tick: 100,
            population: 5,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        let root = build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, 16.0);
        let rect = tree.node_rect(root).unwrap();
        assert_eq!(rect.width, 400.0);
    }

    #[test]
    fn demo_draw_list_not_empty() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let live = DemoLiveData {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, 16.0);
        let mut dl = super::super::DrawList::new();
        tree.draw(&mut dl);
        // Should have panels (root + separators + scroll list) and texts.
        assert!(!dl.panels.is_empty(), "draw list should have panels");
        assert!(
            !dl.texts.is_empty() || !dl.rich_texts.is_empty(),
            "draw list should have text"
        );
    }
}
