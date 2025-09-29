use crate::{error::{AppError, AppResult}, models::DayLog, storage::Storage};
use chrono::NaiveDate;
use directories::ProjectDirs;
use std::{fs, io::Write, path::PathBuf};

pub struct FsStorage {
    root: PathBuf,
}

impl FsStorage {
    pub fn new() -> AppResult<Self> {
        let proj = ProjectDirs::from("com", "aigenda", "aigenda")
            .ok_or_else(|| AppError::Other("cannot resolve data dir".into()))?;
        let root = proj.data_dir().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn day_path(&self, date: NaiveDate) -> PathBuf {
        self.root.join(format!("{}.json", date.format("%Y-%m-%d")))
    }
}

impl Storage for FsStorage {
    fn load_day(&self, date: NaiveDate) -> AppResult<DayLog> {
        let p = self.day_path(date);
        if !p.exists() {
            return Ok(DayLog {
                date: date.format("%Y-%m-%d").to_string(),
                ..Default::default()
            });
        }
        let bytes = fs::read(p)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn save_day(&self, day: &DayLog) -> AppResult<()> {
        let date = NaiveDate::parse_from_str(&day.date, "%Y-%m-%d")?;
        let p = self.day_path(date);
        let json = serde_json::to_vec_pretty(day)?;
        let mut f = fs::File::create(p)?;
        f.write_all(&json)?;
        Ok(())
    }

    fn iter_days(&self) -> AppResult<Box<dyn Iterator<Item = AppResult<DayLog>>>> {
        let mut files = fs::read_dir(&self.root)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
            .collect::<Vec<_>>();
        files.sort();
        Ok(Box::new(files.into_iter().map(|p| {
            let bytes = fs::read(p)?;
            let day: DayLog = serde_json::from_slice(&bytes)?;
            Ok(day)
        })))
    }
}
