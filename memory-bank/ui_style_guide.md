## UI Style Guide

### Popups and Dialogs
- All popups should use the `new_center_popup_window` utility function from `window_utils.rs`
- Popups should be centered on the screen
- Popups should have a close button in the top right corner or use keyboard shortcuts (Esc/q) to close
- Confirmation dialogs (like delete and exit) should follow the pattern in `delete_dialog.rs`
- Help window should follow the pattern in `help_window.rs`
- All popups should have consistent styling with the app's color scheme

### Input Bars
- Search bars should follow the pattern in `search_bar.rs`
- Input bars should have shadows similar to window popups for visual consistency
- Close buttons should be placed in window titles rather than in search bars when possible

### Navigation
- Double-clicking on entries should enter directories or open files
- Terminal popup should be triggered by the 'T' shortcut using the egui_term crate
- Context menu should support file operations: add, rename, delete, copy, cut, and paste
- Context menu options should be enabled/disabled based on context (e.g., paste only enabled when clipboard has content)

### Layout
- The right side preview panel should always be visible
- Panels should be separated with consistent spacing
- UI elements should have consistent padding and margins

### Testing
- UI tests should be written in the @tests/ directory
- Tests should cover right-click context menu functionality
- Tests should verify that the right side preview panel is always visible
