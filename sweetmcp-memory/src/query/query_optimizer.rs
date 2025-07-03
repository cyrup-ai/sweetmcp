//! Query optimization and cost estimation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::query::{QueryPlan, QueryType, QueryStep, Result, QueryError};

/// Query optimizer
pub struct QueryOptimizer {
    /// Cost model
    cost_model: CostModel,
    
    /// Optimization rules
    rules: Vec<Box<dyn OptimizationRule>>,
    
    /// Statistics
    stats: QueryStatistics,
}

/// Cost model for query operations
#[derive(Debug, Clone)]
pub struct CostModel {
    /// Cost factors
    factors: HashMap<String, f64>,
}

impl Default for CostModel {
    fn default() -> Self {
        let mut factors = HashMap::new();
        
        // Basic operation costs
        factors.insert("full_scan".to_string(), 100.0);
        factors.insert("index_scan".to_string(), 10.0);
        factors.insert("vector_search".to_string(), 20.0);
        factors.insert("filter".to_string(), 5.0);
        factors.insert("sort".to_string(), 15.0);
        factors.insert("join".to_string(), 50.0);
        
        // Per-record costs
        factors.insert("record_fetch".to_string(), 0.1);
        factors.insert("record_filter".to_string(), 0.01);
        factors.insert("record_sort".to_string(), 0.05);
        
        Self { factors }
    }
}

/// Query statistics for optimization
#[derive(Debug, Clone, Default)]
pub struct QueryStatistics {
    /// Table sizes
    pub table_sizes: HashMap<String, u64>,
    
    /// Index cardinalities
    pub index_cardinalities: HashMap<String, u64>,
    
    /// Selectivity estimates
    pub selectivities: HashMap<String, f64>,
}

impl QueryOptimizer {
    /// Create a new optimizer
    pub fn new() -> Self {
        Self {
            cost_model: CostModel::default(),
            rules: Self::default_rules(),
            stats: QueryStatistics::default(),
        }
    }
    
    /// Get default optimization rules
    fn default_rules() -> Vec<Box<dyn OptimizationRule>> {
        vec![
            Box::new(PushDownFilterRule),
            Box::new(UseIndexRule),
            Box::new(ParallelizationRule),
            Box::new(CacheRule),
        ]
    }
    
    /// Update statistics
    pub fn update_statistics(&mut self, stats: QueryStatistics) {
        self.stats = stats;
    }
    
    /// Optimize a query plan
    pub fn optimize(&self, mut plan: QueryPlan) -> Result<QueryPlan> {
        // Apply optimization rules
        for rule in &self.rules {
            if rule.applicable(&plan) {
                plan = rule.apply(plan)?;
            }
        }
        
        // Recalculate cost
        plan.cost = self.calculate_cost(&plan);
        
        Ok(plan)
    }
    
    /// Calculate the cost of a query plan
    pub fn calculate_cost(&self, plan: &QueryPlan) -> f64 {
        plan.steps
            .iter()
            .map(|step| self.calculate_step_cost(step))
            .sum()
    }
    
    /// Calculate the cost of a single step
    fn calculate_step_cost(&self, step: &QueryStep) -> f64 {
        // Look up base cost
        let base_cost = self.cost_model.factors
            .get(&step.name.to_lowercase().replace(" ", "_"))
            .copied()
            .unwrap_or(10.0);
        
        // Adjust for parallelization
        if step.parallel {
            base_cost * 0.6 // 40% reduction for parallel execution
        } else {
            base_cost
        }
    }
    
    /// Estimate query selectivity
    pub fn estimate_selectivity(&self, query_type: &QueryType) -> f64 {
        match query_type {
            QueryType::Exact => 0.01, // 1% selectivity
            QueryType::Similarity => 0.1, // 10% selectivity
            QueryType::FullText => 0.05, // 5% selectivity
            QueryType::GraphTraversal => 0.2, // 20% selectivity
            QueryType::Hybrid => 0.15, // 15% selectivity
        }
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization rule trait
pub trait OptimizationRule: Send + Sync {
    /// Check if the rule is applicable
    fn applicable(&self, plan: &QueryPlan) -> bool;
    
    /// Apply the optimization rule
    fn apply(&self, plan: QueryPlan) -> Result<QueryPlan>;
    
    /// Get rule name
    fn name(&self) -> &str;
}

/// Push down filter rule - move filters earlier in the plan
struct PushDownFilterRule;

impl OptimizationRule for PushDownFilterRule {
    fn applicable(&self, plan: &QueryPlan) -> bool {
        // Check if there are filters that can be pushed down
        plan.steps.iter().any(|step| step.name == "Filter Results")
    }
    
    fn apply(&self, mut plan: QueryPlan) -> Result<QueryPlan> {
        // Find filter step
        if let Some(filter_pos) = plan.steps.iter().position(|s| s.name == "Filter Results") {
            // Move filter earlier if possible
            if filter_pos > 0 {
                let filter = plan.steps.remove(filter_pos);
                plan.steps.insert(1, filter); // After initial scan
            }
        }
        
        Ok(plan)
    }
    
    fn name(&self) -> &str {
        "push_down_filter"
    }
}

/// Use index rule - prefer index scans over full scans
struct UseIndexRule;

impl OptimizationRule for UseIndexRule {
    fn applicable(&self, plan: &QueryPlan) -> bool {
        plan.steps.iter().any(|step| step.name == "Full Scan") && !plan.use_index
    }
    
    fn apply(&self, mut plan: QueryPlan) -> Result<QueryPlan> {
        // This would check available indexes and replace full scan with index scan
        // For now, just mark that we checked
        Ok(plan)
    }
    
    fn name(&self) -> &str {
        "use_index"
    }
}

/// Parallelization rule - mark steps that can run in parallel
struct ParallelizationRule;

impl OptimizationRule for ParallelizationRule {
    fn applicable(&self, plan: &QueryPlan) -> bool {
        plan.steps.iter().any(|step| !step.parallel && self.can_parallelize(step))
    }
    
    fn apply(&self, mut plan: QueryPlan) -> Result<QueryPlan> {
        for step in &mut plan.steps {
            if !step.parallel && self.can_parallelize(step) {
                step.parallel = true;
                step.cost *= 0.6; // Reduce cost for parallel execution
            }
        }
        
        Ok(plan)
    }
    
    fn name(&self) -> &str {
        "parallelization"
    }
}

impl ParallelizationRule {
    fn can_parallelize(&self, step: &QueryStep) -> bool {
        matches!(
            step.name.as_str(),
            "Full Scan" | "Filter Results" | "Vector Search"
        )
    }
}

/// Cache rule - add caching for expensive operations
struct CacheRule;

impl OptimizationRule for CacheRule {
    fn applicable(&self, plan: &QueryPlan) -> bool {
        plan.cost > 50.0 // Only cache expensive queries
    }
    
    fn apply(&self, mut plan: QueryPlan) -> Result<QueryPlan> {
        // Add cache check step at the beginning
        plan.steps.insert(0, QueryStep {
            name: "Cache Lookup".to_string(),
            description: "Check if results are cached".to_string(),
            cost: 1.0,
            parallel: false,
        });
        
        // Add cache write step at the end
        plan.steps.push(QueryStep {
            name: "Cache Write".to_string(),
            description: "Store results in cache".to_string(),
            cost: 2.0,
            parallel: false,
        });
        
        Ok(plan)
    }
    
    fn name(&self) -> &str {
        "cache"
    }
}

/// Query optimization hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationHints {
    /// Force use of specific index
    pub use_index: Option<String>,
    
    /// Disable parallelization
    pub no_parallel: bool,
    
    /// Disable caching
    pub no_cache: bool,
    
    /// Custom cost factors
    pub cost_overrides: HashMap<String, f64>,
}