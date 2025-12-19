use super::AppInfo;
use std::path::Path;

pub fn get_apps_for_file_windows(_path: &Path) -> Vec<AppInfo> {
    // TODO: Implement Windows application lookup
    // This could use Windows Registry to find file associations
    Vec::new()
}
