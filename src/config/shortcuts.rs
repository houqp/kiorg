use egui::{Key, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Node in the shortcut prefix tree
#[derive(Debug, Clone)]
pub enum ShortcutTreeNode {
    // A parent node that can have children but no action
    Children(HashMap<ShortcutKey, ShortcutTreeNode>),
    // A leaf node that has an action but no children
    Action(ShortcutAction),
}

// Represents a single key with modifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortcutKey {
    pub key: Key,
    pub modifiers: Modifiers,
}

// Result of traversing a key buffer through the shortcut tree
#[derive(Debug, Clone, PartialEq)]
pub enum TraverseResult {
    // Found a complete action to execute
    Action(ShortcutAction),
    // Partial match - wait for more keys
    Partial,
    // No match found
    NoMatch,
}

impl ShortcutTreeNode {
    pub fn new() -> Self {
        Self::Children(HashMap::new())
    }
}

impl Default for ShortcutTreeNode {
    fn default() -> Self {
        Self::new()
    }
}

// Define a struct to represent a keyboard shortcut
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct KeyboardShortcut {
    pub key: String, // Now supports multi-character sequences like "gg"
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[cfg(target_os = "macos")]
    #[serde(default)]
    pub command: bool,
}

impl KeyboardShortcut {
    #[must_use]
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            shift: false,
            ctrl: false,
            alt: false,
            #[cfg(target_os = "macos")]
            command: false,
        }
    }

    #[must_use]
    pub const fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    #[must_use]
    pub const fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    #[must_use]
    pub const fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    #[cfg(target_os = "macos")]
    #[must_use]
    pub const fn with_cmd(mut self) -> Self {
        self.command = true;
        self
    }

    // Convert the key sequence to a vector of ShortcutKey structs
    pub fn to_shortcut_keys(&self) -> Result<Vec<ShortcutKey>, String> {
        let mut keys = Vec::new();
        let modifiers = Modifiers {
            alt: self.alt,
            ctrl: self.ctrl,
            shift: self.shift,
            #[cfg(not(target_os = "macos"))]
            mac_cmd: false,
            #[cfg(target_os = "macos")]
            mac_cmd: self.command,
            #[cfg(not(target_os = "macos"))]
            command: self.ctrl,
            #[cfg(target_os = "macos")]
            command: self.command,
        };

        // Handle special key names first
        if let Some(special_key) = Self::parse_special_key(&self.key) {
            keys.push(ShortcutKey {
                key: special_key,
                modifiers,
            });
            return Ok(keys);
        }

        // Handle character sequences like "gg", "gd", etc.
        for c in self.key.chars() {
            if let Some(egui_key) = Self::char_to_egui_key(c) {
                // Apply modifiers to every key in the sequence
                keys.push(ShortcutKey {
                    key: egui_key,
                    modifiers,
                });
            } else {
                return Err(format!("Unsupported character in key sequence: '{}'", c));
            }
        }

        if keys.is_empty() {
            Err("Empty key sequence".to_string())
        } else {
            Ok(keys)
        }
    }

    // Parse special key names (like "escape", "enter", etc.)
    fn parse_special_key(key_str: &str) -> Option<Key> {
        match key_str {
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
            "," => Some(Key::Comma),
            _ => None,
        }
    }

    // Convert a single character to egui::Key
    fn char_to_egui_key(c: char) -> Option<Key> {
        match c {
            'a' | 'A' => Some(Key::A),
            'b' | 'B' => Some(Key::B),
            'c' | 'C' => Some(Key::C),
            'd' | 'D' => Some(Key::D),
            'e' | 'E' => Some(Key::E),
            'f' | 'F' => Some(Key::F),
            'g' | 'G' => Some(Key::G),
            'h' | 'H' => Some(Key::H),
            'i' | 'I' => Some(Key::I),
            'j' | 'J' => Some(Key::J),
            'k' | 'K' => Some(Key::K),
            'l' | 'L' => Some(Key::L),
            'm' | 'M' => Some(Key::M),
            'n' | 'N' => Some(Key::N),
            'o' | 'O' => Some(Key::O),
            'p' | 'P' => Some(Key::P),
            'q' | 'Q' => Some(Key::Q),
            'r' | 'R' => Some(Key::R),
            's' | 'S' => Some(Key::S),
            't' | 'T' => Some(Key::T),
            'u' | 'U' => Some(Key::U),
            'v' | 'V' => Some(Key::V),
            'w' | 'W' => Some(Key::W),
            'x' | 'X' => Some(Key::X),
            'y' | 'Y' => Some(Key::Y),
            'z' | 'Z' => Some(Key::Z),
            '0' => Some(Key::Num0),
            '1' => Some(Key::Num1),
            '2' => Some(Key::Num2),
            '3' => Some(Key::Num3),
            '4' => Some(Key::Num4),
            '5' => Some(Key::Num5),
            '6' => Some(Key::Num6),
            '7' => Some(Key::Num7),
            '8' => Some(Key::Num8),
            '9' => Some(Key::Num9),
            '?' => Some(Key::Questionmark),
            '/' => Some(Key::Slash),
            '[' => Some(Key::OpenBracket),
            ']' => Some(Key::CloseBracket),
            '-' => Some(Key::Minus),
            ',' => Some(Key::Comma),
            _ => {
                tracing::warn!("Unsupported character: {}", c);
                None
            }
        }
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
    SwitchToNextTab,
    SwitchToPreviousTab,
    PageUp,
    PageDown,

    // File operations
    DeleteEntry,
    RenameEntry,
    AddEntry,
    SelectEntry,
    SelectAllEntries,
    CopyEntry,
    CutEntry,
    PasteEntry,
    OpenWithCommand,

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

    #[cfg(target_os = "windows")]
    ShowWindowsDrives,

    #[cfg(target_os = "macos")]
    ShowVolumes,

    // UI interaction
    ActivateSearch,
    ShowHelp,
    OpenTerminal,
    ShowFilePreview,
    ShowTeleport,
    ShowSortToggle,
    ShowActionHistory,
    Undo,
    Redo,
    Exit,
    ToggleRangeSelection,
    ToggleHiddenFiles,
    CopyPath,
    CopyName,
}

// Define a struct for the shortcuts map using a prefix tree
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Shortcuts {
    // Main mapping from action to list of shortcuts (for serialization and display)
    #[serde(flatten)]
    action_to_shortcuts: HashMap<ShortcutAction, Vec<KeyboardShortcut>>,
    // Prefix tree for efficient multi-character shortcut matching
    #[serde(skip)]
    shortcut_tree: ShortcutTreeNode,
}

impl Shortcuts {
    #[must_use]
    pub fn new() -> Self {
        Self {
            action_to_shortcuts: HashMap::new(),
            shortcut_tree: ShortcutTreeNode::new(),
        }
    }

    #[must_use]
    pub fn get(&self, action: &ShortcutAction) -> Option<&Vec<KeyboardShortcut>> {
        self.action_to_shortcuts.get(action)
    }

    // Add a shortcut for an action, updating both the action map and tree
    pub fn add_shortcut(
        &mut self,
        shortcut: KeyboardShortcut,
        action: ShortcutAction,
    ) -> Result<(), String> {
        // Add to shortcut tree first to detect conflicts immediately
        if let Ok(keys) = shortcut.to_shortcut_keys() {
            self.insert_into_tree(&keys, action)?;
        }

        // Only add to action_to_shortcuts map if tree insertion succeeded
        self.action_to_shortcuts
            .entry(action)
            .or_default()
            .push(shortcut.clone());

        Ok(())
    }

    // Helper method to insert a key sequence into the prefix tree
    fn insert_into_tree(
        &mut self,
        keys: &[ShortcutKey],
        action: ShortcutAction,
    ) -> Result<(), String> {
        let mut current_node = &mut self.shortcut_tree;

        for (i, key) in keys.iter().enumerate() {
            if i == keys.len() - 1 {
                // Last key in sequence - insert the action
                match current_node {
                    ShortcutTreeNode::Children(children) => {
                        // Check if this key already exists and is an action (conflict)
                        if let Some(existing_node) = children.get(key) {
                            if matches!(existing_node, ShortcutTreeNode::Action(_)) {
                                // This is a prefix conflict - action already exists at this key
                                return Err(
                                    "Shortcut conflict: key sequence conflicts with existing shortcut".to_string()
                                );
                            } else {
                                // Existing parent node - this creates a conflict
                                return Err(
                                    "Prefix conflict: shortcut conflicts with existing longer shortcut".to_string()
                                );
                            }
                        } else {
                            children.insert(key.clone(), ShortcutTreeNode::Action(action));
                        }
                    }
                    ShortcutTreeNode::Action(_) => {
                        // Cannot insert into an action node - this is a structural conflict
                        return Err(
                            "Prefix conflict: cannot insert shortcut through existing action node"
                                .to_string(),
                        );
                    }
                }
            } else {
                // Not the last key - ensure we have a parent node to traverse into
                match current_node {
                    ShortcutTreeNode::Children(children) => {
                        let entry = children
                            .entry(key.clone())
                            .or_insert_with(ShortcutTreeNode::new);
                        current_node = entry;
                    }
                    ShortcutTreeNode::Action(_) => {
                        // Cannot traverse through an action node - this is a prefix conflict
                        return Err(
                            "Prefix conflict: cannot traverse through existing action node"
                                .to_string(),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    // Traverse the shortcut tree with a key buffer, returning the result in a single traversal
    #[must_use]
    pub fn traverse_tree(&self, key_buffer: &[ShortcutKey]) -> TraverseResult {
        let mut current_node = &self.shortcut_tree;

        for shortcut_key in key_buffer {
            match current_node {
                ShortcutTreeNode::Children(children) => {
                    if let Some(child_node) = children.get(shortcut_key) {
                        current_node = child_node;
                    } else {
                        // No matching path in tree
                        return TraverseResult::NoMatch;
                    }
                }
                ShortcutTreeNode::Action(_) => {
                    // Cannot traverse through action node
                    return TraverseResult::NoMatch;
                }
            }
        }

        // Check what we found at the end of traversal
        match current_node {
            ShortcutTreeNode::Action(action) => TraverseResult::Action(*action),
            ShortcutTreeNode::Children(children) => {
                if children.is_empty() {
                    TraverseResult::NoMatch
                } else {
                    TraverseResult::Partial
                }
            }
        }
    }

    // Set all shortcuts for an action, replacing any existing ones
    pub fn set_shortcuts(
        &mut self,
        action: ShortcutAction,
        shortcuts: Vec<KeyboardShortcut>,
    ) -> Result<(), String> {
        // Update the action_to_shortcuts map
        self.action_to_shortcuts.insert(action, shortcuts);

        // Rebuild the entire tree since we can't easily remove specific entries
        self.rebuild_tree()
    }

    pub fn add_shortcuts(
        &mut self,
        action: ShortcutAction,
        mut shortcuts: Vec<KeyboardShortcut>,
    ) -> Result<(), String> {
        // Add to existing shortcuts instead of replacing them
        if let Some(existing_shortcuts) = self.action_to_shortcuts.get_mut(&action) {
            existing_shortcuts.append(&mut shortcuts);
        } else {
            self.action_to_shortcuts.insert(action, shortcuts);
        }

        // Rebuild the entire tree since we can't easily remove specific entries
        self.rebuild_tree()
    }

    // Rebuild the entire shortcut tree from the action_to_shortcuts map
    fn rebuild_tree(&mut self) -> Result<(), String> {
        self.shortcut_tree = ShortcutTreeNode::new();

        // Collect all the actions and shortcuts to avoid borrowing issues
        let shortcuts_to_insert: Vec<(ShortcutAction, Vec<ShortcutKey>)> = self
            .action_to_shortcuts
            .iter()
            .flat_map(|(action, shortcuts)| {
                shortcuts.iter().filter_map(|shortcut| {
                    shortcut.to_shortcut_keys().ok().map(|keys| (*action, keys))
                })
            })
            .collect();

        for (action, keys) in shortcuts_to_insert {
            // Propagate conflicts immediately to the user
            self.insert_into_tree(&keys, action)?;
        }

        Ok(())
    }

    // Ensure tree is built after deserialization
    pub fn ensure_tree_built(&mut self) -> Result<(), String> {
        // Check if tree is empty (happens after deserialization)
        let tree_is_empty = match &self.shortcut_tree {
            ShortcutTreeNode::Children(children) => children.is_empty(),
            ShortcutTreeNode::Action(_) => {
                // Root node should never be an action node - this indicates a structural error
                return Err(
                    "Invalid shortcut definition resulting in action without associated key"
                        .to_string(),
                );
            }
        };

        if tree_is_empty && !self.action_to_shortcuts.is_empty() {
            self.rebuild_tree()?;
        }

        Ok(())
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
#[must_use]
pub fn default_shortcuts() -> Shortcuts {
    let mut shortcuts = Shortcuts::new();

    // Helper function to add a shortcut
    let mut add_shortcut = |shortcut: KeyboardShortcut, action: ShortcutAction| {
        if let Err(e) = shortcuts.add_shortcut(shortcut, action) {
            panic!("Default shortcut conflict: {}", e);
        }
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

    // Preview file in popup
    add_shortcut(
        KeyboardShortcut::new("k").with_shift(),
        ShortcutAction::ShowFilePreview,
    );
    add_shortcut(
        KeyboardShortcut::new("enter"),
        ShortcutAction::OpenDirectoryOrFile,
    );
    add_shortcut(
        KeyboardShortcut::new("o"),
        ShortcutAction::OpenDirectoryOrFile,
    );

    // Go to first entry with "gg"
    add_shortcut(KeyboardShortcut::new("gg"), ShortcutAction::GoToFirstEntry);

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

    add_shortcut(
        KeyboardShortcut::new("a").with_ctrl(),
        ShortcutAction::SelectAllEntries,
    );

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

    // Tab switching shortcuts: Ctrl+number on Windows/Linux, Cmd+number on Mac
    #[cfg(target_os = "macos")]
    {
        add_shortcut(
            KeyboardShortcut::new("1").with_cmd(),
            ShortcutAction::SwitchToTab1,
        );
        add_shortcut(
            KeyboardShortcut::new("2").with_cmd(),
            ShortcutAction::SwitchToTab2,
        );
        add_shortcut(
            KeyboardShortcut::new("3").with_cmd(),
            ShortcutAction::SwitchToTab3,
        );
        add_shortcut(
            KeyboardShortcut::new("4").with_cmd(),
            ShortcutAction::SwitchToTab4,
        );
        add_shortcut(
            KeyboardShortcut::new("5").with_cmd(),
            ShortcutAction::SwitchToTab5,
        );
        add_shortcut(
            KeyboardShortcut::new("6").with_cmd(),
            ShortcutAction::SwitchToTab6,
        );
        add_shortcut(
            KeyboardShortcut::new("7").with_cmd(),
            ShortcutAction::SwitchToTab7,
        );
        add_shortcut(
            KeyboardShortcut::new("8").with_cmd(),
            ShortcutAction::SwitchToTab8,
        );
        add_shortcut(
            KeyboardShortcut::new("9").with_cmd(),
            ShortcutAction::SwitchToTab9,
        );
    }
    #[cfg(not(target_os = "macos"))]
    {
        add_shortcut(
            KeyboardShortcut::new("1").with_ctrl(),
            ShortcutAction::SwitchToTab1,
        );
        add_shortcut(
            KeyboardShortcut::new("2").with_ctrl(),
            ShortcutAction::SwitchToTab2,
        );
        add_shortcut(
            KeyboardShortcut::new("3").with_ctrl(),
            ShortcutAction::SwitchToTab3,
        );
        add_shortcut(
            KeyboardShortcut::new("4").with_ctrl(),
            ShortcutAction::SwitchToTab4,
        );
        add_shortcut(
            KeyboardShortcut::new("5").with_ctrl(),
            ShortcutAction::SwitchToTab5,
        );
        add_shortcut(
            KeyboardShortcut::new("6").with_ctrl(),
            ShortcutAction::SwitchToTab6,
        );
        add_shortcut(
            KeyboardShortcut::new("7").with_ctrl(),
            ShortcutAction::SwitchToTab7,
        );
        add_shortcut(
            KeyboardShortcut::new("8").with_ctrl(),
            ShortcutAction::SwitchToTab8,
        );
        add_shortcut(
            KeyboardShortcut::new("9").with_ctrl(),
            ShortcutAction::SwitchToTab9,
        );
    }

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

    // Volumes
    #[cfg(target_os = "macos")]
    add_shortcut(
        KeyboardShortcut::new("v").with_ctrl().with_shift(),
        ShortcutAction::ShowVolumes,
    );

    // Drives (Windows equivalent of volumes)
    #[cfg(target_os = "windows")]
    add_shortcut(
        KeyboardShortcut::new("d").with_ctrl().with_shift(),
        ShortcutAction::ShowWindowsDrives,
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
    add_shortcut(KeyboardShortcut::new("esc"), ShortcutAction::Exit);

    add_shortcut(KeyboardShortcut::new("/"), ShortcutAction::ActivateSearch);
    add_shortcut(
        KeyboardShortcut::new("f").with_ctrl(),
        ShortcutAction::ActivateSearch,
    );
    add_shortcut(
        KeyboardShortcut::new("p").with_ctrl(),
        ShortcutAction::ShowTeleport,
    );

    // Action history shortcuts
    add_shortcut(
        KeyboardShortcut::new("h").with_ctrl().with_shift(),
        ShortcutAction::ShowActionHistory,
    );
    add_shortcut(KeyboardShortcut::new("u"), ShortcutAction::Undo);
    add_shortcut(KeyboardShortcut::new("r").with_ctrl(), ShortcutAction::Redo);

    // Add new shortcuts for switching to preview tab and next/previous tab
    add_shortcut(KeyboardShortcut::new("]"), ShortcutAction::SwitchToNextTab);
    add_shortcut(
        KeyboardShortcut::new("["),
        ShortcutAction::SwitchToPreviousTab,
    );

    // Add shortcut for opening files with custom command
    add_shortcut(
        KeyboardShortcut::new("o").with_shift(),
        ShortcutAction::OpenWithCommand,
    );

    add_shortcut(
        KeyboardShortcut::new("u").with_ctrl(),
        ShortcutAction::PageUp,
    );
    add_shortcut(
        KeyboardShortcut::new("d").with_ctrl(),
        ShortcutAction::PageDown,
    );
    add_shortcut(KeyboardShortcut::new("pageup"), ShortcutAction::PageUp);
    add_shortcut(KeyboardShortcut::new("pagedown"), ShortcutAction::PageDown);
    add_shortcut(
        KeyboardShortcut::new("v"),
        ShortcutAction::ToggleRangeSelection,
    );
    add_shortcut(KeyboardShortcut::new(","), ShortcutAction::ShowSortToggle);
    // Add the new shortcut for toggling hidden files
    add_shortcut(
        KeyboardShortcut::new("h").with_ctrl(),
        ShortcutAction::ToggleHiddenFiles,
    );

    // Copy operations to system clipboard
    add_shortcut(KeyboardShortcut::new("cp"), ShortcutAction::CopyPath);
    add_shortcut(KeyboardShortcut::new("cn"), ShortcutAction::CopyName);

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

    #[test]
    fn test_default_shortcuts_no_conflicts() {
        // this will panic if default shortcuts have conflicts
        let _ = default_shortcuts();
    }

    #[test]
    fn test_prefix_conflict_detection() {
        let mut shortcuts = Shortcuts::new();

        // Add conflicting shortcuts: "a" and "aa"
        shortcuts
            .add_shortcut(KeyboardShortcut::new("a"), ShortcutAction::AddEntry)
            .unwrap();
        let result = shortcuts.add_shortcut(KeyboardShortcut::new("aa"), ShortcutAction::MoveDown);
        match result {
            Ok(_) => {
                panic!("Expected prefix conflict to be detected");
            }
            Err(conflict_error) => {
                // The conflict should mention both shortcuts
                assert!(
                    conflict_error.contains("conflict") || conflict_error.contains("Conflict"),
                    "Expected error message to mention conflict, got: {conflict_error}"
                );
            }
        }
    }

    #[test]
    fn test_prefix_conflict_with_different_modifiers() {
        let mut shortcuts = Shortcuts::new();

        // Add shortcuts with different modifiers - these should NOT conflict
        shortcuts
            .add_shortcut(KeyboardShortcut::new("a"), ShortcutAction::AddEntry)
            .unwrap();
        let result = shortcuts.add_shortcut(
            KeyboardShortcut::new("aa").with_ctrl(),
            ShortcutAction::MoveDown,
        );
        assert!(
            result.is_ok(),
            "Expected no conflict due to different modifiers"
        );
    }

    #[test]
    fn test_longer_prefix_conflict() {
        let mut shortcuts = Shortcuts::new();

        // Add conflicting shortcuts: "bb" and "bbq"
        shortcuts
            .add_shortcut(KeyboardShortcut::new("bb"), ShortcutAction::ToggleBookmark)
            .unwrap();
        let result =
            shortcuts.add_shortcut(KeyboardShortcut::new("bbq"), ShortcutAction::ShowBookmarks);
        assert!(
            result.is_err(),
            "Expected prefix conflict to be detected for 'bb' and 'bbq'"
        );
    }

    #[test]
    fn test_special_key_no_prefix_conflict() {
        let mut shortcuts = Shortcuts::new();

        // Add shortcuts where one is a character and the other is a special key starting with same letter
        // "p" (character) and "pageup" (special key) should NOT conflict
        shortcuts
            .add_shortcut(KeyboardShortcut::new("p"), ShortcutAction::PasteEntry)
            .unwrap();
        let result =
            shortcuts.add_shortcut(KeyboardShortcut::new("pageup"), ShortcutAction::PageUp);
        assert!(
            result.is_ok(),
            "Expected no conflict between character 'p' and special key 'pageup'"
        );
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_command_modifier_matching_linux_windows() {
        use crate::config::shortcuts::TraverseResult;

        // Create a shortcuts map and add a shortcut for Ctrl+D
        let mut shortcuts = Shortcuts::new();
        let ctrl_d_shortcut = KeyboardShortcut::new("d").with_ctrl();
        shortcuts
            .add_shortcut(ctrl_d_shortcut, ShortcutAction::PageDown)
            .unwrap();

        // Simulate Linux/Windows modifiers where command should equal ctrl
        let linux_windows_modifiers = Modifiers {
            alt: false,
            ctrl: true,
            shift: false,
            mac_cmd: false,
            command: true, // On Linux/Windows, command should be set to same value as ctrl
        };

        // Test that traverse_tree can match the shortcut correctly
        let shortcut_key = ShortcutKey {
            key: Key::D,
            modifiers: linux_windows_modifiers,
        };
        let result = shortcuts.traverse_tree(&[shortcut_key]);

        assert_eq!(result, TraverseResult::Action(ShortcutAction::PageDown));

        // Also test that it doesn't match when ctrl/command are not pressed
        let no_modifiers = Modifiers {
            alt: false,
            ctrl: false,
            shift: false,
            mac_cmd: false,
            command: false,
        };

        let no_match_key = ShortcutKey {
            key: Key::D,
            modifiers: no_modifiers,
        };
        let no_match = shortcuts.traverse_tree(&[no_match_key]);

        assert_eq!(no_match, TraverseResult::NoMatch);

        // Test that it doesn't match when only command is pressed but not ctrl
        let command_only_modifiers = Modifiers {
            alt: false,
            ctrl: false,
            shift: false,
            mac_cmd: false,
            command: true,
        };

        let no_match_command_only_key = ShortcutKey {
            key: Key::D,
            modifiers: command_only_modifiers,
        };
        let no_match_command_only = shortcuts.traverse_tree(&[no_match_command_only_key]);

        assert_eq!(no_match_command_only, TraverseResult::NoMatch);
    }
}

// Helper functions for the Shortcuts type
pub mod shortcuts_helpers {
    use super::{ShortcutAction, Shortcuts};

    // Get a human-readable representation of shortcuts for an action
    #[must_use]
    pub fn get_shortcut_display(shortcuts: &Shortcuts, action: ShortcutAction) -> String {
        let action_shortcuts = shortcuts
            .get(&action)
            .map_or_else(|| &[], std::vec::Vec::as_slice);
        if action_shortcuts.is_empty() {
            return String::from("Not assigned");
        }

        action_shortcuts
            .iter()
            .map(|shortcut| {
                let mut parts = Vec::new();

                if shortcut.ctrl {
                    parts.push("Ctrl".to_string());
                }

                if shortcut.alt {
                    parts.push("Alt".to_string());
                }

                if shortcut.shift {
                    parts.push("Shift".to_string());
                }

                #[cfg(target_os = "macos")]
                if shortcut.command {
                    parts.push("Cmd".to_string());
                }

                // Handle the key display - could be multi-character like "gg" or special keys
                let key_display = {
                    let key_lower = shortcut.key.to_lowercase();
                    // First check for special key conversion regardless of length
                    match key_lower.as_str() {
                        "up" | "arrow_up" => "⬆".to_string(),
                        "down" | "arrow_down" => "⬇".to_string(),
                        "left" | "arrow_left" => "⬅".to_string(),
                        "right" | "arrow_right" => "➡".to_string(),
                        "enter" | "return" => "Enter".to_string(),
                        "space" => "Space".to_string(),
                        "esc" | "escape" => "Esc".to_string(),
                        "tab" => "Tab".to_string(),
                        "backspace" => "Backspace".to_string(),
                        "delete" => "Delete".to_string(),
                        "home" => "Home".to_string(),
                        "end" => "End".to_string(),
                        "pageup" => "PageUp".to_string(),
                        "pagedown" => "PageDown".to_string(),
                        "insert" => "Insert".to_string(),
                        // If not a special key, use the key as-is (could be multi-character like "gg")
                        _ => shortcut.key.clone(),
                    }
                };
                parts.push(key_display);

                parts.join("+")
            })
            .collect::<Vec<_>>()
            .join(" or ")
    }
}
