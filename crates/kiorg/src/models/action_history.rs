use chrono::{DateTime, Local};
use std::path::PathBuf;

/// Individual operation data structures
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateOperation {
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameOperation {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyOperation {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveOperation {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

/// Represents different types of mutation actions that can be performed on files/directories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    /// File or directory creation operations
    Create { operations: Vec<CreateOperation> },
    /// File or directory rename/move operations
    Rename { operations: Vec<RenameOperation> },
    /// File copy operations
    Copy { operations: Vec<CopyOperation> },
    /// File move operations (different from rename as it can cross directory boundaries)
    Move { operations: Vec<MoveOperation> },
}

/// Represents a single action in the history with metadata
#[derive(Debug, Clone)]
pub struct HistoryAction {
    /// Type of action performed
    pub action_type: ActionType,
    /// Timestamp when the action was performed
    pub timestamp: DateTime<Local>,
}

impl HistoryAction {
    /// Get the description for this action
    pub fn get_description(&self) -> String {
        TabActionHistory::generate_description(&self.action_type)
    }
}

/// History manager for a single tab
#[derive(Debug, Clone)]
pub struct TabActionHistory {
    /// Actions that have not been rolled back
    active_actions: Vec<HistoryAction>,
    /// Actions that have been rolled back
    rolled_back_actions: Vec<HistoryAction>,
    /// Maximum number of actions to keep in history
    max_history_size: usize,
}

impl Default for TabActionHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl TabActionHistory {
    /// Create a new empty action history
    pub fn new() -> Self {
        Self::with_max_size(256)
    }

    /// Create a new action history with custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            active_actions: Vec::new(),
            rolled_back_actions: Vec::new(),
            max_history_size: max_size,
        }
    }

    /// Add a new action to the history
    pub fn add_action(&mut self, action_type: ActionType) {
        let action = HistoryAction {
            action_type,
            timestamp: Local::now(),
        };

        self.active_actions.push(action);
        self.rolled_back_actions.truncate(0);

        // Maintain max history size
        let overflow = self.active_actions.len() as i64 - self.max_history_size as i64;
        if overflow > 0 {
            self.active_actions.drain(0..(overflow as usize));
        }
    }

    /// Get the most recent action that hasn't been rolled back
    pub fn get_last_rollbackable_action(&self) -> Option<&HistoryAction> {
        self.active_actions.last()
    }

    /// Check if there are any rolled back actions
    pub fn has_rolled_back_actions(&self) -> bool {
        !self.rolled_back_actions.is_empty()
    }

    /// Undo the last action (linear undo)
    pub fn undo_last_action(&mut self) -> Option<HistoryAction> {
        if let Some(action) = self.active_actions.pop() {
            self.rolled_back_actions.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Redo the most recently rolled back action
    pub fn redo_last_action(&mut self) -> Option<HistoryAction> {
        if let Some(action) = self.rolled_back_actions.pop() {
            self.active_actions.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    /// Get the most recent rolled back action that can be redone
    pub fn get_last_redoable_action(&self) -> Option<&HistoryAction> {
        self.rolled_back_actions.last()
    }

    /// Get active actions only
    pub fn get_active_actions(&self) -> &[HistoryAction] {
        &self.active_actions
    }

    /// Get rolled back actions only
    pub fn get_rolled_back_actions(&self) -> &[HistoryAction] {
        &self.rolled_back_actions
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.active_actions.clear();
        self.rolled_back_actions.clear();
    }

    /// Get the number of actions in history (active + rolled back)
    pub fn len(&self) -> usize {
        self.active_actions.len() + self.rolled_back_actions.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.active_actions.is_empty() && self.rolled_back_actions.is_empty()
    }

    /// Generate a human-readable description for an action
    pub fn generate_description(action_type: &ActionType) -> String {
        match action_type {
            ActionType::Create { operations } => {
                if operations.len() == 1 {
                    format!("Created '{}'", operations[0].path.display())
                } else {
                    format!("Created {} items", operations.len())
                }
            }
            ActionType::Rename { operations } => {
                if operations.len() == 1 {
                    format!(
                        "Renamed '{}' to '{}'",
                        operations[0].old_path.display(),
                        operations[0].new_path.display()
                    )
                } else {
                    format!("Renamed {} items", operations.len())
                }
            }
            ActionType::Copy { operations } => {
                if operations.len() == 1 {
                    format!(
                        "Copied '{}' to '{}'",
                        operations[0].source_path.display(),
                        operations[0].target_path.display()
                    )
                } else {
                    format!("Copied {} items", operations.len())
                }
            }
            ActionType::Move { operations } => {
                if operations.len() == 1 {
                    format!(
                        "Moved '{}' to '{}'",
                        operations[0].source_path.display(),
                        operations[0].target_path.display()
                    )
                } else {
                    format!("Moved {} items", operations.len())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_history_basic_operations() {
        let mut history = TabActionHistory::new();

        // Test adding actions
        history.add_action(ActionType::Create {
            operations: vec![CreateOperation {
                path: PathBuf::from("test.txt"),
                is_dir: false,
            }],
        });

        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());

        // Test that we have active actions
        assert!(history.get_last_rollbackable_action().is_some());
        assert!(!history.has_rolled_back_actions());

        // Test linear undo
        let undone_action = history.undo_last_action();
        assert!(undone_action.is_some());
        assert!(history.has_rolled_back_actions());
    }

    #[test]
    fn test_action_descriptions() {
        let create_action = ActionType::Create {
            operations: vec![CreateOperation {
                path: PathBuf::from("test.txt"),
                is_dir: false,
            }],
        };
        assert_eq!(
            TabActionHistory::generate_description(&create_action),
            "Created 'test.txt'"
        );

        let rename_action = ActionType::Rename {
            operations: vec![RenameOperation {
                old_path: PathBuf::from("old.txt"),
                new_path: PathBuf::from("new.txt"),
            }],
        };
        assert_eq!(
            TabActionHistory::generate_description(&rename_action),
            "Renamed 'old.txt' to 'new.txt'"
        );
    }

    #[test]
    fn test_rollback_capability() {
        let mut history = TabActionHistory::new();

        let create_action = ActionType::Create {
            operations: vec![CreateOperation {
                path: PathBuf::from("test.txt"),
                is_dir: false,
            }],
        };
        // Create actions can be rolled back, so should be recorded
        history.add_action(create_action.clone());

        // Should have 1 recorded action
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_max_history_size() {
        let mut history = TabActionHistory::with_max_size(3);

        // Add more actions than max size
        for i in 0..5 {
            history.add_action(ActionType::Create {
                operations: vec![CreateOperation {
                    path: PathBuf::from(format!("test{}.txt", i)),
                    is_dir: false,
                }],
            });
        }

        // Should only keep the last 3 actions
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_undo_redo_functionality() {
        let mut history = TabActionHistory::new();

        // Add an action
        let create_action = ActionType::Create {
            operations: vec![CreateOperation {
                path: PathBuf::from("test.txt"),
                is_dir: false,
            }],
        };
        history.add_action(create_action);

        // Initially no rolled back actions
        assert!(!history.has_rolled_back_actions());
        assert!(history.get_last_rollbackable_action().is_some());
        assert!(history.get_last_redoable_action().is_none());

        // Undo the action
        let undone_action = history.undo_last_action();
        assert!(undone_action.is_some());
        assert!(history.has_rolled_back_actions());
        assert!(history.get_last_rollbackable_action().is_none());
        assert!(history.get_last_redoable_action().is_some());

        // Redo the action
        let redone_action = history.redo_last_action();
        assert!(redone_action.is_some());
        assert!(!history.has_rolled_back_actions());
        assert!(history.get_last_rollbackable_action().is_some());
        assert!(history.get_last_redoable_action().is_none());

        // Verify the action descriptions match
        assert_eq!(
            undone_action.unwrap().get_description(),
            redone_action.unwrap().get_description()
        );
    }
}
