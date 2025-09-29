use crate::{error::AppResult, models::DayLog};
use chrono::NaiveDate;

pub mod fs;

pub trait Storage: Send + Sync {
    fn load_day(&self, date: NaiveDate) -> AppResult<DayLog>;
    fn save_day(&self, day: &DayLog) -> AppResult<()>;
    fn iter_days(&self) -> AppResult<Vec<DayLog>>;
}
