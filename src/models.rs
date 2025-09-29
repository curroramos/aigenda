use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[cfg(feature = "ai")]
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub when: String, // RFC3339
    pub text: String,
    pub tags: Vec<String>, // keep; we'll use later
}

impl Note {
    #[cfg(feature = "ai")]
    pub fn new(text: String) -> Self {
        Self {
            when: Utc::now().to_rfc3339(),
            text,
            tags: Vec::new(),
        }
    }

    #[cfg(feature = "ai")]
    pub fn when(&self) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&self.when)
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc)
    }

    #[cfg(feature = "ai")]
    pub fn text(&self) -> &str {
        &self.text
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DayLog {
    #[serde(with = "date_format")]
    pub date: NaiveDate,
    pub notes: Vec<Note>,
}

impl DayLog {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            notes: Vec::new(),
        }
    }

    #[cfg(feature = "ai")]
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    #[cfg(feature = "ai")]
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    #[cfg(feature = "ai")]
    pub fn notes_mut(&mut self) -> &mut Vec<Note> {
        &mut self.notes
    }

}

mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
