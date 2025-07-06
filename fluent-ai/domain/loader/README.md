# FileLoader

| Property | Type | Example |
|----------|------|---------|
| `iterator` | `Box<dyn Iterator<Item = T>>` | `Box::new(glob("*.txt").unwrap().map(|p| p.unwrap()))` |