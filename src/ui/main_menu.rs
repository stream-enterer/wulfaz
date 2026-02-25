//! Main menu (UI-415).
//!
//! Displayed on application start before the game world loads.
//! Centered panel with game title and menu buttons.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{Edges, FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

/// Application state machine. Only InGame runs the simulation tick loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    MainMenu,
    Loading,
    InGame,
}

/// Info needed to build the main menu.
pub struct MainMenuInfo {
    pub has_saves: bool,
    pub screen_width: f32,
    pub screen_height: f32,
}

/// Build the main menu (UI-415).
///
/// Returns the panel root ID. Buttons dispatch `UiAction` variants:
/// `MenuNewGame`, `MenuContinue`, `MenuLoad`, `MenuSettings`, `MenuQuit`.
pub fn build_main_menu(tree: &mut WidgetTree, theme: &Theme, info: &MainMenuInfo) -> WidgetId {
    // Full-screen background
    let bg = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: [0.0; 4],
        border_width: 0.0,
        shadow_width: 0.0,
    });
    tree.set_sizing(
        bg,
        Sizing::Fixed(info.screen_width),
        Sizing::Fixed(info.screen_height),
    );

    // Centered menu panel
    let menu_w = theme.s(300.0);
    let menu = tree.insert(
        bg,
        Widget::Panel {
            bg_color: [0.0, 0.0, 0.0, 0.0], // transparent inner panel
            border_color: [0.0; 4],
            border_width: 0.0,
            shadow_width: 0.0,
        },
    );
    tree.set_sizing(menu, Sizing::Fixed(menu_w), Sizing::Fit);
    let menu_x = (info.screen_width - menu_w) / 2.0;
    let menu_y = info.screen_height * 0.25;
    tree.set_position(
        menu,
        Position::Fixed {
            x: menu_x,
            y: menu_y,
        },
    );

    let col = tree.insert(
        menu,
        Widget::Column {
            gap: theme.label_gap * 3.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_position(col, Position::Fixed { x: 0.0, y: 0.0 });
    tree.set_sizing(col, Sizing::Fixed(menu_w), Sizing::Fit);

    // Title
    tree.insert(
        col,
        Widget::Label {
            text: "Wulfaz".to_string(),
            color: theme.gold,
            font_size: theme.font_header_size * 3.0,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    // Separator
    let sep = tree.insert(
        col,
        Widget::Separator {
            color: theme.gold,
            thickness: theme.separator_thickness + 1.0,
            horizontal: true,
        },
    );
    tree.set_sizing(sep, Sizing::Fixed(menu_w * 0.8), Sizing::Fit);

    let button_w = theme.s(200.0);

    // New Game button
    let new_game = make_menu_button(tree, theme, col, "New Game", button_w);
    tree.set_on_click(new_game, super::UiAction::MenuNewGame);

    // Continue button (enabled only if saves exist)
    let continue_btn = make_menu_button(tree, theme, col, "Continue", button_w);
    tree.set_on_click(continue_btn, super::UiAction::MenuContinue);
    if !info.has_saves {
        // Visually disable
        if let Some(node) = tree.get_mut(continue_btn)
            && let Widget::Button {
                color,
                border_color,
                ..
            } = &mut node.widget
        {
            *color = theme.disabled;
            *border_color = theme.disabled;
        }
    }

    // Load Game button
    let load = make_menu_button(tree, theme, col, "Load Game", button_w);
    tree.set_on_click(load, super::UiAction::MenuLoad);

    // Settings button
    let settings = make_menu_button(tree, theme, col, "Settings", button_w);
    tree.set_on_click(settings, super::UiAction::MenuSettings);

    // Quit button
    let quit = make_menu_button(tree, theme, col, "Quit", button_w);
    tree.set_on_click(quit, super::UiAction::MenuQuit);

    bg
}

/// Helper: create a menu button with consistent styling.
fn make_menu_button(
    tree: &mut WidgetTree,
    theme: &Theme,
    parent: WidgetId,
    text: &str,
    width: f32,
) -> WidgetId {
    let btn = tree.insert(
        parent,
        Widget::Button {
            text: text.to_string(),
            color: theme.text_medium,
            bg_color: theme.tab_inactive_color,
            border_color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_sizing(btn, Sizing::Fixed(width), Sizing::Fit);
    tree.set_padding(
        btn,
        Edges {
            top: theme.button_pad_v * 1.5,
            right: theme.button_pad_h,
            bottom: theme.button_pad_v * 1.5,
            left: theme.button_pad_h,
        },
    );
    btn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_has_5_buttons() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = MainMenuInfo {
            has_saves: true,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_main_menu(&mut tree, &theme, &info);
        let bg_node = tree.get(root).unwrap();
        let menu_id = bg_node.children[0];
        let menu_node = tree.get(menu_id).unwrap();
        let col_id = menu_node.children[0];
        let col_node = tree.get(col_id).unwrap();

        // Children: title, separator, new_game, continue, load, settings, quit
        let button_count = col_node
            .children
            .iter()
            .filter(|&&id| {
                tree.get(id)
                    .map(|n| matches!(&n.widget, Widget::Button { .. }))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(button_count, 5, "Main menu should have 5 buttons");
    }

    #[test]
    fn quit_button_has_callback() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = MainMenuInfo {
            has_saves: false,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_main_menu(&mut tree, &theme, &info);
        let bg_node = tree.get(root).unwrap();
        let menu_id = bg_node.children[0];
        let menu_node = tree.get(menu_id).unwrap();
        let col_id = menu_node.children[0];
        let col_node = tree.get(col_id).unwrap();

        // Last button should be Quit
        let last_btn_id = col_node.children.last().unwrap();
        let node = tree.get(*last_btn_id).unwrap();
        assert!(matches!(node.on_click, Some(crate::ui::UiAction::MenuQuit)));
    }

    #[test]
    fn continue_disabled_without_saves() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = MainMenuInfo {
            has_saves: false,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_main_menu(&mut tree, &theme, &info);
        let bg_node = tree.get(root).unwrap();
        let menu_id = bg_node.children[0];
        let menu_node = tree.get(menu_id).unwrap();
        let col_id = menu_node.children[0];
        let col_node = tree.get(col_id).unwrap();

        // Continue button is 3rd child (index 3: title=0, sep=1, new_game=2, continue=3)
        let continue_id = col_node.children[3];
        let node = tree.get(continue_id).unwrap();
        if let Widget::Button { color, .. } = &node.widget {
            assert_eq!(*color, theme.disabled, "Continue button should be disabled");
        }
    }

    #[test]
    fn app_state_variants() {
        assert_ne!(AppState::MainMenu, AppState::Loading);
        assert_ne!(AppState::Loading, AppState::InGame);
        assert_ne!(AppState::MainMenu, AppState::InGame);
    }
}
