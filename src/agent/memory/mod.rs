use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::fs;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub timestamp: DateTime<Utc>,
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub action: String,
    pub parameters: Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub action: String,
    pub result: String,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMemory {
    messages: VecDeque<ConversationMessage>,
    max_messages: usize,
    current_context_tokens: usize,
    max_context_tokens: usize,
}

impl ConversationMemory {
    pub fn new(max_messages: usize, max_context_tokens: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages,
            current_context_tokens: 0,
            max_context_tokens,
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        let message = ConversationMessage {
            timestamp: Utc::now(),
            role: MessageRole::User,
            content,
            tool_calls: None,
            tool_results: None,
        };

        self.add_message(message);
    }

    pub fn add_assistant_message(&mut self, content: String, tool_calls: Option<Vec<ToolCall>>) {
        let message = ConversationMessage {
            timestamp: Utc::now(),
            role: MessageRole::Assistant,
            content,
            tool_calls,
            tool_results: None,
        };

        self.add_message(message);
    }

    pub fn add_tool_results(&mut self, results: Vec<ToolResult>) {
        if let Some(last_message) = self.messages.back_mut() {
            if matches!(last_message.role, MessageRole::Assistant) {
                last_message.tool_results = Some(results);
            }
        }
    }

    fn add_message(&mut self, message: ConversationMessage) {
        // Estimate token count (rough approximation: 1 token ≈ 4 characters)
        let estimated_tokens = message.content.len() / 4;

        self.messages.push_back(message);
        self.current_context_tokens += estimated_tokens;

        // Remove oldest messages if we exceed limits
        while self.messages.len() > self.max_messages ||
              self.current_context_tokens > self.max_context_tokens {
            if let Some(removed) = self.messages.pop_front() {
                let removed_tokens = removed.content.len() / 4;
                self.current_context_tokens = self.current_context_tokens.saturating_sub(removed_tokens);
            }
        }
    }

    pub fn get_context_for_prompt(&self, include_system_info: bool) -> String {
        let mut context = String::new();

        if include_system_info && !self.messages.is_empty() {
            context.push_str("## Conversation History\n");
            context.push_str("Previous messages and tool interactions:\n\n");
        }

        for message in &self.messages {
            match message.role {
                MessageRole::User => {
                    context.push_str(&format!("User: {}\n", message.content));
                }
                MessageRole::Assistant => {
                    context.push_str(&format!("Assistant: {}\n", message.content));

                    if let Some(tool_calls) = &message.tool_calls {
                        for call in tool_calls {
                            context.push_str(&format!(
                                "  → Called {}.{} with: {}\n",
                                call.tool_name, call.action, call.parameters
                            ));
                        }
                    }

                    if let Some(results) = &message.tool_results {
                        for result in results {
                            let status = if result.success { "✓" } else { "✗" };
                            context.push_str(&format!(
                                "  {} {}.{}: {} ({}ms)\n",
                                status, result.tool_name, result.action,
                                result.result, result.execution_time_ms
                            ));
                        }
                    }
                }
                MessageRole::System => {
                    context.push_str(&format!("System: {}\n", message.content));
                }
                MessageRole::Tool => {
                    // Tool messages are usually included in tool_results
                    continue;
                }
            }
        }

        if !context.is_empty() {
            context.push_str("\n---\n\n");
        }

        context
    }

    pub fn get_recent_tool_usage(&self) -> Vec<String> {
        let mut tools_used = Vec::new();

        for message in self.messages.iter().rev().take(5) {
            if let Some(tool_calls) = &message.tool_calls {
                for call in tool_calls {
                    let tool_action = format!("{}.{}", call.tool_name, call.action);
                    if !tools_used.contains(&tool_action) {
                        tools_used.push(tool_action);
                    }
                }
            }
        }

        tools_used
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.current_context_tokens = 0;
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn context_token_count(&self) -> usize {
        self.current_context_tokens
    }

    // Persistent storage methods
    pub fn load_from_file(file_path: &PathBuf, max_messages: usize, max_context_tokens: usize) -> AppResult<Self> {
        if file_path.exists() {
            let contents = fs::read_to_string(file_path)
                .map_err(|e| crate::error::AppError::Storage(format!("Failed to read memory file: {}", e)))?;

            let mut memory: ConversationMemory = serde_json::from_str(&contents)
                .map_err(|e| crate::error::AppError::Storage(format!("Failed to parse memory file: {}", e)))?;

            // Update limits in case they changed
            memory.max_messages = max_messages;
            memory.max_context_tokens = max_context_tokens;

            Ok(memory)
        } else {
            Ok(Self::new(max_messages, max_context_tokens))
        }
    }

    pub fn save_to_file(&self, file_path: &PathBuf) -> AppResult<()> {
        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| crate::error::AppError::Storage(format!("Failed to create memory directory: {}", e)))?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::AppError::Storage(format!("Failed to serialize memory: {}", e)))?;

        fs::write(file_path, contents)
            .map_err(|e| crate::error::AppError::Storage(format!("Failed to write memory file: {}", e)))?;

        Ok(())
    }

    pub fn get_memory_file_path() -> PathBuf {
        // Store in user data directory
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("aigenda");
        path.push("conversation_memory.json");
        path
    }
}