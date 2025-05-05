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
            _ => None,
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

    // Bookmarks
    ToggleBookmark,
    ShowBookmarks,

    // Utils
    OpenTerminal,
    ShowHelp,
    Exit,
    ActivateSearch,
}

// Define a type alias for the shortcuts map to reduce nesting in config
// Now keyed by ShortcutAction for more intuitive configuration
pub type Shortcuts = HashMap<ShortcutAction, Vec<KeyboardShortcut>>;

// Function to get default shortcuts
pub fn default_shortcuts() -> Shortcuts {
    let mut shortcuts: Shortcuts = HashMap::new();

    // Helper function to add a shortcut
    let mut add_shortcut = |shortcut: KeyboardShortcut, action: ShortcutAction| {
        shortcuts.entry(action).or_default().push(shortcut);
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

    add_shortcut(KeyboardShortcut::new("x"), ShortcutAction::CutEntry);

    add_shortcut(KeyboardShortcut::new("p"), ShortcutAction::PasteEntry);

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
        KeyboardShortcut::new("d").with_ctrl(),
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

    shortcuts
}

// Create a static reference to default shortcuts for efficiency
use std::sync::OnceLock;

pub fn get_default_shortcuts() -> &'static Shortcuts {
    static DEFAULT_SHORTCUTS: OnceLock<Shortcuts> = OnceLock::new();
    DEFAULT_SHORTCUTS.get_or_init(default_shortcuts)
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
        // Convert the egui Key to a string representation
        let key_str = match key {
            Key::A => "a",
            Key::B => "b",
            Key::C => "c",
            Key::D => "d",
            Key::E => "e",
            Key::F => "f",
            Key::G => "g",
            Key::H => "h",
            Key::I => "i",
            Key::J => "j",
            Key::K => "k",
            Key::L => "l",
            Key::M => "m",
            Key::N => "n",
            Key::O => "o",
            Key::P => "p",
            Key::Q => "q",
            Key::R => "r",
            Key::S => "s",
            Key::T => "t",
            Key::U => "u",
            Key::V => "v",
            Key::W => "w",
            Key::X => "x",
            Key::Y => "y",
            Key::Z => "z",
            Key::Num0 => "0",
            Key::Num1 => "1",
            Key::Num2 => "2",
            Key::Num3 => "3",
            Key::Num4 => "4",
            Key::Num5 => "5",
            Key::Num6 => "6",
            Key::Num7 => "7",
            Key::Num8 => "8",
            Key::Num9 => "9",
            Key::Escape => "escape",
            Key::Enter => "enter",
            Key::Space => "space",
            Key::Tab => "tab",
            Key::Backspace => "backspace",
            Key::Insert => "insert",
            Key::Delete => "delete",
            Key::Home => "home",
            Key::End => "end",
            Key::PageUp => "pageup",
            Key::PageDown => "pagedown",
            Key::ArrowLeft => "left",
            Key::ArrowRight => "right",
            Key::ArrowUp => "up",
            Key::ArrowDown => "down",
            Key::Questionmark => "?",
            Key::Slash => "/",
            _ => return None, // Unsupported key
        };

        // Search through all actions and their shortcuts
        for (action, shortcuts_list) in shortcuts {
            // Check if any shortcut in the list matches the current key and modifiers
            for shortcut in shortcuts_list {
                if shortcut.key == key_str
                    && shortcut.shift == modifiers.shift
                    && shortcut.ctrl == modifiers.ctrl
                    && shortcut.alt == modifiers.alt
                    && shortcut.mac_cmd == modifiers.mac_cmd
                    && shortcut.namespace == namespace
                {
                    return Some(*action);
                }
            }
        }

        None
    }

    // Get all shortcuts for a specific action
    pub fn get_shortcuts_for_action(
        shortcuts: &Shortcuts,
        action: ShortcutAction,
    ) -> Vec<KeyboardShortcut> {
        // Direct lookup in the HashMap
        shortcuts.get(&action).cloned().unwrap_or_default()
    }

    // Get a human-readable representation of shortcuts for an action
    pub fn get_shortcut_display(shortcuts: &Shortcuts, action: ShortcutAction) -> String {
        let action_shortcuts = get_shortcuts_for_action(shortcuts, action);
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
            .join(" / ")
    }
}
