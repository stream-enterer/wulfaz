//! Character finder (UI-402).
//!
//! Search panel with TextInput filter, sorted ScrollList of matching entities.
//! Registered with PanelManager as `"finder"` (only one open at a time).

use super::theme::Theme;
use super::widget::CrossAlign;
use super::window::build_window_frame;
use super::{FontFamily, Sizing, Widget, WidgetId, WidgetTree};

/// Sort options for the character finder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderSort {
    Name,
    Distance,
    Health,
}

impl FinderSort {
    pub fn label(self) -> &'static str {
        match self {
            FinderSort::Name => "Name",
            FinderSort::Distance => "Distance",
            FinderSort::Health => "Health",
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => FinderSort::Name,
            1 => FinderSort::Distance,
            2 => FinderSort::Health,
            _ => FinderSort::Name,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            FinderSort::Name => 0,
            FinderSort::Distance => 1,
            FinderSort::Health => 2,
        }
    }
}

/// A single entry in the character finder list.
#[derive(Debug, Clone)]
pub struct FinderEntry {
    pub entity_id: u64,
    pub icon: char,
    pub name: String,
    pub action: String,
    pub position: (i32, i32),
}

/// Info needed to build the character finder.
pub struct CharacterFinderInfo {
    pub search_text: String,
    pub sort: FinderSort,
    pub entries: Vec<FinderEntry>,
    pub screen_width: f32,
    pub screen_height: f32,
}

/// Finder panel dimensions.
const PANEL_WIDTH: f32 = 320.0;
const PANEL_HEIGHT: f32 = 400.0;

/// Build the character finder panel (UI-402).
///
/// Returns `(panel_root_id, close_button_id, search_input_id, sort_dropdown_id)`.
pub fn build_character_finder(
    tree: &mut WidgetTree,
    theme: &Theme,
    info: &CharacterFinderInfo,
) -> (WidgetId, WidgetId, WidgetId, WidgetId) {
    let frame = build_window_frame(
        tree,
        theme,
        "Character Finder",
        PANEL_WIDTH,
        Sizing::Fixed(PANEL_HEIGHT),
        true,
    );
    let content_w = frame.content_width;

    // Search row: TextInput + Sort dropdown
    let search_row = tree.insert(
        frame.content,
        Widget::Row {
            gap: theme.label_gap,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(search_row, Sizing::Fixed(content_w), Sizing::Fit);

    let search_input = tree.insert(
        search_row,
        Widget::TextInput {
            text: info.search_text.clone(),
            cursor_pos: info.search_text.len(),
            color: theme.text_medium,
            bg_color: theme.progress_bar_health_bg,
            font_size: theme.font_body_size,
            placeholder: "Search...".to_string(),
            focused: true,
        },
    );
    tree.set_sizing(search_input, Sizing::Fixed(content_w - 100.0), Sizing::Fit);

    let sort_dropdown = tree.insert(
        search_row,
        Widget::Dropdown {
            selected: info.sort.to_index(),
            options: vec![
                FinderSort::Name.label().to_string(),
                FinderSort::Distance.label().to_string(),
                FinderSort::Health.label().to_string(),
            ],
            open: false,
            color: theme.text_medium,
            bg_color: theme.bg_parchment,
            font_size: theme.font_data_size,
        },
    );
    tree.set_on_click(sort_dropdown, "finder::sort");

    // Results count
    tree.insert(
        frame.content,
        Widget::Label {
            text: format!("{} found", info.entries.len()),
            color: theme.disabled,
            font_size: theme.font_data_size,
            font_family: FontFamily::Mono,
            wrap: false,
        },
    );

    // ScrollList of matching entities
    let list = tree.insert(
        frame.content,
        Widget::ScrollList {
            bg_color: [0.0, 0.0, 0.0, 0.03],
            border_color: theme.panel_border_color,
            border_width: 1.0,
            item_height: theme.scroll_item_height,
            scroll_offset: 0.0,
            scrollbar_color: theme.scrollbar_color,
            scrollbar_width: theme.scrollbar_width,
            item_heights: Vec::new(),
        },
    );
    tree.set_sizing(list, Sizing::Fixed(content_w), Sizing::Fixed(250.0));

    let row_w = content_w - theme.scrollbar_width - 4.0;
    for entry in &info.entries {
        let row = tree.insert(
            list,
            Widget::Row {
                gap: theme.label_gap * 2.0,
                align: CrossAlign::Center,
            },
        );
        tree.set_sizing(row, Sizing::Fixed(row_w), Sizing::Fit);
        tree.set_on_click(row, format!("finder::select:{}", entry.entity_id));

        // Icon
        tree.insert(
            row,
            Widget::Label {
                text: entry.icon.to_string(),
                color: theme.gold,
                font_size: theme.font_body_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );

        // Name
        tree.insert(
            row,
            Widget::Label {
                text: entry.name.clone(),
                color: theme.text_medium,
                font_size: theme.font_body_size,
                font_family: FontFamily::Serif,
                wrap: false,
            },
        );

        // Action / position
        tree.insert(
            row,
            Widget::Label {
                text: format!("({},{})", entry.position.0, entry.position.1),
                color: theme.disabled,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
                wrap: false,
            },
        );
    }

    // close_btn is always Some here since closeable=true
    (
        frame.root,
        frame.close_btn.expect("closeable frame"),
        search_input,
        sort_dropdown,
    )
}

/// Filter and sort entities for the character finder.
pub fn collect_finder_entries(
    world: &crate::world::World,
    search: &str,
    sort: FinderSort,
    camera_pos: (i32, i32),
) -> Vec<FinderEntry> {
    let search_lower = search.to_lowercase();
    let mut entries: Vec<FinderEntry> = world
        .alive
        .iter()
        .filter_map(|&entity| {
            let name = world
                .body
                .names
                .get(&entity)
                .map(|n| n.value.clone())
                .unwrap_or_else(|| format!("E{}", entity.0));

            // Filter by search text
            if !search_lower.is_empty() && !name.to_lowercase().contains(&search_lower) {
                return None;
            }

            let icon = world.body.icons.get(&entity).map(|i| i.ch).unwrap_or('?');
            let pos = world
                .body
                .positions
                .get(&entity)
                .map(|p| (p.x, p.y))
                .unwrap_or((0, 0));
            let action = world
                .mind
                .action_states
                .get(&entity)
                .and_then(|a| a.current_action.as_ref().map(|id| format!("{:?}", id)))
                .unwrap_or_else(|| "Idle".to_string());

            Some(FinderEntry {
                entity_id: entity.0,
                icon,
                name,
                action,
                position: pos,
            })
        })
        .collect();

    // Sort
    match sort {
        FinderSort::Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
        FinderSort::Distance => {
            entries.sort_by(|a, b| {
                let da = (a.position.0 - camera_pos.0).abs() + (a.position.1 - camera_pos.1).abs();
                let db = (b.position.0 - camera_pos.0).abs() + (b.position.1 - camera_pos.1).abs();
                da.cmp(&db)
            });
        }
        FinderSort::Health => {
            entries.sort_by(|a, b| {
                // Sort by entity_id as proxy when health isn't available inline
                a.entity_id.cmp(&b.entity_id)
            });
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entries() -> Vec<FinderEntry> {
        vec![
            FinderEntry {
                entity_id: 1,
                icon: 'g',
                name: "Goblin".to_string(),
                action: "Wander".to_string(),
                position: (5, 10),
            },
            FinderEntry {
                entity_id: 2,
                icon: 'w',
                name: "Wolf".to_string(),
                action: "Hunt".to_string(),
                position: (20, 30),
            },
            FinderEntry {
                entity_id: 3,
                icon: 'g',
                name: "Goblin Chief".to_string(),
                action: "Idle".to_string(),
                position: (6, 11),
            },
        ]
    }

    #[test]
    fn finder_builds_with_entries() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = CharacterFinderInfo {
            search_text: "gob".to_string(),
            sort: FinderSort::Name,
            entries: test_entries(),
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let (root, _close, search, _sort) = build_character_finder(&mut tree, &theme, &info);
        assert!(tree.get(root).is_some());
        assert!(tree.get(search).is_some());
    }

    #[test]
    fn finder_search_input_has_text() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = CharacterFinderInfo {
            search_text: "gob".to_string(),
            sort: FinderSort::Name,
            entries: test_entries(),
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let (_root, _close, search, _sort) = build_character_finder(&mut tree, &theme, &info);
        let node = tree.get(search).unwrap();
        if let Widget::TextInput { text, .. } = &node.widget {
            assert_eq!(text, "gob");
        } else {
            panic!("Expected TextInput widget");
        }
    }

    #[test]
    fn finder_sort_roundtrip() {
        for sort in [FinderSort::Name, FinderSort::Distance, FinderSort::Health] {
            assert_eq!(FinderSort::from_index(sort.to_index()), sort);
        }
    }

    #[test]
    fn finder_has_close_button() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let info = CharacterFinderInfo {
            search_text: String::new(),
            sort: FinderSort::Name,
            entries: vec![],
            screen_width: 800.0,
            screen_height: 600.0,
        };
        let (_root, close, _search, _sort) = build_character_finder(&mut tree, &theme, &info);
        let close_node = tree.get(close).unwrap();
        if let Widget::Button { text, .. } = &close_node.widget {
            assert_eq!(text, "X");
        } else {
            panic!("Expected Button widget for close");
        }
    }
}
