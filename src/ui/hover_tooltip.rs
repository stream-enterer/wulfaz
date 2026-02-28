use super::WidgetId;
use super::draw::{FontFamily, TextMeasurer, TextSpan};
use super::geometry::Size;
use super::theme::Theme;
use super::tree::WidgetTree;
use super::widget::Widget;

/// Data for the map hover tooltip (UI-I01b).
/// Extracted from World in main.rs, consumed by `build_hover_tooltip`.
pub struct HoverInfo {
    pub tile_x: i32,
    pub tile_y: i32,
    pub terrain: String,
    pub quartier: Option<String>,
    pub address: Option<String>,
    pub building_name: Option<String>,
    /// (name, activity) pairs for building occupants.
    pub occupants: Vec<(String, String)>,
    /// Year suffix like "[1842]" if data is from a fallback year.
    pub occupant_year_suffix: Option<String>,
    /// (icon_char, name) pairs for alive entities on this tile.
    pub entities: Vec<(char, String)>,
}

/// Maximum number of occupants shown in the hover tooltip.
const HOVER_MAX_OCCUPANTS: usize = 5;

/// Build a hover tooltip panel for the hovered map tile (UI-I01b).
///
/// Created on demand when cursor is over a map tile, destroyed when
/// cursor leaves (per DD-5). Styled like a W04 tooltip panel.
/// Replaces the old string-based `render::render_hover_info()`.
///
/// Returns the root panel's `WidgetId`.
pub fn build_hover_tooltip(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &HoverInfo,
    cursor: (f32, f32),
    screen: Size,
    tm: &mut dyn TextMeasurer,
) -> WidgetId {
    let (panel, col) = tree.insert_tooltip_chrome(theme);

    // Line 1: coordinates + terrain type (always short)
    tree.insert(
        col,
        Widget::RichText {
            spans: vec![
                TextSpan {
                    text: format!("({}, {})", info.tile_x, info.tile_y),
                    color: theme.gold,
                    font_family: FontFamily::Mono,
                },
                TextSpan {
                    text: format!("  {}", info.terrain),
                    color: theme.text_light,
                    font_family: FontFamily::Mono,
                },
            ],
            font_size: theme.font_data_size,
        },
    );

    // Line 2: quartier name (optional)
    if let Some(ref quartier) = info.quartier {
        tree.insert(
            col,
            Widget::Label {
                text: quartier.clone(),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
                wrap: false,
            },
        );
    }

    // Line 3: address + building name (optional, can be long — wraps)
    if let Some(ref address) = info.address {
        // Address on its own line.
        tree.insert(
            col,
            Widget::Label {
                text: address.clone(),
                color: theme.text_light,
                font_size: theme.font_body_size,
                font_family: theme.font_body_family,
                wrap: true,
            },
        );
        // Building name on a separate wrapping line.
        if let Some(ref name) = info.building_name {
            tree.insert(
                col,
                Widget::Label {
                    text: name.clone(),
                    color: theme.gold,
                    font_size: theme.font_body_size,
                    font_family: theme.font_body_family,
                    wrap: true,
                },
            );
        }
    }

    // Occupants section (names can be very long — wraps)
    if !info.occupants.is_empty() {
        let show_count = info.occupants.len().min(HOVER_MAX_OCCUPANTS);
        for (name, activity) in &info.occupants[..show_count] {
            tree.insert(
                col,
                Widget::Label {
                    text: format!("{} ({})", name, activity),
                    color: theme.text_light,
                    font_size: theme.font_data_size,
                    font_family: theme.font_data_family,
                    wrap: true,
                },
            );
        }
        if info.occupants.len() > HOVER_MAX_OCCUPANTS {
            tree.insert(
                col,
                Widget::Label {
                    text: format!("+{} more", info.occupants.len() - HOVER_MAX_OCCUPANTS),
                    color: theme.disabled,
                    font_size: theme.font_data_size,
                    font_family: theme.font_data_family,
                    wrap: false,
                },
            );
        }
        if let Some(ref suffix) = info.occupant_year_suffix {
            tree.insert(
                col,
                Widget::Label {
                    text: suffix.clone(),
                    color: theme.disabled,
                    font_size: theme.font_data_size,
                    font_family: theme.font_data_family,
                    wrap: false,
                },
            );
        }
    }

    // Entities section (usually short)
    for (icon, name) in &info.entities {
        tree.insert(
            col,
            Widget::Label {
                text: format!("{} {}", icon, name),
                color: theme.text_light,
                font_size: theme.font_data_size,
                font_family: theme.font_data_family,
                wrap: true,
            },
        );
    }

    // Position: below-right of cursor, edge-flip if clipping screen.
    tree.position_tooltip(panel, cursor, screen, 0, theme, tm);

    panel
}

#[cfg(test)]
mod tests {
    use super::super::draw::{DrawList, HeuristicMeasurer};
    use super::super::geometry::Position;
    use super::super::node::ZTier;
    use super::super::widget::TooltipContent;
    use super::*;

    fn screen() -> Size {
        Size {
            width: 800.0,
            height: 600.0,
        }
    }

    /// Helper: build a minimal HoverInfo with just terrain.
    fn hover_terrain_only() -> HoverInfo {
        HoverInfo {
            tile_x: 100,
            tile_y: 200,
            terrain: "Road".into(),
            quartier: None,
            address: None,
            building_name: None,
            occupants: Vec::new(),
            occupant_year_suffix: None,
            entities: Vec::new(),
        }
    }

    #[test]
    fn hover_tooltip_terrain_only() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = hover_terrain_only();
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (100.0, 100.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        // One root: the tooltip panel.
        assert_eq!(tree.roots().len(), 1);
        assert_eq!(tree.roots()[0], tip);

        // Panel has one child: the Column wrapper.
        let node = tree.get(tip).expect("tooltip panel");
        assert_eq!(node.children.len(), 1);
        if let Widget::Panel { bg_color, .. } = &node.widget {
            assert_eq!(*bg_color, theme.tooltip_bg_color);
        } else {
            panic!("tooltip root should be a Panel");
        }

        // Column has one child: the coordinates + terrain RichText.
        let col = tree.get(node.children[0]).expect("column");
        assert!(matches!(col.widget, Widget::Column { .. }));
        assert_eq!(col.children.len(), 1);
        let child = tree.get(col.children[0]).expect("child");
        if let Widget::RichText { spans, font_size } = &child.widget {
            assert!((font_size - theme.font_data_size).abs() < 0.01);
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0].text, "(100, 200)");
            assert_eq!(spans[0].color, theme.gold);
            assert!(spans[1].text.contains("Road"));
            assert_eq!(spans[1].color, theme.text_light);
        } else {
            panic!("expected RichText");
        }
    }

    #[test]
    fn hover_tooltip_full_building() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = HoverInfo {
            tile_x: 42,
            tile_y: 99,
            terrain: "Floor".into(),
            quartier: Some("Marais".into()),
            address: Some("42 Rue de Rivoli".into()),
            building_name: Some("Boulangerie".into()),
            occupants: vec![
                ("Jean Dupont".into(), "flour merchant".into()),
                ("Marie".into(), "baker".into()),
            ],
            occupant_year_suffix: None,
            entities: vec![('g', "Goblin".into())],
        };
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (200.0, 200.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        let node = tree.get(tip).expect("panel");
        // Panel has 1 child (Column).
        assert_eq!(node.children.len(), 1);
        let col = tree.get(node.children[0]).expect("column");
        // Column children: coords(1) + quartier(1) + address(1) + building_name(1)
        //                 + 2 occupants(2) + 1 entity(1) = 7
        assert_eq!(col.children.len(), 7);
    }

    #[test]
    fn hover_tooltip_occupant_truncation() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = HoverInfo {
            tile_x: 0,
            tile_y: 0,
            terrain: "Floor".into(),
            quartier: None,
            address: Some("1 Rue X".into()),
            building_name: None,
            occupants: (0..8)
                .map(|i| (format!("Person {}", i), "trade".into()))
                .collect(),
            occupant_year_suffix: Some("[1842]".into()),
            entities: Vec::new(),
        };
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (50.0, 50.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        let node = tree.get(tip).expect("panel");
        // Panel has 1 child (Column).
        assert_eq!(node.children.len(), 1);
        let col = tree.get(node.children[0]).expect("column");
        // Column children: coords(1) + address(1) + 5 occupants(5) + "+3 more"(1) + year(1) = 9
        assert_eq!(col.children.len(), 9);

        // Verify "+3 more" label exists.
        let mut dl = DrawList::new();
        tree.layout(screen(), &mut HeuristicMeasurer);
        tree.draw(&mut dl, &mut HeuristicMeasurer);
        let has_more = dl.texts.iter().any(|t| t.text == "+3 more");
        assert!(has_more, "should show +3 more for 8 occupants (max 5)");
    }

    #[test]
    fn hover_tooltip_entities_shown() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = HoverInfo {
            tile_x: 10,
            tile_y: 20,
            terrain: "Road".into(),
            quartier: None,
            address: None,
            building_name: None,
            occupants: Vec::new(),
            occupant_year_suffix: None,
            entities: vec![('g', "Goblin".into()), ('w', "Wolf".into())],
        };
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (100.0, 100.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        let node = tree.get(tip).expect("panel");
        // Panel has 1 child (Column).
        assert_eq!(node.children.len(), 1);
        let col = tree.get(node.children[0]).expect("column");
        // Column children: coords(1) + 2 entity labels(2) = 3
        assert_eq!(col.children.len(), 3);

        tree.layout(screen(), &mut HeuristicMeasurer);
        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // Entity entries are now Labels (previously RichText).
        let goblin_label = dl.texts.iter().any(|t| t.text.contains("Goblin"));
        assert!(goblin_label, "should have a label containing 'Goblin'");
    }

    #[test]
    fn hover_tooltip_draw_output() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = hover_terrain_only();
        build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (100.0, 100.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        tree.layout(screen(), &mut HeuristicMeasurer);
        let mut dl = DrawList::new();
        tree.draw(&mut dl, &mut HeuristicMeasurer);

        // One panel (tooltip background).
        assert_eq!(dl.panels.len(), 1);
        assert_eq!(dl.panels[0].bg_color, theme.tooltip_bg_color);
        assert_eq!(dl.panels[0].border_color, theme.tooltip_border_color);

        // One rich text (coords + terrain).
        assert_eq!(dl.rich_texts.len(), 1);
        assert_eq!(dl.rich_texts[0].spans.len(), 2);
        assert!(dl.rich_texts[0].spans[0].text.contains("100"));
    }

    #[test]
    fn hover_tooltip_positioned_on_screen() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = hover_terrain_only();
        let tip = build_hover_tooltip(
            &mut tree,
            &theme,
            &info,
            (750.0, 550.0),
            screen(),
            &mut HeuristicMeasurer,
        );

        tree.layout(screen(), &mut HeuristicMeasurer);
        let rect = tree.node_rect(tip).expect("rect");

        // Tooltip should be fully on screen (edge-flipped if necessary).
        assert!(rect.x >= 0.0, "x={} should be >= 0", rect.x);
        assert!(rect.y >= 0.0, "y={} should be >= 0", rect.y);
        assert!(
            rect.x + rect.width <= 800.0,
            "right edge {} should be <= 800",
            rect.x + rect.width
        );
        assert!(
            rect.y + rect.height <= 600.0,
            "bottom edge {} should be <= 600",
            rect.y + rect.height
        );
    }

    /// Verify that both tooltip systems (W04 `show_tooltip` and I01b
    /// `build_hover_tooltip`) use the same standardized wrapping pipeline:
    ///   Panel(constraints=tooltip_max_width) > Column > Label(wrap=true)
    ///
    /// Run with: `cargo test tooltip_wrapping_consistent`
    #[test]
    fn tooltip_wrapping_consistent() {
        use super::super::input::UiState;
        use std::time::{Duration, Instant};

        let theme = Theme::default();
        let max_w = theme.tooltip_max_width;
        let pad = theme.tooltip_padding;

        // --- Helper: assert tooltip panel structure ---
        // Both tooltip types must produce:
        //   Panel (Fit, Fit, constraints.max_width == tooltip_max_width)
        //     -- Column (Fit, Fit, CrossAlign::Start)
        //         -- ... children (Labels with wrap=true for long text)
        fn assert_tooltip_structure(
            tree: &WidgetTree,
            panel_id: WidgetId,
            max_w: f32,
            pad: f32,
            label: &str,
        ) {
            let panel = tree.get(panel_id).expect("panel exists");
            assert!(
                matches!(panel.widget, Widget::Panel { .. }),
                "{label}: root must be Panel"
            );
            // Z-tier must be Tooltip so tooltips render above modals/overlays.
            let tier = tree.z_tier(panel_id);
            assert_eq!(
                tier,
                Some(ZTier::Tooltip),
                "{label}: must be ZTier::Tooltip"
            );
            // Constraint caps width.
            let c = panel
                .constraints
                .as_ref()
                .expect(&format!("{label}: needs constraints"));
            assert!(
                (c.max_width - max_w).abs() < 0.01,
                "{label}: constraint max_width={}, expected {}",
                c.max_width,
                max_w,
            );
            assert!(
                (panel.padding.left - pad).abs() < 0.01,
                "{label}: padding should match tooltip_padding"
            );
            // First child is a Column.
            assert_eq!(
                panel.children.len(),
                1,
                "{label}: Panel should have 1 child (Column)"
            );
            let col = tree.get(panel.children[0]).expect("column");
            assert!(
                matches!(col.widget, Widget::Column { .. }),
                "{label}: Panel child must be Column"
            );
        }

        // === 1. W04 tooltip (show_tooltip via TooltipContent::Text) ===
        let mut tree1 = WidgetTree::new();
        let btn = tree1.insert_root(Widget::Button {
            text: "X".into(),
            color: [1.0; 4],
            bg_color: [0.3; 4],
            border_color: [0.5; 4],
            font_size: 14.0,
            font_family: FontFamily::default(),
        });
        tree1.set_position(btn, Position::Fixed { x: 10.0, y: 10.0 });
        let long = "word ".repeat(80); // long text that must wrap
        tree1.set_tooltip(btn, Some(TooltipContent::Text(long)));
        tree1.layout(screen(), &mut HeuristicMeasurer);

        let mut state = UiState::new();
        let t0 = Instant::now();
        let br = tree1.get(btn).unwrap().rect;
        state.handle_cursor_moved(&mut tree1, br.x + 1.0, br.y + 1.0);
        state.update_tooltips(&mut tree1, &theme, screen(), t0, &mut HeuristicMeasurer);
        let t1 = t0 + Duration::from_millis(theme.tooltip_delay_ms + 1);
        state.update_tooltips(&mut tree1, &theme, screen(), t1, &mut HeuristicMeasurer);
        assert_eq!(state.tooltip_count(), 1);

        let w04_panel = tree1.roots()[1]; // second root = tooltip
        assert_tooltip_structure(&tree1, w04_panel, max_w, pad, "W04");

        // After layout, width must be <= max and height > single line.
        tree1.layout(screen(), &mut HeuristicMeasurer);
        let w04_rect = tree1.get(w04_panel).unwrap().rect;
        assert!(
            w04_rect.width <= max_w + 0.01,
            "W04 width {} should be <= {}",
            w04_rect.width,
            max_w,
        );
        let one_line_h = HeuristicMeasurer
            .measure_text("M", theme.font_body_family, theme.font_body_size)
            .height;
        assert!(
            w04_rect.height > one_line_h + pad * 2.0,
            "W04 height {} should indicate multi-line wrapping (single-line ~{})",
            w04_rect.height,
            one_line_h + pad * 2.0,
        );

        // === 2. I01b hover tooltip (build_hover_tooltip) ===
        let mut tree2 = WidgetTree::new();
        let info = HoverInfo {
            tile_x: 0,
            tile_y: 0,
            terrain: "Floor".into(),
            quartier: None,
            address: Some("Rue de Rivoli, 261".into()),
            building_name: Some("Vandermersch (J.-A.) et Cie (blanchisseurs de calicots et percales, toiles et tissus de coton)".into()),
            occupants: vec![("An extremely long occupant name that should definitely exceed the tooltip max width when combined with their lengthy activity description".into(), "some very long activity description here".into())],
            occupant_year_suffix: None,
            entities: Vec::new(),
        };
        let i01b_panel = build_hover_tooltip(
            &mut tree2,
            &theme,
            &info,
            (100.0, 100.0),
            screen(),
            &mut HeuristicMeasurer,
        );
        assert_tooltip_structure(&tree2, i01b_panel, max_w, pad, "I01b");

        // After layout, width must be <= max and height must be multi-line.
        tree2.layout(screen(), &mut HeuristicMeasurer);
        let i01b_rect = tree2.get(i01b_panel).unwrap().rect;
        assert!(
            i01b_rect.width <= max_w + 0.01,
            "I01b width {} should be <= {}",
            i01b_rect.width,
            max_w,
        );
        // With long building name + long occupant, must exceed a few lines.
        let min_expected_h = one_line_h * 3.0 + pad * 2.0;
        assert!(
            i01b_rect.height > min_expected_h,
            "I01b height {} should indicate multi-line wrapping (min expected ~{})",
            i01b_rect.height,
            min_expected_h,
        );
    }
}
