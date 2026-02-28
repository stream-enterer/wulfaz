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

// Core tree types
pub use tree::WidgetTree;
pub use widget::Widget;

// Geometry
pub use geometry::{Edges, Position, Rect, Size, Sizing};

// Drawing
pub use draw::{DrawList, FontFamily, HeuristicMeasurer, TextMeasurer};

// Node metadata
pub use node::{UiPerfMetrics, ZTier};

// Actions & input
pub use action::UiAction;
pub use input::{MouseButton, UiEvent, UiState};

// Animation
pub use animation::{Anim, Animator, Easing};

// Infrastructure
pub use context::{DismissResult, SidebarState, UiContext};
pub use keybindings::{Action, KeyBindings, KeyCombo, ModifierFlags};
pub use modal::ModalStack;
pub use panel_manager::PanelManager;
pub use theme::Theme;

// Builders (active — called from main.rs)
pub use entity_inspector::{EntityInspectorInfo, build_entity_inspector, collect_inspector_info};
pub use event_log::{EventLogEntry, build_event_log, collect_event_entries};
pub use hover_tooltip::{HoverInfo, build_hover_tooltip};
pub use minimap::{MinimapInfo, MinimapTexture, build_minimap, minimap_click_to_world};
pub use pause_overlay::build_pause_overlay;
pub use sidebar::{
    MAIN_TAB_WIDTH, SIDEBAR_MARGIN, SidebarInfo, build_placeholder_view, build_showcase_view,
    build_tab_strip,
};
pub use status_bar::{StatusBarInfo, build_status_bar};

// Builders (pending integration into main.rs)
pub use character_finder::{
    CharacterFinderInfo, FinderEntry, FinderSort, build_character_finder, collect_finder_entries,
};
pub use character_panel::{CharacterPanelInfo, build_character_panel, collect_character_info};
pub use context_menu::{ContextMenu, MenuItem};
pub use event_popup::{EventChoice, NarrativeEvent, build_event_popup};
pub use loading_screen::{LoadingScreenInfo, LoadingStage, build_loading_screen};
pub use main_menu::{AppState, MainMenuInfo, build_main_menu};
pub use map_mode::{MapMode, MapModeInfo, build_map_mode_selector};
pub use notification::{NotificationManager, NotificationPriority};
pub use opinion_view::{OpinionModifier, OpinionViewInfo, Sentiment, build_opinion_view};
pub use outliner::{
    ActiveEvent, AlertEntry, AlertPriority, OutlinerInfo, PinnedCharacter, build_outliner,
};
pub use save_load::{SaveFileEntry, SaveLoadInfo, build_save_load_screen};
pub use settings::{SettingsInfo, build_settings_screen};
pub use sprite::{SpriteAtlas, SpriteRect};
pub use window::{ConfirmationDialog, WindowFrame, build_confirmation_dialog, build_window_frame};

use slotmap::new_key_type;

new_key_type! {
    /// Handle into the widget arena. Stable across insertions/removals.
    pub struct WidgetId;
}
