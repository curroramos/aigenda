use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub when: String, // RFC3339
    pub text: String,
    pub tags: Vec<String>, // keep; we'll use later
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
