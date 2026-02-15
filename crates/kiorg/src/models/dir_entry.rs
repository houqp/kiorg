use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirEntryMeta {
    pub path: PathBuf,
    pub modified: SystemTime,
}

use std::sync::OnceLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub meta: DirEntryMeta,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    #[serde(skip)]
    pub(crate) formatted_size: OnceLock<String>,
    #[serde(skip)]
    pub(crate) formatted_modified: OnceLock<String>,
}

impl DirEntry {
    pub fn new(
        name: String,
        meta: DirEntryMeta,
        is_dir: bool,
        is_symlink: bool,
        size: u64,
    ) -> Self {
        Self {
            name,
            meta,
            is_dir,
            is_symlink,
            size,
            formatted_size: OnceLock::new(),
            formatted_modified: OnceLock::new(),
        }
    }

    pub fn formatted_size(&self) -> &str {
        self.formatted_size
            .get_or_init(|| crate::utils::format::format_size(self.size, self.is_dir))
    }

    pub fn formatted_modified(&self) -> &str {
        self.formatted_modified
            .get_or_init(|| crate::utils::format::format_modified(self.meta.modified))
    }

    pub fn accessibility_text(&self) -> String {
        let file_type = if self.is_dir { "folder" } else { "file" };

        if self.is_symlink {
            format!(
                "{} {}, symbolic link, modified {}, size {}",
                file_type,
                self.name,
                self.formatted_modified(),
                self.formatted_size()
            )
        } else {
            format!(
                "{} {}, modified {}, size {}",
                file_type,
                self.name,
                self.formatted_modified(),
                self.formatted_size()
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
            formatted_size: OnceLock::new(),
            formatted_modified: OnceLock::new(),
        };

        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.meta.path, PathBuf::from("/tmp/test.txt"));
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 100);
    }
}
