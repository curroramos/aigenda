use crate::{error::AppResult, storage::Storage};
use chrono::NaiveDate;

pub fn run_list<S: Storage>(store: &S, all: bool, date: Option<String>) -> AppResult<()> {
    if all {
        for day in store.iter_days()? {
            print_day(&day);
        }
        return Ok(());
    }

    if let Some(d) = date {
        let parsed = NaiveDate::parse_from_str(&d, "%Y-%m-%d")?;
        let day = store.load_day(parsed)?;
        print_day(&day);
        return Ok(());
    }

    let today = chrono::Local::now().date_naive();
    let day = store.load_day(today)?;
    print_day(&day);
    Ok(())
}

fn print_day(day: &crate::models::DayLog) {
    if day.notes.is_empty() {
        println!("(no notes) {}", day.date);
        return;
    }
    println!("# {}", day.date);
    for (i, n) in day.notes.iter().enumerate() {
        println!("- [{:02}] {}", i + 1, n.text);
    }
    println!();
}
