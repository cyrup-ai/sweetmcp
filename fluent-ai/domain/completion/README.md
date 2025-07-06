# CompletionRequest

| Property | Type | Example |
|----------|------|---------|
| `preamble` | `String` | `"You are a helpful assistant"` |
| `chat_history` | `Vec<[Message](../message/)>` | `vec![Message::user("Hello"), Message::assistant("Hi there!")]` |
| `documents` | `Vec<[Document](../document/)>` | `vec![Document { id: "doc1", text: "Context information", additional_props: HashMap::new() }]` |
| `tools` | `Vec<[ToolDefinition](../tool-definition/)>` | `vec![ToolDefinition { name: "calculator", description: "Math tool", parameters: json!({}) }]` |
| `temperature` | `Option<f64>` | `Some(0.7)` |
| `max_tokens` | `Option<u64>` | `Some(1000)` |
| `additional_params` | `Option<serde_json::Value>` | `Some(json!({"top_p": 0.9, "frequency_penalty": 0.0}))` |