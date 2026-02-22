use std::collections::HashMap;

use winit::keyboard::KeyCode;

/// Modifier flags for a key combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierFlags {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl ModifierFlags {
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
    };
}

/// A key combination: modifier flags + a physical key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub modifiers: ModifierFlags,
    pub key: KeyCode,
}

impl KeyCombo {
    /// Plain key, no modifiers.
    pub const fn plain(key: KeyCode) -> Self {
        Self {
            modifiers: ModifierFlags::NONE,
            key,
        }
    }
}

/// Actions that can be triggered by keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Toggle simulation pause.
    PauseSim,
    /// Set simulation speed (1 = normal, 2-5 = faster).
    SpeedSet(u32),
    /// Close topmost overlay (tooltip → inspector → exit).
    CloseTopmost,
}

/// Configurable keyboard shortcut map.
pub struct KeyBindings {
    map: HashMap<KeyCombo, Action>,
    /// Reverse lookup: action → first combo that maps to it.
    reverse: HashMap<Action, KeyCombo>,
}

impl KeyBindings {
    /// Default keybindings per UI-I03 spec.
    pub fn defaults() -> Self {
        let mut map = HashMap::new();

        map.insert(KeyCombo::plain(KeyCode::Space), Action::PauseSim);
        map.insert(KeyCombo::plain(KeyCode::Escape), Action::CloseTopmost);
        map.insert(KeyCombo::plain(KeyCode::Digit1), Action::SpeedSet(1));
        map.insert(KeyCombo::plain(KeyCode::Digit2), Action::SpeedSet(2));
        map.insert(KeyCombo::plain(KeyCode::Digit3), Action::SpeedSet(3));
        map.insert(KeyCombo::plain(KeyCode::Digit4), Action::SpeedSet(4));
        map.insert(KeyCombo::plain(KeyCode::Digit5), Action::SpeedSet(5));

        let reverse = Self::build_reverse(&map);
        Self { map, reverse }
    }

    /// Look up the action for a key combination.
    pub fn lookup(&self, combo: KeyCombo) -> Option<Action> {
        self.map.get(&combo).copied()
    }

    /// Get the display label for an action's keybinding (e.g. "Space", "Esc", "1").
    pub fn label_for(&self, action: Action) -> Option<String> {
        self.reverse.get(&action).map(|combo| {
            let mut parts = Vec::new();
            if combo.modifiers.ctrl {
                parts.push("Ctrl");
            }
            if combo.modifiers.alt {
                parts.push("Alt");
            }
            if combo.modifiers.shift {
                parts.push("Shift");
            }
            parts.push(key_name(combo.key));
            parts.join("+")
        })
    }

    fn build_reverse(map: &HashMap<KeyCombo, Action>) -> HashMap<Action, KeyCombo> {
        let mut reverse = HashMap::new();
        for (&combo, &action) in map {
            // First combo wins (deterministic for defaults since we insert in order,
            // but HashMap iteration is arbitrary — acceptable for MVP).
            reverse.entry(action).or_insert(combo);
        }
        reverse
    }
}

/// Human-readable name for a key code.
fn key_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Space => "Space",
        KeyCode::Escape => "Esc",
        KeyCode::Tab => "Tab",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::Digit0 => "0",
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::Enter => "Enter",
        KeyCode::Backspace => "Bksp",
        KeyCode::Delete => "Del",
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PgUp",
        KeyCode::PageDown => "PgDn",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_exist() {
        let kb = KeyBindings::defaults();
        assert_eq!(
            kb.lookup(KeyCombo::plain(KeyCode::Space)),
            Some(Action::PauseSim)
        );
        assert_eq!(
            kb.lookup(KeyCombo::plain(KeyCode::Escape)),
            Some(Action::CloseTopmost)
        );
        assert_eq!(
            kb.lookup(KeyCombo::plain(KeyCode::Digit1)),
            Some(Action::SpeedSet(1))
        );
        assert_eq!(
            kb.lookup(KeyCombo::plain(KeyCode::Digit5)),
            Some(Action::SpeedSet(5))
        );
    }

    #[test]
    fn unbound_key_returns_none() {
        let kb = KeyBindings::defaults();
        assert_eq!(kb.lookup(KeyCombo::plain(KeyCode::KeyZ)), None);
    }

    #[test]
    fn label_for_pause() {
        let kb = KeyBindings::defaults();
        let label = kb.label_for(Action::PauseSim);
        assert_eq!(label.as_deref(), Some("Space"));
    }

    #[test]
    fn label_for_close() {
        let kb = KeyBindings::defaults();
        let label = kb.label_for(Action::CloseTopmost);
        assert_eq!(label.as_deref(), Some("Esc"));
    }

    #[test]
    fn label_for_speed() {
        let kb = KeyBindings::defaults();
        assert_eq!(kb.label_for(Action::SpeedSet(1)).as_deref(), Some("1"));
        assert_eq!(kb.label_for(Action::SpeedSet(3)).as_deref(), Some("3"));
    }

    #[test]
    fn modifier_combo_label() {
        let mut map = HashMap::new();
        map.insert(
            KeyCombo {
                modifiers: ModifierFlags {
                    shift: false,
                    ctrl: true,
                    alt: false,
                },
                key: KeyCode::KeyP,
            },
            Action::PauseSim,
        );
        let reverse = KeyBindings::build_reverse(&map);
        let kb = KeyBindings { map, reverse };
        assert_eq!(kb.label_for(Action::PauseSim).as_deref(), Some("Ctrl+P"));
    }

    #[test]
    fn key_name_coverage() {
        assert_eq!(key_name(KeyCode::Space), "Space");
        assert_eq!(key_name(KeyCode::Escape), "Esc");
        assert_eq!(key_name(KeyCode::F11), "F11");
        assert_eq!(key_name(KeyCode::Enter), "Enter");
        assert_eq!(key_name(KeyCode::ArrowUp), "Up");
    }
}
