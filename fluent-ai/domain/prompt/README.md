# PromptRequest

| Property | Type | Example |
|----------|------|---------|
| `prompt` | `[Message](../message/)` | `Message::user("What's the weather?")` |
| `chat_history` | `Option<&mut Vec<[Message](../message/)>>` | `Some(&mut conversation_history)` |
| `max_depth` | `usize` | `10` |
| `agent` | `&[Agent](../01-agent/)` | `&my_agent` |