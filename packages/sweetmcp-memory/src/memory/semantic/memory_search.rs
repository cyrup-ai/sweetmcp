//! Semantic memory advanced search operations
//!
//! This module provides advanced search capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;

impl SemanticMemory {
    /// Find items by fuzzy content matching
    pub fn find_items_by_fuzzy_content(&self, query: &str, max_distance: usize) -> Vec<(&SemanticItem, usize)> {
        let mut results = Vec::new();
        
        for item in &self.items {
            let distance = self.levenshtein_distance(&item.content.to_lowercase(), &query.to_lowercase());
            if distance <= max_distance {
                results.push((item, distance));
            }
        }
        
        // Sort by distance (closest first)
        results.sort_by(|a, b| a.1.cmp(&b.1));
        results
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }

    /// Find items by Jaro-Winkler similarity
    pub fn find_items_by_jaro_winkler(&self, query: &str, min_similarity: f64) -> Vec<(&SemanticItem, f64)> {
        let mut results = Vec::new();
        
        for item in &self.items {
            let similarity = self.jaro_winkler_similarity(&item.content.to_lowercase(), &query.to_lowercase());
            if similarity >= min_similarity {
                results.push((item, similarity));
            }
        }
        
        // Sort by similarity (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Calculate Jaro-Winkler similarity between two strings
    fn jaro_winkler_similarity(&self, s1: &str, s2: &str) -> f64 {
        let jaro_sim = self.jaro_similarity(s1, s2);
        
        if jaro_sim < 0.7 {
            return jaro_sim;
        }
        
        // Calculate common prefix length (up to 4 characters)
        let prefix_len = s1.chars()
            .zip(s2.chars())
            .take(4)
            .take_while(|(c1, c2)| c1 == c2)
            .count() as f64;
        
        jaro_sim + (0.1 * prefix_len * (1.0 - jaro_sim))
    }

    /// Calculate Jaro similarity between two strings
    fn jaro_similarity(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 && len2 == 0 {
            return 1.0;
        }
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }
        
        let match_window = (len1.max(len2) / 2).saturating_sub(1);
        
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        
        let mut matches1 = vec![false; len1];
        let mut matches2 = vec![false; len2];
        
        let mut matches = 0;
        
        // Find matches
        for i in 0..len1 {
            let start = if i >= match_window { i - match_window } else { 0 };
            let end = (i + match_window + 1).min(len2);
            
            for j in start..end {
                if matches2[j] || chars1[i] != chars2[j] {
                    continue;
                }
                matches1[i] = true;
                matches2[j] = true;
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
            if !matches1[i] {
                continue;
            }
            while !matches2[k] {
                k += 1;
            }
            if chars1[i] != chars2[k] {
                transpositions += 1;
            }
            k += 1;
        }
        
        let jaro = (matches as f64 / len1 as f64 + 
                   matches as f64 / len2 as f64 + 
                   (matches as f64 - transpositions as f64 / 2.0) / matches as f64) / 3.0;
        
        jaro
    }

    /// Search items using n-gram similarity
    pub fn search_items_by_ngram(&self, query: &str, n: usize, min_similarity: f64) -> Vec<(&SemanticItem, f64)> {
        let query_ngrams = self.generate_ngrams(query, n);
        let mut results = Vec::new();
        
        for item in &self.items {
            let item_ngrams = self.generate_ngrams(&item.content, n);
            let similarity = self.ngram_similarity(&query_ngrams, &item_ngrams);
            
            if similarity >= min_similarity {
                results.push((item, similarity));
            }
        }
        
        // Sort by similarity (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Generate n-grams from a string
    fn generate_ngrams(&self, text: &str, n: usize) -> std::collections::HashSet<String> {
        let mut ngrams = std::collections::HashSet::new();
        let chars: Vec<char> = text.to_lowercase().chars().collect();
        
        if chars.len() < n {
            ngrams.insert(text.to_lowercase());
            return ngrams;
        }
        
        for i in 0..=chars.len() - n {
            let ngram: String = chars[i..i + n].iter().collect();
            ngrams.insert(ngram);
        }
        
        ngrams
    }

    /// Calculate n-gram similarity between two sets of n-grams
    fn ngram_similarity(&self, ngrams1: &std::collections::HashSet<String>, ngrams2: &std::collections::HashSet<String>) -> f64 {
        if ngrams1.is_empty() && ngrams2.is_empty() {
            return 1.0;
        }
        if ngrams1.is_empty() || ngrams2.is_empty() {
            return 0.0;
        }
        
        let intersection_size = ngrams1.intersection(ngrams2).count() as f64;
        let union_size = ngrams1.union(ngrams2).count() as f64;
        
        intersection_size / union_size
    }

    /// Full-text search across all item content
    pub fn full_text_search(&self, query: &str) -> Vec<(&SemanticItem, f64)> {
        let query_terms: Vec<&str> = query.to_lowercase().split_whitespace().collect();
        let mut results = Vec::new();
        
        for item in &self.items {
            let content_lower = item.content.to_lowercase();
            let mut score = 0.0;
            let mut term_matches = 0;
            
            for term in &query_terms {
                if content_lower.contains(term) {
                    term_matches += 1;
                    // Simple TF scoring
                    let term_frequency = content_lower.matches(term).count() as f64;
                    score += term_frequency;
                }
            }
            
            if term_matches > 0 {
                // Normalize by content length and boost for term coverage
                let coverage_boost = term_matches as f64 / query_terms.len() as f64;
                let length_normalization = score / (content_lower.len() as f64).sqrt();
                let final_score = length_normalization * coverage_boost;
                
                results.push((item, final_score));
            }
        }
        
        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Search items by semantic similarity (placeholder for future vector search)
    pub fn semantic_search(&self, query: &str, top_k: usize) -> Vec<(&SemanticItem, f64)> {
        // For now, use a combination of fuzzy matching and n-gram similarity
        let fuzzy_results = self.find_items_by_fuzzy_content(query, 5);
        let ngram_results = self.search_items_by_ngram(query, 3, 0.1);
        
        let mut combined_scores = std::collections::HashMap::new();
        
        // Combine fuzzy results (inverse distance as score)
        for (item, distance) in fuzzy_results {
            let score = 1.0 / (1.0 + distance as f64);
            *combined_scores.entry(item.id.as_str()).or_insert(0.0) += score * 0.4;
        }
        
        // Combine n-gram results
        for (item, similarity) in ngram_results {
            *combined_scores.entry(item.id.as_str()).or_insert(0.0) += similarity * 0.6;
        }
        
        // Convert back to items with scores
        let mut results: Vec<(&SemanticItem, f64)> = combined_scores
            .into_iter()
            .filter_map(|(item_id, score)| {
                self.get_item(item_id).map(|item| (item, score))
            })
            .collect();
        
        // Sort by score (highest first) and take top_k
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(top_k).collect()
    }

    /// Extract a snippet around the query match
    pub fn extract_snippet(&self, text: &str, query: &str, max_length: usize) -> String {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();
        
        if let Some(match_pos) = text_lower.find(&query_lower) {
            let start = if match_pos >= max_length / 2 {
                match_pos - max_length / 2
            } else {
                0
            };
            
            let end = (start + max_length).min(text.len());
            let snippet = &text[start..end];
            
            if start > 0 {
                format!("...{}", snippet)
            } else if end < text.len() {
                format!("{}...", snippet)
            } else {
                snippet.to_string()
            }
        } else {
            // Fallback to beginning of text
            if text.len() <= max_length {
                text.to_string()
            } else {
                format!("{}...", &text[..max_length])
            }
        }
    }
}