pub(crate) mod action;
mod animation;
pub(crate) mod character_finder;
pub(crate) mod character_panel;
pub(crate) mod context;
mod context_menu;
mod draw;
pub(crate) mod entity_inspector;
pub(crate) mod event_log;
pub(crate) mod event_popup;
mod geometry;
pub(crate) mod hover_tooltip;
mod input;
mod keybindings;
pub(crate) mod loading_screen;
pub(crate) mod main_menu;
pub(crate) mod map_mode;
pub(crate) mod minimap;
pub(crate) mod modal;
mod node;
mod notification;
pub(crate) mod opinion_view;
pub(crate) mod outliner;
mod panel_manager;
pub(crate) mod pause_overlay;
pub(crate) mod save_load;
pub(crate) mod settings;
pub(crate) mod sidebar;
pub(crate) mod sprite;
pub(crate) mod status_bar;
#[cfg(test)]
mod test_helpers;
mod theme;
mod tree;
mod tree_anim;
mod tree_draw;
mod tree_hit_test;
mod tree_layout;
mod tree_scroll;
mod tree_tooltip;
mod widget;
pub(crate) mod window;

#[allow(unused_imports)] // Public API: used by main.rs for typed callbacks (UI-P1).
pub use action::{PanelKind, UiAction};
#[allow(unused_imports)] // Public API: used by main.rs for animation (UI-W05).
pub use animation::{Anim, Animator, Easing};
#[allow(unused_imports)] // Public API: used by main.rs for character finder (UI-402).
pub use character_finder::{
    CharacterFinderInfo, FinderEntry, FinderSort, build_character_finder, collect_finder_entries,
};
#[allow(unused_imports)] // Public API: used by main.rs for character panel (UI-400).
pub use character_panel::{CharacterPanelInfo, build_character_panel, collect_character_info};
#[allow(unused_imports)] // Public API: used by main.rs for UiContext (UI-P2).
pub use context::{DismissResult, SidebarState, UiContext};
#[allow(unused_imports)] // Public API: used by main.rs for context menus (UI-303).
pub use context_menu::{ContextMenu, MenuItem};
#[allow(unused_imports)] // Public API: used by game panels constructing widgets.
pub use draw::{
    DrawList, FontFamily, HeuristicMeasurer, PanelCommand, RichTextCommand, SpriteCommand,
    TextCommand, TextMeasurer, TextSpan,
};
#[allow(unused_imports)] // Public API: used by main.rs for entity inspector (UI-I01d).
pub use entity_inspector::{EntityInspectorInfo, build_entity_inspector, collect_inspector_info};
#[allow(unused_imports)] // Public API: used by main.rs for event log (UI-I01c).
pub use event_log::{EventLogEntry, build_event_log, collect_event_entries};
#[allow(unused_imports)] // Public API: used by main.rs for event popups (UI-401).
pub use event_popup::{EventChoice, NarrativeEvent, build_event_popup};
#[allow(unused_imports)] // Public API: used by builders and main.rs.
pub use geometry::{Constraints, Edges, Position, Rect, Size, Sizing};
#[allow(unused_imports)] // Public API: used by main.rs for hover tooltip (UI-I01b).
pub use hover_tooltip::{HoverInfo, build_hover_tooltip};
#[allow(unused_imports)] // Public API: used by main.rs for input routing (UI-W02).
pub use input::{MapClick, MouseButton, UiEvent, UiState};
#[allow(unused_imports)] // Public API: used by main.rs for keyboard shortcuts (UI-I03).
pub use keybindings::{Action, KeyBindings, KeyCombo, ModifierFlags};
#[allow(unused_imports)] // Public API: used by main.rs for loading screen (UI-414).
pub use loading_screen::{LoadingScreenInfo, LoadingStage, build_loading_screen};
#[allow(unused_imports)] // Public API: used by main.rs for main menu (UI-415).
pub use main_menu::{AppState, MainMenuInfo, build_main_menu};
#[allow(unused_imports)] // Public API: used by main.rs for map mode selector (UI-403).
pub use map_mode::{MapMode, MapModeInfo, build_map_mode_selector};
#[allow(unused_imports)] // Public API: used by main.rs for minimap (UI-407).
pub use minimap::{MinimapInfo, MinimapTexture, build_minimap, minimap_click_to_world};
#[allow(unused_imports)] // Public API: used by main.rs for modal management (UI-300).
pub use modal::{ModalOptions, ModalPop, ModalStack};
#[allow(unused_imports)] // Internal API: used by tree modules.
pub(crate) use node::WidgetNode;
#[allow(unused_imports)] // Public API: used by main.rs for perf metrics and z-tiers.
pub use node::{UiPerfMetrics, ZTier};
#[allow(unused_imports)] // Public API: used by main.rs for notification system (UI-302).
pub use notification::{NotificationManager, NotificationPriority};
#[allow(unused_imports)] // Public API: used by main.rs for opinion view (UI-406).
pub use opinion_view::{OpinionModifier, OpinionViewInfo, Sentiment, build_opinion_view};
#[allow(unused_imports)] // Public API: used by main.rs for outliner (UI-405).
pub use outliner::{
    ActiveEvent, AlertEntry, AlertPriority, OutlinerInfo, PinnedCharacter, build_outliner,
};
#[allow(unused_imports)] // Public API: used by main.rs for panel management (UI-306).
pub use panel_manager::PanelManager;
#[allow(unused_imports)] // Public API: used by main.rs for pause overlay (UI-105).
pub use pause_overlay::build_pause_overlay;
#[allow(unused_imports)] // Public API: used by main.rs for save/load screen (UI-412).
pub use save_load::{SaveFileEntry, SaveLoadInfo, build_save_load_screen};
#[allow(unused_imports)] // Public API: used by main.rs for settings screen (UI-413).
pub use settings::{SettingsInfo, build_settings_screen};
#[allow(unused_imports)] // Public API: used by main.rs for sidebar (UI-407).
pub use sidebar::{
    MAIN_TAB_WIDTH, SIDEBAR_MARGIN, SidebarInfo, TAB_COUNT, build_placeholder_view,
    build_showcase_view, build_tab_strip,
};
#[allow(unused_imports)] // Public API: used by sprite renderer (UI-202b).
pub use sprite::{SpriteAtlas, SpriteRect};
#[allow(unused_imports)] // Public API: used by main.rs for status bar (UI-I01a).
pub use status_bar::{StatusBarInfo, build_status_bar};
pub use theme::Theme;
pub use tree::WidgetTree;
#[allow(unused_imports)] // Public API: used by game panels setting tooltip content.
pub use widget::{CrossAlign, TooltipContent, Widget};
#[allow(unused_imports)]
// Public API: used by screen builders for shared window frame (UI-600).
pub use window::{ConfirmationDialog, WindowFrame, build_confirmation_dialog, build_window_frame};

use slotmap::new_key_type;

new_key_type! {
    /// Handle into the widget arena. Stable across insertions/removals.
    pub struct WidgetId;
}
