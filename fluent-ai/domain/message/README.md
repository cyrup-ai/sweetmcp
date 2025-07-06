# Message

| Property | Type | Example |
|----------|------|---------|
| `User` | `{ content: [OneOrMany](../one-or-many/)<[UserContent](../user-content/)> }` | `Message::User { content: OneOrMany::one(UserContent::Text("Hello")) }` |
| `Assistant` | `{ content: [OneOrMany](../one-or-many/)<[AssistantContent](../assistant-content/)> }` | `Message::Assistant { content: OneOrMany::one(AssistantContent::Text("Hi there!")) }` |