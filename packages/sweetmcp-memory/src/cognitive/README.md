# Cognitive Optimization System

## Overview

This system uses **committee-based evaluation** with **Monte Carlo Tree Search (MCTS)** to optimize code according to user objectives. Unlike traditional optimization that relies on hardcoded metrics, this system uses LLM agents to evaluate how well each modification achieves the specified goal.

## Key Components

### 1. Committee-based Evaluation (`committee.rs`)
- Multiple LLM agents with different perspectives (performance, memory, quality)
- Multi-round evaluation process:
  - **Initial**: Independent evaluation
  - **Review**: Agents see others' scores and can revise
  - **Refine**: Committee provides steering feedback
  - **Finalize**: Final consensus reached
- No hardcoded performance values - agents determine impact based on the objective

### 2. MCTS with Committee (`mcts.rs`)
- Explores code modification tree
- Uses committee evaluations as rewards
- Actions include: optimize hot paths, reduce allocations, improve cache locality, etc.
- Each action is evaluated by the committee for its impact

### 3. Performance Analysis (`performance.rs`)
- Uses committee evaluations instead of hardcoded metrics
- Tracks performance trends
- Calculates rewards based on objective achievement

### 4. Evolution Engine (`evolution.rs`)
- Orchestrates MCTS runs
- Manages committee creation
- Reports significant improvements

### 5. Infinite Orchestrator (`orchestrator.rs`)
- Runs continuous optimization iterations
- Tracks best states
- Saves results for each iteration

## How It Works

1. **User provides objective**: "Optimize search relevance while maintaining performance"

2. **Committee evaluates**: Each modification is scored by how well it achieves the objective

3. **MCTS explores**: Uses committee scores to guide exploration of modification space

4. **Multi-round consensus**: Agents review each other's evaluations and refine

5. **Best path selected**: MCTS identifies modifications with highest objective achievement

## Example Usage

```rust
// Define your objective
let user_objective = "Optimize the quantum memory search function for better relevance 
                     while maintaining performance constraints.";

// Create orchestrator
let orchestrator = InfiniteOrchestrator::new(
    "spec.md",
    "output/",
    initial_code,
    10.0,   // baseline latency
    100.0,  // baseline memory  
    50.0,   // baseline relevance
    user_objective,
)?;

// Run optimization
orchestrator.run_infinite().await?;
```

## Key Differences from Traditional Optimization

### Traditional Approach:
```rust
// Hardcoded impact values
match action {
    "inline_function" => (0.90, 1.05, 1.10), // latency, memory, relevance
    "add_caching" => (0.80, 1.15, 1.20),
    // ...
}
```

### This Approach:
```rust
// Committee evaluates based on objective
let factors = committee.evaluate_action(
    state,
    action,
    spec,
    user_objective, // <-- Drives evaluation
).await?;
```

## Multi-Round Evaluation Process

The committee uses multiple rounds to reach consensus:

1. **Round 1**: Each agent independently evaluates
2. **Round 2**: Agents see others' evaluations and can revise
3. **Round 3**: Committee steering based on disagreements
4. **Round 4**: Final consensus

This mirrors how human committees work - initial positions, discussion, guidance, and final decision.

## Configuration

Set environment variables for LLM access:
- `OPENAI_API_KEY` - For GPT-4 agent
- `ANTHROPIC_API_KEY` - For Claude agent
- `COHERE_API_KEY` - For additional agents

## Benefits

1. **Objective-driven**: Optimizations target actual user goals
2. **Adaptive**: Committee learns what works for each objective
3. **Transparent**: See exactly why each modification was chosen
4. **Collaborative**: Multiple perspectives prevent blind spots
5. **No hardcoding**: Impact determined by AI evaluation, not fixed values