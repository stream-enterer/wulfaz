//! Settings screen (UI-413).
//!
//! Tabs: Display (ui_scale, window mode), Audio (placeholder), Controls (keybindings list).
//! Registered with PanelManager as `"settings"`.

use super::keybindings::{Action, KeyBindings};
use super::theme::Theme;
use super::widget::CrossAlign;
use super::window::build_window_frame;
use super::{FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

/// Info needed to build the settings screen.
pub struct SettingsInfo<'a> {
    pub ui_scale: f32,
    pub keybindings: &'a KeyBindings,
    pub screen_width: f32,
    pub screen_height: f32,
}

/// Settings panel dimensions.
const PANEL_WIDTH: f32 = 400.0;
const PANEL_HEIGHT: f32 = 350.0;

/// Build the settings screen (UI-413).
///
/// Returns `(panel_root_id, close_button_id, scale_slider_id)`.
pub fn build_settings_screen(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &SettingsInfo,
) -> (WidgetId, WidgetId, WidgetId) {
    let frame = build_window_frame(
        tree,
        theme,
        "Settings",
        PANEL_WIDTH,
        Sizing::Fixed(PANEL_HEIGHT),
        true,
    );

    // Center on screen
    let px = (info.screen_width - PANEL_WIDTH) / 2.0;
    let py = (info.screen_height - PANEL_HEIGHT) / 2.0;
    tree.set_position(frame.root, Position::Fixed { x: px, y: py });

    let content_w = frame.content_width;

    // TabContainer
    let tabs = tree.insert(
        frame.content,
        Widget::TabContainer {
            tabs: vec![
                "Display".to_string(),
                "Audio".to_string(),
                "Controls".to_string(),
            ],
            active: 0,
            tab_color: theme.tab_inactive_color,
            active_color: theme.tab_active_color,
            font_size: theme.font_body_size,
        },
    );
    tree.set_sizing(tabs, Sizing::Fixed(content_w), Sizing::Fit);

    // === Display tab (child 0) ===
    let display_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(display_col, Sizing::Fixed(content_w), Sizing::Fit);

    // UI Scale slider
    let scale_row = tree.insert(
        display_col,
        Widget::Row {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(scale_row, Sizing::Fixed(content_w), Sizing::Fit);

    tree.insert(
        scale_row,
        Widget::Label {
            text: "UI Scale:".to_string(),
            color: theme.text_dark,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    let scale_slider = tree.insert(
        scale_row,
        Widget::Slider {
            value: info.ui_scale,
            min: 0.5,
            max: 2.0,
            track_color: theme.progress_bar_health_bg,
            thumb_color: theme.gold,
            width: 120.0,
        },
    );
    tree.set_on_click(scale_slider, "settings::ui_scale");

    tree.insert(
        scale_row,
        Widget::Label {
            text: format!("{:.1}x", info.ui_scale),
            color: theme.gold,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );

    // Window mode dropdown
    let mode_row = tree.insert(
        display_col,
        Widget::Row {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(mode_row, Sizing::Fixed(content_w), Sizing::Fit);

    tree.insert(
        mode_row,
        Widget::Label {
            text: "Window:".to_string(),
            color: theme.text_dark,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    let window_dropdown = tree.insert(
        mode_row,
        Widget::Dropdown {
            selected: 0,
            options: vec![
                "Windowed".to_string(),
                "Borderless".to_string(),
                "Fullscreen".to_string(),
            ],
            open: false,
            color: theme.text_dark,
            bg_color: theme.bg_parchment,
            font_size: theme.font_data_size,
        },
    );
    tree.set_on_click(window_dropdown, "settings::window_mode");

    // === Audio tab (child 1) -- placeholder ===
    let audio_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(audio_col, Sizing::Fixed(content_w), Sizing::Fit);

    tree.insert(
        audio_col,
        Widget::Label {
            text: "Audio settings will be available when the audio system is implemented."
                .to_string(),
            color: theme.disabled,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: true,
        },
    );

    // === Controls tab (child 2) -- read-only keybinding list ===
    let controls_col = tree.insert(
        tabs,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(controls_col, Sizing::Fixed(content_w), Sizing::Fit);

    tree.insert(
        controls_col,
        Widget::Label {
            text: "Keyboard Shortcuts".to_string(),
            color: theme.text_dark,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    let sep = tree.insert(
        controls_col,
        Widget::Separator {
            color: theme.panel_border_color,
            thickness: theme.separator_thickness,
            horizontal: true,
        },
    );
    tree.set_sizing(sep, Sizing::Fixed(content_w), Sizing::Fit);

    // List keybindings
    let bindings = [
        (Action::PauseSim, "Pause/Resume"),
        (Action::CloseTopmost, "Close Panel"),
        (Action::ToggleDemo, "Widget Showcase"),
        (Action::SpeedSet(1), "Speed 1x"),
        (Action::SpeedSet(2), "Speed 2x"),
        (Action::SpeedSet(3), "Speed 3x"),
    ];

    for (action, description) in &bindings {
        let key_label = info
            .keybindings
            .label_for(*action)
            .unwrap_or_else(|| "?".to_string());

        let row = tree.insert(
            controls_col,
            Widget::Row {
                gap: theme.label_gap * 4.0,
                align: CrossAlign::Center,
            },
        );
        tree.set_sizing(row, Sizing::Fixed(content_w), Sizing::Fit);

        tree.insert(
            row,
            Widget::Label {
                text: key_label,
                color: theme.gold,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
        tree.insert(
            row,
            Widget::Label {
                text: description.to_string(),
                color: theme.text_dark,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );
    }

    // close_btn is always Some here since closeable=true
    (
        frame.root,
        frame.close_btn.expect("closeable frame"),
        scale_slider,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_has_3_tabs() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let mut tree = WidgetTree::new();
        let info = SettingsInfo {
            ui_scale: 1.0,
            keybindings: &kb,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let (root, _close, _slider) = build_settings_screen(&mut tree, &theme, &info);

        // Navigate: root -> frame_col -> content -> TabContainer
        let root_node = tree.get(root).unwrap();
        let frame_col = root_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();
        let tab_id = content_node.children[0];
        let tab_node = tree.get(tab_id).unwrap();
        if let Widget::TabContainer { tabs, .. } = &tab_node.widget {
            assert_eq!(tabs.len(), 3);
            assert_eq!(tabs[0], "Display");
            assert_eq!(tabs[1], "Audio");
            assert_eq!(tabs[2], "Controls");
        } else {
            panic!("Expected TabContainer widget");
        }
    }

    #[test]
    fn display_tab_has_scale_slider() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let mut tree = WidgetTree::new();
        let info = SettingsInfo {
            ui_scale: 1.5,
            keybindings: &kb,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let (_root, _close, slider) = build_settings_screen(&mut tree, &theme, &info);

        let node = tree.get(slider).unwrap();
        if let Widget::Slider {
            value, min, max, ..
        } = &node.widget
        {
            assert!((*value - 1.5).abs() < 0.01);
            assert_eq!(*min, 0.5);
            assert_eq!(*max, 2.0);
        } else {
            panic!("Expected Slider widget");
        }
    }

    #[test]
    fn settings_has_close_button() {
        let theme = Theme::default();
        let kb = KeyBindings::defaults();
        let mut tree = WidgetTree::new();
        let info = SettingsInfo {
            ui_scale: 1.0,
            keybindings: &kb,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let (_root, close, _slider) = build_settings_screen(&mut tree, &theme, &info);
        let close_node = tree.get(close).unwrap();
        if let Widget::Button { text, .. } = &close_node.widget {
            assert_eq!(text, "X");
        } else {
            panic!("Expected Button widget for close");
        }
    }
}
