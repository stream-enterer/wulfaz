use crate::components::Entity;

/// Every UI interaction the app can dispatch. Exhaustive match enforces handling.
#[derive(Debug, Clone)]
pub enum UiAction {
    // Inspector (src/ui/mod.rs — build_entity_inspector)
    InspectorClose,

    // Sidebar (src/ui/sidebar.rs)
    SelectTab(usize),

    // Modal (src/ui/modal.rs)
    ModalDismiss,

    // Dialog (src/ui/window.rs — build_confirmation_dialog)
    DialogAccept,
    DialogCancel,

    // Main menu (src/ui/main_menu.rs)
    MenuNewGame,
    MenuContinue,
    MenuLoad,
    MenuSettings,
    MenuQuit,

    // Outliner (src/ui/outliner.rs)
    OutlinerSelectCharacter(Entity),
    OutlinerSelectEvent(String),

    // Character finder (src/ui/character_finder.rs)
    FinderSort,
    FinderSelect(Entity),

    // Settings (src/ui/settings.rs)
    SettingsUiScale,
    SettingsWindowMode,

    // Save/Load (src/ui/save_load.rs)
    SaveLoadSave,
    SaveLoadLoad,
    SaveLoadSelect(String),

    // Map mode (src/ui/map_mode.rs)
    MapModeChange,
    MapModeSpeed,

    // Event popup (src/ui/event_popup.rs) — data-driven, callback from KDL
    EventChoice(String),

    // Context menu (src/ui/context_menu.rs) — data-driven, action from MenuItem
    ContextAction(String),
}

/// Known panel types. Used as keys in PanelManager and scroll offset maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelKind {
    Sidebar,
    Outliner,
    CharacterPanel,
    CharacterFinder,
    OpinionView,
    Settings,
    SaveLoad,
    MapMode,
    EventPopup,
}
