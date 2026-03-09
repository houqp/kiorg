use crate::{
    Kiorg,
    models::action_history::{ActionType, RenameOperation},
};

#[derive(Debug)]
pub struct Rename {
    original_index: usize,
    original_name: String,
    new_name: String,
}

impl Rename {
    pub fn new(original_index: usize, original_name: String, new_name: String) -> Self {
        Self {
            original_index,
            original_name,
            new_name,
        }
    }

    pub fn confirm(&mut self, app: &mut Kiorg) {
        let new_name = self.new_name.trim().to_string();

        if new_name.is_empty() || new_name == self.original_name {
            return;
        }

        let tab = app.tab_manager.current_tab_mut();
        if let Some(entry) = tab.entries.get(tab.selected_index) {
            let parent = entry.meta.path.parent().unwrap_or(&tab.current_path);
            let new_path = parent.join(new_name);

            if let Err(e) = crate::utils::file_operations::omni_rename(&entry.meta.path, &new_path)
            {
                app.notify_error(format!("Failed to rename: {e}"));
            } else {
                // Delete preview cache for the old path (all associated versions)
                crate::utils::preview_cache::delete_previews_for_path(&entry.meta.path);

                // Record rename action in history
                let old_path = entry.meta.path.clone();
                tab.action_history.add_action(ActionType::Rename {
                    operations: vec![RenameOperation { old_path, new_path }],
                });

                app.refresh_entries();
            }
        }

        self.clear(app);
    }

    pub fn clear(&mut self, app: &mut Kiorg) {
        app.inline_rename = None
    }
}
