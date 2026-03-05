use std::path::Path;

/// Recursively copy a directory from src to dst
pub fn copy_dir_recursively(src: &Path, dst: &Path) -> std::io::Result<()> {
    // Create the destination directory if it doesn't exist
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    // Iterate through the source directory entries
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(file_name);

        if entry_path.is_dir() {
            // Recursively copy subdirectories
            copy_dir_recursively(&entry_path, &dst_path)?;
        } else {
            // Copy files
            std::fs::copy(&entry_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Move a file or directory from src to dst, handling cross-device links by falling back to copy and delete.
pub fn omni_rename(src: &Path, dst: &Path) -> std::io::Result<()> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(18) => {
            // Error 18 is "Invalid cross-device link"
            if src.is_dir() {
                copy_dir_recursively(src, dst)?;
                std::fs::remove_dir_all(src)?;
            } else {
                std::fs::copy(src, dst)?;
                std::fs::remove_file(src)?;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_dir_recursively() {
        // Test the helper function logic (without actual file operations)
        // This is a unit test for the function signature and basic error handling
        let src = Path::new("/nonexistent/src");
        let dst = Path::new("/nonexistent/dst");

        // Should return an error for non-existent paths
        let result = copy_dir_recursively(src, dst);
        assert!(result.is_err());
    }
}
