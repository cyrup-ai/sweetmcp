# Kalosm Integration Examples

This directory contains reference examples from the Kalosm library that can be used to implement AI capabilities in our browser automation MCP. These examples demonstrate patterns that follow our project conventions, particularly around asynchronous code and API design.

## Example Overview

### LLM Integration

- `llm_chat_example.rs` - Basic chat model using Phi-4 with a system prompt
- `anthropic_example.rs` - Alternative model using Claude 3.7 Sonnet
- `character_chat_example.rs` - Advanced chat with constraints and persona customization
- `constrained_example.rs` - Structured generation with validation constraints

### Vision & Image Processing

- `segment_anything_example.rs` - Image segmentation for visual element extraction
- `ocr_example.rs` - Optical character recognition for extracting text from images

### Embedding & Search

- `embedding_cache_example.rs` - Embedding with caching for efficient semantic search

## Usage in Browser Automation

These examples should be adapted to follow our project conventions:

1. No `async_trait` or `async fn` in traits
2. No returning `Box<dyn Future>` or `Pin<Box<dyn Future>>` from client interfaces
3. Sync interfaces with `.await()` called internally
4. Hiding async complexity behind channels and task spawning
5. Returning domain-specific types instead of generic futures/streams

For browser automation, these capabilities can be integrated to:

- Process browser screenshots with segmentation and OCR
- Generate structured browser actions with constraints
- Use embeddings for semantic search of page elements 
- Create LLM agents that reason about browser state

## Implementation Notes

When implementing these patterns:

1. Create domain-specific return types (e.g., `AgentResponse`, `BrowserAction`)
2. Use proper error handling with custom types (no `unwrap()`)
3. Keep modules under 300 lines by proper decomposition
4. Follow Rust's official style guidelines
