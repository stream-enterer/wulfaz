//! Save/Load screen (UI-412).
//!
//! Two-tab panel: Save (TextInput + button) and Load (ScrollList of saves).
//! Registered with PanelManager as `"save_load"`.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{Edges, FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

/// Info about a save file for the load list.
#[derive(Debug, Clone)]
pub struct SaveFileEntry {
    pub name: String,
    pub timestamp: String,
}

/// Info needed to build the save/load screen.
pub struct SaveLoadInfo {
    pub saves: Vec<SaveFileEntry>,
    pub screen_width: f32,
    pub screen_height: f32,
}

/// Save/load panel dimensions.
const PANEL_WIDTH: f32 = 400.0;
const PANEL_HEIGHT: f32 = 350.0;

/// Build the save/load screen (UI-412).
///
/// Returns the panel root ID. Register with PanelManager as `"save_load"`.
pub fn build_save_load_screen(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &SaveLoadInfo,
) -> WidgetId {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    tree.set_sizing(
        panel,
        Sizing::Fixed(PANEL_WIDTH),
        Sizing::Fixed(PANEL_HEIGHT),
    );
    tree.set_padding(panel, Edges::all(theme.panel_padding));
    // Center on screen
    let px = (info.screen_width - PANEL_WIDTH) / 2.0;
    let py = (info.screen_height - PANEL_HEIGHT) / 2.0;
    tree.set_position(panel, Position::Fixed { x: px, y: py });

    let content_w = PANEL_WIDTH - theme.panel_padding * 2.0;

    // Title
    let title = tree.insert(
        panel,
        Widget::Label {
            text: "Save / Load".to_string(),
            color: theme.gold,
            font_size: theme.font_header_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );
    tree.set_position(title, Position::Fixed { x: 0.0, y: 0.0 });

    // TabContainer: Save and Load tabs
    let tabs = tree.insert(
        panel,
        Widget::TabContainer {
            tabs: vec!["Save".to_string(), "Load".to_string()],
            active: 0,
            tab_color: theme.tab_inactive_color,
            active_color: theme.tab_active_color,
            font_size: theme.font_body_size,
        },
    );
    let tab_y = theme.font_header_size + theme.label_gap * 2.0;
    tree.set_position(tabs, Position::Fixed { x: 0.0, y: tab_y });
    tree.set_sizing(tabs, Sizing::Fixed(content_w), Sizing::Fit);

    // === Save tab content (child 0) ===
    let save_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(save_col, Sizing::Fixed(content_w), Sizing::Fit);

    // Save name input
    tree.insert(
        save_col,
        Widget::Label {
            text: "Save name:".to_string(),
            color: theme.text_dark,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );
    let input = tree.insert(
        save_col,
        Widget::TextInput {
            text: String::new(),
            cursor_pos: 0,
            color: theme.text_dark,
            bg_color: theme.progress_bar_health_bg,
            font_size: theme.font_body_size,
            placeholder: "Enter save name...".to_string(),
            focused: false,
        },
    );
    tree.set_sizing(input, Sizing::Fixed(content_w - 20.0), Sizing::Fit);

    // Save button
    let save_btn = tree.insert(
        save_col,
        Widget::Button {
            text: "Save Game".to_string(),
            color: theme.text_dark,
            bg_color: theme.tab_inactive_color,
            border_color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_on_click(save_btn, "save_load::save");
    tree.set_padding(
        save_btn,
        Edges {
            top: theme.button_pad_v,
            right: theme.button_pad_h,
            bottom: theme.button_pad_v,
            left: theme.button_pad_h,
        },
    );

    // === Load tab content (child 1) ===
    let load_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(load_col, Sizing::Fixed(content_w), Sizing::Fit);

    if info.saves.is_empty() {
        tree.insert(
            load_col,
            Widget::Label {
                text: "No saves found.".to_string(),
                color: theme.disabled,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
    } else {
        // ScrollList of save files
        let list = tree.insert(
            load_col,
            Widget::ScrollList {
                bg_color: [0.0, 0.0, 0.0, 0.05],
                border_color: theme.panel_border_color,
                border_width: 1.0,
                item_height: theme.scroll_item_height,
                scroll_offset: 0.0,
                scrollbar_color: theme.scrollbar_color,
                scrollbar_width: theme.scrollbar_width,
                item_heights: Vec::new(),
            },
        );
        tree.set_sizing(list, Sizing::Fixed(content_w), Sizing::Fixed(200.0));

        for save in &info.saves {
            let row = tree.insert(
                list,
                Widget::Row {
                    gap: theme.label_gap * 2.0,
                    align: CrossAlign::Center,
                },
            );
            tree.set_sizing(
                row,
                Sizing::Fixed(content_w - theme.scrollbar_width - 4.0),
                Sizing::Fit,
            );
            tree.set_on_click(row, format!("save_load::select:{}", save.name));

            tree.insert(
                row,
                Widget::Label {
                    text: save.name.clone(),
                    color: theme.text_dark,
                    font_size: theme.font_body_size,
                    font_family: FontFamily::Serif,
                    wrap: false,
                },
            );
            tree.insert(
                row,
                Widget::Label {
                    text: save.timestamp.clone(),
                    color: theme.disabled,
                    font_size: theme.font_data_size,
                    font_family: FontFamily::Mono,
                    wrap: false,
                },
            );
        }

        // Load button
        let load_btn = tree.insert(
            load_col,
            Widget::Button {
                text: "Load Game".to_string(),
                color: theme.text_dark,
                bg_color: theme.tab_inactive_color,
                border_color: theme.gold,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
            },
        );
        tree.set_on_click(load_btn, "save_load::load");
        tree.set_padding(
            load_btn,
            Edges {
                top: theme.button_pad_v,
                right: theme.button_pad_h,
                bottom: theme.button_pad_v,
                left: theme.button_pad_h,
            },
        );
    }

    panel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_has_2_tabs() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = SaveLoadInfo {
            saves: vec![],
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_save_load_screen(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();

        // Find TabContainer (second child after title)
        let tab_id = panel_node.children[1];
        let tab_node = tree.get(tab_id).unwrap();
        if let Widget::TabContainer { tabs, .. } = &tab_node.widget {
            assert_eq!(tabs.len(), 2);
            assert_eq!(tabs[0], "Save");
            assert_eq!(tabs[1], "Load");
        } else {
            panic!("Expected TabContainer widget");
        }
    }

    #[test]
    fn save_tab_has_text_input() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = SaveLoadInfo {
            saves: vec![],
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_save_load_screen(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();
        let tab_id = panel_node.children[1];
        let tab_node = tree.get(tab_id).unwrap();

        // Tab child 0 = save column
        let save_col_id = tab_node.children[0];
        let save_col_node = tree.get(save_col_id).unwrap();

        let has_text_input = save_col_node.children.iter().any(|&child_id| {
            if let Some(node) = tree.get(child_id) {
                matches!(&node.widget, Widget::TextInput { .. })
            } else {
                false
            }
        });
        assert!(has_text_input, "Save tab should contain a TextInput");
    }

    #[test]
    fn load_tab_with_saves_shows_list() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = SaveLoadInfo {
            saves: vec![
                SaveFileEntry {
                    name: "Save 1".to_string(),
                    timestamp: "2025-01-01".to_string(),
                },
                SaveFileEntry {
                    name: "Save 2".to_string(),
                    timestamp: "2025-01-02".to_string(),
                },
            ],
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_save_load_screen(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();
        let tab_id = panel_node.children[1];
        let tab_node = tree.get(tab_id).unwrap();

        // Tab child 1 = load column
        let load_col_id = tab_node.children[1];
        let load_col_node = tree.get(load_col_id).unwrap();

        let has_scroll_list = load_col_node.children.iter().any(|&child_id| {
            if let Some(node) = tree.get(child_id) {
                matches!(&node.widget, Widget::ScrollList { .. })
            } else {
                false
            }
        });
        assert!(
            has_scroll_list,
            "Load tab should contain a ScrollList with saves"
        );
    }
}
