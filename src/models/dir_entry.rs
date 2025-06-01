use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub modified: SystemTime,
    pub size: u64,
    pub formatted_modified: String,
    pub formatted_size: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    #[test]
    fn test_dir_entry_creation() {
        let entry = DirEntry {
            name: "test.txt".to_string(),
            path: PathBuf::from("/tmp/test.txt"),
            is_dir: false,
            is_symlink: false,
            modified: UNIX_EPOCH,
            size: 100,
            formatted_modified: "1970-01-01 00:00:00".to_string(),
            formatted_size: "100 B".to_string(),
        };

        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.path, PathBuf::from("/tmp/test.txt"));
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 100);
        assert_eq!(entry.formatted_size, "100 B");
    }
}
