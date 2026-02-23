//! Loading screen (UI-414).
//!
//! Full-screen panel displayed during startup while data loads.
//! Shows title, progress bar, and status label.

use super::theme::Theme;
use super::{FontFamily, Position, Sizing, Widget, WidgetId, WidgetTree};

/// Loading stage identifiers. Each stage advances the progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingStage {
    Terrain,
    Creatures,
    Sprites,
    Entities,
    Done,
}

impl LoadingStage {
    /// Progress fraction for this stage (0.0 to 1.0).
    pub fn progress(self) -> f32 {
        match self {
            Self::Terrain => 0.0,
            Self::Creatures => 0.25,
            Self::Sprites => 0.50,
            Self::Entities => 0.75,
            Self::Done => 1.0,
        }
    }

    /// Status text displayed during this stage.
    pub fn label(self) -> &'static str {
        match self {
            Self::Terrain => "Loading terrain...",
            Self::Creatures => "Loading creatures...",
            Self::Sprites => "Loading sprites...",
            Self::Entities => "Spawning entities...",
            Self::Done => "Ready",
        }
    }
}

/// Info needed to build the loading screen.
pub struct LoadingScreenInfo {
    pub stage: LoadingStage,
    pub screen_width: f32,
    pub screen_height: f32,
}

/// Build the loading screen (UI-414).
///
/// Full-screen panel with centered title, progress bar, and status label.
pub fn build_loading_screen(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &LoadingScreenInfo,
) -> WidgetId {
    let panel = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: [0.0; 4],
        border_width: 0.0,
        shadow_width: 0.0,
    });
    tree.set_sizing(
        panel,
        Sizing::Fixed(info.screen_width),
        Sizing::Fixed(info.screen_height),
    );

    // Center column: title + progress bar + status label
    let col = tree.insert(
        panel,
        Widget::Column {
            gap: theme.label_gap * 4.0,
            align: super::widget::CrossAlign::Center,
        },
    );
    let col_w = theme.s(300.0);
    tree.set_sizing(col, Sizing::Fixed(col_w), Sizing::Fit);
    // Center the column in the screen
    let col_x = (info.screen_width - col_w) / 2.0;
    let col_y = (info.screen_height - 100.0) / 2.0;
    tree.set_position(col, Position::Fixed { x: col_x, y: col_y });

    // Title
    tree.insert(
        col,
        Widget::Label {
            text: "Wulfaz".to_string(),
            color: theme.gold,
            font_size: theme.font_header_size * 2.0,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    // Progress bar
    let fraction = info.stage.progress();
    let bar = tree.insert(
        col,
        Widget::ProgressBar {
            fraction,
            fg_color: theme.gold,
            bg_color: theme.progress_bar_health_bg,
            border_color: theme.panel_border_color,
            border_width: theme.progress_bar_border_width,
            height: theme.progress_bar_height * 2.0,
        },
    );
    tree.set_sizing(bar, Sizing::Fixed(col_w), Sizing::Fit);

    // Status label
    tree.insert(
        col,
        Widget::Label {
            text: info.stage.label().to_string(),
            color: theme.text_medium,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    panel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_screen_progress_bar_fraction() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = LoadingScreenInfo {
            stage: LoadingStage::Sprites,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_loading_screen(&mut tree, &theme, &info);
        assert!(tree.get(root).is_some());

        // Find the progress bar in the tree
        let panel_node = tree.get(root).unwrap();
        let col_id = panel_node.children[0];
        let col_node = tree.get(col_id).unwrap();
        // Children: title, progress bar, label
        assert!(col_node.children.len() >= 3);
        let bar_id = col_node.children[1];
        let bar_node = tree.get(bar_id).unwrap();
        if let Widget::ProgressBar { fraction, .. } = &bar_node.widget {
            assert!(
                (*fraction - 0.5).abs() < 0.01,
                "Expected 0.5, got {}",
                fraction
            );
        } else {
            panic!("Expected ProgressBar widget");
        }
    }

    #[test]
    fn loading_screen_status_label() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = LoadingScreenInfo {
            stage: LoadingStage::Terrain,
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let root = build_loading_screen(&mut tree, &theme, &info);
        let panel_node = tree.get(root).unwrap();
        let col_id = panel_node.children[0];
        let col_node = tree.get(col_id).unwrap();
        let label_id = col_node.children[2];
        let label_node = tree.get(label_id).unwrap();
        if let Widget::Label { text, .. } = &label_node.widget {
            assert_eq!(text, "Loading terrain...");
        } else {
            panic!("Expected Label widget");
        }
    }

    #[test]
    fn loading_stage_progress_values() {
        assert_eq!(LoadingStage::Terrain.progress(), 0.0);
        assert_eq!(LoadingStage::Creatures.progress(), 0.25);
        assert_eq!(LoadingStage::Sprites.progress(), 0.50);
        assert_eq!(LoadingStage::Entities.progress(), 0.75);
        assert_eq!(LoadingStage::Done.progress(), 1.0);
    }
}
