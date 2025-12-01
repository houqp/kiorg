use crate::models::action_history::ActionType;
use std::path::PathBuf;

/// Result of a rollback operation
#[derive(Debug, Clone)]
pub enum RollbackResult {
    Success(String),
    PartialSuccess {
        success: String,
        errors: Vec<String>,
    },
    Error(String),
}

/// Rollback utility for undoing file system operations
pub struct RollbackManager;

impl Default for RollbackManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RollbackManager {
    pub fn new() -> Self {
        Self
    }

    /// Perform rollback for an action type
    pub fn rollback_action(&self, action_type: &ActionType) -> Result<String, String> {
        match action_type {
            ActionType::Create { operations } => {
                let mut success_count = 0;
                let mut errors = Vec::new();
                let mut single_result = None;

                for op in operations.iter().rev() {
                    match Self::rollback_create(&op.path, op.is_dir) {
                        RollbackResult::Success(msg) => {
                            success_count += 1;
                            if success_count == 1 {
                                single_result = Some(msg);
                            }
                        }
                        RollbackResult::Error(e) => errors.push(e),
                        RollbackResult::PartialSuccess { errors: errs, .. } => errors.extend(errs),
                    }
                }

                if !errors.is_empty() {
                    Err(errors.join("; "))
                } else if success_count == 1 {
                    Ok(single_result.unwrap())
                } else {
                    Ok(format!("Rolled back {} create operations", success_count))
                }
            }
            ActionType::Rename { operations } => {
                let mut success_count = 0;
                let mut errors = Vec::new();
                let mut single_result = None;

                for op in operations.iter().rev() {
                    match Self::rollback_rename(&op.old_path, &op.new_path) {
                        RollbackResult::Success(msg) => {
                            success_count += 1;
                            if success_count == 1 {
                                single_result = Some(msg);
                            }
                        }
                        RollbackResult::Error(e) => errors.push(e),
                        RollbackResult::PartialSuccess { errors: errs, .. } => errors.extend(errs),
                    }
                }

                if !errors.is_empty() {
                    Err(errors.join("; "))
                } else if success_count == 1 {
                    Ok(single_result.unwrap())
                } else {
                    Ok(format!("Rolled back {} rename operations", success_count))
                }
            }
            ActionType::Copy { operations } => {
                let mut success_count = 0;
                let mut errors = Vec::new();
                let mut single_result = None;

                for op in operations.iter().rev() {
                    let is_dir = op.target_path.is_dir();
                    match Self::rollback_copy(&op.target_path, is_dir) {
                        RollbackResult::Success(msg) => {
                            success_count += 1;
                            if success_count == 1 {
                                single_result = Some(msg);
                            }
                        }
                        RollbackResult::Error(e) => errors.push(e),
                        RollbackResult::PartialSuccess { errors: errs, .. } => errors.extend(errs),
                    }
                }

                if !errors.is_empty() {
                    Err(errors.join("; "))
                } else if success_count == 1 {
                    Ok(single_result.unwrap())
                } else {
                    Ok(format!("Rolled back {} copy operations", success_count))
                }
            }
            ActionType::Move { operations } => {
                let mut success_count = 0;
                let mut errors = Vec::new();
                let mut single_result = None;

                for op in operations.iter().rev() {
                    match Self::rollback_move(&op.source_path, &op.target_path) {
                        RollbackResult::Success(msg) => {
                            success_count += 1;
                            if success_count == 1 {
                                single_result = Some(msg);
                            }
                        }
                        RollbackResult::Error(e) => errors.push(e),
                        RollbackResult::PartialSuccess { errors: errs, .. } => errors.extend(errs),
                    }
                }

                if !errors.is_empty() {
                    Err(errors.join("; "))
                } else if success_count == 1 {
                    Ok(single_result.unwrap())
                } else {
                    Ok(format!("Rolled back {} move operations", success_count))
                }
            }
        }
    }

    /// Rollback a create operation by deleting the created file/directory
    fn rollback_create(path: &PathBuf, is_directory: bool) -> RollbackResult {
        if !path.exists() {
            return RollbackResult::Error(format!(
                "Cannot rollback create: {} no longer exists",
                path.display()
            ));
        }

        let result = if is_directory {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        match result {
            Ok(()) => {
                let item_type = if is_directory { "directory" } else { "file" };
                RollbackResult::Success(format!("Deleted {} '{}'", item_type, path.display()))
            }
            Err(e) => RollbackResult::Error(format!("Failed to delete {}: {}", path.display(), e)),
        }
    }

    /// Rollback a rename operation by renaming back to original name
    fn rollback_rename(original_path: &PathBuf, current_path: &PathBuf) -> RollbackResult {
        if !current_path.exists() {
            return RollbackResult::Error(format!(
                "Cannot rollback rename: {} no longer exists",
                current_path.display()
            ));
        }

        if original_path.exists() {
            return RollbackResult::Error(format!(
                "Cannot rollback rename: {} already exists",
                original_path.display()
            ));
        }

        match std::fs::rename(current_path, original_path) {
            Ok(()) => RollbackResult::Success(format!(
                "Renamed '{}' back to '{}'",
                current_path.display(),
                original_path.display()
            )),
            Err(e) => RollbackResult::Error(format!(
                "Failed to rename {} back to {}: {}",
                current_path.display(),
                original_path.display(),
                e
            )),
        }
    }

    /// Rollback a copy operation by deleting the copied file/directory
    fn rollback_copy(target_path: &PathBuf, is_directory: bool) -> RollbackResult {
        Self::rollback_create(target_path, is_directory)
    }

    /// Rollback a move operation by moving back to original location
    fn rollback_move(original_path: &PathBuf, current_path: &PathBuf) -> RollbackResult {
        Self::rollback_rename(original_path, current_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::action_history::{ActionType, CreateOperation, RenameOperation};
    use tempfile::tempdir;

    #[test]
    fn test_rollback_create_file() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Create a file
        std::fs::write(&test_file, "test content").unwrap();
        assert!(test_file.exists());

        let action = ActionType::Create {
            operations: vec![CreateOperation {
                path: test_file.clone(),
                is_dir: false,
            }],
        };

        let manager = RollbackManager::new();
        let result = manager.rollback_action(&action);

        assert!(result.is_ok(), "Rollback should succeed");
        assert!(!test_file.exists(), "File should be deleted");
    }

    #[test]
    fn test_rollback_create_directory() {
        let temp_dir = tempdir().unwrap();
        let test_dir = temp_dir.path().join("test_dir");

        // Create a directory
        std::fs::create_dir(&test_dir).unwrap();
        assert!(test_dir.exists());

        let action = ActionType::Create {
            operations: vec![CreateOperation {
                path: test_dir.clone(),
                is_dir: true,
            }],
        };

        let manager = RollbackManager::new();
        let result = manager.rollback_action(&action);

        assert!(result.is_ok(), "Rollback should succeed");
        assert!(!test_dir.exists(), "Directory should be deleted");
    }

    #[test]
    fn test_rollback_rename() {
        let temp_dir = tempdir().unwrap();
        let old_file = temp_dir.path().join("old.txt");
        let new_file = temp_dir.path().join("new.txt");

        // Create and rename a file
        std::fs::write(&old_file, "test content").unwrap();
        std::fs::rename(&old_file, &new_file).unwrap();
        assert!(!old_file.exists());
        assert!(new_file.exists());

        let action = ActionType::Rename {
            operations: vec![RenameOperation {
                old_path: old_file.clone(),
                new_path: new_file.clone(),
            }],
        };

        let manager = RollbackManager::new();
        let result = manager.rollback_action(&action);

        assert!(result.is_ok(), "Rollback should succeed");
        assert!(old_file.exists(), "File should be renamed back");
        assert!(!new_file.exists(), "New file should not exist");
    }
}
