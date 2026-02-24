//! UI-DEMO: Widget showcase panel.
//!
//! A persistent developer reference panel (toggled with F11 or `--ui-demo`)
//! that renders every available widget type. Uses Column/Row auto-layout
//! throughout — no manual position tracking.

use super::draw::{FontFamily, TextSpan};
use super::keybindings::{Action, KeyBindings};
use super::theme::Theme;
use super::widget::{CrossAlign, TooltipContent, Widget};
use super::{Edges, EntityInspectorInfo, Position, Size, Sizing, WidgetId, WidgetTree};

/// Live simulation data for the demo's live-data section.
pub struct DemoLiveData<'a> {
    pub entity_info: Option<&'a EntityInspectorInfo>,
    pub tick: u64,
    pub population: usize,
}

/// Build the demo widget showcase into an existing tree.
///
/// Returns the root panel `WidgetId` so the caller can apply slide-in
/// animation. The demo occupies a 400px-wide panel on the right side.
pub fn build_demo(
    tree: &mut WidgetTree,
    theme: &Theme,
    keybindings: &KeyBindings,
    live: &DemoLiveData,
    screen: Size,
) -> WidgetId {
    let panel_w = 400.0_f32;
    let panel_h = screen.height - 8.0; // 4px margin top+bottom
    let content_w = panel_w - theme.panel_padding * 2.0;

    // Root panel — parchment background with gold border.
    let root = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    tree.set_position(
        root,
        Position::Fixed {
            x: screen.width - panel_w - 4.0,
            y: 4.0,
        },
    );
    tree.set_sizing(root, Sizing::Fixed(panel_w), Sizing::Fixed(panel_h));
    tree.set_padding(root, Edges::all(theme.panel_padding));

    // Main content column — all sections flow top-to-bottom.
    let col = tree.insert(
        root,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(col, Sizing::Fixed(content_w), Sizing::Fit);

    // -------------------------------------------------------------------
    // Title
    // -------------------------------------------------------------------
    tree.insert(
        col,
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
    insert_sep(tree, col, theme, content_w);

    // -------------------------------------------------------------------
    // Typography
    // -------------------------------------------------------------------
    section_header(tree, col, theme, "Typography");

    tree.insert(
        col,
        Widget::Label {
            text: "Serif Header 21px".into(),
            color: theme.text_light,
            font_size: theme.font_header_size,
            font_family: theme.font_header_family,
            wrap: false,
        },
    );
    tree.insert(
        col,
        Widget::Label {
            text: "Serif Body 16px".into(),
            color: theme.text_light,
            font_size: theme.font_body_size,
            font_family: theme.font_body_family,
            wrap: false,
        },
    );
    tree.insert(
        col,
        Widget::Label {
            text: "Mono Data 12px".into(),
            color: theme.text_light,
            font_size: theme.font_data_size,
            font_family: theme.font_data_family,
            wrap: false,
        },
    );

    // Semantic colors in a row.
    let color_row = tree.insert(
        col,
        Widget::Row {
            gap: theme.label_gap * 3.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(color_row, Sizing::Fixed(content_w), Sizing::Fit);
    for (text, color) in [
        ("Danger", theme.danger),
        ("Positive", theme.text_positive),
        ("Warning", theme.text_warning),
        ("Disabled", theme.disabled),
    ] {
        tree.insert(
            color_row,
            Widget::Label {
                text: text.into(),
                color,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
    }
    insert_sep(tree, col, theme, content_w);

    // -------------------------------------------------------------------
    // Rich Text
    // -------------------------------------------------------------------
    section_header(tree, col, theme, "Rich Text");

    tree.insert(
        col,
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
    insert_sep(tree, col, theme, content_w);

    // -------------------------------------------------------------------
    // Progress Bars
    // -------------------------------------------------------------------
    section_header(tree, col, theme, "Progress Bars");

    for (fraction, fg, label) in [
        (0.75, theme.progress_bar_health_fg, "Health 75%"),
        (0.40, theme.gold, "Hunger 40%"),
        (0.12, theme.danger, "Low 12%"),
    ] {
        tree.insert(
            col,
            Widget::Label {
                text: label.into(),
                color: theme.text_low,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        let bar = tree.insert(
            col,
            Widget::ProgressBar {
                fraction,
                fg_color: fg,
                bg_color: theme.progress_bar_health_bg,
                border_color: theme.panel_border_color,
                border_width: theme.progress_bar_border_width,
                height: theme.progress_bar_height,
            },
        );
        tree.set_sizing(bar, Sizing::Fixed(content_w), Sizing::Fit);
    }
    insert_sep(tree, col, theme, content_w);

    // -------------------------------------------------------------------
    // Buttons + Keybindings
    // -------------------------------------------------------------------
    section_header(tree, col, theme, "Buttons");

    // Pause + Close in a row.
    let btn_row = tree.insert(
        col,
        Widget::Row {
            gap: theme.label_gap,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(btn_row, Sizing::Fixed(content_w), Sizing::Fit);

    let pause_label = keybindings
        .label_for(Action::PauseSim)
        .unwrap_or_else(|| "?".into());
    let pause_btn = tree.insert(
        btn_row,
        Widget::Button {
            text: format!("Pause ({})", pause_label),
            color: theme.text_light,
            bg_color: [0.0, 0.0, 0.0, 0.0],
            border_color: theme.panel_border_color,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_tooltip(
        pause_btn,
        Some(TooltipContent::Text("Toggle simulation pause".into())),
    );

    tree.insert(btn_row, Widget::Expand);

    let close_label = keybindings
        .label_for(Action::CloseTopmost)
        .unwrap_or_else(|| "?".into());
    let close_btn = tree.insert(
        btn_row,
        Widget::Button {
            text: format!("Close ({})", close_label),
            color: theme.danger,
            bg_color: [0.0, 0.0, 0.0, 0.0],
            border_color: theme.danger,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_tooltip(
        close_btn,
        Some(TooltipContent::Text("Close topmost overlay".into())),
    );

    // Speed buttons in a row.
    let speed_row = tree.insert(
        col,
        Widget::Row {
            gap: theme.label_gap,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(speed_row, Sizing::Fixed(content_w), Sizing::Fit);

    for speed in 1..=5 {
        let speed_label = keybindings
            .label_for(Action::SpeedSet(speed))
            .unwrap_or_else(|| format!("{}", speed));
        tree.insert(
            speed_row,
            Widget::Button {
                text: format!("{}x ({})", speed, speed_label),
                color: theme.text_light,
                bg_color: [0.0, 0.0, 0.0, 0.0],
                border_color: theme.panel_border_color,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
            },
        );
    }
    insert_sep(tree, col, theme, content_w);

    // -------------------------------------------------------------------
    // Controls (Checkbox, Slider, Dropdown, TextInput)
    // -------------------------------------------------------------------
    section_header(tree, col, theme, "Controls");

    // Checkboxes in a row.
    let check_row = tree.insert(
        col,
        Widget::Row {
            gap: theme.label_gap * 4.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(check_row, Sizing::Fixed(content_w), Sizing::Fit);

    tree.insert(
        check_row,
        Widget::Checkbox {
            checked: true,
            label: "Show grid".into(),
            color: theme.text_medium,
            font_size: theme.font_body_size,
        },
    );
    tree.insert(
        check_row,
        Widget::Checkbox {
            checked: false,
            label: "Debug mode".into(),
            color: theme.text_medium,
            font_size: theme.font_body_size,
        },
    );

    // Slider row: label + slider + value.
    let slider_row = tree.insert(
        col,
        Widget::Row {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(slider_row, Sizing::Fixed(content_w), Sizing::Fit);

    tree.insert(
        slider_row,
        Widget::Label {
            text: "Speed:".into(),
            color: theme.text_medium,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );
    tree.insert(
        slider_row,
        Widget::Slider {
            value: 1.5,
            min: 0.5,
            max: 3.0,
            track_color: theme.progress_bar_health_bg,
            thumb_color: theme.gold,
            width: 120.0,
        },
    );
    tree.insert(
        slider_row,
        Widget::Label {
            text: "1.5x".into(),
            color: theme.gold,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );

    // Dropdown.
    tree.insert(
        col,
        Widget::Dropdown {
            selected: 0,
            options: vec!["Windowed".into(), "Borderless".into(), "Fullscreen".into()],
            open: false,
            color: theme.text_medium,
            bg_color: theme.bg_parchment,
            font_size: theme.font_data_size,
        },
    );

    // Text input.
    let text_input = tree.insert(
        col,
        Widget::TextInput {
            text: String::new(),
            cursor_pos: 0,
            color: theme.text_medium,
            bg_color: [
                theme.bg_parchment[0] * 0.8,
                theme.bg_parchment[1] * 0.8,
                theme.bg_parchment[2] * 0.8,
                theme.bg_parchment[3],
            ],
            font_size: theme.font_body_size,
            placeholder: "Search...".into(),
            focused: false,
        },
    );
    tree.set_sizing(text_input, Sizing::Fixed(content_w), Sizing::Fit);
    insert_sep(tree, col, theme, content_w);

    // -------------------------------------------------------------------
    // Scroll List
    // -------------------------------------------------------------------
    section_header(tree, col, theme, "Scroll List");

    let scroll_h = 80.0_f32;
    let scroll_list = tree.insert(
        col,
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
            item_heights: Vec::new(),
            empty_text: None,
        },
    );
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
                color: theme.text_medium,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
                wrap: false,
            },
        );
    }
    insert_sep(tree, col, theme, content_w);

    // -------------------------------------------------------------------
    // Collapsible: Live Data
    // -------------------------------------------------------------------
    let live_section = tree.insert(
        col,
        Widget::Collapsible {
            header: "Live Data".into(),
            expanded: true,
            color: theme.gold,
            font_size: theme.font_body_size,
        },
    );
    tree.set_sizing(live_section, Sizing::Fixed(content_w), Sizing::Fit);

    let live_col = tree.insert(
        live_section,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(live_col, Sizing::Fixed(content_w), Sizing::Fit);

    // Tick + population.
    tree.insert(
        live_col,
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

    // Entity details.
    if let Some(info) = live.entity_info {
        tree.insert(
            live_col,
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

        // Health + hunger stats.
        let mut stat_spans = Vec::new();
        if let Some((cur, max)) = info.health {
            let ratio = if max > 0.0 { cur / max } else { 0.0 };
            let color = severity_color(theme, ratio);
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
            let color = severity_color(theme, ratio);
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
            tree.insert(
                live_col,
                Widget::RichText {
                    spans: stat_spans,
                    font_size: theme.font_data_size,
                },
            );
        }

        if let Some(ref action) = info.action {
            tree.insert(
                live_col,
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
        }
    } else {
        tree.insert(
            live_col,
            Widget::Label {
                text: "No entities alive".into(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
                wrap: false,
            },
        );
    }

    // -------------------------------------------------------------------
    // Collapsible: Tooltips
    // -------------------------------------------------------------------
    let tooltip_section = tree.insert(
        col,
        Widget::Collapsible {
            header: "Tooltips".into(),
            expanded: true,
            color: theme.gold,
            font_size: theme.font_body_size,
        },
    );
    tree.set_sizing(tooltip_section, Sizing::Fixed(content_w), Sizing::Fit);

    let tooltip_btn = tree.insert(
        tooltip_section,
        Widget::Button {
            text: "Hover for nested tooltips".into(),
            color: theme.text_light,
            bg_color: [0.0, 0.0, 0.0, 0.0],
            border_color: theme.panel_border_color,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );

    // 3-level nested tooltip chain.
    let level3 = TooltipContent::Text("Level 3: deepest tooltip".into());
    let level2 = TooltipContent::Custom(vec![
        (
            Widget::Label {
                text: "Level 2 tooltip".into(),
                color: theme.text_light,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
            None,
        ),
        (
            Widget::Label {
                text: "[hover for level 3]".into(),
                color: theme.gold,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
            Some(level3),
        ),
    ]);
    let level1 = TooltipContent::Custom(vec![
        (
            Widget::Label {
                text: "Level 1 tooltip".into(),
                color: theme.text_light,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
            None,
        ),
        (
            Widget::Label {
                text: "[hover for level 2]".into(),
                color: theme.gold,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
            Some(level2),
        ),
    ]);
    tree.set_tooltip(tooltip_btn, Some(level1));

    root
}

/// Insert a gold section header label.
fn section_header(tree: &mut WidgetTree, parent: WidgetId, theme: &Theme, text: &str) {
    tree.insert(
        parent,
        Widget::Label {
            text: text.into(),
            color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );
}

/// Insert a horizontal separator spanning `width`.
fn insert_sep(tree: &mut WidgetTree, parent: WidgetId, theme: &Theme, width: f32) {
    let sep = tree.insert(
        parent,
        Widget::Separator {
            color: theme.separator_color,
            thickness: theme.separator_thickness,
            horizontal: true,
        },
    );
    tree.set_sizing(sep, Sizing::Fixed(width), Sizing::Fit);
}

/// Pick color by severity ratio (current/max).
fn severity_color(theme: &Theme, ratio: f32) -> [f32; 4] {
    if ratio > 0.5 {
        theme.text_positive
    } else if ratio > 0.25 {
        theme.text_warning
    } else {
        theme.text_negative
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::HeuristicMeasurer;

    fn default_live() -> DemoLiveData<'static> {
        DemoLiveData {
            entity_info: None,
            tick: 42,
            population: 0,
        }
    }

    fn default_screen() -> Size {
        Size {
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn demo_builds_without_entity() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let live = default_live();
        let screen = default_screen();
        let mut tree = WidgetTree::new();
        let root = build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, &mut HeuristicMeasurer);
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
        let screen = default_screen();
        let mut tree = WidgetTree::new();
        let root = build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, &mut HeuristicMeasurer);
        let rect = tree.node_rect(root).unwrap();
        assert_eq!(rect.width, 400.0);
    }

    #[test]
    fn demo_draw_list_not_empty() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let live = default_live();
        let screen = default_screen();
        let mut tree = WidgetTree::new();
        build_demo(&mut tree, &theme, &kb, &live, screen);
        tree.layout(screen, &mut HeuristicMeasurer);
        let mut dl = super::super::DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);
        assert!(!dl.panels.is_empty(), "draw list should have panels");
        assert!(
            !dl.texts.is_empty() || !dl.rich_texts.is_empty(),
            "draw list should have text"
        );
    }

    /// Collect all widget type discriminants by walking the tree from roots.
    fn collect_widget_types(tree: &WidgetTree, id: WidgetId, types: &mut Vec<String>) {
        if let Some(node) = tree.get(id) {
            let name = match &node.widget {
                Widget::Panel { .. } => "Panel",
                Widget::Column { .. } => "Column",
                Widget::Row { .. } => "Row",
                Widget::Label { .. } => "Label",
                Widget::Button { .. } => "Button",
                Widget::RichText { .. } => "RichText",
                Widget::ScrollList { .. } => "ScrollList",
                Widget::ProgressBar { .. } => "ProgressBar",
                Widget::Separator { .. } => "Separator",
                Widget::Checkbox { .. } => "Checkbox",
                Widget::Dropdown { .. } => "Dropdown",
                Widget::Slider { .. } => "Slider",
                Widget::TextInput { .. } => "TextInput",
                Widget::Collapsible { .. } => "Collapsible",
                Widget::Expand => "Expand",
                _ => "Other",
            };
            types.push(name.to_string());
            for &child in &node.children {
                collect_widget_types(tree, child, types);
            }
        }
    }

    #[test]
    fn demo_has_all_widget_types() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let info = EntityInspectorInfo {
            name: "Test".into(),
            icon: 't',
            position: (0, 0),
            health: Some((50.0, 100.0)),
            hunger: Some((25.0, 100.0)),
            fatigue: None,
            combat: None,
            action: Some("Idle".into()),
            gait: None,
        };
        let live = DemoLiveData {
            entity_info: Some(&info),
            tick: 1,
            population: 1,
        };
        let screen = default_screen();
        let mut tree = WidgetTree::new();
        build_demo(&mut tree, &theme, &kb, &live, screen);

        // Walk tree from roots, collect all widget types.
        let mut types = Vec::new();
        for root_id in tree.roots() {
            collect_widget_types(&tree, root_id, &mut types);
        }

        for expected in [
            "Panel",
            "Column",
            "Row",
            "Label",
            "Button",
            "RichText",
            "ScrollList",
            "ProgressBar",
            "Separator",
            "Checkbox",
            "Dropdown",
            "Slider",
            "TextInput",
            "Collapsible",
            "Expand",
        ] {
            assert!(
                types.contains(&expected.to_string()),
                "missing {}",
                expected
            );
        }
    }
}
