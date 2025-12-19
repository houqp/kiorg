use super::AppInfo;
use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSArray, NSFileManager, NSString, NSURL};
use std::path::Path;

pub fn get_apps_for_file_macos(path: &Path) -> Vec<AppInfo> {
    let Some(path_str) = path.to_str() else {
        return Vec::new();
    };
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let file_manager = NSFileManager::defaultManager();
        let file_url = NSURL::fileURLWithPath(&NSString::from_str(path_str));

        let app_urls: Retained<NSArray<NSURL>> = objc2::msg_send![
            &workspace,
            URLsForApplicationsToOpenURL: &*file_url
        ];

        (*app_urls)
            .iter()
            .filter_map(|url| {
                let path = url.path()?;
                Some(AppInfo {
                    path: path.to_string(),
                    name: file_manager.displayNameAtPath(&path).to_string(),
                })
            })
            .collect()
    }
}
