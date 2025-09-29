use crate::agent::memory::ConversationMemory;
use crate::agent::ToolRegistry;
use crate::error::AppResult;

/// Handles dynamic prompt generation for different contexts
pub struct PromptGenerator;

impl PromptGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generates initial prompt for first iteration
    pub fn generate_initial_prompt(
        &self,
        user_input: &str,
        memory: &ConversationMemory,
        registry: &ToolRegistry,
    ) -> AppResult<String> {
        let tools_description = registry.generate_enhanced_tools_description();
        let conversation_context = memory.get_context_for_prompt(true);
        let recent_tools = memory.get_recent_tool_usage();

        let recent_tools_hint = if !recent_tools.is_empty() {
            format!("Recently used tools: {}\n", recent_tools.join(", "))
        } else {
            String::new()
        };

        let prompt = format!(
            r#"You are a helpful AI assistant with access to various tools for managing notes and tasks. Your personality should be conversational, helpful, and similar to Claude Code's style.

{}Available Tools:
{}

{}Current User Request: {}

## Chain of Thought Instructions:

You will work through this request step by step, potentially using multiple tools to complete the task. Think through your approach and execute tools as needed.

**Execution Pattern:**
1. **Analyze the request** and explain your understanding
2. **Plan your approach** - what steps will you take?
3. **Execute tools** as needed with JSON format
4. **If more actions are needed**, indicate this clearly in your response
5. **Continue until the task is complete**

**Tool Usage Format:**
```json
{{
    "tool": "tool_name",
    "action": "action_name",
    "parameters": {{
        "param1": "value1"
    }}
}}
```

**Continuation Signals:**
If you need to continue with more actions after seeing tool results, include phrases like:
- "Let me also..."
- "I need to..."
- "Next, I'll..."
- "Additionally..."

**Conversational Style:**
- Be natural and explain your thinking process
- Show your reasoning for each step
- Explain what you're doing and why
- Continue until the user's request is fully satisfied

Start by analyzing the request and explaining your approach.
"#,
            conversation_context, tools_description, recent_tools_hint, user_input
        );

        Ok(prompt)
    }

    /// Generates continuation prompt for subsequent iterations
    pub fn generate_continuation_prompt(
        &self,
        original_request: &str,
        execution_context: &str,
        memory: &ConversationMemory,
        registry: &ToolRegistry,
    ) -> AppResult<String> {
        let tools_description = registry.generate_enhanced_tools_description();
        let conversation_context = memory.get_context_for_prompt(true);

        let prompt = format!(
            r#"{}

Available Tools:
{}

## Continuation Context:

Original User Request: {}

Execution History:
{}

## Continuation Instructions:

You are continuing to work on the user's original request. Based on what you've already done:

1. **Review** what has been accomplished so far
2. **Determine** if the original request is fully satisfied
3. **If more actions are needed**:
   - Explain what you need to do next
   - Execute the appropriate tools with JSON format
   - Use continuation signals like "Let me also...", "Next, I'll...", etc.
4. **If the task is complete**:
   - Provide a natural conclusion
   - Summarize what was accomplished
   - Don't include any tool JSON

**Tool Usage Format (if needed):**
```json
{{
    "tool": "tool_name",
    "action": "action_name",
    "parameters": {{
        "param1": "value1"
    }}
}}
```

**Your Goal:** Complete the user's original request fully. Continue if more actions would be helpful.
"#,
            conversation_context, tools_description, original_request, execution_context
        );

        Ok(prompt)
    }
}