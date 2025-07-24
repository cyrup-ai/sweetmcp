//! Semantic memory multi-field search operations
//!
//! This module provides multi-field search capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;

impl SemanticMemory {
    /// Multi-field search across content, category, and tags
    pub fn multi_field_search(&self, query: &str) -> Vec<(&SemanticItem, SearchMatch)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for item in &self.items {
            let mut matches = Vec::new();
            let mut total_score = 0.0;
            
            // Search in content
            if item.content.to_lowercase().contains(&query_lower) {
                let match_count = item.content.to_lowercase().matches(&query_lower).count();
                let score = match_count as f64 * 1.0; // Base score for content matches
                matches.push(FieldMatch {
                    field: SearchField::Content,
                    score,
                    snippet: self.extract_snippet(&item.content, query, 100),
                });
                total_score += score;
            }
            
            // Search in category
            if item.category.to_lowercase().contains(&query_lower) {
                let score = 2.0; // Higher score for category matches
                matches.push(FieldMatch {
                    field: SearchField::Category,
                    score,
                    snippet: item.category.clone(),
                });
                total_score += score;
            }
            
            // Search in tags
            for tag in &item.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    let score = 1.5; // Medium score for tag matches
                    matches.push(FieldMatch {
                        field: SearchField::Tag,
                        score,
                        snippet: tag.clone(),
                    });
                    total_score += score;
                }
            }
            
            // Search in metadata keys and values
            for (key, value) in &item.metadata {
                let key_lower = key.to_lowercase();
                let value_str = value.to_string().to_lowercase();
                
                if key_lower.contains(&query_lower) {
                    let score = 1.2; // Score for metadata key matches
                    matches.push(FieldMatch {
                        field: SearchField::MetadataKey,
                        score,
                        snippet: key.clone(),
                    });
                    total_score += score;
                }
                
                if value_str.contains(&query_lower) {
                    let score = 1.0; // Score for metadata value matches
                    matches.push(FieldMatch {
                        field: SearchField::MetadataValue,
                        score,
                        snippet: value_str,
                    });
                    total_score += score;
                }
            }
            
            if !matches.is_empty() {
                results.push((item, SearchMatch {
                    total_score,
                    field_matches: matches,
                }));
            }
        }
        
        // Sort by total score (highest first)
        results.sort_by(|a, b| b.1.total_score.partial_cmp(&a.1.total_score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Advanced multi-field search with field-specific weights
    pub fn weighted_multi_field_search(&self, query: &str, weights: &SearchWeights) -> Vec<(&SemanticItem, SearchMatch)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for item in &self.items {
            let mut matches = Vec::new();
            let mut total_score = 0.0;
            
            // Search in content
            if item.content.to_lowercase().contains(&query_lower) {
                let match_count = item.content.to_lowercase().matches(&query_lower).count();
                let base_score = match_count as f64;
                let weighted_score = base_score * weights.content_weight;
                matches.push(FieldMatch {
                    field: SearchField::Content,
                    score: weighted_score,
                    snippet: self.extract_snippet(&item.content, query, 100),
                });
                total_score += weighted_score;
            }
            
            // Search in category
            if item.category.to_lowercase().contains(&query_lower) {
                let weighted_score = weights.category_weight;
                matches.push(FieldMatch {
                    field: SearchField::Category,
                    score: weighted_score,
                    snippet: item.category.clone(),
                });
                total_score += weighted_score;
            }
            
            // Search in tags
            for tag in &item.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    let weighted_score = weights.tag_weight;
                    matches.push(FieldMatch {
                        field: SearchField::Tag,
                        score: weighted_score,
                        snippet: tag.clone(),
                    });
                    total_score += weighted_score;
                }
            }
            
            // Search in metadata
            for (key, value) in &item.metadata {
                let key_lower = key.to_lowercase();
                let value_str = value.to_string().to_lowercase();
                
                if key_lower.contains(&query_lower) {
                    let weighted_score = weights.metadata_key_weight;
                    matches.push(FieldMatch {
                        field: SearchField::MetadataKey,
                        score: weighted_score,
                        snippet: key.clone(),
                    });
                    total_score += weighted_score;
                }
                
                if value_str.contains(&query_lower) {
                    let weighted_score = weights.metadata_value_weight;
                    matches.push(FieldMatch {
                        field: SearchField::MetadataValue,
                        score: weighted_score,
                        snippet: value_str,
                    });
                    total_score += weighted_score;
                }
            }
            
            if !matches.is_empty() {
                results.push((item, SearchMatch {
                    total_score,
                    field_matches: matches,
                }));
            }
        }
        
        // Sort by total score (highest first)
        results.sort_by(|a, b| b.1.total_score.partial_cmp(&a.1.total_score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Search with faceted results
    pub fn faceted_search(&self, query: &str) -> FacetedSearchResult {
        let multi_field_results = self.multi_field_search(query);
        
        let mut content_matches = Vec::new();
        let mut category_matches = Vec::new();
        let mut tag_matches = Vec::new();
        let mut metadata_matches = Vec::new();
        
        for (item, search_match) in multi_field_results {
            for field_match in search_match.field_matches {
                match field_match.field {
                    SearchField::Content => content_matches.push((item, field_match.score)),
                    SearchField::Category => category_matches.push((item, field_match.score)),
                    SearchField::Tag => tag_matches.push((item, field_match.score)),
                    SearchField::MetadataKey | SearchField::MetadataValue => {
                        metadata_matches.push((item, field_match.score));
                    }
                }
            }
        }
        
        let content_score: f64 = content_matches.iter().map(|(_, score)| score).sum();
        let category_score: f64 = category_matches.iter().map(|(_, score)| score).sum();
        let tag_score: f64 = tag_matches.iter().map(|(_, score)| score).sum();
        let metadata_score: f64 = metadata_matches.iter().map(|(_, score)| score).sum();
        
        FacetedSearchResult {
            content_facet: SearchFacet {
                field_type: SearchField::Content,
                matches: content_matches,
                total_score: content_score,
            },
            category_facet: SearchFacet {
                field_type: SearchField::Category,
                matches: category_matches,
                total_score: category_score,
            },
            tag_facet: SearchFacet {
                field_type: SearchField::Tag,
                matches: tag_matches,
                total_score: tag_score,
            },
            metadata_facet: SearchFacet {
                field_type: SearchField::MetadataKey, // Represents both key and value
                matches: metadata_matches,
                total_score: metadata_score,
            },
        }
    }
}

/// Search field types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchField {
    Content,
    Category,
    Tag,
    MetadataKey,
    MetadataValue,
}

/// Field match information
#[derive(Debug, Clone)]
pub struct FieldMatch {
    pub field: SearchField,
    pub score: f64,
    pub snippet: String,
}

/// Search match result
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub total_score: f64,
    pub field_matches: Vec<FieldMatch>,
}

/// Search weights for different fields
#[derive(Debug, Clone)]
pub struct SearchWeights {
    pub content_weight: f64,
    pub category_weight: f64,
    pub tag_weight: f64,
    pub metadata_key_weight: f64,
    pub metadata_value_weight: f64,
}

impl Default for SearchWeights {
    fn default() -> Self {
        Self {
            content_weight: 1.0,
            category_weight: 2.0,
            tag_weight: 1.5,
            metadata_key_weight: 1.2,
            metadata_value_weight: 1.0,
        }
    }
}

/// Search facet for a specific field type
#[derive(Debug, Clone)]
pub struct SearchFacet<'a> {
    pub field_type: SearchField,
    pub matches: Vec<(&'a SemanticItem, f64)>,
    pub total_score: f64,
}

/// Faceted search result
#[derive(Debug, Clone)]
pub struct FacetedSearchResult<'a> {
    pub content_facet: SearchFacet<'a>,
    pub category_facet: SearchFacet<'a>,
    pub tag_facet: SearchFacet<'a>,
    pub metadata_facet: SearchFacet<'a>,
}