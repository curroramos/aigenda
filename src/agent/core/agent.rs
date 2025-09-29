use crate::agent::execution::{ChainExecutor, ContinuationDetector};
use crate::agent::memory::ConversationMemory;
use crate::agent::prompts::PromptGenerator;
use crate::agent::streaming::{StreamingHandler, ConsoleStreamingHandler};
use crate::agent::tool_executor::ToolExecutor;
use crate::agent::ToolRegistry;
use crate::error::AppResult;

#[cfg(feature = "ai")]
use uuid::Uuid;

/// Main agent orchestrator - coordinates all components
pub struct Agent {
    registry: ToolRegistry,
    claude_client: Option<crate::ai::claude::ClaudeClient>,
    memory: ConversationMemory,
    session_id: String,

    // Specialized components
    chain_executor: ChainExecutor,
    continuation_detector: ContinuationDetector,
    tool_executor: ToolExecutor,
    prompt_generator: PromptGenerator,
}

impl Agent {
    pub fn new() -> AppResult<Self> {
        let mut registry = ToolRegistry::new();
        registry.auto_discover_tools()?;

        // Load persistent memory
        let memory_path = ConversationMemory::get_memory_file_path();
        let memory = ConversationMemory::load_from_file(&memory_path, 50, 8000)?;

        Ok(Self {
            claude_client: None,
            memory,
            session_id: Uuid::new_v4().to_string(),
            chain_executor: ChainExecutor::new(5), // max 5 iterations
            continuation_detector: ContinuationDetector,
            tool_executor: ToolExecutor::new(),
            prompt_generator: PromptGenerator::new(),
            registry,
        })
    }

    pub fn new_with_memory_limits(max_messages: usize, max_tokens: usize) -> AppResult<Self> {
        let mut registry = ToolRegistry::new();
        registry.auto_discover_tools()?;

        // Load persistent memory with custom limits
        let memory_path = ConversationMemory::get_memory_file_path();
        let memory = ConversationMemory::load_from_file(&memory_path, max_messages, max_tokens)?;

        Ok(Self {
            claude_client: None,
            memory,
            session_id: Uuid::new_v4().to_string(),
            chain_executor: ChainExecutor::new(5),
            continuation_detector: ContinuationDetector,
            tool_executor: ToolExecutor::new(),
            prompt_generator: PromptGenerator::new(),
            registry,
        })
    }

    pub fn with_claude_client(mut self, client: crate::ai::claude::ClaudeClient) -> Self {
        self.claude_client = Some(client);
        self
    }

    /// Main execution entry point with streaming support
    pub async fn execute_command_streaming<H>(
        &mut self,
        input: &str,
        streaming_handler: &mut H
    ) -> AppResult<String>
    where
        H: StreamingHandler,
    {
        // Store user message in memory
        self.memory.add_user_message(input.to_string());

        let mut full_conversation = String::new();
        let mut execution_context = String::new();
        let mut continue_loop = true;
        let mut loop_count = 0;
        let max_loops = 5;

        while continue_loop && loop_count < max_loops {
            loop_count += 1;

            streaming_handler.on_iteration_start(loop_count)?;

            // Generate prompt for this iteration
            let prompt = self.generate_prompt_for_iteration(input, &execution_context, loop_count)?;

            // Handle this iteration with streaming
            let (iteration_result, should_continue) = self.handle_iteration_streaming(&prompt, streaming_handler).await?;

            streaming_handler.on_iteration_end(loop_count, &iteration_result)?;

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

        // Save memory to disk after complete execution
        self.save_memory()?;

        Ok(full_conversation)
    }

    /// Main execution entry point (legacy, non-streaming)
    pub async fn execute_command(&mut self, input: &str) -> AppResult<String> {
        let mut default_handler = ConsoleStreamingHandler::new();
        self.execute_command_streaming(input, &mut default_handler).await
    }

    /// Generates prompt for a specific iteration
    fn generate_prompt_for_iteration(
        &self,
        user_input: &str,
        execution_context: &str,
        iteration: usize,
    ) -> AppResult<String> {
        if iteration == 1 {
            self.prompt_generator.generate_initial_prompt(user_input, &self.memory, &self.registry)
        } else {
            self.prompt_generator.generate_continuation_prompt(
                user_input,
                execution_context,
                &self.memory,
                &self.registry,
            )
        }
    }

    /// Handles a single iteration of the chain with streaming
    async fn handle_iteration_streaming<H>(
        &mut self,
        prompt: &str,
        streaming_handler: &mut H
    ) -> AppResult<(String, bool)>
    where
        H: StreamingHandler,
    {
        // Send to Claude API
        let claude_client = self.claude_client.as_ref()
            .ok_or_else(|| crate::error::AppError::Storage("Claude client not configured".to_string()))?;

        let assistant_response = claude_client.chat(prompt).await?;

        // Stream the LLM response immediately
        streaming_handler.on_llm_response(&assistant_response)?;

        // Execute any tool calls in the response with streaming
        let (tool_calls, tool_results, tool_output) = self.tool_executor
            .execute_tools_from_response_streaming(&assistant_response, &self.registry, streaming_handler)
            .await?;

        // Build iteration result
        let mut iteration_result = assistant_response.clone();
        if !tool_output.is_empty() {
            iteration_result.push_str("\n\n**Tool Results:**\n");
            iteration_result.push_str(&tool_output);
        }

        // Store complete response in memory
        self.memory.add_assistant_message(assistant_response.clone(), Some(tool_calls));
        for result in tool_results {
            self.memory.add_tool_results(vec![result]);
        }

        // Determine if we should continue the chain
        let should_continue = !tool_output.is_empty() &&
            self.continuation_detector.should_continue(&assistant_response);

        Ok((iteration_result, should_continue))
    }

    /// Handles a single iteration of the chain (legacy, non-streaming)
    async fn handle_iteration(&mut self, prompt: &str) -> AppResult<(String, bool)> {
        // Send to Claude API
        let claude_client = self.claude_client.as_ref()
            .ok_or_else(|| crate::error::AppError::Storage("Claude client not configured".to_string()))?;

        let assistant_response = claude_client.chat(prompt).await?;

        // Execute any tool calls in the response
        let (tool_calls, tool_results, tool_output) = self.tool_executor
            .execute_tools_from_response(&assistant_response, &self.registry)
            .await?;

        // Build iteration result
        let mut iteration_result = assistant_response.clone();
        if !tool_output.is_empty() {
            iteration_result.push_str("\n\n**Tool Results:**\n");
            iteration_result.push_str(&tool_output);
        }

        // Store complete response in memory
        self.memory.add_assistant_message(assistant_response.clone(), Some(tool_calls));
        for result in tool_results {
            self.memory.add_tool_results(vec![result]);
        }

        // Determine if we should continue the chain
        let should_continue = !tool_output.is_empty() &&
            self.continuation_detector.should_continue(&assistant_response);

        Ok((iteration_result, should_continue))
    }

    // Utility methods
    pub fn list_available_tools(&self) -> Vec<String> {
        self.registry.list_tools()
    }

    pub fn get_conversation_history(&self) -> String {
        self.memory.get_context_for_prompt(false)
    }

    pub fn clear_memory(&mut self) {
        self.memory.clear();
    }

    pub fn get_memory_stats(&self) -> (usize, usize) {
        (self.memory.message_count(), self.memory.context_token_count())
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    pub fn get_tool_schemas(&self) -> Vec<String> {
        self.registry.get_enhanced_schemas()
    }

    fn save_memory(&self) -> AppResult<()> {
        let memory_path = ConversationMemory::get_memory_file_path();
        self.memory.save_to_file(&memory_path)
    }
}