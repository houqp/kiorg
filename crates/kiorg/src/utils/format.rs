use chrono::{DateTime, Local};
use humansize::{BINARY, format_size as humansize_format};
use std::time::SystemTime;

/// Formats a SystemTime into a string with the format "%Y-%m-%d %H:%M:%S"
pub fn format_modified(modified: SystemTime) -> String {
    DateTime::<Local>::from(modified)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Formats a size in bytes into a human-readable string.
/// If it's a directory, returns an empty string (as directories don't have a single "size" in this context).
pub fn format_size(size: u64, is_dir: bool) -> String {
    if is_dir {
        String::new()
    } else {
        humansize_format(size, BINARY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_modified() {
        use chrono::TimeZone;
        let dt_str = "2023-10-27 10:30:00";
        let naive = chrono::NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S").unwrap();
        let local_dt = Local.from_local_datetime(&naive).single().unwrap();
        let ts = SystemTime::from(local_dt);
        let formatted = format_modified(ts);
        assert_eq!(formatted, dt_str);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0, false), "0 B");
        assert_eq!(format_size(1024, false), "1 KiB");
        assert_eq!(format_size(100, true), "");
    }
}
