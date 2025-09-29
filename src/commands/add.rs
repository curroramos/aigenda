use crate::{
    error::AppResult,
    models::{DayLog, Note},
    storage::Storage,
};
use chrono::Local;

pub fn run_add<S: Storage>(store: &S, words: Vec<String>) -> AppResult<()> {
    let now = Local::now();
    let text = words.join(" ");
    let mut day = store.load_day(now.date_naive())?;
    day.notes.push(Note {
        when: now.to_rfc3339(),
        text,
        tags: vec![],
    });
    store.save_day(&day)?;
    println!("Added note to {}.", day.date);
    Ok(())
}
