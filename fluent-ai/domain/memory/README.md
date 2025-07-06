# VectorStoreIndex

| Property | Type | Example |
|----------|------|---------|
| `top_n` | `BoxFuture<Result<Vec<(f64, String, Value)>, VectorStoreError>>` | `async { Ok(vec![(0.95, "doc1".to_string(), json!({"title": "Document 1"})), (0.87, "doc2".to_string(), json!({"title": "Document 2"}))]) }` |
| `top_n_ids` | `BoxFuture<Result<Vec<(f64, String)>, VectorStoreError>>` | `async { Ok(vec![(0.95, "doc1".to_string()), (0.87, "doc2".to_string())]) }` |
