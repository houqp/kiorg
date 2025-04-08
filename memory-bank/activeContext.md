## Active Context

### Current work focus
Improved search filter feature with visual highlighting and fixed entry name display issues.

### Recent changes
1. Fixed entry name display and search highlighting:
   - Added `search_query` and `search_active` fields to `EntryRowParams` struct
   - Implemented proper text highlighting in `draw_entry_row` using `LayoutJob`
   - Improved search UX by:
     - Highlighting matching text in yellow
     - Showing non-matching entries in gray
     - Maintaining search state between sessions
2. Implemented column sort order persistence feature:
   - Updated the `Config` struct to include sort preferences
   - Modified the application to save sort preferences when they change
   - Updated the application to load and apply sort preferences when creating new tabs
3. Updated the productContext.md file with the project's purpose, problems it solves, how it should work, and user experience goals.
4. Implemented search filter functionality:
   - Added state variables (`search_mode`, `search_query`, `search_focus`) to `Kiorg` struct.
   - Added `search_query` and `search_active` to `Tab` struct.
   - Updated `handle_key_press` in `app.rs` to handle `/`, Enter, and Esc for search.
   - Implemented `get_filtered_entries`, `clear_filter`, `get_first_filtered_entry_index` in `Tab`.
   - Updated `center_panel.rs` to display search bar and use filtered entries.
   - Resolved borrow checker issues in `center_panel.rs` by deferring state updates.
   - Updated help window and `projectbrief.md` with the new shortcut.

### Next steps
1. Test the search filter feature thoroughly, especially with:
   - Large directories
   - Unicode characters
   - Very long filenames
   - Empty search queries
2. Consider implementing other features from the backlog (e.g., PDF preview, create new file/folder shortcuts).
3. Continue maintaining and updating the Memory Bank.

### Active decisions and considerations
*   Decided to filter entries in real-time as the user types in the search bar.
*   Pressing Enter confirms the current filter, allowing navigation within filtered results.
*   Pressing Esc cancels search mode and clears the filter.
*   Decided to store sort preferences in the existing TOML config file rather than creating a separate file
*   Chose to update the sort preferences immediately when changed rather than only on application exit
*   Implemented the feature in a way that maintains backward compatibility with existing configurations
*   How to best structure the systemPatterns.md file to document the system architecture and key technical decisions
*   How to document the technologies used, development setup, and technical constraints in techContext.md
*   How to track the project's progress, current status, and known issues in progress.md

### Important patterns and preferences
*   Prefer composition over inheritance.
*   Always run `cargo clippy` on every change.
*   Break code into modules logically instead of keeping all of them into large files.
*   Avoid deeply nested code structure by breaking up implementation into pure functions.
*   Write new tests whenever applicable to prevent regressions.
*   Prefer integration tests in `tests` folder over unit tests for more complex test cases.
*   When dealing with borrow checker issues:
    1. Move data access before closures when possible
    2. Consider restructuring code to minimize mutable borrows

### Learnings and project insights
*   The importance of maintaining accurate and up-to-date documentation for long-term project success
*   The value of clear and concise communication within the development team
*   Persisting user preferences enhances the user experience significantly
*   The serde library makes it straightforward to serialize and deserialize Rust structs to and from TOML
*   Modular code organization makes it easier to implement new features without introducing bugs