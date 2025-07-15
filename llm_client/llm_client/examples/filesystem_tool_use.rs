use futures_util::StreamExt;
use llm_client::*;
use llm_client::tools::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// An example showing how to use streaming with tools to implement filesystem functionality
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    // Create a client that supports tool use
    // For this example, we'll use Anthropic Claude which has strong tool use capabilities
    let llm_client = LlmClient::anthropic().claude_3_sonnet().init().unwrap();

    // Create a tool-enabled completion request
    let mut tool_completion = llm_client.basic_completion();

    // Enable streaming for real-time tool interactions
    tool_completion.stream_response(true);

    // Define our filesystem tools
    let list_files_tool = ToolDefinition::new(
        "list_files",
        "List files in a directory",
        ListFilesParams::schema(),
        handle_list_files,
    );

    let read_file_tool = ToolDefinition::new(
        "read_file",
        "Read the contents of a file",
        ReadFileParams::schema(),
        handle_read_file,
    );

    // Add tools to the completion request
    tool_completion.add_tool(list_files_tool);
    tool_completion.add_tool(read_file_tool);

    // Set up the prompt
    tool_completion
        .prompt()
        .add_system_message()
        .unwrap()
        .set_content("You're a helpful file system assistant that can list files and read their contents.");
    
    tool_completion
        .prompt()
        .add_user_message()
        .unwrap()
        .set_content("List the files in the current directory and then read the first Rust file you find.");

    // Process the streaming response with tool calls
    println!("Starting interactive tool session...\n");
    
    // Get a stream of response chunks
    let mut stream = tool_completion.stream().await.unwrap();
    
    // Variables to track the current tool call
    let mut current_tool = String::new();
    let mut current_tool_id = String::new();
    let mut current_tool_input = String::new();
    
    // Process each chunk as it arrives
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // Regular text output from the model
                if let Some(text) = chunk.text_delta {
                    print!("{}", text);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
                
                // Tool call name
                if let Some(tool_name) = chunk.tool_call_name {
                    if current_tool.is_empty() {
                        println!("\n[Starting tool call: {}]", tool_name);
                    }
                    current_tool = tool_name;
                }
                
                // Tool call ID
                if let Some(tool_id) = chunk.tool_call_id {
                    current_tool_id = tool_id;
                }
                
                // Tool input parameters (streaming)
                if let Some(input_delta) = chunk.tool_call_input_delta {
                    current_tool_input.push_str(&input_delta);
                }
                
                // Check if we've reached the end of the response
                if let Some(finish_reason) = chunk.finish_reason {
                    // Check if this is a tool call completion (using a pattern match instead of direct comparison)
                    let is_tool_call = matches!(finish_reason, CompletionFinishReason::NonMatchingStoppingSequence(Some(ref reason)) if reason == "tool_call");
                    
                    if is_tool_call && !current_tool.is_empty() {
                        println!("\n[Executing tool: {} with input: {}]", current_tool, current_tool_input);
                        
                        // Execute the tool and add the result to the conversation
                        if let Ok(tool_result) = execute_tool(&current_tool, &current_tool_id, &current_tool_input) {
                            println!("\n[Tool result: {}]", tool_result);
                            
                            // Add the tool result to the conversation
                            tool_completion
                                .prompt()
                                .add_tool_result_message(&current_tool_id, &tool_result)
                                .unwrap();
                            
                            // Reset for the next tool call
                            current_tool = String::new();
                            current_tool_id = String::new();
                            current_tool_input = String::new();
                            
                            // Continue the conversation with the tool result
                            stream = tool_completion.stream().await.unwrap();
                        } else {
                            println!("\n[Tool execution failed]");
                            break;
                        }
                    } else {
                        println!("\n\nStream finished: {:?}", finish_reason);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error in stream: {:?}", e);
                break;
            }
        }
    }
}

// Tool parameter definitions

#[derive(Debug, Serialize, Deserialize)]
struct ListFilesParams {
    /// The directory path to list files from. Default is the current directory.
    #[serde(default = "default_dir")]
    directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadFileParams {
    /// The full path to the file to read
    file_path: String,
}

fn default_dir() -> String {
    ".".to_string()
}

// Tool schemas
impl ListFilesParams {
    fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "directory": {
                    "type": "string",
                    "description": "The directory path to list files from. Default is the current directory."
                }
            },
            "required": []
        })
    }
}

impl ReadFileParams {
    fn schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The full path to the file to read"
                }
            },
            "required": ["file_path"]
        })
    }
}

// Tool handlers
fn handle_list_files(params_json: &str) -> Result<String, String> {
    let params: ListFilesParams = serde_json::from_str(params_json)
        .map_err(|e| format!("Failed to parse parameters: {}", e))?;
    
    let dir_path = PathBuf::from(params.directory);
    
    let entries = std::fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut result = Vec::new();
    
    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                "directory"
            } else {
                "file"
            };
            
            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            
            result.push(format!("{} ({}, {} bytes)", file_name, file_type, size));
        }
    }
    
    if result.is_empty() {
        Ok("No files found in the directory.".to_string())
    } else {
        Ok(format!("Files in directory:\n{}", result.join("\n")))
    }
}

fn handle_read_file(params_json: &str) -> Result<String, String> {
    let params: ReadFileParams = serde_json::from_str(params_json)
        .map_err(|e| format!("Failed to parse parameters: {}", e))?;
    
    let file_path = PathBuf::from(&params.file_path);
    
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", params.file_path));
    }
    
    if !file_path.is_file() {
        return Err(format!("Path is not a file: {}", params.file_path));
    }
    
    std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

// Helper function to execute a tool by name
fn execute_tool(tool_name: &str, tool_id: &str, params_json: &str) -> Result<String, String> {
    match tool_name {
        "list_files" => handle_list_files(params_json),
        "read_file" => handle_read_file(params_json),
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}
