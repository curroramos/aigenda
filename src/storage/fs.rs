use std::fs;
use std::path::PathBuf;
use chrono::NaiveDate;
use directories::ProjectDirs;

use crate::{
    error::{AppError, AppResult},
    models::DayLog,
};
use super::Storage;

pub struct FsStorage {
    data_dir: PathBuf,
}

impl FsStorage {
    pub fn new() -> AppResult<Self> {
        let dirs = ProjectDirs::from("com", "example", "aigenda")
            .ok_or_else(|| AppError::Storage("Could not determine data directory".to_string()))?;

        let data_dir = dirs.data_dir().to_path_buf();
        fs::create_dir_all(&data_dir)
            .map_err(|e| AppError::Storage(format!("Could not create data directory: {}", e)))?;

        Ok(Self { data_dir })
    }

    fn day_file_path(&self, date: NaiveDate) -> PathBuf {
        self.data_dir.join(format!("{}.json", date.format("%Y-%m-%d")))
    }
}

impl Storage for FsStorage {
    fn load_day(&self, date: NaiveDate) -> AppResult<DayLog> {
        let path = self.day_file_path(date);

        if !path.exists() {
            return Ok(DayLog::new(date));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::Storage(format!("Could not read file {}: {}", path.display(), e)))?;

        serde_json::from_str(&content)
            .map_err(|e| AppError::Storage(format!("Could not parse JSON from {}: {}", path.display(), e)))
    }

    fn save_day(&self, day: &DayLog) -> AppResult<()> {
        let path = self.day_file_path(day.date);

        let content = serde_json::to_string_pretty(day)
            .map_err(|e| AppError::Storage(format!("Could not serialize day log: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| AppError::Storage(format!("Could not write to {}: {}", path.display(), e)))
    }

    fn iter_days(&self) -> AppResult<Box<dyn Iterator<Item = AppResult<DayLog>>>> {
        let entries = fs::read_dir(&self.data_dir)
            .map_err(|e| AppError::Storage(format!("Could not read data directory: {}", e)))?;

        let mut day_logs = Vec::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| AppError::Storage(format!("Could not read directory entry: {}", e)))?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| AppError::Storage(format!("Could not read file {}: {}", path.display(), e)))?;

                match serde_json::from_str::<DayLog>(&content) {
                    Ok(day_log) => day_logs.push(Ok(day_log)),
                    Err(e) => day_logs.push(Err(AppError::Storage(format!(
                        "Could not parse JSON from {}: {}", path.display(), e
                    )))),
                }
            }
        }

        Ok(Box::new(day_logs.into_iter()))
    }
}