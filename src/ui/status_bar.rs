use super::WidgetId;
use super::draw::{FontFamily, TextSpan};
use super::geometry::{Edges, Position, Sizing};
use super::keybindings::{self, KeyBindings};
use super::node::UiPerfMetrics;
use super::theme::Theme;
use super::tree::WidgetTree;
use super::widget::Widget;

/// Build the status bar panel at the top of the screen (UI-I01a).
///
/// Chrome panel: permanent, rebuilt every frame with live simulation data.
/// Replaces the old string-based `render::render_status()`.
///
/// Returns the root panel's `WidgetId` so the caller can read its
/// computed height after layout (via `WidgetTree::node_rect`).
/// Status bar configuration for pause/speed display (UI-I03).
pub struct StatusBarInfo<'a> {
    pub tick: u64,
    pub date: String,
    pub population: usize,
    pub is_turn_based: bool,
    pub player_name: Option<&'a str>,
    pub paused: bool,
    pub sim_speed: u32,
    pub keybindings: &'a KeyBindings,
    pub screen_width: f32,
    /// Previous frame's perf metrics (UI-505). None = don't display.
    pub perf: Option<UiPerfMetrics>,
}

pub fn build_status_bar(tree: &mut WidgetTree, theme: &Theme, info: &StatusBarInfo) -> WidgetId {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.status_bar_bg,
        border_color: theme.panel_border_color,
        border_width: theme.tooltip_border_width, // thin 1px border
        shadow_width: 0.0,                        // no shadow for a flat bar
    });
    tree.set_position(panel, Position::Fixed { x: 0.0, y: 0.0 });
    tree.set_sizing(panel, Sizing::Fixed(info.screen_width), Sizing::Fit);
    tree.set_padding(
        panel,
        Edges {
            top: theme.status_bar_padding_v,
            right: theme.status_bar_padding_h,
            bottom: theme.status_bar_padding_v,
            left: theme.status_bar_padding_h,
        },
    );

    let sep = || TextSpan {
        text: "  |  ".to_string(),
        color: theme.disabled,
        font_family: FontFamily::Mono,
    };

    let mut spans = vec![
        TextSpan {
            text: info.date.clone(),
            color: theme.gold,
            font_family: FontFamily::Mono,
        },
        sep(),
        TextSpan {
            text: format!("Pop: {}", info.population),
            color: theme.text_light,
            font_family: FontFamily::Mono,
        },
        sep(),
    ];

    if info.is_turn_based {
        spans.push(TextSpan {
            text: "TURN-BASED".to_string(),
            color: theme.gold,
            font_family: FontFamily::Mono,
        });
    } else if info.paused {
        // Show "PAUSED (Space)" with keybinding label.
        let pause_label = info
            .keybindings
            .label_for(keybindings::Action::Pause)
            .map(|k| format!("PAUSED ({k})"))
            .unwrap_or_else(|| "PAUSED".to_string());
        spans.push(TextSpan {
            text: pause_label,
            color: theme.danger,
            font_family: FontFamily::Mono,
        });
    } else {
        // Show speed indicator: "Speed: 1x (1)" through "Speed: 5x (5)"
        let speed_label = info
            .keybindings
            .label_for(keybindings::Action::SpeedSet(info.sim_speed))
            .map(|k| format!("Speed: {}x ({k})", info.sim_speed))
            .unwrap_or_else(|| format!("Speed: {}x", info.sim_speed));
        let color = if info.sim_speed > 1 {
            theme.gold
        } else {
            theme.text_light
        };
        spans.push(TextSpan {
            text: speed_label,
            color,
            font_family: FontFamily::Mono,
        });
    }

    if let Some(name) = info.player_name {
        spans.push(sep());
        spans.push(TextSpan {
            text: format!("@{name}"),
            color: theme.gold,
            font_family: FontFamily::Mono,
        });
    }

    // Perf metrics (UI-505): right-aligned debug info from previous frame.
    if let Some(perf) = &info.perf {
        spans.push(sep());
        spans.push(TextSpan {
            text: format!(
                "UI: build {:.1}ms | layout {:.1}ms | draw {:.1}ms | render {:.1}ms | {}w",
                perf.build_us as f64 / 1000.0,
                perf.layout_us as f64 / 1000.0,
                perf.draw_us as f64 / 1000.0,
                perf.render_us as f64 / 1000.0,
                perf.widget_count,
            ),
            color: theme.disabled,
            font_family: FontFamily::Mono,
        });
    }

    tree.insert(
        panel,
        Widget::RichText {
            spans,
            font_size: theme.font_data_size,
        },
    );

    panel
}

#[cfg(test)]
mod tests {
    use super::super::draw::{DrawList, HeuristicMeasurer};
    use super::super::geometry::Size;
    use super::super::sidebar;
    use super::*;

    #[test]
    fn widget_count_on_showcase() {
        let theme = Theme::default();
        let kb = keybindings::KeyBindings::defaults();
        let live = sidebar::SidebarInfo {
            entity_info: None,
            tick: 0,
            population: 0,
        };
        let screen = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut tree = WidgetTree::new();
        sidebar::build_showcase_view(&mut tree, &theme, &kb, &live, screen, 0.0);
        assert!(tree.widget_count() > 0, "sidebar tree should have widgets");
    }

    #[test]
    fn perf_metrics_default() {
        let m = UiPerfMetrics::default();
        assert_eq!(m.build_us, 0);
        assert_eq!(m.widget_count, 0);
    }

    #[test]
    fn status_bar_with_perf_metrics() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let perf = UiPerfMetrics {
            build_us: 300,
            layout_us: 100,
            draw_us: 200,
            render_us: 400,
            widget_count: 42,
            panel_cmds: 10,
            text_cmds: 20,
            sprite_cmds: 0,
        };
        let info = StatusBarInfo {
            tick: 0,
            date: "1 January 1845, 00:00".to_string(),
            population: 0,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: Some(perf),
        };
        let bar = build_status_bar(&mut tree, &theme, &info);
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            // With perf: 5 normal + sep + perf = 7 spans.
            assert_eq!(spans.len(), 7);
            let perf_span = &spans[6];
            assert!(perf_span.text.contains("build 0.3ms"));
            assert!(perf_span.text.contains("layout 0.1ms"));
            assert!(perf_span.text.contains("42w"));
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn status_bar_structure() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 42,
            date: "1 January 1845, 00:42".to_string(),
            population: 15,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        let bar = build_status_bar(&mut tree, &theme, &info);

        // One root: the status bar panel.
        assert_eq!(tree.roots().len(), 1);
        assert_eq!(tree.roots()[0], bar);

        // Panel has one child: the RichText.
        let node = tree.get(bar).expect("bar exists");
        assert_eq!(node.children.len(), 1);
        if let Widget::Panel { bg_color, .. } = &node.widget {
            assert_eq!(*bg_color, theme.status_bar_bg);
        } else {
            panic!("status bar root should be a Panel");
        }

        // Child is RichText with data font size.
        let child = tree.get(node.children[0]).expect("child exists");
        if let Widget::RichText { spans, font_size } = &child.widget {
            assert!((font_size - theme.font_data_size).abs() < 0.01);
            // Real-time mode, speed 1, no player: 5 spans (tick, sep, pop, sep, speed).
            assert_eq!(spans.len(), 5);
            assert_eq!(spans[0].text, "1 January 1845, 00:42");
            assert_eq!(spans[0].color, theme.gold);
            assert_eq!(spans[2].text, "Pop: 15");
            assert_eq!(spans[2].color, theme.text_light);
            assert!(spans[4].text.starts_with("Speed: 1x"));
        } else {
            panic!("status bar child should be RichText");
        }
    }

    #[test]
    fn status_bar_turn_based_with_player() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 100,
            date: "1 January 1845, 01:40".to_string(),
            population: 3,
            is_turn_based: true,
            player_name: Some("Goblin"),
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        let bar = tree.roots()[0];
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            // Turn-based + player: 7 spans (tick, sep, pop, sep, mode, sep, @name).
            assert_eq!(spans.len(), 7);
            assert_eq!(spans[4].text, "TURN-BASED");
            assert_eq!(spans[4].color, theme.gold);
            assert_eq!(spans[6].text, "@Goblin");
            assert_eq!(spans[6].color, theme.gold);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn status_bar_layout_full_width() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 0,
            date: "1 January 1845, 00:00".to_string(),
            population: 0,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 1024.0,
            perf: None,
        };
        let bar = build_status_bar(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 1024.0,
                height: 768.0,
            },
            &mut HeuristicMeasurer,
        );

        let rect = tree.node_rect(bar).expect("rect after layout");
        assert!((rect.x - 0.0).abs() < 0.01);
        assert!((rect.y - 0.0).abs() < 0.01);
        assert!((rect.width - 1024.0).abs() < 0.01);
        // Height = padding_v*2 + content (Fit sizing).
        assert!(rect.height > 0.0);
        assert!(rect.height < 100.0); // sanity: a single-line bar shouldn't be huge
    }

    #[test]
    fn status_bar_draw_output() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 7,
            date: "1 January 1845, 00:07".to_string(),
            population: 200,
            is_turn_based: true,
            player_name: Some("Wolf"),
            paused: false,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            &mut HeuristicMeasurer,
        );

        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // One panel (the status bar background).
        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.panels[0].bg_color, theme.status_bar_bg);
        assert!((dl.panels[0].width - 800.0).abs() < 0.01);

        // One rich text command with 7 spans.
        assert_eq!(dl.rich_texts.len(), 1);
        assert_eq!(dl.rich_texts[0].spans.len(), 7);
        assert!(dl.rich_texts[0].spans[0].text.contains("7"));
        assert!(dl.rich_texts[0].spans[2].text.contains("200"));
        assert_eq!(dl.rich_texts[0].spans[6].text, "@Wolf");

        // No plain text commands (only rich text).
        assert_eq!(dl.texts.len(), 0);
    }

    #[test]
    fn status_bar_paused_display() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 10,
            date: "1 January 1845, 00:10".to_string(),
            population: 5,
            is_turn_based: false,
            player_name: None,
            paused: true,
            sim_speed: 1,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        let bar = tree.roots()[0];
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            // Paused: 5 spans (tick, sep, pop, sep, "PAUSED (Space)").
            assert_eq!(spans.len(), 5);
            assert!(spans[4].text.contains("PAUSED"));
            assert!(spans[4].text.contains("Space"));
            assert_eq!(spans[4].color, theme.danger);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn status_bar_speed_display() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let kb = KeyBindings::defaults();
        let info = StatusBarInfo {
            tick: 10,
            date: "1 January 1845, 00:10".to_string(),
            population: 5,
            is_turn_based: false,
            player_name: None,
            paused: false,
            sim_speed: 3,
            keybindings: &kb,
            screen_width: 800.0,
            perf: None,
        };
        build_status_bar(&mut tree, &theme, &info);

        let bar = tree.roots()[0];
        let child_id = tree.get(bar).expect("bar").children[0];
        let child = tree.get(child_id).expect("child");
        if let Widget::RichText { spans, .. } = &child.widget {
            assert_eq!(spans.len(), 5);
            assert!(spans[4].text.contains("3x"));
            assert!(spans[4].text.contains("(3)"));
            // Speed > 1 gets gold highlight.
            assert_eq!(spans[4].color, theme.gold);
        } else {
            panic!("expected RichText");
        }
    }
}
