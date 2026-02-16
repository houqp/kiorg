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

    pub fn purge_cache_dir() {}
}

pub use imp::{get_cache_dir, purge_cache_dir};

pub fn calculate_cache_key(entry: &DirEntryMeta) -> String {
    let path_str = entry.path.to_string_lossy();
    let hasher = RandomState::with_seeds(0, 0, 0, 0);
    let path_hash = hasher.hash_one(path_str.as_bytes());

    let mtime = entry
        .modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{path_hash:x}.{mtime}")
}

pub fn get_cache_path(key: &str) -> Option<PathBuf> {
    get_cache_dir().map(|mut d| {
        d.push(key);
        d
    })
}

pub fn save_preview(
    key: &str,
    cached: &CachedPreviewContent,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_cache_dir()?;
    if let Some(path) = get_cache_path(key) {
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

// FIXME: initialize it once globally, and return the dir in O(1)
pub fn ensure_cache_dir() -> std::io::Result<()> {
    if let Some(dir) = get_cache_dir() {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
    }
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
}
