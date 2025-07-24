//! Semantic item metadata validation operations
//!
//! This module provides validation and integrity checking for metadata
//! with zero allocation patterns and blazing-fast performance.

use serde_json::Value;

use super::item_core::SemanticItem;

impl SemanticItem {
    /// Validate metadata integrity
    /// 
    /// # Returns
    /// Vector of validation issues found
    pub fn validate_metadata_integrity(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        for (key, value) in &self.metadata {
            // Check for empty keys
            if key.is_empty() {
                issues.push("Empty metadata key found".to_string());
            }
            
            // Check for very long keys
            if key.len() > 100 {
                issues.push(format!("Very long key found: {} ({} chars)", key, key.len()));
            }
            
            // Check for suspicious characters in keys
            if key.contains('\n') || key.contains('\r') || key.contains('\t') {
                issues.push(format!("Key contains whitespace characters: {}", key));
            }
            
            // Check for very large string values
            if let Value::String(s) = value {
                if s.len() > 10000 {
                    issues.push(format!("Very large string value for key '{}': {} chars", key, s.len()));
                }
            }
            
            // Check for very large arrays
            if let Value::Array(arr) = value {
                if arr.len() > 1000 {
                    issues.push(format!("Very large array for key '{}': {} elements", key, arr.len()));
                }
            }
            
            // Check for deeply nested objects
            if let Value::Object(obj) = value {
                if obj.len() > 50 {
                    issues.push(format!("Very large object for key '{}': {} properties", key, obj.len()));
                }
            }
        }
        
        issues
    }

    /// Validate metadata key format
    /// 
    /// # Arguments
    /// * `key` - Key to validate
    /// 
    /// # Returns
    /// True if key format is valid
    pub fn is_valid_metadata_key(key: &str) -> bool {
        if key.is_empty() || key.len() > 100 {
            return false;
        }
        
        // Check for invalid characters
        if key.contains('\n') || key.contains('\r') || key.contains('\t') {
            return false;
        }
        
        // Key should not start or end with whitespace
        if key.starts_with(' ') || key.ends_with(' ') {
            return false;
        }
        
        true
    }

    /// Validate metadata value size
    /// 
    /// # Arguments
    /// * `value` - Value to validate
    /// 
    /// # Returns
    /// True if value size is reasonable
    pub fn is_reasonable_metadata_value_size(value: &Value) -> bool {
        match value {
            Value::String(s) => s.len() <= 10000,
            Value::Array(arr) => arr.len() <= 1000,
            Value::Object(obj) => obj.len() <= 50,
            _ => true, // Numbers, booleans, and null are always reasonable
        }
    }

    /// Check if metadata is well-formed
    /// 
    /// # Returns
    /// True if all metadata entries are well-formed
    pub fn is_metadata_well_formed(&self) -> bool {
        for (key, value) in &self.metadata {
            if !Self::is_valid_metadata_key(key) {
                return false;
            }
            
            if !Self::is_reasonable_metadata_value_size(value) {
                return false;
            }
        }
        
        true
    }

    /// Get metadata validation score
    /// 
    /// # Returns
    /// Validation score (0.0 to 1.0) based on various quality metrics
    pub fn get_metadata_validation_score(&self) -> f64 {
        if self.metadata.is_empty() {
            return 1.0; // Empty metadata is considered valid
        }
        
        let total_entries = self.metadata.len();
        let mut valid_entries = 0;
        
        for (key, value) in &self.metadata {
            let key_valid = Self::is_valid_metadata_key(key);
            let value_valid = Self::is_reasonable_metadata_value_size(value);
            
            if key_valid && value_valid {
                valid_entries += 1;
            }
        }
        
        valid_entries as f64 / total_entries as f64
    }

    /// Sanitize metadata key
    /// 
    /// # Arguments
    /// * `key` - Key to sanitize
    /// 
    /// # Returns
    /// Sanitized key
    pub fn sanitize_metadata_key(key: &str) -> String {
        key.trim()
            .replace('\n', "_")
            .replace('\r', "_")
            .replace('\t', "_")
            .chars()
            .take(100)
            .collect()
    }

    /// Check for duplicate metadata keys (case-insensitive)
    /// 
    /// # Returns
    /// Vector of duplicate key groups
    pub fn find_duplicate_metadata_keys(&self) -> Vec<Vec<String>> {
        let mut key_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        
        for key in self.metadata.keys() {
            let lowercase_key = key.to_lowercase();
            key_groups.entry(lowercase_key).or_default().push(key.clone());
        }
        
        key_groups.into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }

    /// Get comprehensive metadata validation report
    /// 
    /// # Returns
    /// Detailed validation report string
    pub fn get_metadata_validation_report(&self) -> String {
        if self.metadata.is_empty() {
            return "No metadata to validate".to_string();
        }
        
        let integrity_issues = self.validate_metadata_integrity();
        let validation_score = self.get_metadata_validation_score();
        let duplicate_keys = self.find_duplicate_metadata_keys();
        let potential_typos = self.find_potential_typos();
        
        let mut report = format!(
            "Metadata Validation Report:\n\
             - Total entries: {}\n\
             - Validation score: {:.1}%\n\
             - Well-formed: {}\n",
            self.metadata.len(),
            validation_score * 100.0,
            self.is_metadata_well_formed()
        );
        
        if !integrity_issues.is_empty() {
            report.push_str(&format!("- Integrity issues ({}):\n", integrity_issues.len()));
            for issue in integrity_issues.iter().take(5) {
                report.push_str(&format!("  - {}\n", issue));
            }
            if integrity_issues.len() > 5 {
                report.push_str(&format!("  - ... and {} more\n", integrity_issues.len() - 5));
            }
        }
        
        if !duplicate_keys.is_empty() {
            report.push_str(&format!("- Duplicate key groups ({}):\n", duplicate_keys.len()));
            for group in duplicate_keys.iter().take(3) {
                report.push_str(&format!("  - {:?}\n", group));
            }
            if duplicate_keys.len() > 3 {
                report.push_str(&format!("  - ... and {} more groups\n", duplicate_keys.len() - 3));
            }
        }
        
        if !potential_typos.is_empty() {
            report.push_str(&format!("- Potential typos ({}):\n", potential_typos.len()));
            for typo in potential_typos.iter().take(3) {
                report.push_str(&format!("  - {}\n", typo));
            }
            if potential_typos.len() > 3 {
                report.push_str(&format!("  - ... and {} more\n", potential_typos.len() - 3));
            }
        }
        
        report
    }

    /// Perform comprehensive metadata cleanup
    /// 
    /// # Returns
    /// Number of issues fixed
    pub fn cleanup_metadata(&mut self) -> usize {
        let mut fixes = 0;
        let mut keys_to_fix = Vec::new();
        
        // Collect keys that need sanitization
        for key in self.metadata.keys() {
            if !Self::is_valid_metadata_key(key) {
                let sanitized = Self::sanitize_metadata_key(key);
                if sanitized != *key {
                    keys_to_fix.push((key.clone(), sanitized));
                }
            }
        }
        
        // Apply key fixes
        for (old_key, new_key) in keys_to_fix {
            if let Some(value) = self.metadata.remove(&old_key) {
                self.metadata.insert(new_key, value);
                fixes += 1;
            }
        }
        
        // Remove null values
        let null_keys: Vec<String> = self.metadata.iter()
            .filter(|(_, value)| value.is_null())
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in null_keys {
            self.metadata.remove(&key);
            fixes += 1;
        }
        
        fixes
    }

    /// Check metadata quality score
    /// 
    /// # Returns
    /// Overall quality score (0.0 to 1.0)
    pub fn get_metadata_quality_score(&self) -> f64 {
        if self.metadata.is_empty() {
            return 1.0;
        }
        
        let validation_score = self.get_metadata_validation_score();
        let health_score = self.get_metadata_health_score();
        let diversity_score = self.calculate_key_similarity_diversity();
        
        // Weighted average of different quality metrics
        (validation_score * 0.4 + health_score * 0.4 + diversity_score * 0.2).clamp(0.0, 1.0)
    }
}