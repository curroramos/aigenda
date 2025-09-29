use async_trait::async_trait;
use serde_json::Value;
use crate::agent::tools::{Tool, ToolAction, ToolSchema, ToolCategory, ActionSchema, ParameterSchema, ParameterType, ReturnSchema, ToolExample};
use crate::error::AppResult;
use crate::storage::Storage;
use crate::models::{Note, DayLog};
use chrono::{NaiveDate, Utc};
use std::sync::Arc;

pub struct NotesTool {
    storage: Arc<dyn Storage>,
}

impl NotesTool {
    pub fn new() -> AppResult<Self> {
        let storage = Arc::new(crate::storage::fs::FsStorage::new()?);
        Ok(Self { storage })
    }

    async fn create_note(&self, text: &str, date: Option<&str>) -> AppResult<String> {
        let target_date = if let Some(date_str) = date {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| crate::error::AppError::ChronoParse(e))?
        } else {
            Utc::now().date_naive()
        };

        let note = Note::new(text.to_string());

        let mut day_log = self.storage.load_day(target_date)
            .unwrap_or_else(|_| DayLog::new(target_date));

        day_log.add_note(note);
        self.storage.save_day(&day_log)?;

        Ok(format!("Note added successfully for {}", target_date))
    }

    async fn read_notes(&self, date: Option<&str>, limit: Option<u32>) -> AppResult<String> {
        if let Some(date_str) = date {
            let target_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| crate::error::AppError::ChronoParse(e))?;

            let day_log = self.storage.load_day(target_date)?;
            let notes = day_log.notes();

            let limited_notes: Vec<_> = if let Some(limit) = limit {
                notes.iter().take(limit as usize).collect()
            } else {
                notes.iter().collect()
            };

            if limited_notes.is_empty() {
                Ok(format!("No notes found for {}", target_date))
            } else {
                let mut result = format!("Notes for {}:\n", target_date);
                for (i, note) in limited_notes.iter().enumerate() {
                    result.push_str(&format!("{}. [{}] {}\n",
                        i + 1,
                        note.when().format("%H:%M"),
                        note.text()
                    ));
                }
                Ok(result)
            }
        } else {
            // List recent notes from multiple days
            let mut days = self.storage.iter_days()?;
            days.reverse(); // Most recent first
            let mut result = String::from("Recent notes:\n");
            let mut count = 0;
            let max_count = limit.unwrap_or(10);

            for day_log in days {
                if count >= max_count {
                    break;
                }

                for note in day_log.notes().iter().rev() {
                    result.push_str(&format!("[{}] {}\n",
                        note.when().format("%Y-%m-%d %H:%M"),
                        note.text()
                    ));
                    count += 1;
                    if count >= max_count {
                        break;
                    }
                }
            }

            if count == 0 {
                Ok("No notes found".to_string())
            } else {
                Ok(result)
            }
        }
    }

    async fn update_note(&self, date: &str, index: u32, new_text: &str) -> AppResult<String> {
        let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| crate::error::AppError::ChronoParse(e))?;

        let mut day_log = self.storage.load_day(target_date)?;

        if let Some(note) = day_log.notes_mut().get_mut(index as usize) {
            *note = Note::new(new_text.to_string());
            self.storage.save_day(&day_log)?;
            Ok(format!("Note {} updated successfully for {}", index + 1, target_date))
        } else {
            Err(crate::error::AppError::Storage(
                format!("Note {} not found for {}", index + 1, target_date)
            ))
        }
    }

    async fn delete_note(&self, date: &str, index: u32) -> AppResult<String> {
        let target_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| crate::error::AppError::ChronoParse(e))?;

        let mut day_log = self.storage.load_day(target_date)?;

        if (index as usize) < day_log.notes().len() {
            day_log.notes_mut().remove(index as usize);
            self.storage.save_day(&day_log)?;
            Ok(format!("Note {} deleted successfully from {}", index + 1, target_date))
        } else {
            Err(crate::error::AppError::Storage(
                format!("Note {} not found for {}", index + 1, target_date)
            ))
        }
    }
}

#[async_trait]
impl Tool for NotesTool {
    fn name(&self) -> &str {
        "notes"
    }

    fn description(&self) -> &str {
        "Manage daily notes with full CRUD operations"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Internal
    }

    fn get_schema(&self) -> ToolSchema {
        ToolSchema {
            name: "notes".to_string(),
            description: "Manage daily notes with full CRUD operations. Notes are automatically organized by date and stored locally as JSON files.".to_string(),
            category: ToolCategory::Internal,
            actions: vec![
                ActionSchema {
                    name: "create".to_string(),
                    description: "Add a new note to today's log or a specific date".to_string(),
                    parameters: vec![
                        ParameterSchema {
                            name: "text".to_string(),
                            description: "The content of the note".to_string(),
                            param_type: ParameterType::String { max_length: Some(5000) },
                            required: true,
                            default_value: None,
                            validation: None,
                        },
                        ParameterSchema {
                            name: "date".to_string(),
                            description: "Date for the note in YYYY-MM-DD format".to_string(),
                            param_type: ParameterType::Date,
                            required: false,
                            default_value: Some(serde_json::Value::String("today".to_string())),
                            validation: None,
                        },
                    ],
                    returns: ReturnSchema {
                        description: "Confirmation message with the date the note was added".to_string(),
                        return_type: ParameterType::String { max_length: None },
                        possible_errors: vec!["Invalid date format".to_string()],
                    },
                },
                ActionSchema {
                    name: "read".to_string(),
                    description: "Read notes from a specific date or recent notes across multiple days".to_string(),
                    parameters: vec![
                        ParameterSchema {
                            name: "date".to_string(),
                            description: "Date in YYYY-MM-DD format to read notes from".to_string(),
                            param_type: ParameterType::Date,
                            required: false,
                            default_value: None,
                            validation: None,
                        },
                        ParameterSchema {
                            name: "limit".to_string(),
                            description: "Maximum number of notes to return".to_string(),
                            param_type: ParameterType::Integer { min: Some(1), max: Some(100) },
                            required: false,
                            default_value: Some(serde_json::Value::Number(serde_json::Number::from(10))),
                            validation: None,
                        },
                    ],
                    returns: ReturnSchema {
                        description: "List of notes with timestamps and content".to_string(),
                        return_type: ParameterType::String { max_length: None },
                        possible_errors: vec!["Date not found".to_string(), "Invalid date format".to_string()],
                    },
                },
                ActionSchema {
                    name: "update".to_string(),
                    description: "Update an existing note by its position/index".to_string(),
                    parameters: vec![
                        ParameterSchema {
                            name: "date".to_string(),
                            description: "Date of the note in YYYY-MM-DD format".to_string(),
                            param_type: ParameterType::Date,
                            required: true,
                            default_value: None,
                            validation: None,
                        },
                        ParameterSchema {
                            name: "index".to_string(),
                            description: "Position of the note to update (1-based index)".to_string(),
                            param_type: ParameterType::Integer { min: Some(1), max: None },
                            required: true,
                            default_value: None,
                            validation: None,
                        },
                        ParameterSchema {
                            name: "text".to_string(),
                            description: "New content for the note".to_string(),
                            param_type: ParameterType::String { max_length: Some(5000) },
                            required: true,
                            default_value: None,
                            validation: None,
                        },
                    ],
                    returns: ReturnSchema {
                        description: "Confirmation message with the updated note details".to_string(),
                        return_type: ParameterType::String { max_length: None },
                        possible_errors: vec!["Note not found".to_string(), "Invalid date format".to_string(), "Invalid index".to_string()],
                    },
                },
                ActionSchema {
                    name: "delete".to_string(),
                    description: "Delete a specific note by its position/index".to_string(),
                    parameters: vec![
                        ParameterSchema {
                            name: "date".to_string(),
                            description: "Date of the note in YYYY-MM-DD format".to_string(),
                            param_type: ParameterType::Date,
                            required: true,
                            default_value: None,
                            validation: None,
                        },
                        ParameterSchema {
                            name: "index".to_string(),
                            description: "Position of the note to delete (1-based index)".to_string(),
                            param_type: ParameterType::Integer { min: Some(1), max: None },
                            required: true,
                            default_value: None,
                            validation: None,
                        },
                    ],
                    returns: ReturnSchema {
                        description: "Confirmation message with the deleted note details".to_string(),
                        return_type: ParameterType::String { max_length: None },
                        possible_errors: vec!["Note not found".to_string(), "Invalid date format".to_string(), "Invalid index".to_string()],
                    },
                },
            ],
            examples: vec![
                ToolExample {
                    description: "Create a simple note for today".to_string(),
                    user_request: "add a note about finishing the AI implementation".to_string(),
                    tool_call: serde_json::json!({
                        "tool": "notes",
                        "action": "create",
                        "parameters": {
                            "text": "Finished implementing AI tools with enhanced memory and schema support"
                        }
                    }),
                    expected_result: "Note added successfully for 2025-09-28".to_string(),
                },
                ToolExample {
                    description: "Read today's notes".to_string(),
                    user_request: "show me today's notes".to_string(),
                    tool_call: serde_json::json!({
                        "tool": "notes",
                        "action": "read",
                        "parameters": {}
                    }),
                    expected_result: "Notes for 2025-09-28:\n1. [20:45] Finished implementing AI tools".to_string(),
                },
                ToolExample {
                    description: "Update a specific note".to_string(),
                    user_request: "update my first note from today".to_string(),
                    tool_call: serde_json::json!({
                        "tool": "notes",
                        "action": "update",
                        "parameters": {
                            "date": "2025-09-28",
                            "index": 1,
                            "text": "Updated: Completed implementing AI tools with memory and enhanced schemas"
                        }
                    }),
                    expected_result: "Note 1 updated successfully for 2025-09-28".to_string(),
                },
            ],
        }
    }

    fn actions(&self) -> Vec<ToolAction> {
        vec![
            ToolAction::new("create", "Add a new note")
                .with_parameter("text", "The note content", true, "string")
                .with_parameter("date", "Date in YYYY-MM-DD format (defaults to today)", false, "string"),

            ToolAction::new("read", "Read notes")
                .with_parameter("date", "Date in YYYY-MM-DD format (optional, shows recent notes if omitted)", false, "string")
                .with_parameter("limit", "Maximum number of notes to show", false, "number"),

            ToolAction::new("update", "Update an existing note")
                .with_parameter("date", "Date in YYYY-MM-DD format", true, "string")
                .with_parameter("index", "Note index (1-based)", true, "number")
                .with_parameter("text", "New note content", true, "string"),

            ToolAction::new("delete", "Delete a note")
                .with_parameter("date", "Date in YYYY-MM-DD format", true, "string")
                .with_parameter("index", "Note index (1-based)", true, "number"),
        ]
    }

    async fn execute(&self, action: &str, parameters: &Value) -> AppResult<String> {
        match action {
            "create" => {
                let text = parameters["text"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing text parameter".to_string()))?;
                let date = parameters["date"].as_str();
                self.create_note(text, date).await
            }
            "read" => {
                let date = parameters["date"].as_str();
                let limit = parameters["limit"].as_u64().map(|l| l as u32);
                self.read_notes(date, limit).await
            }
            "update" => {
                let date = parameters["date"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing date parameter".to_string()))?;
                let index = parameters["index"].as_u64()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing index parameter".to_string()))? as u32;
                let text = parameters["text"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing text parameter".to_string()))?;
                self.update_note(date, index.saturating_sub(1), text).await
            }
            "delete" => {
                let date = parameters["date"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing date parameter".to_string()))?;
                let index = parameters["index"].as_u64()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing index parameter".to_string()))? as u32;
                self.delete_note(date, index.saturating_sub(1)).await
            }
            _ => Err(crate::error::AppError::Storage(format!("Unknown action: {}", action)))
        }
    }
}