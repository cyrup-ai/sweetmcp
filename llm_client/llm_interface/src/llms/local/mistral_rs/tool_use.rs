use crate::requests::common::tools::{ToolCall, ToolChoice, ToolDefinition};
use mistralrs::{Tool as MistralTool, ToolCall as MistralToolCall, ToolChoice as MistralToolChoice};


/// Convert a common ToolDefinition to a MistralTool
pub fn to_mistral_tool(tool: &ToolDefinition) -> MistralTool {
    MistralTool {
        function: mistralrs::FunctionTool {
            name: tool.name.clone(),
            description: Some(tool.description.clone()),
            parameters: tool.input_schema.clone(),
        },
    }
}

/// Convert a ToolChoice to a MistralToolChoice
pub fn to_mistral_tool_choice(choice: &ToolChoice) -> MistralToolChoice {
    match choice {
        ToolChoice::Auto => MistralToolChoice::Auto,
        ToolChoice::None => MistralToolChoice::None,
        ToolChoice::Any => MistralToolChoice::Any,
        ToolChoice::Tool { name } => MistralToolChoice::Tool {
            name: name.clone(),
        },
    }
}

/// Convert a MistralToolCall to a common ToolCall
pub fn from_mistral_tool_call(call: &MistralToolCall) -> ToolCall {
    ToolCall {
        id: call.id.clone(),
        name: call.function.name.clone(),
        arguments: call.function.arguments.clone(),
    }
}

/// Convert a vector of common ToolDefinition to a vector of MistralTool
pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<MistralTool> {
    tools.iter().map(to_mistral_tool).collect()
}
