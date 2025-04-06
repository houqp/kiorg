use std::path::PathBuf;
use crate::models::dir_entry::DirEntry;

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub enum SortColumn {
    Name,
    Modified,
    Size,
    None,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone)]
pub struct Tab {
    pub current_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub parent_entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub parent_selected_index: usize,
    pub selected_entries: std::collections::HashSet<PathBuf>,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
}

impl Tab {
    pub fn new(path: PathBuf) -> Self {
        Self {
            current_path: path,
            entries: Vec::new(),
            parent_entries: Vec::new(),
            selected_index: 0,
            parent_selected_index: 0,
            selected_entries: std::collections::HashSet::new(),
            sort_column: SortColumn::None,
            sort_order: SortOrder::Ascending,
        }
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
            // If clicking a different column, start with ascending
            self.sort_column = column;
            self.sort_order = SortOrder::Descending;
        }
    }

    pub fn sort_entries(&mut self) {
        self.entries.sort_by(|a, b| {
            // Always keep folders first regardless of sort column
            if a.is_dir != b.is_dir {
                return if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }

            // If no sort column is selected, sort by name ascending
            if self.sort_column == SortColumn::None {
                return a.name.cmp(&b.name);
            }

            let primary_order = match self.sort_column {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Modified => a.modified.cmp(&b.modified),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::None => unreachable!(), // Already handled above
            };

            match self.sort_order {
                SortOrder::Ascending => primary_order,
                SortOrder::Descending => primary_order.reverse(),
            }
        });
    }
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub current_tab_index: usize,
}

impl TabManager {
    pub fn new(initial_path: PathBuf) -> Self {
        Self {
            tabs: vec![Tab::new(initial_path)],
            current_tab_index: 0,
        }
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

    pub fn current_tab(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab_index]
    }

    pub fn current_tab_ref(&self) -> &Tab {
        &self.tabs[self.current_tab_index]
    }
} 