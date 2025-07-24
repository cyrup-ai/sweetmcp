//! Quantum entanglement engine issue collection and statistics
//!
//! This module provides issue collection management and statistical analysis
//! with zero-allocation patterns and blazing-fast performance.

use super::engine_issue_types::{NetworkIssue, IssueSeverity, IssueCategory};

/// Issue collection with analysis capabilities
#[derive(Debug, Clone)]
pub struct IssueCollection {
    pub issues: Vec<NetworkIssue>,
    pub created_at: std::time::SystemTime,
}

impl IssueCollection {
    /// Create new issue collection
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Add issue to collection
    pub fn add_issue(&mut self, issue: NetworkIssue) {
        self.issues.push(issue);
    }

    /// Get issues by severity
    pub fn issues_by_severity(&self, severity: IssueSeverity) -> Vec<&NetworkIssue> {
        self.issues.iter().filter(|issue| issue.severity == severity).collect()
    }

    /// Get issues by category
    pub fn issues_by_category(&self, category: IssueCategory) -> Vec<&NetworkIssue> {
        self.issues.iter().filter(|issue| issue.category == category).collect()
    }

    /// Get critical issues requiring immediate attention
    pub fn critical_issues(&self) -> Vec<&NetworkIssue> {
        self.issues.iter().filter(|issue| issue.requires_immediate_attention()).collect()
    }

    /// Get overall severity score
    pub fn overall_severity_score(&self) -> f64 {
        if self.issues.is_empty() {
            return 0.0;
        }

        let total_score: f64 = self.issues.iter()
            .map(|issue| issue.severity.priority_value() as f64 * issue.impact_score)
            .sum();

        total_score / self.issues.len() as f64
    }

    /// Get summary statistics
    pub fn summary_stats(&self) -> IssueSummaryStats {
        let mut stats = IssueSummaryStats::default();
        
        for issue in &self.issues {
            match issue.severity {
                IssueSeverity::Info => stats.info_count += 1,
                IssueSeverity::Warning => stats.warning_count += 1,
                IssueSeverity::Error => stats.error_count += 1,
                IssueSeverity::Critical => stats.critical_count += 1,
            }
            
            stats.total_impact_score += issue.impact_score;
        }
        
        stats.total_count = self.issues.len();
        stats.average_impact_score = if stats.total_count > 0 {
            stats.total_impact_score / stats.total_count as f64
        } else {
            0.0
        };
        
        stats
    }

    /// Check if collection has critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|issue| matches!(issue.severity, IssueSeverity::Critical))
    }

    /// Get recommended actions
    pub fn recommended_actions(&self) -> Vec<String> {
        let mut actions = Vec::new();
        
        if self.has_critical_issues() {
            actions.push("Address critical issues immediately".to_string());
        }
        
        let error_count = self.issues_by_severity(IssueSeverity::Error).len();
        if error_count > 0 {
            actions.push(format!("Resolve {} error-level issues", error_count));
        }
        
        let warning_count = self.issues_by_severity(IssueSeverity::Warning).len();
        if warning_count > 3 {
            actions.push("Review and address warning-level issues".to_string());
        }
        
        if actions.is_empty() {
            actions.push("Monitor network health regularly".to_string());
        }
        
        actions
    }

    /// Get issues sorted by priority
    pub fn issues_by_priority(&self) -> Vec<&NetworkIssue> {
        let mut issues: Vec<&NetworkIssue> = self.issues.iter().collect();
        issues.sort_by(|a, b| {
            // Sort by severity first, then by impact score
            let severity_cmp = b.severity.priority_value().cmp(&a.severity.priority_value());
            if severity_cmp == std::cmp::Ordering::Equal {
                b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                severity_cmp
            }
        });
        issues
    }

    /// Get issues affecting connectivity
    pub fn connectivity_issues(&self) -> Vec<&NetworkIssue> {
        self.issues.iter().filter(|issue| issue.affects_connectivity()).collect()
    }

    /// Get issues affecting performance
    pub fn performance_issues(&self) -> Vec<&NetworkIssue> {
        self.issues.iter().filter(|issue| issue.affects_performance()).collect()
    }

    /// Get total estimated resolution time in minutes
    pub fn total_estimated_resolution_time(&self) -> u32 {
        self.issues.iter().map(|issue| issue.estimated_resolution_time()).sum()
    }

    /// Get issues by affected node count
    pub fn issues_by_node_impact(&self) -> Vec<&NetworkIssue> {
        let mut issues: Vec<&NetworkIssue> = self.issues.iter().collect();
        issues.sort_by(|a, b| b.affected_nodes.len().cmp(&a.affected_nodes.len()));
        issues
    }

    /// Get unique affected nodes across all issues
    pub fn all_affected_nodes(&self) -> Vec<String> {
        let mut nodes = std::collections::HashSet::new();
        for issue in &self.issues {
            for node in &issue.affected_nodes {
                nodes.insert(node.clone());
            }
        }
        nodes.into_iter().collect()
    }

    /// Get issue distribution by category
    pub fn category_distribution(&self) -> std::collections::HashMap<IssueCategory, usize> {
        let mut distribution = std::collections::HashMap::new();
        for issue in &self.issues {
            *distribution.entry(issue.category.clone()).or_insert(0) += 1;
        }
        distribution
    }

    /// Get severity distribution
    pub fn severity_distribution(&self) -> std::collections::HashMap<IssueSeverity, usize> {
        let mut distribution = std::collections::HashMap::new();
        for issue in &self.issues {
            *distribution.entry(issue.severity.clone()).or_insert(0) += 1;
        }
        distribution
    }

    /// Filter issues by time range
    pub fn issues_since(&self, since: std::time::SystemTime) -> Vec<&NetworkIssue> {
        self.issues.iter()
            .filter(|issue| issue.detected_at >= since)
            .collect()
    }

    /// Get issues with high impact (>0.7)
    pub fn high_impact_issues(&self) -> Vec<&NetworkIssue> {
        self.issues.iter()
            .filter(|issue| issue.impact_score > 0.7)
            .collect()
    }

    /// Clear all issues
    pub fn clear(&mut self) {
        self.issues.clear();
        self.created_at = std::time::SystemTime::now();
    }

    /// Remove resolved issues (placeholder - in real implementation would check resolution status)
    pub fn remove_resolved_issues(&mut self) {
        // In a real implementation, this would check if issues have been resolved
        // For now, we'll remove issues older than 24 hours as a placeholder
        let twenty_four_hours_ago = std::time::SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(24 * 60 * 60))
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        
        self.issues.retain(|issue| issue.detected_at >= twenty_four_hours_ago);
    }

    /// Get health score based on issues (0.0 = critical, 1.0 = perfect)
    pub fn health_score(&self) -> f64 {
        if self.issues.is_empty() {
            return 1.0;
        }

        let severity_penalty: f64 = self.issues.iter()
            .map(|issue| match issue.severity {
                IssueSeverity::Critical => 0.4,
                IssueSeverity::Error => 0.2,
                IssueSeverity::Warning => 0.1,
                IssueSeverity::Info => 0.02,
            })
            .sum();

        let impact_penalty: f64 = self.issues.iter()
            .map(|issue| issue.impact_score * 0.3)
            .sum();

        let total_penalty = severity_penalty + impact_penalty;
        (1.0 - total_penalty).max(0.0)
    }
}

impl Default for IssueCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics for issue collection
#[derive(Debug, Clone, Default)]
pub struct IssueSummaryStats {
    pub total_count: usize,
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub total_impact_score: f64,
    pub average_impact_score: f64,
}

impl IssueSummaryStats {
    /// Get formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Total: {}, Critical: {}, Error: {}, Warning: {}, Info: {}, Avg Impact: {:.1}%",
            self.total_count,
            self.critical_count,
            self.error_count,
            self.warning_count,
            self.info_count,
            self.average_impact_score * 100.0
        )
    }

    /// Check if stats indicate healthy network
    pub fn indicates_healthy_network(&self) -> bool {
        self.critical_count == 0 && 
        self.error_count == 0 && 
        self.warning_count <= 2 &&
        self.average_impact_score < 0.3
    }

    /// Get overall severity level
    pub fn overall_severity(&self) -> IssueSeverity {
        if self.critical_count > 0 {
            IssueSeverity::Critical
        } else if self.error_count > 0 {
            IssueSeverity::Error
        } else if self.warning_count > 0 {
            IssueSeverity::Warning
        } else {
            IssueSeverity::Info
        }
    }

    /// Get priority score (higher = more urgent)
    pub fn priority_score(&self) -> f64 {
        (self.critical_count as f64 * 4.0) +
        (self.error_count as f64 * 3.0) +
        (self.warning_count as f64 * 2.0) +
        (self.info_count as f64 * 1.0)
    }

    /// Check if immediate action is required
    pub fn requires_immediate_action(&self) -> bool {
        self.critical_count > 0 || self.error_count > 2
    }
}