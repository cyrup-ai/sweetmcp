use chrono::Utc;
use extism_pdk::*;
use serde_json::{Value, json};
use sweetmcp_plugin_builder::prelude::*;
use sweetmcp_plugin_builder::{CallToolResult, Ready};

/// Time tool using plugin-builder
struct TimeTool;

impl McpTool for TimeTool {
    const NAME: &'static str = "time";

    fn description(builder: DescriptionBuilder) -> DescriptionBuilder {
        builder
            .does("Get current time in various formats and parse time strings")
            .when("you need to get the current UTC time")
            .when("you need to parse or format time strings")
            .when("you need to work with timestamps")
            .perfect_for("scheduling, logging, time-based calculations, and date/time operations")
    }

    fn schema(builder: SchemaBuilder) -> Value {
        builder
            .required_enum(
                "name",
                "Time operation to perform",
                &["get_time_utc", "parse_time"],
            )
            .optional_string(
                "time_string",
                "Time string to parse (for parse_time operation)",
            )
            .build()
    }

    fn execute(args: Value) -> Result<CallToolResult, Error> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("name parameter required"))?;

        match name {
            "get_time_utc" => {
                let now = Utc::now();
                let timestamp = now.timestamp().to_string();
                let rfc2822 = now.to_rfc2822().to_string();
                Ok(ContentBuilder::text(
                    json!({
                        "utc_time": timestamp,
                        "utc_time_rfc2822": rfc2822,
                    })
                    .to_string(),
                ))
            }
            "parse_time" => {
                let time_string = args
                    .get("time_string")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::msg("time_string parameter required for parse_time"))?;

                match chrono::DateTime::parse_from_rfc2822(time_string) {
                    Ok(dt) => Ok(ContentBuilder::text(
                        json!({
                            "parsed_time": dt.timestamp().to_string(),
                            "formatted": dt.to_rfc2822().to_string(),
                        })
                        .to_string(),
                    )),
                    Err(e) => Ok(ContentBuilder::error(format!(
                        "Failed to parse time: {}",
                        e
                    ))),
                }
            }
            _ => Ok(ContentBuilder::error(format!(
                "Unknown time operation: {}",
                name
            ))),
        }
    }
}

/// Create the plugin instance
#[allow(dead_code)]
fn plugin() -> McpPlugin<Ready> {
    mcp_plugin("time")
        .description("Time operations including getting current time and parsing time strings")
        .tool::<TimeTool>()
        .serve()
}

// Generate standard MCP entry points
sweetmcp_plugin_builder::generate_mcp_functions!(plugin);
