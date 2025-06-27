use log::{error, info};
use mcpr::{client::Client, error::MCPError, transport::stdio::StdioTransport};
use serde_json::Value;
use std::io::Write;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc,
};

#[tokio::main]
async fn main() -> Result<(), MCPError> {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Create a transport
    let transport = StdioTransport::new();

    // Create a client
    let mut client = Client::new(transport);

    // Initialize the client
    info!("Initializing client...");
    let init_result = client.initialize().await?;

    info!("Connection established");

    // Get server information
    if let Some(server_info) = init_result.get("serverInfo") {
        if let (Some(name), Some(version), Some(protocol_version)) = (
            server_info.get("name").and_then(|v| v.as_str()),
            server_info.get("version").and_then(|v| v.as_str()),
            init_result.get("protocolVersion").and_then(|v| v.as_str()),
        ) {
            info!(
                "Connected to server: {} v{} (protocol {})",
                name, version, protocol_version
            );
        }
    }

    // Retrieve available tools
    let tools_result = client.call_tool::<_, Value>("tools/list", &()).await?;
    let tools = tools_result
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    let name = t.get("name")?.as_str()?;
                    let description = t.get("description").and_then(|d| d.as_str()).unwrap_or("");
                    Some((name.to_string(), description.to_string()))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if !tools.is_empty() {
        info!("Available tools:");
        for (name, description) in &tools {
            info!("  - {} - {}", name, description);
        }
    } else {
        info!("No tools available");
    }

    // Create channels for user input
    let (input_tx, mut input_rx) = mpsc::channel(10);

    // Spawn a task to read user input
    tokio::spawn(async move {
        // Use Tokio's stdin
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut buffer = String::new();

        loop {
            // We need to use the standard io for output since tokio doesn't have a direct equivalent
            print!("> ");
            std::io::stdout().flush().unwrap();

            buffer.clear();
            match reader.read_line(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Err(_) = input_tx.send(buffer.trim().to_string()).await {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }
    });

    // Main loop
    info!("Enter commands in the format: <tool_name> <json_params>");
    info!("Example: echo {{\"message\": \"Hello, world!\"}}");
    info!("Type 'exit' to quit");

    while let Some(input) = input_rx.recv().await {
        if input.trim() == "exit" {
            break;
        }

        // Parse the input
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        if parts.len() < 2 {
            error!("Invalid input format. Use: <tool_name> <json_params>");
            continue;
        }

        let tool_name = parts[0];
        let params_str = parts[1];

        // Parse the parameters
        let params: Value = match serde_json::from_str(params_str) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid JSON parameters: {}", e);
                continue;
            }
        };

        // Call the tool
        info!(
            "Calling tool: {} with parameters: {}",
            tool_name, params_str
        );
        match client.call_tool::<_, Value>(tool_name, &params).await {
            Ok(result) => {
                println!("Result: {}", serde_json::to_string_pretty(&result).unwrap());
            }
            Err(e) => {
                error!("Error calling tool: {}", e);
            }
        }
    }

    // Shutdown the client
    info!("Shutting down client...");
    client.shutdown().await?;

    Ok(())
}
