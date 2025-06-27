mod plugin;

use extism_pdk::*;
use plugin::types::*;
use serde_json::json;

pub(crate) fn call(_input: CallToolRequest) -> Result<CallToolResult, Error> {
    let request = HttpRequest::new("https://1.1.1.1/cdn-cgi/trace");
    let response = http::request::<Vec<u8>>(&request, None)
        .map_err(|e| Error::msg(format!("Failed to make HTTP request: {}", e)))?;

    let text = String::from_utf8(response.body().to_vec())
        .map_err(|e| Error::msg(format!("Failed to parse response as UTF-8: {}", e)))?;

    // Parse the response to extract IP address
    let ip = text
        .lines()
        .find(|line| line.starts_with("ip="))
        .map(|line| line.trim_start_matches("ip="))
        .ok_or_else(|| Error::msg("Could not find IP address in response"))?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(ip.to_string()),
            mime_type: Some("text/plain".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![ToolDescription {
            name: "myip".into(),
            description: "Get the current IP address using Cloudflare's service".into(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": [],
            })
            .as_object()
            .unwrap()
            .clone(),
        }],
    })
}
