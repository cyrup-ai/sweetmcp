# ToolType

| Property | Type | Example |
|----------|------|---------|
| `Simple` | `Box<dyn ToolDyn>` | `ToolType::Simple(Box::new(CalculatorTool))` |
| `Embedding` | `Box<dyn ToolEmbeddingDyn>` | `ToolType::Embedding(Box::new(SearchTool))` |