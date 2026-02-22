//! Map mode selector (UI-403).
//!
//! Dropdown for map modes + game speed slider, integrates into the status bar area.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{FontFamily, Sizing, Widget, WidgetId, WidgetTree};

/// Available map rendering modes. Open-ended for future extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapMode {
    Terrain,
    Political,
    PopulationDensity,
}

impl MapMode {
    /// All currently available map modes.
    pub fn all() -> &'static [MapMode] {
        &[
            MapMode::Terrain,
            MapMode::Political,
            MapMode::PopulationDensity,
        ]
    }

    /// Display label for the map mode.
    pub fn label(self) -> &'static str {
        match self {
            MapMode::Terrain => "Terrain",
            MapMode::Political => "Political",
            MapMode::PopulationDensity => "Population",
        }
    }

    /// Convert index (from Dropdown selection) to MapMode.
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => MapMode::Terrain,
            1 => MapMode::Political,
            2 => MapMode::PopulationDensity,
            _ => MapMode::Terrain,
        }
    }

    /// Index for the Dropdown widget.
    pub fn to_index(self) -> usize {
        match self {
            MapMode::Terrain => 0,
            MapMode::Political => 1,
            MapMode::PopulationDensity => 2,
        }
    }
}

/// Info needed to build the map mode selector.
pub struct MapModeInfo {
    pub current_mode: MapMode,
    pub sim_speed: u32,
}

/// Build the map mode selector + speed slider row (UI-403).
///
/// Returns `(row_id, dropdown_id, slider_id)` for callback dispatch.
pub fn build_map_mode_selector(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &MapModeInfo,
) -> (WidgetId, WidgetId, WidgetId) {
    let row = tree.insert_root(Widget::Row {
        gap: theme.label_gap * 2.0,
        align: CrossAlign::Center,
    });
    tree.set_sizing(row, Sizing::Fit, Sizing::Fit);

    // Map mode label
    tree.insert(
        row,
        Widget::Label {
            text: "Map:".to_string(),
            color: theme.text_dark,
            font_size: theme.font_data_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    // Map mode dropdown
    let options: Vec<String> = MapMode::all()
        .iter()
        .map(|m| m.label().to_string())
        .collect();
    let dropdown = tree.insert(
        row,
        Widget::Dropdown {
            selected: info.current_mode.to_index(),
            options,
            open: false,
            color: theme.text_dark,
            bg_color: theme.bg_parchment,
            font_size: theme.font_data_size,
        },
    );
    tree.set_on_click(dropdown, "map_mode::change");

    // Speed label
    tree.insert(
        row,
        Widget::Label {
            text: "Speed:".to_string(),
            color: theme.text_dark,
            font_size: theme.font_data_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    // Speed slider (1-5)
    let slider = tree.insert(
        row,
        Widget::Slider {
            value: info.sim_speed as f32,
            min: 1.0,
            max: 5.0,
            track_color: theme.progress_bar_health_bg,
            thumb_color: theme.gold,
            width: 80.0,
        },
    );
    tree.set_on_click(slider, "map_mode::speed");

    // Speed value label
    tree.insert(
        row,
        Widget::Label {
            text: format!("{}x", info.sim_speed),
            color: theme.gold,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );

    (row, dropdown, slider)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_mode_dropdown_has_correct_selection() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = MapModeInfo {
            current_mode: MapMode::Political,
            sim_speed: 2,
        };
        let (_row, dropdown, _slider) = build_map_mode_selector(&mut tree, &theme, &info);

        let node = tree.get(dropdown).unwrap();
        if let Widget::Dropdown {
            selected, options, ..
        } = &node.widget
        {
            assert_eq!(*selected, 1); // Political is index 1
            assert_eq!(options.len(), 3);
        } else {
            panic!("Expected Dropdown widget");
        }
    }

    #[test]
    fn speed_slider_value() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = MapModeInfo {
            current_mode: MapMode::Terrain,
            sim_speed: 3,
        };
        let (_row, _dropdown, slider) = build_map_mode_selector(&mut tree, &theme, &info);

        let node = tree.get(slider).unwrap();
        if let Widget::Slider {
            value, min, max, ..
        } = &node.widget
        {
            assert_eq!(*value, 3.0);
            assert_eq!(*min, 1.0);
            assert_eq!(*max, 5.0);
        } else {
            panic!("Expected Slider widget");
        }
    }

    #[test]
    fn map_mode_roundtrip() {
        for mode in MapMode::all() {
            assert_eq!(MapMode::from_index(mode.to_index()), *mode);
        }
    }
}
