use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub modified: SystemTime,
    pub size: u64,
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
            modified: UNIX_EPOCH,
            size: 100,
        };

        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.path, PathBuf::from("/tmp/test.txt"));
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 100);
    }
} 