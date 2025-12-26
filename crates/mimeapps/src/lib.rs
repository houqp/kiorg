//! Cross-platform library for looking up registered applications for a given file path.
//!
//! This crate supports Linux (XDG specification), macOS (NSWorkspace), and
//! provides a placeholder for Windows.
//!
//! # Specification
//! <https://specifications.freedesktop.org/mime-apps-spec/mime-apps-spec-latest.html>

/// Information about an application that can open a file.
#[derive(Clone, Debug)]
pub struct AppInfo {
    /// The absolute path to the application executable or bundle.
    pub path: String,
    /// The user-visible name of the application.
    pub name: String,
}

#[cfg(target_os = "linux")]
mod linux_impl;

/// Returns a list of applications that can open the file at the given path.
///
/// On Linux, it implements the XDG MIME apps specification.
/// On macOS, it uses NSWorkspace.
/// On Windows, it is currently a placeholder and returns an empty list.
#[cfg(target_os = "linux")]
pub use linux_impl::get_apps_for_file_linux as get_apps_for_file;

#[cfg(target_os = "macos")]
mod macos_impl;

/// Returns a list of applications that can open the file at the given path.
///
/// On Linux, it implements the XDG MIME apps specification.
/// On macOS, it uses NSWorkspace.
/// On Windows, it is currently a placeholder and returns an empty list.
#[cfg(target_os = "macos")]
pub use macos_impl::get_apps_for_file_macos as get_apps_for_file;

#[cfg(target_os = "windows")]
mod windows_impl;

/// Returns a list of applications that can open the file at the given path.
///
/// On Linux, it implements the XDG MIME apps specification.
/// On macOS, it uses NSWorkspace.
/// On Windows, it is currently a placeholder and returns an empty list.
#[cfg(target_os = "windows")]
pub use windows_impl::get_apps_for_file_windows as get_apps_for_file;
