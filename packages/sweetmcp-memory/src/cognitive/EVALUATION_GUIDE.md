# Committee Evaluation System - Progress-Focused Design

## Core Philosophy

Instead of artificial metrics (latency, memory, relevance), the committee answers one key question:

> **"Does this solution represent incremental progress toward the user objective?"**

## Evaluation Criteria

Each agent evaluates based on:

1. **makes_progress** (boolean) - Does this move us closer to the objective?
2. **objective_alignment** (0-1) - How well aligned with what the user wants?
3. **implementation_quality** (0-1) - Is it well-implemented or just a hack?
4. **risk_assessment** (0-1) - How safe is this change? (1 = very safe)
5. **suggested_improvements** - Concrete ways to make it better

## Example Prompts

### Initial Evaluation Prompt
```
You are an expert performance evaluator on an optimization committee.

USER OBJECTIVE: Optimize the quantum memory search function for better relevance 
                while maintaining performance constraints.

CURRENT CODE:
```rust
pub struct QuantumMemory {
    cache: HashMap<String, Vec<f64>>,
}

impl QuantumMemory {
    pub fn search(&self, query: &str) -> Vec<f64> {
        // Linear search
        for (key, value) in &self.cache {
            if key.contains(query) {
                return value.clone();
            }
        }
        vec![]
    }
}
```

PROPOSED ACTION: add_quantum_entanglement_scoring

Your task is to evaluate whether this action makes incremental progress toward the user objective.

Consider from your performance perspective:
1. Does this action move us closer to the objective? (even small steps count)
2. How well does it align with what the user wants?
3. Is it well-implemented or just a hack?
4. What are the risks?

Provide your evaluation in exactly this JSON format:
{
    "makes_progress": true,
    "objective_alignment": 0.85,
    "implementation_quality": 0.70,
    "risk_assessment": 0.90,
    "reasoning": "Adding quantum entanglement scoring directly addresses the 'better relevance' objective. This represents clear progress even if the implementation needs refinement.",
    "suggested_improvements": [
        "Consider caching entanglement scores",
        "Add benchmarks to verify performance constraints",
        "Implement fallback for non-quantum systems"
    ]
}
```

### Review Phase (Seeing Other Evaluations)
```
OTHER EVALUATIONS:
memory_agent: progress=false, alignment=0.40, quality=0.60, risk=0.70
Reasoning: This adds complexity without clear memory benefits
Suggestions: Profile memory usage first

quality_agent: progress=true, alignment=0.90, quality=0.65, risk=0.85
Reasoning: Directly improves search relevance as requested
Suggestions: Add unit tests for quantum scoring

CONSENSUS: Most agents think this makes progress

Consider their perspectives and either:
1. Maintain your position with stronger reasoning
2. Revise based on insights you missed
```

### When All Solutions Fail

If all committee members vote `makes_progress: false`, the system:

1. **Preserves the original code** (no regression)
2. **Enhances the prompt** with failure understanding:

```
The committee rejected all proposed solutions for the following reasons:
- "add_quantum_entanglement": Too complex without proven relevance improvement
- "optimize_cache_locality": Doesn't address the relevance objective
- "parallelize_search": Adds overhead without relevance gains

Please generate NEW approaches that:
1. Directly improve search relevance (the main objective)
2. Use simpler, proven techniques before complex ones
3. Include measurement/benchmarking in the solution

Focus on incremental progress - even small improvements count.
```

## Multi-Round Process

1. **Initial** - Independent evaluation
2. **Review** - See others' scores, can revise
3. **Refine** - If no consensus, steering feedback helps
4. **Finalize** - Last chance for consensus

## Key Advantages

- **No fake metrics** - Just "does this help?"
- **Forward momentum** - Always use best solution so far
- **Learning from failure** - Failed attempts improve next prompt
- **Transparent reasoning** - Know WHY decisions were made
- **Incremental progress** - Small steps are celebrated

## Example Decision Flow

```rust
// Round 1: Initial evaluation
let consensus = ConsensusDecision {
    makes_progress: true,  // 2/3 agents agree
    confidence: 0.67,
    overall_score: 0.75,   // Weighted: alignment(0.85) * 0.5 + quality(0.70) * 0.3 + risk(0.90) * 0.2
    improvement_suggestions: vec![
        "Add benchmarks",
        "Cache entanglement scores",
        "Document the algorithm"
    ],
    dissenting_opinions: vec![
        "memory_agent: This adds complexity without clear benefits"
    ]
};

// If makes_progress = true → Apply this solution
// If makes_progress = false → Keep original, enhance prompt with failure reasons
```

## Configuration

```rust
// Adjust for your needs
let committee = EvaluationCommittee::new(
    agent_count: 3,      // Number of perspectives
    max_rounds: 4,       // How many rounds of discussion
    consensus_threshold: 0.85, // Skip rounds if confidence high
)?;
```