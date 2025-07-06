# Extractor

| Property | Type | Example |
|----------|------|---------|
| `agent` | `[Agent](../01-agent/)<M>` | `Agent::new(completion_model).preamble("Extract structured data").build()` |
| `_t` | `PhantomData<T>` | `PhantomData::<PersonData>` |