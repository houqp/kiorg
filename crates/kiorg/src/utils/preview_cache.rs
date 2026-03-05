use ahash::RandomState;

use rkyv::rancor::Error;
use std::fs;

use std::path::PathBuf;

use crate::models::dir_entry::DirEntryMeta;
use crate::models::preview_content::CachedPreviewContent;

#[cfg(any(test, feature = "testing"))]
mod imp {
    use super::*;
    use std::sync::Mutex;

    static TEST_CACHE_DIR: Mutex<Option<tempfile::TempDir>> = Mutex::new(None);

    pub fn get_cache_dir() -> Option<PathBuf> {
        let mut guard = TEST_CACHE_DIR.lock().unwrap();
        if guard.is_none() {
            *guard = Some(
                tempfile::Builder::new()
                    .prefix("kiorg_test_")
                    .tempdir()
                    .expect("failed to create temp test cache dir"),
            );
        }
        Some(guard.as_ref().unwrap().path().to_path_buf())
    }

    pub fn purge_cache_dir() {
        if let Ok(mut guard) = TEST_CACHE_DIR.lock() {
            let _ = guard.take();
        }
    }
}

#[cfg(not(any(test, feature = "testing")))]
mod imp {
    use super::*;

    pub fn get_cache_dir() -> Option<PathBuf> {
        dirs::cache_dir().map(|mut d| {
            d.push("kiorg");
            d
        })
    }

    pub fn purge_cache_dir() {
        if let Some(dir) = get_cache_dir() {
            if dir.exists() {
                let _ = fs::remove_dir_all(dir);
            }
        }
    }
}

pub use imp::{get_cache_dir, purge_cache_dir};

pub fn calculate_path_hash(path: &std::path::Path) -> u64 {
    let path_str = path.to_string_lossy();
    let hasher = RandomState::with_seeds(0, 0, 0, 0);
    hasher.hash_one(path_str.as_bytes())
}

pub fn calculate_cache_key(entry: &DirEntryMeta) -> String {
    let path_hash = calculate_path_hash(&entry.path);

    let mtime = entry
        .modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{path_hash:x}.{mtime}")
}

pub fn delete_previews_for_path(path: &std::path::Path) {
    let path_hash = calculate_path_hash(path);
    let hash_hex = format!("{path_hash:x}");
    let prefix = format!("{hash_hex}.");

    if let Some(mut d) = get_cache_dir() {
        if hash_hex.len() >= 2 {
            d.push(&hash_hex[0..2]);
        }
        if d.exists() && d.is_dir() {
            if let Ok(entries) = fs::read_dir(d) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.starts_with(&prefix) {
                            let _ = fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}

pub fn get_cache_path(key: &str) -> Option<PathBuf> {
    get_cache_dir().map(|mut d| {
        if key.len() >= 2 {
            d.push(&key[0..2]);
        }
        d.push(key);
        d
    })
}

pub fn save_preview(
    key: &str,
    cached: &CachedPreviewContent,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = get_cache_path(key) {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let bytes = rkyv::to_bytes::<Error>(cached)?;
        atomic_write(&path, &bytes)?;
    }
    Ok(())
}

pub fn load_preview(key: &str) -> Option<CachedPreviewContent> {
    let path = get_cache_path(key)?;
    if !path.exists() {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    rkyv::from_bytes::<CachedPreviewContent, Error>(&bytes).ok()
}

pub fn delete_preview(key: &str) {
    if let Some(path) = get_cache_path(key) {
        let _ = fs::remove_file(path);
    }
}

pub fn atomic_write(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut temp_path = path.to_path_buf();
    if let Some(file_name) = temp_path.file_name() {
        let mut new_name = file_name.to_os_string();
        new_name.push(".tmp");
        temp_path.set_file_name(new_name);
    }

    {
        let mut file = fs::File::create(&temp_path)?;
        std::io::Write::write_all(&mut file, bytes)?;
        file.sync_all()?;
    }

    fs::rename(temp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::preview_content::ZipEntry;
    use std::time::SystemTime;

    #[test]
    fn test_calculate_cache_key() {
        let entry = DirEntryMeta {
            path: PathBuf::from("/tmp/test.txt"),
            modified: SystemTime::UNIX_EPOCH,
        };
        let key = calculate_cache_key(&entry);
        assert_eq!(key, "f32edd2249c84742.0");
    }

    #[test]
    fn test_preview_serialization() {
        let cached = CachedPreviewContent::Zip(vec![ZipEntry {
            name: "test.txt".to_string(),
            size: 100,
            is_dir: false,
        }]);
        let key = "test_zip_cache";

        save_preview(key, &cached).expect("Failed to save");
        let loaded = load_preview(key).expect("Failed to load");

        if let CachedPreviewContent::Zip(entries) = loaded {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].name, "test.txt");
        } else {
            panic!("Wrong content type loaded");
        }

        // Clean up
        if let Some(path) = get_cache_path(key) {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_delete_previews_for_path() {
        let path = PathBuf::from("/tmp/test_delete.txt");
        let cached = CachedPreviewContent::Zip(vec![]);

        // Create multiple cache entries for the same path with different mtimes
        let mut keys = Vec::new();
        for i in 0..3 {
            let entry = DirEntryMeta {
                path: path.clone(),
                modified: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(i),
            };
            let key = calculate_cache_key(&entry);
            save_preview(&key, &cached).expect("Failed to save");
            assert!(get_cache_path(&key).unwrap().exists());
            keys.push(key);
        }

        // Verify another path's cache is NOT deleted
        let other_path = PathBuf::from("/tmp/other_file.txt");
        let other_entry = DirEntryMeta {
            path: other_path.clone(),
            modified: std::time::SystemTime::now(),
        };
        let other_key = calculate_cache_key(&other_entry);
        save_preview(&other_key, &cached).expect("Failed to save");
        assert!(get_cache_path(&other_key).unwrap().exists());

        delete_previews_for_path(&path);

        // All keys for the target path should be gone
        for key in keys {
            assert!(
                !get_cache_path(&key).unwrap().exists(),
                "Cache key {key} should have been deleted"
            );
        }

        // The other path's cache should still exist
        assert!(
            get_cache_path(&other_key).unwrap().exists(),
            "Other path's cache should not have been deleted"
        );

        // Clean up other path's cache
        let _ = fs::remove_file(get_cache_path(&other_key).unwrap());
    }
}
