# sweetmcp-voice-tools

MCP tool definitions for voice operations in SweetMCP.

## Overview

This crate provides the MCP (Model Context Protocol) tool definitions for voice operations, enabling LLMs to interact with text-to-speech (TTS) and speech-to-text (STT) capabilities through a clean, intuitive interface.

## Tools Provided

### `speak`
Convert text to speech and play it through system audio.

**Parameters:**
- `text` (required): Text to convert to speech
- `voice_id` (optional): Voice ID to use
- `speed` (optional): Speech speed (0.5-2.0)

### `listen`
Listen to audio from the microphone and transcribe to text.

**Parameters:**
- `microphone_id` (required): Microphone device ID
- `duration_seconds` (required): Duration to listen (1-300 seconds)
- `wake_word` (optional): Wake word for activation

## Integration

This package is used by:
- **sweetmcp-axum**: Registers the voice tools in the MCP tool registry
- **sweetmcp-voice**: Implements the actual voice functionality using fluent-voice

## Protocol

Communication between the MCP plugin and voice service uses QUIC (via cryypt) with the request/response protocol defined in `protocol.rs`.

## License

MIT OR Apache-2.0
