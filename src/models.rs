use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub when: String,      // RFC3339
    pub text: String,
    pub tags: Vec<String>, // keep; we’ll use later
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DayLog {
    pub date: String,      // YYYY-MM-DD
    pub notes: Vec<Note>,
}
