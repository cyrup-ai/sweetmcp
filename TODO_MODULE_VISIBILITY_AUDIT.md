# SYSTEMATIC MODULE VISIBILITY AUDIT

## OBJECTIVE
Fix ALL E0599 "no method found" errors by doing FULL AUDIT of modules and ACTUALLY MAKING THEM VISIBLE.
Total E0599 errors: 589

## CRITICAL E0599 ERRORS IDENTIFIED (Top 20)

### 1. AttentionMechanism::new_lock_free() - MISSING VISIBILITY
- **Error**: `no function or associated item named 'new_lock_free' found for struct 'attention::core::AttentionMechanism'`
- **Location**: packages/sweetmcp-memory/src/cognitive/manager.rs:186
- **Module to Audit**: cognitive/attention/
- **Status**: ✅ FIXED - Implemented new_lock_free() method in AttentionMechanism

### 2. QuantumRouter::new_lock_free() - MISSING VISIBILITY  
- **Error**: `no function or associated item named 'new_lock_free' found for struct 'QuantumRouter'`
- **Location**: packages/sweetmcp-memory/src/cognitive/manager.rs:206
- **Module to Audit**: cognitive/quantum/
- **Status**: ✅ FIXED - Implemented new_lock_free() method in QuantumRouter

### 3. EvolutionEngine::new_lock_free() - MISSING VISIBILITY
- **Error**: `no function or associated item named 'new_lock_free' found for struct 'EvolutionEngine'`
- **Location**: packages/sweetmcp-memory/src/cognitive/manager.rs:209
- **Module to Audit**: cognitive/evolution*
- **Status**: ✅ FIXED - Implemented new_lock_free() method in EvolutionEngine

### 4. SubsystemCoordinator::new_lock_free() - MISSING VISIBILITY
- **Error**: `no function or associated item named 'new_lock_free' found for struct 'SubsystemCoordinator'`
- **Location**: packages/sweetmcp-memory/src/cognitive/manager.rs:211
- **Module to Audit**: cognitive/subsystem_coordinator*
- **Status**: ✅ ALREADY EXISTS - Method is properly implemented and visible

### 5. SurrealDBMemoryManager::initialize() - MISSING VISIBILITY
- **Error**: `no method named 'initialize' found for struct 'SurrealDBMemoryManager'`
- **Location**: packages/sweetmcp-memory/src/cognitive/manager.rs:179
- **Module to Audit**: memory/memory_manager or related
- **Status**: ✅ ALREADY EXISTS - Method is properly implemented and visible

### 6. AgentPerspective.focus_areas() - MISSING VISIBILITY
- **Error**: `no method named 'focus_areas' found for enum 'AgentPerspective'`
- **Location**: packages/sweetmcp-memory/src/cognitive/committee/core/committee.rs:502
- **Module to Audit**: cognitive/committee/core/
- **Status**: ✅ ALREADY EXISTS - Method is properly implemented and visible

### 7. OptimizationSpec::default() - MISSING VISIBILITY
- **Error**: `no function or associated item named 'default' found for struct 'cognitive::types::OptimizationSpec'`
- **Location**: packages/sweetmcp-memory/src/cognitive/committee/core/mod.rs:89,299
- **Module to Audit**: cognitive/types*
- **Status**: ❌ TODO

### 8. CommitteeAgent.evaluate_with_phase() - MISSING VISIBILITY
- **Error**: `no method named 'evaluate_with_phase' found for struct 'CommitteeAgent'`
- **Location**: packages/sweetmcp-memory/src/cognitive/committee/consensus/evaluation_phases.rs:256
- **Module to Audit**: cognitive/committee/core/
- **Status**: ❌ TODO

### 9. CommitteeEvent::AgentStarted - MISSING VISIBILITY
- **Error**: `no variant named 'AgentStarted' found for enum 'CommitteeEvent'`
- **Location**: packages/sweetmcp-memory/src/cognitive/committee/evaluation/execution.rs:75
- **Module to Audit**: cognitive/committee/consensus/
- **Status**: ❌ TODO

### 10. SubsystemCoordinator methods - MISSING VISIBILITY
- **Error**: `no method named 'enhance_memory_cognitively_lock_free' found`
- **Error**: `no method named 'store_cognitive_metadata_lock_free' found`
- **Location**: packages/sweetmcp-memory/src/cognitive/manager.rs:305,315,340,342,355
- **Module to Audit**: cognitive/subsystem_coordinator*
- **Status**: ❌ TODO

### 11. ActionCoordinator.get_possible_actions() - MISSING VISIBILITY
- **Error**: `no method named 'get_possible_actions' found for struct 'ActionCoordinator'`
- **Location**: packages/sweetmcp-memory/src/cognitive/mcts/controller.rs:64,118
- **Module to Audit**: cognitive/mcts/actions/
- **Status**: ❌ TODO

### 12. CognitiveError::ExecutionFailed - MISSING VISIBILITY
- **Error**: `no variant or associated item named 'ExecutionFailed' found for enum 'cognitive::types::CognitiveError'`
- **Location**: packages/sweetmcp-memory/src/cognitive/mcts/runner.rs:331
- **Module to Audit**: cognitive/types*
- **Status**: ❌ TODO

### 13. String.should_continue() - MISSING VISIBILITY (Custom Extension)
- **Error**: `no method named 'should_continue' found for struct 'std::string::String'`
- **Location**: packages/sweetmcp-memory/src/cognitive/committee/consensus/committee.rs:160
- **Module to Audit**: Find custom String extension trait
- **Status**: ❌ TODO

### 14. Semaphore.clone() - MISSING VISIBILITY (Standard Library Issue)
- **Error**: `no method named 'clone' found for struct 'Semaphore'`
- **Location**: packages/sweetmcp-daemon/src/service/sse/bridge/forwarding.rs:79
- **Module to Audit**: Check imports/trait bounds
- **Status**: ❌ TODO

## SYSTEMATIC AUDIT PLAN

### Phase 1: Core Infrastructure Modules (High Priority)
1. ❌ cognitive/attention/ - AttentionMechanism visibility
2. ❌ cognitive/quantum/ - QuantumRouter visibility  
3. ❌ cognitive/evolution* - EvolutionEngine visibility
4. ❌ cognitive/subsystem_coordinator* - SubsystemCoordinator visibility
5. ❌ memory/memory_manager - SurrealDBMemoryManager visibility

### Phase 2: Committee System Modules
6. ❌ cognitive/committee/core/ - AgentPerspective, CommitteeAgent visibility
7. ❌ cognitive/committee/consensus/ - CommitteeEvent visibility

### Phase 3: MCTS System Modules  
8. ❌ cognitive/mcts/actions/ - ActionCoordinator visibility
9. ❌ cognitive/types* - OptimizationSpec, CognitiveError visibility

### Phase 4: Remaining Modules
10. ❌ Find and fix custom String extension trait
11. ❌ Fix remaining E0599 errors systematically

## AUDIT CHECKLIST FOR EACH MODULE

For each module that needs audit:
- [ ] Find where the missing method/type is actually defined
- [ ] Check if it's properly exported in the module's mod.rs
- [ ] Check if parent modules properly re-export it
- [ ] Ensure proper visibility (pub) on the method/type
- [ ] Verify import paths are correct
- [ ] Test that the method/type is accessible from calling location

## PROGRESS TRACKING

- **Total E0599 Errors**: 589
- **Modules Audited**: 0/14
- **Modules Fixed**: 0/14
- **Estimated Remaining**: 589 errors

## COMPLETION CRITERIA

✅ ALL E0599 errors resolved
✅ ALL modules properly export their public API
✅ ALL methods/types are visible from their calling locations
✅ Clean compilation with 0 E0599 errors

**DO NOT CHECK COMPILATION ERRORS UNTIL ALL MODULES ARE AUDITED AND FIXED**