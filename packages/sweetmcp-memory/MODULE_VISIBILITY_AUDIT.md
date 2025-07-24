# 🔍 SYSTEMATIC MODULE VISIBILITY AUDIT - E0599 ERRORS

**CRITICAL: ALL modules with E0599 errors need FULL AUDIT and proper exports**
**DO NOT CHECK ERRORS until ALL modules are audited and fixed**

## IDENTIFIED MODULES WITH E0599 ERRORS

### 1. COGNITIVE COMMITTEE MODULES
- [ ] `cognitive/committee/consensus/committee.rs` - AUDIT NEEDED
- [ ] `cognitive/committee/consensus/evaluation_phases.rs` - AUDIT NEEDED  
- [ ] `cognitive/committee/core/committee.rs` - AUDIT NEEDED
- [ ] `cognitive/committee/core/mod.rs` - AUDIT NEEDED
- [ ] `cognitive/committee/evaluation/builder.rs` - AUDIT NEEDED
- [ ] `cognitive/committee/evaluation/execution.rs` - AUDIT NEEDED

### 2. COGNITIVE MANAGER MODULE
- [ ] `cognitive/manager.rs` - AUDIT NEEDED (24 E0599 errors)

### 3. COGNITIVE EVOLUTION MODULE  
- [ ] `cognitive/evolution.rs` - AUDIT NEEDED

### 4. COGNITIVE MCTS MODULES
- [ ] `cognitive/mcts/analysis/node_search_advanced.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/analysis/node_search_basic.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/analysis/node_search_bottleneck.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/analysis/node_search_statistics.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/analysis/node_search_types.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/actions/action_applicator.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/actions/action_generator.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/actions/action_validator.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/controller.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/execution.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/factory.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/results.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/runner.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/types/action_types.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/types/node_types.rs` - AUDIT NEEDED
- [ ] `cognitive/mcts/types/tree_types.rs` - AUDIT NEEDED

### 5. COGNITIVE QUANTUM MCTS MODULES
- [ ] `cognitive/quantum_mcts/entanglement/analysis/bottleneck_detection.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/analysis/neighborhood_analysis.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/analysis/network_topology.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/analysis/quality_assessment.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/balancing/balance_analysis.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/balancing/balancing_operations.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/balancing/balancing_strategy.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/core.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/factory.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/health.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/operations.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/optimization.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/engine/pruning.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/metrics/benchmarking.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/metrics/reporting.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum_mcts/entanglement/metrics/tracking.rs` - AUDIT NEEDED

### 6. COGNITIVE QUANTUM ERROR CORRECTION MODULES
- [ ] `cognitive/quantum/error_correction/stabilizer.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/error_correction/surface_code/correction_algorithms.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/error_correction/surface_code/layout_management.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/error_correction/surface_code/logical_operations.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/error_correction/surface_code/syndrome_detection.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/error_correction/syndrome.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/error_correction/topological.rs` - AUDIT NEEDED

### 7. COGNITIVE QUANTUM ML DECODER MODULES
- [ ] `cognitive/quantum/ml_decoder/config.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/ml_decoder/decoding.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/ml_decoder/gradients.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/ml_decoder/optimizers.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/ml_decoder/quantum_ops.rs` - AUDIT NEEDED
- [ ] `cognitive/quantum/ml_decoder/training.rs` - AUDIT NEEDED

### 8. GRAPH MODULES
- [ ] `graph/entity/core.rs` - AUDIT NEEDED
- [ ] `graph/entity/queries.rs` - AUDIT NEEDED
- [ ] `graph/entity/relationships.rs` - AUDIT NEEDED
- [ ] `graph/entity/validation.rs` - AUDIT NEEDED

### 9. MEMORY MODULES
- [ ] `memory/memory_manager/core.rs` - AUDIT NEEDED
- [ ] `memory/memory_manager/crud.rs` - AUDIT NEEDED
- [ ] `memory/memory_manager/relationships.rs` - AUDIT NEEDED
- [ ] `memory/memory_manager/search.rs` - AUDIT NEEDED
- [ ] `memory/memory_manager/trait_def.rs` - AUDIT NEEDED
- [ ] `memory/memory_manager/types.rs` - AUDIT NEEDED
- [ ] `memory/query/builder/conditions.rs` - AUDIT NEEDED
- [ ] `memory/query/builder/convenience.rs` - AUDIT NEEDED
- [ ] `memory/query/builder/core.rs` - AUDIT NEEDED
- [ ] `memory/query/builder/executor.rs` - AUDIT NEEDED
- [ ] `memory/query/builder/validation.rs` - AUDIT NEEDED
- [ ] `memory/query/executor.rs` - AUDIT NEEDED

### 10. MEMORY SEMANTIC MODULES
- [ ] `memory/semantic/item_types.rs` - AUDIT NEEDED
- [ ] `memory/semantic/memory_management.rs` - AUDIT NEEDED
- [ ] `memory/semantic/memory_optimization/health_check.rs` - AUDIT NEEDED
- [ ] `memory/semantic/memory_optimization/operations_utilities.rs` - AUDIT NEEDED
- [ ] `memory/semantic/memory_optimization/optimization_operations.rs` - AUDIT NEEDED
- [ ] `memory/semantic/memory_optimization/optimization_recommendations.rs` - AUDIT NEEDED
- [ ] `memory/semantic/relationships/mod.rs` - AUDIT NEEDED
- [ ] `memory/semantic/semantic_relationship.rs` - AUDIT NEEDED

### 11. VECTOR MODULES
- [ ] `vector/async_vector_core.rs` - AUDIT NEEDED
- [ ] `vector/async_vector_operations.rs` - AUDIT NEEDED
- [ ] `vector/async_vector_optimization/coordinator_config.rs` - AUDIT NEEDED
- [ ] `vector/async_vector_optimization/coordinator_core.rs` - AUDIT NEEDED
- [ ] `vector/async_vector_optimization/coordinator_operations.rs` - AUDIT NEEDED
- [ ] `vector/async_vector_optimization/coordinator_types.rs` - AUDIT NEEDED
- [ ] `vector/async_vector_optimization/mod.rs` - AUDIT NEEDED
- [ ] `vector/async_vector_optimization/search_strategies.rs` - AUDIT NEEDED

## AUDIT PROCESS FOR EACH MODULE

### For each module above:
1. **Read the module file completely**
2. **Identify all structs, enums, traits, functions, constants**
3. **Check what's currently exported in the module's mod.rs**
4. **Add missing exports to make everything visible**
5. **Verify parent mod.rs files export the module properly**
6. **Mark as COMPLETED in this list**

### Quality Standards:
- ✅ All public types must be exported
- ✅ All public methods must be on public types
- ✅ All public functions must be exported
- ✅ All public constants must be exported
- ✅ Module hierarchy must properly re-export everything
- ✅ No private items should be used publicly

## EXECUTION ORDER
1. Start with **COGNITIVE MANAGER** (highest error count - 24 errors)
2. Then **COGNITIVE MCTS** modules (core functionality)
3. Then **COGNITIVE COMMITTEE** modules
4. Then **MEMORY** modules
5. Then **VECTOR** modules
6. Then **QUANTUM** modules
7. Then **GRAPH** modules

**CRITICAL: DO NOT RUN CARGO CHECK UNTIL ALL MODULES ARE AUDITED AND COMPLETED**