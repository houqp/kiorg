//! XDG mimeapps.list parser and application lookup
//!
//! This crate implements the freedesktop.org mime-apps specification for finding
//! applications associated with file types on Linux systems.
//!
//! # Specification
//! <https://specifications.freedesktop.org/mime-apps-spec/mime-apps-spec-latest.html>

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub path: String,
    pub name: String,
}

#[cfg(target_os = "linux")]
mod linux_impl;

#[cfg(target_os = "linux")]
pub use linux_impl::get_apps_for_file_linux as get_apps_for_file;

#[cfg(target_os = "macos")]
mod macos_impl;

#[cfg(target_os = "macos")]
pub use macos_impl::get_apps_for_file_macos as get_apps_for_file;

#[cfg(target_os = "windows")]
mod windows_impl;

#[cfg(target_os = "windows")]
pub use windows_impl::get_apps_for_file_windows as get_apps_for_file;
