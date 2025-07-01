# MCP Reasoner Plugin

A WebAssembly plugin for the [hyper-mcp](https://github.com/tuananh/hyper-mcp) server, providing reasoning capabilities using Beam Search and Monte Carlo Tree Search.

## Features

- Advanced reasoning engine with multiple strategies
- Stateful reasoning that tracks thought paths
- Support for branching thoughts and exploring multiple paths
- Compatible with the Model Context Protocol (MCP)

## Usage with hyper-mcp

Add this plugin to your hyper-mcp configuration:

```json
{
  "plugins": [
    {
      "name": "mcp-reasoner",
      "path": "oci://ghcr.io/yourusername/mcp-reasoner-plugin:latest"
    }
  ]
}
```

## Plugin Interface

The plugin exposes a single tool `mcp-reasoner` with the following parameters:

- `thought` (string, required): The current reasoning step
- `thoughtNumber` (integer, required): The current step number (1-indexed)
- `totalThoughts` (integer, required): Total expected steps
- `nextThoughtNeeded` (boolean, required): Whether another step is needed
- `parentId` (string, optional): Optional parent thought ID for branching
- `strategyType` (string, optional): Reasoning strategy to use (beam_search, mcts, mcts_002_alpha, or mcts_002alt_alpha)
- `beamWidth` (integer, optional): Number of top paths to maintain (1-10)
- `numSimulations` (integer, optional): Number of MCTS simulations to run (1-150)

## Building

```bash
# Build locally
cargo build --target wasm32-wasip1 --release

# Build with Docker
docker build -t ghcr.io/yourusername/mcp-reasoner-plugin:latest .

# Push to registry
docker push ghcr.io/yourusername/mcp-reasoner-plugin:latest
```

## License

MIT