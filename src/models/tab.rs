use crate::config::Config;
use crate::models::dir_entry::DirEntry;
use std::path::PathBuf;

#[derive(Clone, PartialEq, Debug, Hash, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum SortColumn {
    Name,
    Modified,
    Size,
    None,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

// TabState is the minimal state that gets serialized/deserialized
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TabState {
    pub current_path: PathBuf,
}

// Tab contains the full runtime state, but only TabState is persisted
#[derive(Clone)]
pub struct Tab {
    pub current_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub parent_entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub parent_selected_index: usize,
    pub marked_entries: std::collections::HashSet<PathBuf>,
    // History of visited directories
    pub history: Vec<PathBuf>,
    pub history_position: usize,
    // Reverse index mapping DirEntry path to index in entries (private)
    path_to_index: std::collections::HashMap<PathBuf, usize>,
}

// Private helper function for sorting DirEntry slices
fn sort_entries_by(entries: &mut [DirEntry], sort_column: SortColumn, sort_order: SortOrder) {
    let primary_order_fn = match sort_column {
        SortColumn::Name => |a: &DirEntry, b: &DirEntry| a.name.cmp(&b.name),
        SortColumn::Modified => |a: &DirEntry, b: &DirEntry| a.modified.cmp(&b.modified),
        SortColumn::Size => |a: &DirEntry, b: &DirEntry| a.size.cmp(&b.size),
        SortColumn::None => {
            return;
        }
    };
    match sort_order {
        SortOrder::Ascending => entries.sort_by(|a, b| {
            // Always keep folders first regardless of sort column
            if a.is_dir != b.is_dir {
                return if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            primary_order_fn(a, b)
        }),
        SortOrder::Descending => entries.sort_by(|a, b| {
            // Always keep folders first regardless of sort column
            if a.is_dir != b.is_dir {
                return if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            primary_order_fn(b, a)
        }),
    };
}

// Build the reverse index mapping paths to indices
fn refresh_path_to_index(tab: &mut Tab) {
    tab.path_to_index.clear();
    for (index, entry) in tab.entries.iter().enumerate() {
        tab.path_to_index.insert(entry.path.clone(), index);
    }
}

impl TabState {
    pub fn new(path: PathBuf) -> Self {
        Self { current_path: path }
    }
}

impl Tab {
    pub fn new(path: PathBuf) -> Self {
        let mut tab = Self {
            current_path: path.clone(),
            entries: Vec::new(),
            parent_entries: Vec::new(),
            selected_index: 0,
            parent_selected_index: 0,
            marked_entries: std::collections::HashSet::new(),
            history: Vec::new(),
            history_position: 0,
            path_to_index: std::collections::HashMap::new(),
        };
        // Add the initial path to history
        tab.add_to_history(path);
        tab
    }

    // Convert Tab to TabState for serialization
    pub fn to_state(&self) -> TabState {
        TabState {
            current_path: self.current_path.clone(),
        }
    }

    // Create Tab from TabState
    pub fn from_state(state: TabState) -> Self {
        let path = state.current_path.clone();
        let mut tab = Self {
            current_path: state.current_path,
            entries: Vec::new(),
            parent_entries: Vec::new(),
            selected_index: 0,
            parent_selected_index: 0,
            marked_entries: std::collections::HashSet::new(),
            history: Vec::new(),
            history_position: 0,
            path_to_index: std::collections::HashMap::new(),
        };
        // Add the initial path to history
        tab.add_to_history(path);
        tab
    }

    // Add a path to the history
    pub fn add_to_history(&mut self, path: PathBuf) {
        // If we're not at the end of the history, truncate the forward history
        if self.history_position < self.history.len() {
            self.history.truncate(self.history_position);
        }

        // Don't add if it's the same as the current path at the end of history
        if self.history.last().is_none_or(|last| *last != path) {
            self.history.push(path);
            self.history_position = self.history.len();
        }
    }

    // Go back in history
    pub fn history_back(&mut self) -> Option<PathBuf> {
        if self.history_position > 1 {
            self.history_position -= 1;
            Some(self.history[self.history_position - 1].clone())
        } else {
            None
        }
    }

    // Go forward in history
    pub fn history_forward(&mut self) -> Option<PathBuf> {
        if self.history_position < self.history.len() {
            self.history_position += 1;
            Some(self.history[self.history_position - 1].clone())
        } else {
            None
        }
    }

    pub fn update_selection(&mut self, new_index: usize) {
        if new_index < self.entries.len() {
            self.selected_index = new_index;
        }
    }

    pub fn selected_entry(&self) -> Option<&DirEntry> {
        if self.entries.is_empty() {
            None
        } else {
            Some(&self.entries[self.selected_index])
        }
    }

    // Get the index of an entry by its path using the reverse index
    pub fn get_index_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.path_to_index.get(path).copied()
    }

    // Returns the index of the first entry that matches the current search query
    pub fn get_first_filtered_entry_index(&self, query: &str) -> Option<usize> {
        if query.is_empty() {
            // If query is empty, return the current selection or 0
            return if !self.entries.is_empty() {
                Some(self.selected_index.min(self.entries.len() - 1))
            } else {
                None
            };
        }
        self.entries
            .iter()
            .position(|entry| entry.name.to_lowercase().contains(&query.to_lowercase()))
    }

    // Returns a filtered list of entries based on the search query
    pub fn get_filtered_entries(&self, query: &Option<String>) -> Vec<&DirEntry> {
        match query {
            Some(query) => self
                .entries
                .iter()
                .filter(|entry| entry.name.to_lowercase().contains(&query.to_lowercase()))
                .collect(),
            None => self.entries.iter().collect(),
        }
    }

    // Returns a filtered list of entries along with their original indices
    pub fn get_filtered_entries_with_indices(
        &self,
        query: &Option<String>,
    ) -> Vec<(&DirEntry, usize)> {
        match query {
            Some(q) => {
                let lower_query = q.to_lowercase();
                self.entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| entry.name.to_lowercase().contains(&lower_query))
                    .map(|(i, e)| (e, i))
                    .collect()
            }
            None => self
                .entries
                .iter()
                .enumerate()
                .map(|(i, e)| (e, i))
                .collect(),
        }
    }
}

fn read_dir_entries(path: &PathBuf) -> Vec<DirEntry> {
    if let Ok(read_dir) = std::fs::read_dir(path) {
        read_dir
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();

                let metadata = entry.metadata().ok()?;
                let is_symlink = metadata.file_type().is_symlink();
                // For symlinks, we need to check if the target is a directory
                let is_dir = path.is_dir();
                let modified = metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let size = if is_dir { 0 } else { metadata.len() };

                Some(DirEntry {
                    name,
                    path,
                    is_dir,
                    is_symlink,
                    modified,
                    size,
                })
            })
            .collect()
    } else {
        Vec::new()
    }
}

// TabManagerState is the minimal state that gets serialized/deserialized
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TabManagerState {
    tab_states: Vec<TabState>,
    current_tab_index: usize,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
}

#[derive(Clone)]
pub struct TabManager {
    tabs: Vec<Tab>,
    current_tab_index: usize,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
}

impl TabManager {
    pub fn new(initial_path: PathBuf) -> Self {
        Self::new_with_config(initial_path, None)
    }

    pub fn new_with_config(initial_path: PathBuf, config: Option<&Config>) -> Self {
        let sort_preference = config.and_then(|c| c.sort_preference.as_ref());

        // Initialize sort settings from config
        let (sort_column, sort_order) = if let Some(pref) = sort_preference {
            (pref.column, pref.order)
        } else {
            (SortColumn::None, SortOrder::Ascending)
        };

        Self {
            tabs: vec![Tab::new(initial_path)],
            current_tab_index: 0,
            sort_column,
            sort_order,
        }
    }

    // Convert TabManager to TabManagerState for serialization
    pub fn to_state(&self) -> TabManagerState {
        TabManagerState {
            tab_states: self.tabs.iter().map(|tab| tab.to_state()).collect(),
            current_tab_index: self.current_tab_index,
            sort_column: self.sort_column,
            sort_order: self.sort_order,
        }
    }

    // Create TabManager from TabManagerState
    pub fn from_state(state: TabManagerState) -> Self {
        Self {
            tabs: state.tab_states.into_iter().map(Tab::from_state).collect(),
            current_tab_index: state.current_tab_index,
            sort_column: state.sort_column,
            sort_order: state.sort_order,
        }
    }

    pub fn tab_indexes(&self) -> Vec<(usize, bool)> {
        (0..self.tabs.len())
            .map(|i| (i, i == self.current_tab_index))
            .collect()
    }

    pub fn add_tab(&mut self, path: PathBuf) {
        self.tabs.push(Tab::new(path));
        self.current_tab_index = self.tabs.len() - 1;
    }

    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.current_tab_index = index;
        }
    }

    pub fn close_current_tab(&mut self) -> bool {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.current_tab_index);
            // Adjust the current tab index if necessary
            if self.current_tab_index >= self.tabs.len() {
                self.current_tab_index = self.tabs.len() - 1;
            }
            return true;
        }
        // Do nothing if it's the last tab
        false
    }

    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab_index]
    }

    pub fn current_tab_ref(&self) -> &Tab {
        &self.tabs[self.current_tab_index]
    }

    // Get the current tab index
    pub fn get_current_tab_index(&self) -> usize {
        self.current_tab_index
    }

    // Get the total number of tabs
    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }

    // Get the index of an entry by its path in the current tab
    pub fn get_entry_index_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.current_tab_ref().get_index_by_path(path)
    }

    pub fn reset_selection(&mut self) {
        let tab = self.current_tab_mut();
        tab.selected_index = 0;
    }

    pub fn select_child(&mut self, child: &PathBuf) -> bool {
        let tab = self.current_tab_mut();
        if child.parent().is_some_and(|p| p == tab.current_path) {
            if let Some(pos) = tab.entries.iter().position(|e| &e.path == child) {
                tab.update_selection(pos);
                return true;
            }
        }
        false
    }

    pub fn toggle_sort(&mut self, column: SortColumn) {
        // If clicking the same column, cycle through: Desc -> Asc -> None
        if self.sort_column == column {
            match self.sort_order {
                SortOrder::Ascending => self.sort_column = SortColumn::None,
                SortOrder::Descending => {
                    self.sort_order = SortOrder::Ascending;
                }
            }
        } else {
            // If clicking a different column, start with descending
            self.sort_column = column;
            self.sort_order = SortOrder::Descending;
        }

        let (column, order) = (self.sort_column, self.sort_order);
        let tab = self.current_tab_mut();
        sort_entries_by(&mut tab.entries, column, order);
        sort_entries_by(&mut tab.parent_entries, column, order);
        refresh_path_to_index(tab);
    }

    pub fn refresh_entries(&mut self) {
        // Store sort settings before borrowing self mutably
        let sort_column = self.sort_column;
        let sort_order = self.sort_order;

        let tab = self.current_tab_mut();
        let current_path = tab.current_path.clone(); // Get current path from the tab

        // Path changed or first load, perform full refresh
        // --- Start: Parent Directory Logic ---
        tab.parent_entries.clear();
        tab.parent_selected_index = 0; // Default selection

        if let Some(parent) = current_path.parent() {
            tab.parent_entries = read_dir_entries(&parent.to_path_buf());
            // Sort parent entries using the global sort settings
            sort_entries_by(&mut tab.parent_entries, sort_column, sort_order);

            // Find current directory in parent entries after sorting
            if let Some(pos) = tab
                .parent_entries
                .iter()
                .position(|e| e.path == current_path)
            {
                tab.parent_selected_index = pos;
            }
        } // else: No parent (e.g., root), parent_entries remains empty
          // --- End: Parent Directory Logic ---

        // --- Start: Current Directory Logic ---
        tab.entries = read_dir_entries(&current_path); // Read entries for the current path
                                                       // Sort entries using the global sort settings
        sort_entries_by(&mut tab.entries, sort_column, sort_order);
        refresh_path_to_index(tab);

        // Reset selection index if it's out of bounds (can happen after rehydrating from TabState)
        if tab.selected_index >= tab.entries.len() && !tab.entries.is_empty() {
            tab.selected_index = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    // Helper to create DirEntry instances for testing
    fn create_entry(name: &str, is_dir: bool, modified_secs_ago: u64, size: u64) -> DirEntry {
        let modified = SystemTime::now() - Duration::from_secs(modified_secs_ago);
        DirEntry {
            path: PathBuf::from(name),
            name: name.to_string(),
            is_dir,
            is_symlink: false, // Default to false for test entries
            modified,          // Use the calculated SystemTime directly
            size: if is_dir { 0 } else { size }, // Use 0 for dirs, provided size for files
        }
    }

    // Helper to extract names for assertion
    fn get_names(entries: &[DirEntry]) -> Vec<String> {
        entries.iter().map(|e| e.name.clone()).collect()
    }

    #[test]
    fn test_sort_empty() {
        let mut entries: Vec<DirEntry> = vec![];
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Ascending);
        assert!(entries.is_empty());
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Descending);
        assert!(entries.is_empty());
        sort_entries_by(&mut entries, SortColumn::None, SortOrder::Ascending);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_sort_none() {
        let mut entries = vec![
            create_entry("b", false, 10, 100),
            create_entry("a", true, 20, 0),
            create_entry("c", false, 5, 50),
        ];
        let initial_names = get_names(&entries);
        sort_entries_by(&mut entries, SortColumn::None, SortOrder::Ascending);
        assert_eq!(get_names(&entries), initial_names);
        sort_entries_by(&mut entries, SortColumn::None, SortOrder::Descending);
        assert_eq!(get_names(&entries), initial_names);
    }

    #[test]
    fn test_sort_name_ascending() {
        let mut entries = vec![
            create_entry("file_b", false, 10, 100),
            create_entry("dir_a", true, 20, 0),
            create_entry("file_c", false, 5, 50),
            create_entry("dir_z", true, 15, 0),
        ];
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Ascending);
        // Dirs first, sorted by name, then files sorted by name
        assert_eq!(
            get_names(&entries),
            vec!["dir_a", "dir_z", "file_b", "file_c"]
        );
    }

    #[test]
    fn test_sort_name_descending() {
        let mut entries = vec![
            create_entry("file_b", false, 10, 100),
            create_entry("dir_a", true, 20, 0),
            create_entry("file_c", false, 5, 50),
            create_entry("dir_z", true, 15, 0),
        ];
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Descending);
        // Dirs first, sorted by name descending, then files sorted by name descending
        assert_eq!(
            get_names(&entries),
            vec!["dir_z", "dir_a", "file_c", "file_b"]
        );
    }

    #[test]
    fn test_sort_modified_ascending() {
        let mut entries = vec![
            create_entry("newest_file", false, 5, 100), // 5 secs ago
            create_entry("old_dir", true, 20, 0),       // 20 secs ago
            create_entry("mid_file", false, 10, 50),    // 10 secs ago
            create_entry("new_dir", true, 2, 0),        // 2 secs ago
        ];
        sort_entries_by(&mut entries, SortColumn::Modified, SortOrder::Ascending);
        // Dirs first (oldest to newest), then files (oldest to newest)
        assert_eq!(
            get_names(&entries),
            vec!["old_dir", "new_dir", "mid_file", "newest_file"]
        );
    }

    #[test]
    fn test_sort_modified_descending() {
        let mut entries = vec![
            create_entry("newest_file", false, 5, 100), // 5 secs ago
            create_entry("old_dir", true, 20, 0),       // 20 secs ago
            create_entry("mid_file", false, 10, 50),    // 10 secs ago
            create_entry("new_dir", true, 2, 0),        // 2 secs ago
        ];
        sort_entries_by(&mut entries, SortColumn::Modified, SortOrder::Descending);
        // Dirs first (newest to oldest), then files (newest to oldest)
        assert_eq!(
            get_names(&entries),
            vec!["new_dir", "old_dir", "newest_file", "mid_file"]
        );
    }

    #[test]
    fn test_sort_size_ascending() {
        let mut entries = vec![
            create_entry("large_file", false, 10, 1000),
            create_entry("dir_a", true, 20, 0),
            create_entry("small_file", false, 5, 50),
            create_entry("dir_b", true, 15, 0),
            create_entry("medium_file", false, 12, 200),
        ];
        sort_entries_by(&mut entries, SortColumn::Size, SortOrder::Ascending);
        // Dirs first (order among dirs undefined by size, likely stable based on input), then files by size ascending
        // We check files part specifically. Dirs should just be before files.
        let names = get_names(&entries);
        assert!(names[0] == "dir_a" || names[0] == "dir_b");
        assert!(names[1] == "dir_a" || names[1] == "dir_b");
        assert_ne!(names[0], names[1]);
        assert_eq!(&names[2..], &["small_file", "medium_file", "large_file"]);
    }

    #[test]
    fn test_sort_size_descending() {
        let mut entries = vec![
            create_entry("large_file", false, 10, 1000),
            create_entry("dir_a", true, 20, 0),
            create_entry("small_file", false, 5, 50),
            create_entry("dir_b", true, 15, 0),
            create_entry("medium_file", false, 12, 200),
        ];
        sort_entries_by(&mut entries, SortColumn::Size, SortOrder::Descending);
        // Dirs first (order among dirs undefined by size), then files by size descending
        let names = get_names(&entries);
        assert!(names[0] == "dir_a" || names[0] == "dir_b");
        assert!(names[1] == "dir_a" || names[1] == "dir_b");
        assert_ne!(names[0], names[1]);
        assert_eq!(&names[2..], &["large_file", "medium_file", "small_file"]);
    }

    #[test]
    fn test_sort_only_dirs() {
        let mut entries = vec![
            create_entry("dir_b", true, 10, 0),
            create_entry("dir_a", true, 20, 0),
            create_entry("dir_c", true, 5, 0),
        ];
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Ascending);
        assert_eq!(get_names(&entries), vec!["dir_a", "dir_b", "dir_c"]);
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Descending);
        assert_eq!(get_names(&entries), vec!["dir_c", "dir_b", "dir_a"]);
    }

    #[test]
    fn test_sort_only_files() {
        let mut entries = vec![
            create_entry("file_b", false, 10, 100),
            create_entry("file_a", false, 20, 200),
            create_entry("file_c", false, 5, 50),
        ];
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Ascending);
        assert_eq!(get_names(&entries), vec!["file_a", "file_b", "file_c"]);
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Descending);
        assert_eq!(get_names(&entries), vec!["file_c", "file_b", "file_a"]);
    }

    #[test]
    fn test_sort_stability_equal_primary_key() {
        // Test stability when primary sort key is the same (e.g., two files with the same name)
        // The current sort is stable by default Rust sort, but let's confirm dirs stay first.
        let mut entries = vec![
            create_entry("same_name", false, 10, 100), // File 1
            create_entry("dir_a", true, 20, 0),
            create_entry("same_name", false, 5, 50), // File 2 (newer, smaller)
            create_entry("dir_b", true, 15, 0),
        ];
        // Sort by name ascending. Dirs first, then files. Order between 'same_name' files should be stable.
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Ascending);
        let names = get_names(&entries);
        assert_eq!(names, vec!["dir_a", "dir_b", "same_name", "same_name"]);

        // Sort by name descending. Dirs first (desc), then files (desc). Order between 'same_name' files should be stable.
        sort_entries_by(&mut entries, SortColumn::Name, SortOrder::Descending);
        let names = get_names(&entries);
        assert_eq!(names, vec!["dir_b", "dir_a", "same_name", "same_name"]);

        // Sort by size ascending. Dirs first, then files by size.
        sort_entries_by(&mut entries, SortColumn::Size, SortOrder::Ascending);
        let names = get_names(&entries);
        assert!(names[0] == "dir_a" || names[0] == "dir_b"); // Dirs first
        assert!(names[1] == "dir_a" || names[1] == "dir_b");
        assert_ne!(names[0], names[1]);
        assert_eq!(entries[2].name, "same_name"); // Smaller file
        assert_eq!(entries[2].size, 50);
        assert_eq!(entries[3].name, "same_name"); // Larger file
        assert_eq!(entries[3].size, 100);

        // Sort by size descending. Dirs first, then files by size desc.
        sort_entries_by(&mut entries, SortColumn::Size, SortOrder::Descending);
        let names = get_names(&entries);
        assert!(names[0] == "dir_a" || names[0] == "dir_b"); // Dirs first
        assert!(names[1] == "dir_a" || names[1] == "dir_b");
        assert_ne!(names[0], names[1]);
        assert_eq!(entries[2].name, "same_name"); // Larger file
        assert_eq!(entries[2].size, 100);
        assert_eq!(entries[3].name, "same_name"); // Smaller file
        assert_eq!(entries[3].size, 50);
    }

    #[test]
    fn test_tab_selection_preservation() {
        // Create a tab manager with two tabs
        let mut tab_manager = TabManager::new(PathBuf::from("/path1"));
        tab_manager.add_tab(PathBuf::from("/path2"));

        // Set up some entries for each tab
        let entries1 = vec![
            create_entry("file1", false, 10, 100),
            create_entry("file2", false, 20, 200),
            create_entry("file3", false, 30, 300),
        ];
        let entries2 = vec![
            create_entry("fileA", false, 10, 100),
            create_entry("fileB", false, 20, 200),
            create_entry("fileC", false, 30, 300),
        ];

        // Set entries for tab 1
        tab_manager.tabs[0].entries = entries1;
        // Set entries for tab 2
        tab_manager.tabs[1].entries = entries2;

        // Set different selection indices for each tab
        tab_manager.tabs[0].update_selection(2); // Select "file3" in tab 1
        tab_manager.tabs[1].update_selection(1); // Select "fileB" in tab 2

        // Switch to tab 2
        tab_manager.switch_to_tab(1);
        assert_eq!(tab_manager.current_tab_ref().selected_index, 1);

        // Switch back to tab 1
        tab_manager.switch_to_tab(0);
        assert_eq!(tab_manager.current_tab_ref().selected_index, 2);

        // Switch to tab 2 again
        tab_manager.switch_to_tab(1);
        assert_eq!(tab_manager.current_tab_ref().selected_index, 1);
    }

    #[test]
    fn test_tab_state_serialization() {
        // Create a tab with a specific selection
        let mut tab = Tab::new(PathBuf::from("/test/path"));
        tab.entries = vec![
            create_entry("file1", false, 10, 100),
            create_entry("file2", false, 20, 200),
            create_entry("file3", false, 30, 300),
        ];
        tab.update_selection(2); // Select "file3"
        tab.parent_selected_index = 1; // Set a parent selection index

        // Convert to TabState
        let state = tab.to_state();

        // Verify the state contains only the path
        assert_eq!(state.current_path, PathBuf::from("/test/path"));

        // Create a new Tab from the state
        let new_tab = Tab::from_state(state);

        // Verify the indices are reset to default
        assert_eq!(new_tab.selected_index, 0);
        assert_eq!(new_tab.parent_selected_index, 0);
    }
}
