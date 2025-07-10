use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;

// Constants
const HISTORY_FILE_NAME: &str = "history.csv";

/// Represents a folder visit entry in the history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitHistoryEntry {
    pub path: PathBuf,
    pub accessed_ts: u64,
    pub count: u64,
}

/// Message types for the async history saver thread
#[derive(Debug, Clone)]
pub enum HistorySaveMessage {
    Save(HashMap<PathBuf, VisitHistoryEntry>, Option<PathBuf>), // history data + config_dir_override
    Shutdown,
}

/// Async history saver handle
pub struct HistorySaver {
    sender: mpsc::Sender<HistorySaveMessage>,
    _handle: std::thread::JoinHandle<()>,
}

impl HistorySaver {
    /// Create a new async history saver with background thread
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<HistorySaveMessage>();

        let handle = std::thread::spawn(move || {
            while let Ok(message) = receiver.recv() {
                match message {
                    HistorySaveMessage::Save(history, config_dir_override) => {
                        if let Err(e) = save_visit_history(&history, config_dir_override.as_ref()) {
                            tracing::error!(err = ?e, "Failed to save visit history in background thread");
                        }
                    }
                    HistorySaveMessage::Shutdown => {
                        tracing::debug!("History saver thread shutting down");
                        break;
                    }
                }
            }
        });

        Self {
            sender,
            _handle: handle,
        }
    }

    /// Queue a save operation (non-blocking)
    pub fn save_async(
        &self,
        history: &HashMap<PathBuf, VisitHistoryEntry>,
        config_dir_override: Option<&PathBuf>,
    ) {
        let message = HistorySaveMessage::Save(history.clone(), config_dir_override.cloned());

        if let Err(e) = self.sender.send(message) {
            tracing::error!(err = ?e, "Failed to send save message to history saver thread");
        }
    }

    /// Shutdown the background thread gracefully
    pub fn shutdown(&self) {
        let _ = self.sender.send(HistorySaveMessage::Shutdown);
    }
}

impl Default for HistorySaver {
    fn default() -> Self {
        Self::new()
    }
}

/// Load visit history from CSV file
pub fn load_visit_history(
    config_dir_override: Option<&PathBuf>,
) -> Result<HashMap<PathBuf, VisitHistoryEntry>, Box<dyn std::error::Error>> {
    let config_dir = config::get_kiorg_config_dir(config_dir_override);
    let history_path = config_dir.join(HISTORY_FILE_NAME);

    let mut history = HashMap::new();

    if !history_path.exists() {
        return Ok(history);
    }

    let content = std::fs::read_to_string(&history_path)?;

    for (line_number, line) in content.lines().skip(1).enumerate() {
        // Skip header, line_number starts from 0 (representing line 2 in the file)
        if line.trim().is_empty() {
            continue; // Skip empty lines
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 {
            return Err(format!(
                "Invalid CSV format at line {}: expected at least 3 fields, found {}",
                line_number + 1, // +1 because we skip header
                parts.len()
            )
            .into());
        }

        // Handle paths with commas: use last two parts for timestamp and count,
        // join the remaining leading parts as the path
        let timestamp_part = parts[parts.len() - 2];
        let count_part = parts[parts.len() - 1];
        let path_parts = &parts[0..parts.len() - 2];
        let path_str = path_parts.join(",");

        let accessed_ts = timestamp_part.parse::<u64>().map_err(|_| {
            format!(
                "Invalid timestamp at line {}: '{}'",
                line_number + 1, // +1 because we skip header
                timestamp_part
            )
        })?;

        let count = count_part.parse::<u64>().map_err(|_| {
            format!(
                "Invalid count at line {}: '{}'",
                line_number + 1, // +1 because we skip header
                count_part
            )
        })?;

        let path = PathBuf::from(path_str);
        let entry = VisitHistoryEntry {
            path: path.clone(),
            accessed_ts,
            count,
        };
        history.insert(path, entry);
    }

    Ok(history)
}

/// Save visit history to CSV file
pub fn save_visit_history(
    history: &HashMap<PathBuf, VisitHistoryEntry>,
    config_dir_override: Option<&PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = config::get_kiorg_config_dir(config_dir_override);

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    let history_path = config_dir.join(HISTORY_FILE_NAME);
    let mut content = String::from("path,accessed_ts,count\n");

    for entry in history.values() {
        content.push_str(&format!(
            "{},{},{}\n",
            entry.path.display(),
            entry.accessed_ts,
            entry.count
        ));
    }

    std::fs::write(&history_path, content)?;
    Ok(())
}

/// Update visit history for a given path
pub fn update_visit_history(history: &mut HashMap<PathBuf, VisitHistoryEntry>, path: &Path) {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    match history.get_mut(path) {
        Some(entry) => {
            entry.accessed_ts = current_time;
            entry.count += 1;
        }
        None => {
            let entry = VisitHistoryEntry {
                path: path.to_path_buf(),
                accessed_ts: current_time,
                count: 1,
            };
            history.insert(path.to_path_buf(), entry);
        }
    }
}
