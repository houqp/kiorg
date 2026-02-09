use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirEntryMeta {
    pub path: PathBuf,
    pub modified: SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub meta: DirEntryMeta,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub formatted_modified: String,
    pub formatted_size: String,
}

impl DirEntry {
    pub fn accessibility_text(&self) -> String {
        let file_type = if self.is_dir { "folder" } else { "file" };
        if self.is_symlink {
            format!(
                "{} {}, symbolic link, modified {}, size {}",
                file_type, self.name, self.formatted_modified, self.formatted_size
            )
        } else {
            format!(
                "{} {}, modified {}, size {}",
                file_type, self.name, self.formatted_modified, self.formatted_size
            )
        }
    }
}

#[cfg(test)] // Added cfg(test) for the test module
mod tests {
    // Wrapped tests in a module
    use super::*; // Import everything from the outer scope
    use std::time::UNIX_EPOCH; // Re-import UNIX_EPOCH for the test module

    #[test]
    fn test_dir_entry_creation() {
        let meta = DirEntryMeta {
            path: PathBuf::from("/tmp/test.txt"),
            modified: UNIX_EPOCH,
        };
        let entry = DirEntry {
            name: "test.txt".to_string(),
            meta,
            is_dir: false,
            is_symlink: false,
            size: 100,
            formatted_modified: "1970-01-01 00:00:00".to_string(),
            formatted_size: "100 B".to_string(),
        };

        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.meta.path, PathBuf::from("/tmp/test.txt"));
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 100);
        assert_eq!(entry.formatted_size, "100 B");
    }
}
