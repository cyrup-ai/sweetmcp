//! Topological error correction types and data structures
//!
//! This module provides the core types and data structures for topological quantum error correction
//! with zero-allocation patterns and blazing-fast performance.

use crate::cognitive::quantum::{
    Complex64,
    types::{CognitiveError, CognitiveResult},
};
use std::collections::{HashMap, HashSet};
use smallvec::SmallVec;

/// Types of topological codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TopologicalCodeType {
    /// Toric code (periodic boundaries)
    ToricCode,
    
    /// Planar code (open boundaries)
    PlanarCode,
    
    /// Color code (triangular/hexagonal lattice)
    ColorCode,
    
    /// Hyperbolic code
    HyperbolicCode,
}

impl TopologicalCodeType {
    /// Get the typical error threshold for this code type
    pub fn error_threshold(&self) -> f64 {
        match self {
            TopologicalCodeType::ToricCode => 0.1075,
            TopologicalCodeType::PlanarCode => 0.1031,
            TopologicalCodeType::ColorCode => 0.109,
            TopologicalCodeType::HyperbolicCode => 0.12,
        }
    }

    /// Get the optimal decoder for this code type
    pub fn optimal_decoder(&self) -> TopologicalDecoderType {
        match self {
            TopologicalCodeType::ToricCode => TopologicalDecoderType::MWPM,
            TopologicalCodeType::PlanarCode => TopologicalDecoderType::MWPM,
            TopologicalCodeType::ColorCode => TopologicalDecoderType::UnionFind,
            TopologicalCodeType::HyperbolicCode => TopologicalDecoderType::BeliefPropagation,
        }
    }

    /// Check if code supports periodic boundaries
    pub fn supports_periodic_boundaries(&self) -> bool {
        matches!(self, TopologicalCodeType::ToricCode | TopologicalCodeType::HyperbolicCode)
    }

    /// Get typical lattice dimensions for given distance
    pub fn lattice_dimensions(&self, distance: usize) -> (usize, usize) {
        match self {
            TopologicalCodeType::ToricCode | TopologicalCodeType::PlanarCode => (distance, distance),
            TopologicalCodeType::ColorCode => (distance * 2, distance * 2),
            TopologicalCodeType::HyperbolicCode => (distance * 3, distance * 3),
        }
    }
}

/// Vertex types in topological codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VertexType {
    /// Regular vertex
    Regular,
    
    /// Boundary vertex
    Boundary,
    
    /// Corner vertex
    Corner,
}

impl VertexType {
    /// Get the typical degree (number of edges) for this vertex type
    pub fn typical_degree(&self) -> usize {
        match self {
            VertexType::Regular => 4,
            VertexType::Boundary => 2,
            VertexType::Corner => 2,
        }
    }

    /// Check if vertex is on boundary
    pub fn is_boundary(&self) -> bool {
        matches!(self, VertexType::Boundary | VertexType::Corner)
    }
}

/// Edge orientation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeOrientation {
    Horizontal,
    Vertical,
    Diagonal,
}

impl EdgeOrientation {
    /// Get the perpendicular orientation
    pub fn perpendicular(&self) -> Self {
        match self {
            EdgeOrientation::Horizontal => EdgeOrientation::Vertical,
            EdgeOrientation::Vertical => EdgeOrientation::Horizontal,
            EdgeOrientation::Diagonal => EdgeOrientation::Diagonal, // Self-perpendicular
        }
    }

    /// Get unit vector for this orientation
    pub fn unit_vector(&self) -> (f64, f64) {
        match self {
            EdgeOrientation::Horizontal => (1.0, 0.0),
            EdgeOrientation::Vertical => (0.0, 1.0),
            EdgeOrientation::Diagonal => (0.7071067811865476, 0.7071067811865476), // 1/sqrt(2)
        }
    }
}

/// Face types for different codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceType {
    /// Square plaquette (toric/planar code)
    Square,
    
    /// Triangular face (color code)
    Triangle,
    
    /// Hexagonal face (color code)
    Hexagon,
}

impl FaceType {
    /// Get the number of edges for this face type
    pub fn edge_count(&self) -> usize {
        match self {
            FaceType::Square => 4,
            FaceType::Triangle => 3,
            FaceType::Hexagon => 6,
        }
    }

    /// Get the interior angle for regular face
    pub fn interior_angle(&self) -> f64 {
        match self {
            FaceType::Square => std::f64::consts::PI / 2.0,      // 90 degrees
            FaceType::Triangle => std::f64::consts::PI / 3.0,    // 60 degrees
            FaceType::Hexagon => 2.0 * std::f64::consts::PI / 3.0, // 120 degrees
        }
    }
}

/// Types of boundaries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryType {
    /// Open boundaries
    Open,
    
    /// Periodic boundaries (torus)
    Periodic,
    
    /// Twisted boundaries
    Twisted,
    
    /// Mixed boundaries
    Mixed,
}

impl BoundaryType {
    /// Check if boundary type preserves topology
    pub fn preserves_topology(&self) -> bool {
        matches!(self, BoundaryType::Periodic | BoundaryType::Twisted)
    }

    /// Get the genus of the resulting surface
    pub fn surface_genus(&self) -> usize {
        match self {
            BoundaryType::Open => 0,          // Disk (genus 0)
            BoundaryType::Periodic => 1,      // Torus (genus 1)
            BoundaryType::Twisted => 1,       // Klein bottle (genus 1)
            BoundaryType::Mixed => 0,         // Varies
        }
    }
}

/// Lattice element reference
#[derive(Debug, Clone, Copy)]
pub enum LatticeElement {
    Vertex(usize),
    Face(usize),
    Edge(usize),
}

impl LatticeElement {
    /// Get the element ID
    pub fn id(&self) -> usize {
        match self {
            LatticeElement::Vertex(id) => *id,
            LatticeElement::Face(id) => *id,
            LatticeElement::Edge(id) => *id,
        }
    }

    /// Check if element is a vertex
    pub fn is_vertex(&self) -> bool {
        matches!(self, LatticeElement::Vertex(_))
    }

    /// Check if element is a face
    pub fn is_face(&self) -> bool {
        matches!(self, LatticeElement::Face(_))
    }

    /// Check if element is an edge
    pub fn is_edge(&self) -> bool {
        matches!(self, LatticeElement::Edge(_))
    }
}

/// Decoder types for topological codes
#[derive(Debug, Clone, Copy)]
pub enum TopologicalDecoderType {
    /// Minimum weight perfect matching
    MWPM,
    
    /// Union-Find decoder
    UnionFind,
    
    /// Belief propagation
    BeliefPropagation,
    
    /// Machine learning decoder
    MLDecoder,
}

impl TopologicalDecoderType {
    /// Get the typical time complexity for this decoder
    pub fn time_complexity_description(&self) -> &'static str {
        match self {
            TopologicalDecoderType::MWPM => "O(n^3) for n syndrome bits",
            TopologicalDecoderType::UnionFind => "O(n) for n syndrome bits",
            TopologicalDecoderType::BeliefPropagation => "O(n * iterations) for n syndrome bits",
            TopologicalDecoderType::MLDecoder => "O(n) for n syndrome bits (after training)",
        }
    }

    /// Check if decoder is optimal for threshold performance
    pub fn is_threshold_optimal(&self) -> bool {
        matches!(self, TopologicalDecoderType::MWPM | TopologicalDecoderType::MLDecoder)
    }

    /// Check if decoder is fast (sub-cubic complexity)
    pub fn is_fast(&self) -> bool {
        matches!(self, TopologicalDecoderType::UnionFind | TopologicalDecoderType::MLDecoder)
    }
}