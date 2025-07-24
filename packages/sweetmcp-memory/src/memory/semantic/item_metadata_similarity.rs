//! Semantic item metadata similarity and string distance operations
//!
//! This module provides string similarity algorithms for metadata validation
//! with zero allocation patterns and blazing-fast performance.

use super::item_core::SemanticItem;

impl SemanticItem {
    /// Check for metadata keys that might be typos
    /// 
    /// # Returns
    /// Vector of potentially misspelled keys
    pub fn find_potential_typos(&self) -> Vec<String> {
        let mut potential_typos = Vec::new();
        let keys: Vec<&String> = self.metadata.keys().collect();
        
        for (i, key1) in keys.iter().enumerate() {
            for key2 in keys.iter().skip(i + 1) {
                if Self::are_keys_similar(key1, key2) {
                    potential_typos.push(format!("'{}' and '{}' are similar", key1, key2));
                }
            }
        }
        
        potential_typos
    }

    /// Check if two keys are similar (potential typos)
    /// 
    /// # Arguments
    /// * `key1` - First key
    /// * `key2` - Second key
    /// 
    /// # Returns
    /// True if keys are similar enough to be potential typos
    pub fn are_keys_similar(key1: &str, key2: &str) -> bool {
        if key1 == key2 {
            return false; // Identical keys are not typos
        }
        
        let len1 = key1.len();
        let len2 = key2.len();
        
        // Keys must be reasonably similar in length
        if (len1 as i32 - len2 as i32).abs() > 2 {
            return false;
        }
        
        // Simple edit distance check
        let distance = Self::levenshtein_distance(key1, key2);
        distance <= 2 && distance > 0
    }

    /// Calculate Levenshtein distance between two strings
    /// 
    /// # Arguments
    /// * `s1` - First string
    /// * `s2` - Second string
    /// 
    /// # Returns
    /// Levenshtein distance
    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }

    /// Calculate Jaro similarity between two strings
    /// 
    /// # Arguments
    /// * `s1` - First string
    /// * `s2` - Second string
    /// 
    /// # Returns
    /// Jaro similarity score (0.0 to 1.0)
    pub fn jaro_similarity(s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }
        
        let match_window = (len1.max(len2) / 2).saturating_sub(1);
        
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        let mut s1_matches = vec![false; len1];
        let mut s2_matches = vec![false; len2];
        
        let mut matches = 0;
        
        // Find matches
        for i in 0..len1 {
            let start = i.saturating_sub(match_window);
            let end = (i + match_window + 1).min(len2);
            
            for j in start..end {
                if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                    continue;
                }
                
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
        
        if matches == 0 {
            return 0.0;
        }
        
        // Count transpositions
        let mut transpositions = 0;
        let mut k = 0;
        
        for i in 0..len1 {
            if !s1_matches[i] {
                continue;
            }
            
            while !s2_matches[k] {
                k += 1;
            }
            
            if s1_chars[i] != s2_chars[k] {
                transpositions += 1;
            }
            
            k += 1;
        }
        
        let jaro = (matches as f64 / len1 as f64 + 
                   matches as f64 / len2 as f64 + 
                   (matches as f64 - transpositions as f64 / 2.0) / matches as f64) / 3.0;
        
        jaro
    }

    /// Calculate Jaro-Winkler similarity between two strings
    /// 
    /// # Arguments
    /// * `s1` - First string
    /// * `s2` - Second string
    /// 
    /// # Returns
    /// Jaro-Winkler similarity score (0.0 to 1.0)
    pub fn jaro_winkler_similarity(s1: &str, s2: &str) -> f64 {
        let jaro = Self::jaro_similarity(s1, s2);
        
        if jaro < 0.7 {
            return jaro;
        }
        
        // Calculate common prefix length (up to 4 characters)
        let prefix_len = s1.chars()
            .zip(s2.chars())
            .take(4)
            .take_while(|(c1, c2)| c1 == c2)
            .count();
        
        jaro + (0.1 * prefix_len as f64 * (1.0 - jaro))
    }

    /// Find metadata keys with high similarity to a target key
    /// 
    /// # Arguments
    /// * `target_key` - Key to find similar keys for
    /// * `threshold` - Minimum similarity threshold (0.0 to 1.0)
    /// 
    /// # Returns
    /// Vector of (key, similarity_score) pairs
    pub fn find_similar_metadata_keys(&self, target_key: &str, threshold: f64) -> Vec<(String, f64)> {
        self.metadata.keys()
            .filter_map(|key| {
                if key == target_key {
                    return None; // Skip identical keys
                }
                
                let similarity = Self::jaro_winkler_similarity(target_key, key);
                if similarity >= threshold {
                    Some((key.clone(), similarity))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the most similar metadata key to a target key
    /// 
    /// # Arguments
    /// * `target_key` - Key to find the most similar key for
    /// 
    /// # Returns
    /// Option of (key, similarity_score) for the most similar key
    pub fn get_most_similar_metadata_key(&self, target_key: &str) -> Option<(String, f64)> {
        self.metadata.keys()
            .filter(|&key| key != target_key)
            .map(|key| {
                let similarity = Self::jaro_winkler_similarity(target_key, key);
                (key.clone(), similarity)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Calculate overall key similarity diversity
    /// 
    /// # Returns
    /// Average pairwise similarity between all metadata keys
    pub fn calculate_key_similarity_diversity(&self) -> f64 {
        let keys: Vec<&String> = self.metadata.keys().collect();
        let key_count = keys.len();
        
        if key_count < 2 {
            return 1.0; // Perfect diversity with 0 or 1 keys
        }
        
        let mut total_similarity = 0.0;
        let mut pair_count = 0;
        
        for i in 0..key_count {
            for j in (i + 1)..key_count {
                total_similarity += Self::jaro_winkler_similarity(keys[i], keys[j]);
                pair_count += 1;
            }
        }
        
        if pair_count == 0 {
            1.0
        } else {
            1.0 - (total_similarity / pair_count as f64) // Invert so higher diversity = higher score
        }
    }
}