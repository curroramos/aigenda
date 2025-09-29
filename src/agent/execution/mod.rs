use crate::agent::memory::ConversationMemory;
use crate::error::AppResult;
use serde_json::Value;

/// Handles the chain of thoughts execution loop
pub struct ChainExecutor {
    max_iterations: usize,
}

impl ChainExecutor {
    pub fn new(max_iterations: usize) -> Self {
        Self { max_iterations }
    }

    pub async fn execute_chain<F, G>(
        &self,
        user_input: &str,
        memory: &mut ConversationMemory,
        mut prompt_generator: F,
        mut iteration_handler: G,
    ) -> AppResult<String>
    where
        F: FnMut(&str, &str, usize) -> AppResult<String>, // (user_input, execution_context, iteration) -> prompt
        G: FnMut(&str) -> AppResult<(String, bool)>, // (prompt) -> (result, should_continue)
    {
        // Store user message in memory
        memory.add_user_message(user_input.to_string());

        let mut full_conversation = String::new();
        let mut execution_context = String::new();
        let mut continue_loop = true;
        let mut loop_count = 0;

        while continue_loop && loop_count < self.max_iterations {
            loop_count += 1;

            // Generate prompt for this iteration
            let prompt = prompt_generator(user_input, &execution_context, loop_count)?;

            // Handle this iteration
            let (iteration_result, should_continue) = iteration_handler(&prompt)?;

            // Update conversation
            if !full_conversation.is_empty() {
                full_conversation.push_str("\n\n---\n\n");
            }
            full_conversation.push_str(&iteration_result);

            // Update execution context for next iteration
            if should_continue {
                execution_context.push_str(&format!(
                    "\nIteration {}:\n{}\n",
                    loop_count,
                    iteration_result
                ));
            }

            continue_loop = should_continue;
        }

        Ok(full_conversation)
    }
}

/// Detects continuation signals in agent responses
pub struct ContinuationDetector;

impl ContinuationDetector {
    pub fn should_continue(&self, response: &str) -> bool {
        let response_lower = response.to_lowercase();

        response_lower.contains("let me also") ||
        response_lower.contains("i'll also") ||
        response_lower.contains("next, i'll") ||
        response_lower.contains("additionally") ||
        response_lower.contains("i need to") ||
        response_lower.contains("i should also") ||
        response_lower.contains("now i'll") ||
        response_lower.contains("then i'll")
    }
}