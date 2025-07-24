//! Specialized evaluation algorithms for different assessment dimensions
//!
//! This module provides the core evaluation algorithms used by agents
//! to assess actions across different dimensions with zero allocation patterns.

use crate::cognitive::mcts::CodeState;

/// Core evaluation algorithms
/// 
/// Provides static methods for evaluating different aspects of actions
/// with specialized assessment logic for alignment, quality, and risk.
pub struct EvaluationAlgorithms;

impl EvaluationAlgorithms {
    /// Evaluate alignment with objective
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// * `objective` - The objective to align with
    /// 
    /// # Returns
    /// Alignment score (0.0 to 1.0)
    pub fn evaluate_alignment(action: &str, objective: &str) -> f64 {
        // Simplified alignment evaluation based on keyword matching
        let action_words: std::collections::HashSet<&str> = action.split_whitespace().collect();
        let objective_words: std::collections::HashSet<&str> = objective.split_whitespace().collect();
        
        let intersection_count = action_words.intersection(&objective_words).count();
        let union_count = action_words.union(&objective_words).count();
        
        if union_count == 0 {
            0.5 // Default neutral score
        } else {
            let jaccard_similarity = intersection_count as f64 / union_count as f64;
            // Apply semantic boost for common programming terms
            let semantic_boost = Self::calculate_semantic_boost(action, objective);
            (jaccard_similarity + semantic_boost).clamp(0.0, 1.0)
        }
    }

    /// Calculate semantic boost for alignment evaluation
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// * `objective` - The objective to align with
    /// 
    /// # Returns
    /// Semantic boost factor (0.0 to 0.3)
    fn calculate_semantic_boost(action: &str, objective: &str) -> f64 {
        let programming_terms = [
            "implement", "create", "build", "develop", "code", "function", "method",
            "class", "module", "test", "debug", "optimize", "refactor", "fix"
        ];
        
        let action_lower = action.to_lowercase();
        let objective_lower = objective.to_lowercase();
        
        let matching_terms = programming_terms.iter()
            .filter(|&term| action_lower.contains(term) && objective_lower.contains(term))
            .count();
        
        (matching_terms as f64 * 0.1).min(0.3)
    }

    /// Evaluate implementation quality
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// * `code_state` - Current code state
    /// 
    /// # Returns
    /// Quality score (0.0 to 1.0)
    pub fn evaluate_quality(action: &str, code_state: &CodeState) -> f64 {
        // Simplified quality evaluation based on action complexity and patterns
        let mut quality_score = 0.5; // Base score
        
        // Positive quality indicators
        let quality_keywords = ["test", "validate", "check", "verify", "document", "optimize"];
        let quality_boost = quality_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count() as f64 * 0.1;
        
        // Negative quality indicators
        let anti_patterns = ["hack", "quick fix", "temporary", "todo", "fixme"];
        let quality_penalty = anti_patterns.iter()
            .filter(|&pattern| action.to_lowercase().contains(pattern))
            .count() as f64 * 0.15;
        
        // Action length factor (moderate length is better)
        let length_factor = if action.len() < 20 {
            0.7 // Too short, might be incomplete
        } else if action.len() > 200 {
            0.8 // Too long, might be complex
        } else {
            1.0 // Good length
        };
        
        quality_score = (quality_score + quality_boost - quality_penalty) * length_factor;
        quality_score.clamp(0.0, 1.0)
    }

    /// Evaluate risk assessment
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// * `code_state` - Current code state
    /// 
    /// # Returns
    /// Risk score (0.0 = high risk, 1.0 = safe)
    pub fn evaluate_risk(action: &str, code_state: &CodeState) -> f64 {
        let mut safety_score = 0.8; // Base safety score
        
        // High-risk keywords
        let high_risk_keywords = ["delete", "remove", "drop", "unsafe", "panic", "unwrap"];
        let high_risk_count = high_risk_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count();
        
        // Medium-risk keywords
        let medium_risk_keywords = ["modify", "change", "alter", "replace", "update"];
        let medium_risk_count = medium_risk_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count();
        
        // Safety-enhancing keywords
        let safety_keywords = ["test", "validate", "backup", "check", "verify", "safe"];
        let safety_boost = safety_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count() as f64 * 0.1;
        
        // Calculate risk penalties
        let high_risk_penalty = high_risk_count as f64 * 0.3;
        let medium_risk_penalty = medium_risk_count as f64 * 0.1;
        
        safety_score = safety_score - high_risk_penalty - medium_risk_penalty + safety_boost;
        safety_score.clamp(0.0, 1.0)
    }

    /// Evaluate security-specific risk factors
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// 
    /// # Returns
    /// Security risk score (0.0 = high risk, 1.0 = safe)
    pub fn evaluate_security_risk(action: &str) -> f64 {
        let security_risks = ["password", "token", "key", "secret", "auth", "login", "admin"];
        let risk_count = security_risks.iter()
            .filter(|&risk| action.to_lowercase().contains(risk))
            .count();
        
        (1.0 - (risk_count as f64 * 0.2)).clamp(0.0, 1.0)
    }

    /// Evaluate performance-specific quality factors
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// 
    /// # Returns
    /// Performance quality score (0.0 to 1.0)
    pub fn evaluate_performance_quality(action: &str) -> f64 {
        let performance_keywords = ["optimize", "cache", "async", "parallel", "efficient", "fast"];
        let boost = performance_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count() as f64 * 0.15;
        
        (0.5 + boost).clamp(0.0, 1.0)
    }

    /// Evaluate maintainability factors
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// 
    /// # Returns
    /// Maintainability score (0.0 to 1.0)
    pub fn evaluate_maintainability(action: &str) -> f64 {
        let maintainability_keywords = ["document", "comment", "refactor", "clean", "organize", "structure"];
        let boost = maintainability_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count() as f64 * 0.12;
        
        (0.6 + boost).clamp(0.0, 1.0)
    }

    /// Evaluate complexity factors
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// 
    /// # Returns
    /// Complexity score (0.0 = very complex, 1.0 = simple)
    pub fn evaluate_complexity(action: &str) -> f64 {
        let complexity_indicators = ["complex", "complicated", "intricate", "nested", "recursive"];
        let simplicity_indicators = ["simple", "straightforward", "basic", "minimal", "clean"];
        
        let complexity_count = complexity_indicators.iter()
            .filter(|&indicator| action.to_lowercase().contains(indicator))
            .count();
        
        let simplicity_count = simplicity_indicators.iter()
            .filter(|&indicator| action.to_lowercase().contains(indicator))
            .count();
        
        // Base complexity score based on action length
        let length_complexity = if action.len() > 150 {
            0.3 // Long actions are typically more complex
        } else if action.len() < 30 {
            0.9 // Short actions are typically simpler
        } else {
            0.6 // Medium length actions
        };
        
        let complexity_penalty = complexity_count as f64 * 0.2;
        let simplicity_boost = simplicity_count as f64 * 0.15;
        
        (length_complexity - complexity_penalty + simplicity_boost).clamp(0.0, 1.0)
    }

    /// Evaluate innovation factors
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// 
    /// # Returns
    /// Innovation score (0.0 to 1.0)
    pub fn evaluate_innovation(action: &str) -> f64 {
        let innovation_keywords = ["new", "novel", "innovative", "creative", "original", "unique"];
        let conventional_keywords = ["standard", "typical", "conventional", "traditional", "common"];
        
        let innovation_count = innovation_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count();
        
        let conventional_count = conventional_keywords.iter()
            .filter(|&keyword| action.to_lowercase().contains(keyword))
            .count();
        
        let innovation_boost = innovation_count as f64 * 0.2;
        let conventional_penalty = conventional_count as f64 * 0.1;
        
        (0.5 + innovation_boost - conventional_penalty).clamp(0.0, 1.0)
    }

    /// Evaluate completeness factors
    /// 
    /// # Arguments
    /// * `action` - The action to evaluate
    /// 
    /// # Returns
    /// Completeness score (0.0 to 1.0)
    pub fn evaluate_completeness(action: &str) -> f64 {
        let completeness_indicators = ["complete", "full", "comprehensive", "thorough", "detailed"];
        let incompleteness_indicators = ["partial", "incomplete", "draft", "stub", "placeholder"];
        
        let complete_count = completeness_indicators.iter()
            .filter(|&indicator| action.to_lowercase().contains(indicator))
            .count();
        
        let incomplete_count = incompleteness_indicators.iter()
            .filter(|&indicator| action.to_lowercase().contains(indicator))
            .count();
        
        let completeness_boost = complete_count as f64 * 0.15;
        let incompleteness_penalty = incomplete_count as f64 * 0.25;
        
        (0.7 + completeness_boost - incompleteness_penalty).clamp(0.0, 1.0)
    }

    /// Calculate weighted score based on multiple dimensions
    /// 
    /// # Arguments
    /// * `scores` - Vector of (score, weight) tuples
    /// 
    /// # Returns
    /// Weighted average score (0.0 to 1.0)
    pub fn calculate_weighted_score(scores: &[(f64, f64)]) -> f64 {
        if scores.is_empty() {
            return 0.0;
        }
        
        let total_weighted_score: f64 = scores.iter().map(|(score, weight)| score * weight).sum();
        let total_weight: f64 = scores.iter().map(|(_, weight)| weight).sum();
        
        if total_weight == 0.0 {
            0.0
        } else {
            (total_weighted_score / total_weight).clamp(0.0, 1.0)
        }
    }
}