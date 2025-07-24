//! Vector collection metadata types extracted from vector repository

use serde::{Deserialize, Serialize};

use crate::vector::{DistanceMetric, VectorIndex};

/// Vector collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCollection {
    /// Collection ID
    pub id: String,

    /// Collection name
    pub name: String,

    /// Vector dimensions
    pub dimensions: usize,

    /// Distance metric
    pub metric: DistanceMetric,

    /// Number of vectors
    pub count: usize,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Handle to a vector collection - cache-line aligned for optimal performance
#[repr(align(64))] // Cache-line alignment for CPU cache efficiency
pub struct VectorCollectionHandle {
    // Hot field - frequently accessed during vector operations
    pub index: Box<dyn VectorIndex>,
    // Cold field - accessed less frequently during metadata operations
    pub metadata: VectorCollection,
}

impl VectorCollection {
    /// Create a new vector collection metadata
    pub fn new(
        id: String,
        name: String,
        dimensions: usize,
        metric: DistanceMetric,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            dimensions,
            metric,
            count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the vector count and timestamp
    pub fn update_count(&mut self, count: usize) {
        self.count = count;
        self.updated_at = chrono::Utc::now();
    }

    /// Update only the timestamp (for metadata changes)
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }
}

impl VectorCollectionHandle {
    /// Create a new vector collection handle
    pub fn new(index: Box<dyn VectorIndex>, metadata: VectorCollection) -> Self {
        Self { index, metadata }
    }

    /// Get a reference to the metadata
    pub fn metadata(&self) -> &VectorCollection {
        &self.metadata
    }

    /// Get a mutable reference to the metadata
    pub fn metadata_mut(&mut self) -> &mut VectorCollection {
        &mut self.metadata
    }

    /// Get a reference to the index
    pub fn index(&self) -> &dyn VectorIndex {
        self.index.as_ref()
    }

    /// Get a mutable reference to the index
    pub fn index_mut(&mut self) -> &mut dyn VectorIndex {
        self.index.as_mut()
    }
}