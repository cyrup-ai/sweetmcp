# 🔥 SURGICAL METHOD RECOVERY PLAN - E0599 ERRORS

**SURGICAL APPROACH: Keep decomposed modules, place missing methods in correct submodules**

## IDENTIFIED MISSING METHODS FROM E0599 ERRORS

### 1. MCTS Methods (cognitive/mcts/)
- `MCTSNode::new_root` → Place in `cognitive/mcts/types/node_types.rs`
- `ActionCoordinator::get_possible_actions` → Place in `cognitive/mcts/actions/action_generator.rs`
- `CodeState::cache_key` → Place in `cognitive/mcts/types/node_types.rs`

### 2. Committee Methods (cognitive/committee/)
- `AgentPerspective::focus_areas` → Place in `cognitive/committee/core/agents.rs`
- `OptimizationSpec::default` → Place in `cognitive/types.rs`
- `CommitteeAgent::evaluate_with_phase` → Place in `cognitive/committee/evaluation/agent_orchestration.rs`
- `CommitteeEvent::AgentStarted` variant → Place in `cognitive/committee/evaluation/types.rs`

### 3. Manager Methods (cognitive/manager.rs)
- `SurrealDBMemoryManager::initialize` → Place in `memory/memory_manager/core.rs`
- `AttentionMechanism::new_lock_free` → Place in `cognitive/attention/core.rs`
- `SubsystemCoordinator::new_lock_free` → Place in `cognitive/manager.rs`
- `SubsystemCoordinator::enhance_memory_cognitively_lock_free` → Place in `cognitive/manager.rs`
- `SubsystemCoordinator::store_cognitive_metadata_lock_free` → Place in `cognitive/manager.rs`

## SURGICAL RECOVERY TASKS

### Task SR1: MCTS Node Methods Recovery
- [ ] **Extract MCTSNode::new_root from backup** - Place in node_types.rs
  - **Source**: cognitive/mcts.rs.backup (lines ~50-80)
  - **Target**: cognitive/mcts/types/node_types.rs
  - **Method**: Extract constructor method for root node creation
  - **Constraints**: Zero allocation, blazing-fast, no unsafe, elegant ergonomic API

### Task SR2: MCTS Action Methods Recovery  
- [ ] **Extract ActionCoordinator::get_possible_actions from backup** - Place in action_generator.rs
  - **Source**: cognitive/mcts.rs.backup (search for get_possible_actions)
  - **Target**: cognitive/mcts/actions/action_generator.rs
  - **Method**: Extract action generation logic
  - **Constraints**: Lock-free operations, efficient action enumeration

### Task SR3: CodeState Methods Recovery
- [ ] **Extract CodeState::cache_key from backup** - Place in node_types.rs
  - **Source**: cognitive/mcts.rs.backup (search for cache_key)
  - **Target**: cognitive/mcts/types/node_types.rs
  - **Method**: Extract caching key generation method
  - **Constraints**: Zero allocation, cache-friendly operations

### Task SR4: Committee Agent Methods Recovery
- [ ] **Extract AgentPerspective::focus_areas from backup** - Place in agents.rs
  - **Source**: cognitive/committee/evaluation.rs.backup
  - **Target**: cognitive/committee/core/agents.rs
  - **Method**: Extract agent perspective analysis method
  - **Constraints**: Efficient perspective enumeration, no allocations

### Task SR5: OptimizationSpec Default Implementation
- [ ] **Add OptimizationSpec::default implementation** - Place in types.rs
  - **Source**: cognitive/types.rs or backup files
  - **Target**: cognitive/types.rs
  - **Method**: Add Default trait implementation
  - **Constraints**: Sensible defaults, zero allocation

### Task SR6: Memory Manager Methods Recovery
- [ ] **Extract SurrealDBMemoryManager::initialize from backup** - Place in core.rs
  - **Source**: memory/memory_manager.rs.backup
  - **Target**: memory/memory_manager/core.rs
  - **Method**: Extract initialization logic
  - **Constraints**: Async initialization, error handling, no blocking

### Task SR7: Attention Mechanism Methods Recovery
- [ ] **Extract AttentionMechanism::new_lock_free from backup** - Place in core.rs
  - **Source**: cognitive/attention.rs.backup or related files
  - **Target**: cognitive/attention/core.rs
  - **Method**: Extract lock-free constructor
  - **Constraints**: Lock-free operations, zero allocation, thread-safe

### Task SR8: SubsystemCoordinator Methods Recovery
- [ ] **Extract SubsystemCoordinator methods from backup** - Place in manager.rs
  - **Source**: cognitive/manager.rs or backup files
  - **Target**: cognitive/manager.rs
  - **Methods**: 
    - `new_lock_free` - Lock-free constructor
    - `enhance_memory_cognitively_lock_free` - Memory enhancement
    - `store_cognitive_metadata_lock_free` - Metadata storage
  - **Constraints**: Lock-free operations, async processing, zero allocation

## EXECUTION STRATEGY

### Phase 1: Critical MCTS Methods (Highest Impact)
1. Extract and place `MCTSNode::new_root` in `node_types.rs`
2. Extract and place `ActionCoordinator::get_possible_actions` in `action_generator.rs`
3. Extract and place `CodeState::cache_key` in `node_types.rs`

### Phase 2: Committee Methods (Core Functionality)
1. Extract and place `AgentPerspective::focus_areas` in `agents.rs`
2. Add `OptimizationSpec::default` implementation in `types.rs`
3. Extract and place `CommitteeAgent::evaluate_with_phase` in `agent_orchestration.rs`

### Phase 3: Manager Methods (System Integration)
1. Extract and place `SurrealDBMemoryManager::initialize` in `core.rs`
2. Extract and place `AttentionMechanism::new_lock_free` in `core.rs`
3. Extract and place `SubsystemCoordinator` methods in `manager.rs`

### Quality Assurance for Each Method:
- ✅ Zero allocation in hot paths
- ✅ Lock-free operations where applicable
- ✅ No unsafe code
- ✅ No unwrap/expect in src/
- ✅ Proper error handling with semantic error types
- ✅ Maintain original behavior and performance
- ✅ Elegant ergonomic APIs with type safety
- ✅ Cache-friendly data structures
- ✅ SIMD optimization in computational kernels where applicable

### Validation Process:
1. Extract method from backup file
2. Analyze method dependencies and imports
3. Place method in appropriate decomposed module
4. Update module exports if needed
5. Verify method signature matches usage sites
6. Ensure all constraints are met
7. Test compilation of affected modules