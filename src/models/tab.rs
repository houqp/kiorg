use std::path::PathBuf;
use crate::models::dir_entry::DirEntry;

#[derive(Clone)]
pub struct Tab {
    pub current_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub parent_entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub parent_selected_index: usize,
    pub selected_entries: std::collections::HashSet<PathBuf>,
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
        }
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