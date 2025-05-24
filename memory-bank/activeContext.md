## Active Context

### Current work focus

- Column sort order persistence feature is now fully implemented and tested. User sort preferences are reliably loaded and saved between sessions, enhancing UX.
- Improved search filter feature with robust visual highlighting and support for large directories, Unicode, and long filenames.
- Enhanced preview functionality with support for PDF and EPUB files, displaying metadata and rendered content.

### Next steps

1. Test the search filter feature thoroughly, especially with:
   - Large directories
   - Unicode characters
   - Very long filenames
   - Empty search queries
2. Consider implementing other features from the backlog (e.g., create new file/folder shortcuts, case-sensitive/insensitive search toggle).
3. Ensure all future UI development follows the patterns documented in the UI style guide.
4. Continue maintaining and updating the Memory Bank.

### Important patterns and preferences

- Prefer composition over inheritance.
- Always run `cargo clippy` on every change.
- Break code into modules logically instead of keeping all of them into large files.
- Avoid deeply nested code structure by breaking up implementation into pure functions.
- Write new tests whenever applicable to prevent regressions.
- Prefer integration tests in `tests` folder over unit tests for more complex test cases.
- When dealing with borrow checker issues:
    1. Move data access before closures when possible
    2. Consider restructuring code to minimize mutable borrows
- Always avoid unsafe rust code.
- Follow UI style guidelines documented in `ui_style_guide.md` for consistent user experience.

### Learnings and project insights

- The importance of maintaining accurate and up-to-date documentation for long-term project success
- The value of clear and concise communication within the development team
- Persisting user preferences enhances the user experience significantly
- The serde library makes it straightforward to serialize and deserialize Rust structs to and from TOML
- Modular code organization makes it easier to implement new features without introducing bugs
- Confirmed that persisting user preferences (like sort order) significantly improves user experience and is technically straightforward with serde/TOML.
- Ensured backward compatibility when expanding the config file for new preferences.
- Reinforced the value of immediate persistence (saving on change) for user settings, rather than on exit.
- Returning final data structures directly from functions rather than intermediate results improves code organization and reduces duplication.
