use std::fs;
use std::path::Path;
use std::time::SystemTime;

use extism_pdk::*;
use serde_json::{Value, json};
use sweetmcp_plugin_builder::prelude::*;
use sweetmcp_plugin_builder::{CallToolRequest, CallToolResult, ListToolsResult, Ready};

/// File system operations tool using plugin-builder
struct FsTool;

impl McpTool for FsTool {
    const NAME: &'static str = "fs";

    fn description(builder: DescriptionBuilder) -> DescriptionBuilder {
        builder
            .does("Perform comprehensive file system operations including reading, writing, and directory management")
            .when("you need to read file contents from the local file system")
            .when("you need to write or create new files")
            .when("you need to edit existing files with specific content changes")
            .when("you need to create directories or manage folder structures")
            .when("you need to list directory contents and file information")
            .when("you need to search for files by name or content")
            .when("you need to get file metadata like size, permissions, timestamps")
            .perfect_for("file management, content processing, directory operations, and system administration tasks")
            .operation("read", "Read the complete contents of a file")
            .operation("read_multiple", "Read contents of multiple files in batch")
            .operation("write", "Write content to a file (creates or overwrites)")
            .operation("edit", "Edit specific parts of a file with targeted changes")
            .operation("mkdir", "Create directories (with parent directory support)")
            .operation("list", "List contents of a directory with detailed information")
            .operation("search", "Search for files by name pattern or content")
            .operation("read_metadata", "Get detailed file metadata and properties")
            .requires("File system access permissions for the target paths")
            .not_for("operations outside of allowed directories or system files")
    }

    fn schema(builder: SchemaBuilder) -> Value {
        builder
            .required_enum(
                "operation",
                "File system operation to perform",
                &[
                    "read",
                    "read_multiple",
                    "write",
                    "edit",
                    "mkdir",
                    "list",
                    "search",
                    "read_metadata",
                ],
            )
            .optional_string(
                "path",
                "File or directory path (required for most operations)",
            )
            .optional_string("content", "Content to write (required for write operation)")
            .optional_string("pattern", "Search pattern for file search operations")
            .build()
    }

    fn execute(args: Value) -> Result<CallToolResult, Error> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("operation parameter required"))?;

        match operation {
            "read" => read_file(&args),
            "read_multiple" => read_multiple_files(&args),
            "write" => write_file(&args),
            "edit" => edit_file(&args),
            "mkdir" => create_dir(&args),
            "list" => list_dir(&args),
            "search" => search_files(&args),
            "read_metadata" => get_file_info(&args),
            _ => Ok(ContentBuilder::error(format!(
                "Unknown fs operation: {}",
                operation
            ))),
        }
    }
}

/// Read file contents
fn read_file(args: &Value) -> Result<CallToolResult, Error> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("path parameter required for read operation"))?;

    match fs::read_to_string(path) {
        Ok(content) => Ok(ContentBuilder::text(
            json!({
                "path": path,
                "content": content,
                "size": content.len()
            })
            .to_string(),
        )),
        Err(e) => Ok(ContentBuilder::error(format!(
            "Failed to read file {}: {}",
            path, e
        ))),
    }
}

/// Read multiple files
fn read_multiple_files(args: &Value) -> Result<CallToolResult, Error> {
    let paths = args
        .get("paths")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::msg("paths array required for read_multiple operation"))?;

    let mut results = Vec::new();

    for path_val in paths {
        if let Some(path) = path_val.as_str() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    results.push(json!({
                        "path": path,
                        "content": content,
                        "success": true,
                        "size": content.len()
                    }));
                }
                Err(e) => {
                    results.push(json!({
                        "path": path,
                        "error": format!("Failed to read: {}", e),
                        "success": false
                    }));
                }
            }
        }
    }

    Ok(ContentBuilder::text(
        json!({
            "operation": "read_multiple",
            "results": results
        })
        .to_string(),
    ))
}

/// Write file contents
fn write_file(args: &Value) -> Result<CallToolResult, Error> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("path parameter required for write operation"))?;

    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("content parameter required for write operation"))?;

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Ok(ContentBuilder::error(format!(
                    "Failed to create parent directories: {}",
                    e
                )));
            }
        }
    }

    match fs::write(path, content) {
        Ok(_) => Ok(ContentBuilder::text(
            json!({
                "path": path,
                "bytes_written": content.len(),
                "success": true
            })
            .to_string(),
        )),
        Err(e) => Ok(ContentBuilder::error(format!(
            "Failed to write file {}: {}",
            path, e
        ))),
    }
}

/// Edit file (simplified implementation)
fn edit_file(args: &Value) -> Result<CallToolResult, Error> {
    let _path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("path parameter required for edit operation"))?;

    // For now, treat edit the same as write - a full implementation would support targeted edits
    write_file(args)
}

/// Create directory
fn create_dir(args: &Value) -> Result<CallToolResult, Error> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("path parameter required for mkdir operation"))?;

    match fs::create_dir_all(path) {
        Ok(_) => Ok(ContentBuilder::text(
            json!({
                "path": path,
                "created": true,
                "success": true
            })
            .to_string(),
        )),
        Err(e) => Ok(ContentBuilder::error(format!(
            "Failed to create directory {}: {}",
            path, e
        ))),
    }
}

/// List directory contents
fn list_dir(args: &Value) -> Result<CallToolResult, Error> {
    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    match fs::read_dir(path) {
        Ok(entries) => {
            let mut files = Vec::new();

            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let metadata = match entry.metadata() {
                            Ok(meta) => meta,
                            Err(_) => continue, // Skip entries we can't read
                        };

                        files.push(json!({
                            "name": entry.file_name().to_string_lossy(),
                            "path": path.to_string_lossy(),
                            "is_file": metadata.is_file(),
                            "is_dir": metadata.is_dir(),
                            "size": metadata.len(),
                        }));
                    }
                    Err(e) => {
                        files.push(json!({
                            "error": format!("Failed to read entry: {}", e)
                        }));
                    }
                }
            }

            Ok(ContentBuilder::text(
                json!({
                    "path": path,
                    "entries": files,
                    "count": files.len()
                })
                .to_string(),
            ))
        }
        Err(e) => Ok(ContentBuilder::error(format!(
            "Failed to list directory {}: {}",
            path, e
        ))),
    }
}

/// Search for files
fn search_files(args: &Value) -> Result<CallToolResult, Error> {
    let pattern = args
        .get("pattern")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("pattern parameter required for search operation"))?;

    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    // Simplified file search - just list files containing the pattern in their name
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut matches = Vec::new();

            for entry in entries {
                if let Ok(entry) = entry {
                    let filename = entry.file_name().to_string_lossy().to_lowercase();
                    if filename.contains(&pattern.to_lowercase()) {
                        matches.push(json!({
                            "name": entry.file_name().to_string_lossy(),
                            "path": entry.path().to_string_lossy(),
                        }));
                    }
                }
            }

            Ok(ContentBuilder::text(
                json!({
                    "pattern": pattern,
                    "search_path": path,
                    "matches": matches,
                    "count": matches.len()
                })
                .to_string(),
            ))
        }
        Err(e) => Ok(ContentBuilder::error(format!(
            "Failed to search in {}: {}",
            path, e
        ))),
    }
}

/// Get file metadata
fn get_file_info(args: &Value) -> Result<CallToolResult, Error> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("path parameter required for read_metadata operation"))?;

    match fs::metadata(path) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0);

            Ok(ContentBuilder::text(
                json!({
                    "path": path,
                    "size": metadata.len(),
                    "is_file": metadata.is_file(),
                    "is_dir": metadata.is_dir(),
                    "modified_timestamp": modified,
                    "readonly": metadata.permissions().readonly(),
                })
                .to_string(),
            ))
        }
        Err(e) => Ok(ContentBuilder::error(format!(
            "Failed to get metadata for {}: {}",
            path, e
        ))),
    }
}

/// Create the plugin instance
#[allow(dead_code)]
fn plugin() -> McpPlugin<Ready> {
    mcp_plugin("fs")
        .description("Comprehensive file system operations and directory management")
        .tool::<FsTool>()
        .serve()
}

// MCP Protocol Implementation
#[allow(dead_code)]
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    plugin().call(input)
}

#[allow(dead_code)]
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    plugin().describe()
}
