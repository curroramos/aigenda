use crate::error::AppResult;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

pub struct ClaudeClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ClaudeClient {
    pub fn new() -> AppResult<Self> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| crate::error::AppError::Storage(
                "ANTHROPIC_API_KEY environment variable not set".to_string()
            ))?;

        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        })
    }

    pub async fn chat(&self, prompt: &str) -> AppResult<String> {
        let request_body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 1024,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::error::AppError::Storage(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::AppError::Storage(
                format!("API request failed with status {}: {}", status, error_text)
            ));
        }

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| crate::error::AppError::Storage(format!("Failed to parse response: {}", e)))?;

        // Extract the content from Claude's response
        if let Some(content) = response_json
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str())
        {
            Ok(content.to_string())
        } else {
            Err(crate::error::AppError::Storage(
                "Unexpected response format from Claude API".to_string()
            ))
        }
    }
}