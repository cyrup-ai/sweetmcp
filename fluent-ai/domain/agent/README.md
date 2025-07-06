# Agent

| Property | Type | Example |
|----------|------|---------|
| `model` | `M: CompletionModel` | `openai_client.completion_model("o3")` |
| `preamble` | `Option<String>` | `Some("You are a helpful assistant")` |
| `static_context` | `Vec<[Document](../document/)>` | `vec![Document { id: "doc1", text: "Context info", additional_props: HashMap::new() }]` |
| `static_tools` | `Vec<String>` | `vec!["calculator", "weather_tool"]` |
| `temperature` | `Option<f64>` | `Some(0.3)` |
| `max_tokens` | `Option<u64>` | `Some(1000)` |
| `additional_params` | `Option<serde_json::Value>` | `Some(json!({"top_p": 0.9}))` |
| `dynamic_context` | `Vec<(usize, Box<dyn [VectorStoreIndexDyn](../vector-store-index-dyn/)>)>` | `vec![(5, Box::new(vector_index))]` |
| `dynamic_tools` | `Vec<(usize, Box<dyn [VectorStoreIndexDyn](../vector-store-index-dyn/)>)>` | `vec![(3, Box::new(tool_index))]` |
| `tools` | `[ToolSet](../tool-set/)` | `ToolSet::from_tools(vec![calculator, weather])` |
