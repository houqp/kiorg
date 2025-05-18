use egui::{Key, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Define a struct to represent a keyboard shortcut
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct KeyboardShortcut {
    pub key: String,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub mac_cmd: bool,
    #[serde(default)]
    pub namespace: bool,
}

impl KeyboardShortcut {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            shift: false,
            ctrl: false,
            alt: false,
            mac_cmd: false,
            namespace: false,
        }
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_mac_cmd(mut self) -> Self {
        self.mac_cmd = true;
        self
    }

    pub fn with_namespace(mut self) -> Self {
        self.namespace = true;
        self
    }

    // Convert a string key name to egui::Key
    pub fn to_egui_key(&self) -> Option<Key> {
        match self.key.as_str() {
            "a" | "A" => Some(Key::A),
            "b" | "B" => Some(Key::B),
            "c" | "C" => Some(Key::C),
            "d" | "D" => Some(Key::D),
            "e" | "E" => Some(Key::E),
            "f" | "F" => Some(Key::F),
            "g" | "G" => Some(Key::G),
            "h" | "H" => Some(Key::H),
            "i" | "I" => Some(Key::I),
            "j" | "J" => Some(Key::J),
            "k" | "K" => Some(Key::K),
            "l" | "L" => Some(Key::L),
            "m" | "M" => Some(Key::M),
            "n" | "N" => Some(Key::N),
            "o" | "O" => Some(Key::O),
            "p" | "P" => Some(Key::P),
            "q" | "Q" => Some(Key::Q),
            "r" | "R" => Some(Key::R),
            "s" | "S" => Some(Key::S),
            "t" | "T" => Some(Key::T),
            "u" | "U" => Some(Key::U),
            "v" | "V" => Some(Key::V),
            "w" | "W" => Some(Key::W),
            "x" | "X" => Some(Key::X),
            "y" | "Y" => Some(Key::Y),
            "z" | "Z" => Some(Key::Z),
            "0" => Some(Key::Num0),
            "1" => Some(Key::Num1),
            "2" => Some(Key::Num2),
            "3" => Some(Key::Num3),
            "4" => Some(Key::Num4),
            "5" => Some(Key::Num5),
            "6" => Some(Key::Num6),
            "7" => Some(Key::Num7),
            "8" => Some(Key::Num8),
            "9" => Some(Key::Num9),
            "escape" | "esc" => Some(Key::Escape),
            "enter" | "return" => Some(Key::Enter),
            "space" => Some(Key::Space),
            "tab" => Some(Key::Tab),
            "backspace" => Some(Key::Backspace),
            "insert" => Some(Key::Insert),
            "delete" => Some(Key::Delete),
            "home" => Some(Key::Home),
            "end" => Some(Key::End),
            "pageup" => Some(Key::PageUp),
            "pagedown" => Some(Key::PageDown),
            "left" | "arrow_left" => Some(Key::ArrowLeft),
            "right" | "arrow_right" => Some(Key::ArrowRight),
            "up" | "arrow_up" => Some(Key::ArrowUp),
            "down" | "arrow_down" => Some(Key::ArrowDown),
            "?" | "question" | "questionmark" => Some(Key::Questionmark),
            "/" | "slash" => Some(Key::Slash),
            "[" => Some(Key::OpenBracket),
            "]" => Some(Key::CloseBracket),
            "-" => Some(Key::Minus),
            key => {
                tracing::warn!("Unsupported key: {}", key);
                None
            }
        }
    }

    // Check if this shortcut matches the given key and modifiers
    pub fn matches(&self, key: Key, modifiers: Modifiers, namespace: bool) -> bool {
        let key_matches = match self.to_egui_key() {
            Some(shortcut_key) => shortcut_key == key,
            None => false,
        };

        key_matches
            && self.shift == modifiers.shift
            && self.ctrl == modifiers.ctrl
            && self.alt == modifiers.alt
            && self.mac_cmd == modifiers.mac_cmd
            && self.namespace == namespace
    }
}

// Define an enum for all possible shortcut actions
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    // Navigation
    MoveDown,
    MoveUp,
    GoToParentDirectory,
    OpenDirectory,
    OpenDirectoryOrFile,
    GoToFirstEntry,
    GoToLastEntry,
    GoBackInHistory,
    GoForwardInHistory,

    // File operations
    DeleteEntry,
    RenameEntry,
    AddEntry,
    SelectEntry,
    CopyEntry,
    CutEntry,
    PasteEntry,

    // Tabs
    CreateTab,
    SwitchToTab1,
    SwitchToTab2,
    SwitchToTab3,
    SwitchToTab4,
    SwitchToTab5,
    SwitchToTab6,
    SwitchToTab7,
    SwitchToTab8,
    SwitchToTab9,
    CloseCurrentTab,
    SwitchToNextTab,
    SwitchToPreviousTab,

    // Bookmarks
    ToggleBookmark,
    ShowBookmarks,

    // Utils
    OpenTerminal,
    ShowHelp,
    Exit,
    ActivateSearch,
}

// Define a struct to represent an egui key combination for efficient lookups
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EguiKeyCombo {
    pub key: Key,
    pub modifiers: Modifiers,
    pub namespace: bool,
}

// Define a struct for the shortcuts map to reduce nesting in config
// Contains both action->shortcuts mapping and shortcut->action mapping for efficient lookups
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Shortcuts {
    // Main mapping from action to list of shortcuts
    #[serde(flatten)]
    action_to_shortcuts: HashMap<ShortcutAction, Vec<KeyboardShortcut>>,
    // Direct mapping from egui key combination to action for O(1) lookups
    #[serde(skip)]
    key_to_action: HashMap<EguiKeyCombo, ShortcutAction>,
}

impl Shortcuts {
    pub fn new() -> Self {
        Self {
            action_to_shortcuts: HashMap::new(),
            key_to_action: HashMap::new(),
        }
    }

    pub fn get(&self, action: &ShortcutAction) -> Option<&Vec<KeyboardShortcut>> {
        self.action_to_shortcuts.get(action)
    }

    // Add a shortcut for an action, updating both maps
    pub fn add_shortcut(&mut self, shortcut: KeyboardShortcut, action: ShortcutAction) {
        // Add to action_to_shortcuts map
        self.action_to_shortcuts
            .entry(action)
            .or_default()
            .push(shortcut.clone());

        // Add to key_to_action map if possible
        if let Some(egui_key) = shortcut.to_egui_key() {
            let key_combo = EguiKeyCombo {
                key: egui_key,
                modifiers: Modifiers {
                    alt: shortcut.alt,
                    ctrl: shortcut.ctrl,
                    shift: shortcut.shift,
                    mac_cmd: shortcut.mac_cmd,
                    command: shortcut.mac_cmd,
                },
                namespace: shortcut.namespace,
            };
            self.key_to_action.insert(key_combo, action);
        }
    }

    // Set all shortcuts for an action, replacing any existing ones
    pub fn set_shortcuts(&mut self, action: ShortcutAction, shortcuts: Vec<KeyboardShortcut>) {
        // First, remove any existing shortcuts for this action from key_to_action
        if let Some(existing_shortcuts) = self.action_to_shortcuts.get(&action) {
            for shortcut in existing_shortcuts {
                if let Some(egui_key) = shortcut.to_egui_key() {
                    let key_combo = EguiKeyCombo {
                        key: egui_key,
                        modifiers: Modifiers {
                            alt: shortcut.alt,
                            ctrl: shortcut.ctrl,
                            shift: shortcut.shift,
                            mac_cmd: shortcut.mac_cmd,
                            command: shortcut.mac_cmd,
                        },
                        namespace: shortcut.namespace,
                    };
                    self.key_to_action.remove(&key_combo);
                }
            }
        }

        // Now add the new shortcuts
        self.action_to_shortcuts.insert(action, shortcuts.clone());

        // Update key_to_action map
        for shortcut in &shortcuts {
            if let Some(egui_key) = shortcut.to_egui_key() {
                let key_combo = EguiKeyCombo {
                    key: egui_key,
                    modifiers: Modifiers {
                        alt: shortcut.alt,
                        ctrl: shortcut.ctrl,
                        shift: shortcut.shift,
                        mac_cmd: shortcut.mac_cmd,
                        command: shortcut.mac_cmd,
                    },
                    namespace: shortcut.namespace,
                };
                self.key_to_action.insert(key_combo, action);
            }
        }
    }
}

// Implement IntoIterator for &Shortcuts to make it work with for loops
impl<'a> IntoIterator for &'a Shortcuts {
    type Item = (&'a ShortcutAction, &'a Vec<KeyboardShortcut>);
    type IntoIter = std::collections::hash_map::Iter<'a, ShortcutAction, Vec<KeyboardShortcut>>;

    fn into_iter(self) -> Self::IntoIter {
        self.action_to_shortcuts.iter()
    }
}

// Function to get default shortcuts
pub fn default_shortcuts() -> Shortcuts {
    let mut shortcuts = Shortcuts::new();

    // Helper function to add a shortcut
    let mut add_shortcut = |shortcut: KeyboardShortcut, action: ShortcutAction| {
        shortcuts.add_shortcut(shortcut, action);
    };

    // Navigation shortcuts
    add_shortcut(KeyboardShortcut::new("j"), ShortcutAction::MoveDown);
    add_shortcut(KeyboardShortcut::new("down"), ShortcutAction::MoveDown);

    add_shortcut(KeyboardShortcut::new("k"), ShortcutAction::MoveUp);
    add_shortcut(KeyboardShortcut::new("up"), ShortcutAction::MoveUp);

    add_shortcut(
        KeyboardShortcut::new("h"),
        ShortcutAction::GoToParentDirectory,
    );
    add_shortcut(
        KeyboardShortcut::new("-"),
        ShortcutAction::GoToParentDirectory,
    );
    add_shortcut(
        KeyboardShortcut::new("left"),
        ShortcutAction::GoToParentDirectory,
    );

    add_shortcut(KeyboardShortcut::new("l"), ShortcutAction::OpenDirectory);
    add_shortcut(
        KeyboardShortcut::new("right"),
        ShortcutAction::OpenDirectory,
    );
    add_shortcut(
        KeyboardShortcut::new("enter"),
        ShortcutAction::OpenDirectoryOrFile,
    );
    add_shortcut(
        KeyboardShortcut::new("o"),
        ShortcutAction::OpenDirectoryOrFile,
    );

    // Add a shortcut for g in namespace mode (after pressing g once)
    add_shortcut(
        KeyboardShortcut::new("g").with_namespace(),
        ShortcutAction::GoToFirstEntry,
    );

    add_shortcut(
        KeyboardShortcut::new("g").with_shift(),
        ShortcutAction::GoToLastEntry,
    );

    // History navigation
    add_shortcut(
        KeyboardShortcut::new("o").with_ctrl(),
        ShortcutAction::GoBackInHistory,
    );

    add_shortcut(
        KeyboardShortcut::new("i").with_ctrl(),
        ShortcutAction::GoForwardInHistory,
    );

    // File operations
    add_shortcut(KeyboardShortcut::new("d"), ShortcutAction::DeleteEntry);

    add_shortcut(KeyboardShortcut::new("r"), ShortcutAction::RenameEntry);

    add_shortcut(KeyboardShortcut::new("a"), ShortcutAction::AddEntry);

    add_shortcut(KeyboardShortcut::new("space"), ShortcutAction::SelectEntry);

    add_shortcut(KeyboardShortcut::new("y"), ShortcutAction::CopyEntry);
    add_shortcut(
        KeyboardShortcut::new("c").with_ctrl(),
        ShortcutAction::CopyEntry,
    );

    add_shortcut(KeyboardShortcut::new("x"), ShortcutAction::CutEntry);
    add_shortcut(
        KeyboardShortcut::new("x").with_ctrl(),
        ShortcutAction::CutEntry,
    );

    add_shortcut(KeyboardShortcut::new("p"), ShortcutAction::PasteEntry);
    add_shortcut(
        KeyboardShortcut::new("v").with_ctrl(),
        ShortcutAction::PasteEntry,
    );

    // Tabs
    add_shortcut(KeyboardShortcut::new("t"), ShortcutAction::CreateTab);

    add_shortcut(KeyboardShortcut::new("1"), ShortcutAction::SwitchToTab1);

    add_shortcut(KeyboardShortcut::new("2"), ShortcutAction::SwitchToTab2);

    add_shortcut(KeyboardShortcut::new("3"), ShortcutAction::SwitchToTab3);

    add_shortcut(KeyboardShortcut::new("4"), ShortcutAction::SwitchToTab4);

    add_shortcut(KeyboardShortcut::new("5"), ShortcutAction::SwitchToTab5);

    add_shortcut(KeyboardShortcut::new("6"), ShortcutAction::SwitchToTab6);

    add_shortcut(KeyboardShortcut::new("7"), ShortcutAction::SwitchToTab7);

    add_shortcut(KeyboardShortcut::new("8"), ShortcutAction::SwitchToTab8);

    add_shortcut(KeyboardShortcut::new("9"), ShortcutAction::SwitchToTab9);

    add_shortcut(
        KeyboardShortcut::new("q").with_ctrl(),
        ShortcutAction::CloseCurrentTab,
    );

    // Bookmarks
    add_shortcut(KeyboardShortcut::new("b"), ShortcutAction::ToggleBookmark);

    add_shortcut(
        KeyboardShortcut::new("b").with_shift(),
        ShortcutAction::ShowBookmarks,
    );

    // Utils
    add_shortcut(
        KeyboardShortcut::new("t").with_shift(),
        ShortcutAction::OpenTerminal,
    );

    add_shortcut(
        KeyboardShortcut::new("?").with_shift(),
        ShortcutAction::ShowHelp,
    );
    add_shortcut(KeyboardShortcut::new("?"), ShortcutAction::ShowHelp);

    add_shortcut(KeyboardShortcut::new("q"), ShortcutAction::Exit);

    add_shortcut(KeyboardShortcut::new("/"), ShortcutAction::ActivateSearch);
    add_shortcut(
        KeyboardShortcut::new("f").with_ctrl(),
        ShortcutAction::ActivateSearch,
    );

    // Add new shortcuts for switching to preview tab and next/previous tab
    add_shortcut(KeyboardShortcut::new("]"), ShortcutAction::SwitchToNextTab);
    add_shortcut(
        KeyboardShortcut::new("["),
        ShortcutAction::SwitchToPreviousTab,
    );

    shortcuts
}

// Create a static reference to default shortcuts for efficiency
use std::sync::OnceLock;

pub fn get_default_shortcuts() -> &'static Shortcuts {
    static DEFAULT_SHORTCUTS: OnceLock<Shortcuts> = OnceLock::new();
    DEFAULT_SHORTCUTS.get_or_init(default_shortcuts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default_shortcuts_no_conflicts() {
        // Get the default shortcuts
        let shortcuts = default_shortcuts();

        // Create a map to track which shortcut is assigned to which action
        let mut shortcut_map: HashMap<KeyboardShortcut, Vec<ShortcutAction>> = HashMap::new();

        // Populate the map
        for (action, shortcuts_list) in &shortcuts {
            for shortcut in shortcuts_list {
                shortcut_map
                    .entry(shortcut.clone())
                    .or_default()
                    .push(*action);
            }
        }

        // Check for conflicts (shortcuts assigned to multiple actions)
        let mut conflicts = Vec::new();
        for (shortcut, actions) in &shortcut_map {
            if actions.len() > 1 {
                conflicts.push((shortcut.clone(), actions.clone()));
            }
        }

        // Format error message if conflicts are found
        if !conflicts.is_empty() {
            let mut error_msg = String::from("Conflicts found in default shortcuts:\n");
            for (shortcut, actions) in conflicts {
                let mut shortcut_str = format!("Key: {}", shortcut.key);
                if shortcut.shift {
                    shortcut_str.push_str(", Shift");
                }
                if shortcut.ctrl {
                    shortcut_str.push_str(", Ctrl");
                }
                if shortcut.alt {
                    shortcut_str.push_str(", Alt");
                }
                if shortcut.mac_cmd {
                    shortcut_str.push_str(", Cmd");
                }
                if shortcut.namespace {
                    shortcut_str.push_str(", Namespace");
                }

                error_msg.push_str(&format!("\n{} is assigned to: ", shortcut_str));
                for action in actions {
                    error_msg.push_str(&format!("{:?}, ", action));
                }
            }

            panic!("{}", error_msg);
        }
    }
}

// Helper functions for the Shortcuts type
pub mod shortcuts_helpers {
    use super::*;

    // Find the action for a given key and modifiers
    pub fn find_action(
        shortcuts: &Shortcuts,
        key: Key,
        modifiers: Modifiers,
        namespace: bool,
    ) -> Option<ShortcutAction> {
        // Create a key combo for direct lookup
        let key_combo = EguiKeyCombo {
            key,
            modifiers,
            namespace,
        };

        // Use the direct key_to_action mapping for O(1) lookup
        shortcuts.key_to_action.get(&key_combo).copied()
    }

    // Get a human-readable representation of shortcuts for an action
    pub fn get_shortcut_display(shortcuts: &Shortcuts, action: ShortcutAction) -> String {
        let action_shortcuts = shortcuts
            .get(&action)
            .map(|v| v.as_slice())
            .unwrap_or_else(|| &[]);
        if action_shortcuts.is_empty() {
            return String::from("Not assigned");
        }

        action_shortcuts
            .iter()
            .map(|shortcut| {
                let mut parts = Vec::new();

                // Add namespace prefix if applicable
                if shortcut.namespace {
                    parts.push("g".to_string());
                }

                if shortcut.ctrl {
                    parts.push("Ctrl".to_string());
                }

                if shortcut.alt {
                    parts.push("Alt".to_string());
                }

                if shortcut.shift {
                    parts.push("Shift".to_string());
                }

                if shortcut.mac_cmd {
                    parts.push("Cmd".to_string());
                }

                // Convert arrow keys to Unicode arrow emojis
                let key_lower = shortcut.key.to_lowercase();
                let key_display = match key_lower.as_str() {
                    "up" | "arrow_up" => "⬆".to_string(),
                    "down" | "arrow_down" => "⬇".to_string(),
                    "left" | "arrow_left" => "⬅".to_string(),
                    "right" | "arrow_right" => "➡".to_string(),
                    "enter" => "Enter".to_string(),
                    "space" => "Space".to_string(),
                    _ => key_lower,
                };
                parts.push(key_display);

                parts.join("+")
            })
            .collect::<Vec<_>>()
            .join(" or ")
    }
}
