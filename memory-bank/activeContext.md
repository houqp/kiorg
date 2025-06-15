## Active Context

### Current work focus

- Async delete operations have been implemented with progress dialog to prevent UI blocking during large file deletions
- Performance optimizations completed for search functionality and center panel rendering to reduce UI lag
- Theme and colorscheme system has been refactored for improved maintainability and code organization
- Benchmarking infrastructure added to monitor and improve performance
- Various UI improvements including fixed navigation path truncation and left panel scrolling issues

### Next steps

1. Continue optimizing performance based on benchmark results
2. Consider implementing remaining features from the backlog:
   - Shortcut to toggle sort
   - Fuzzy directory jump (integrate with fzf)
   - Regular expression search support
3. Fix known issues:
   - Renaming a file doesn't clear the image rendering cache, so it still displays the old image
   - Implement PDF preview using pdfium or pathfinder_rasterize
4. Ensure all future UI development follows the patterns documented in the UI style guide
5. Continue maintaining and updating the Memory Bank

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
- Confirmed that persisting user preferences (like sort order) significantly improves user experience and is technically straightforward with serde/TOML
- Ensured backward compatibility when expanding the config file for new preferences
- Reinforced the value of immediate persistence (saving on change) for user settings, rather than on exit
- Returning final data structures directly from functions rather than intermediate results improves code organization and reduces duplication
- Async operations are crucial for maintaining responsive UI, especially for potentially long-running tasks like file operations
- Performance benchmarking provides valuable insights for optimization efforts
- Breaking large operations into background threads with progress reporting greatly improves user experience
