//! Topological lattice type definitions
//!
//! This module provides core data structures for topological lattices
//! with zero-allocation patterns and blazing-fast performance.

use smallvec::SmallVec;
use super::topological_types::{VertexType, EdgeOrientation, FaceType, BoundaryType};

/// Topological lattice structure
#[derive(Debug, Clone)]
pub struct TopologicalLattice {
    /// Lattice dimensions
    pub dimensions: (usize, usize),
    
    /// Vertex positions
    pub vertices: Vec<LatticeVertex>,
    
    /// Edge connections
    pub edges: Vec<LatticeEdge>,
    
    /// Face (plaquette) definitions
    pub faces: Vec<LatticeFace>,
    
    /// Boundary conditions
    pub boundary: BoundaryConditions,
}

/// Vertex in topological lattice
#[derive(Debug, Clone)]
pub struct LatticeVertex {
    /// Vertex ID
    pub id: usize,
    
    /// Position coordinates
    pub position: (f64, f64),
    
    /// Connected edges
    pub edges: SmallVec<[usize; 4]>,
    
    /// Vertex type
    pub vertex_type: VertexType,
}

/// Edge in topological lattice
#[derive(Debug, Clone)]
pub struct LatticeEdge {
    /// Edge ID
    pub id: usize,
    
    /// Connected vertices
    pub vertices: (usize, usize),
    
    /// Adjacent faces
    pub faces: SmallVec<[usize; 2]>,
    
    /// Edge orientation
    pub orientation: EdgeOrientation,
    
    /// Physical qubit on this edge
    pub qubit_id: Option<usize>,
}

/// Face (plaquette) in topological lattice
#[derive(Debug, Clone)]
pub struct LatticeFace {
    /// Face ID
    pub id: usize,
    
    /// Boundary edges (ordered)
    pub edges: SmallVec<[usize; 6]>,
    
    /// Face type
    pub face_type: FaceType,
    
    /// Syndrome qubit for this face
    pub syndrome_qubit: Option<usize>,
}

/// Boundary conditions for lattice
#[derive(Debug, Clone)]
pub struct BoundaryConditions {
    /// Periodic in x-direction
    pub periodic_x: bool,
    
    /// Periodic in y-direction
    pub periodic_y: bool,
    
    /// Boundary type
    pub boundary_type: BoundaryType,
}

/// Lattice statistics
#[derive(Debug, Clone)]
pub struct LatticeStatistics {
    pub num_vertices: usize,
    pub num_edges: usize,
    pub num_faces: usize,
    pub num_qubits: usize,
    pub average_vertex_degree: f64,
    pub boundary_vertices: usize,
}

impl BoundaryConditions {
    /// Create open boundary conditions
    pub fn open() -> Self {
        Self {
            periodic_x: false,
            periodic_y: false,
            boundary_type: BoundaryType::Open,
        }
    }

    /// Create periodic boundary conditions (torus)
    pub fn periodic() -> Self {
        Self {
            periodic_x: true,
            periodic_y: true,
            boundary_type: BoundaryType::Periodic,
        }
    }

    /// Create mixed boundary conditions
    pub fn mixed(periodic_x: bool, periodic_y: bool) -> Self {
        let boundary_type = match (periodic_x, periodic_y) {
            (true, true) => BoundaryType::Periodic,
            (false, false) => BoundaryType::Open,
            _ => BoundaryType::Mixed,
        };

        Self {
            periodic_x,
            periodic_y,
            boundary_type,
        }
    }
}

impl TopologicalLattice {
    /// Get vertex by ID
    pub fn get_vertex(&self, id: usize) -> Option<&LatticeVertex> {
        self.vertices.get(id)
    }

    /// Get edge by ID
    pub fn get_edge(&self, id: usize) -> Option<&LatticeEdge> {
        self.edges.get(id)
    }

    /// Get face by ID
    pub fn get_face(&self, id: usize) -> Option<&LatticeFace> {
        self.faces.get(id)
    }

    /// Get all edges connected to a vertex
    pub fn vertex_edges(&self, vertex_id: usize) -> Vec<&LatticeEdge> {
        if let Some(vertex) = self.get_vertex(vertex_id) {
            vertex.edges.iter()
                .filter_map(|&edge_id| self.get_edge(edge_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all faces adjacent to an edge
    pub fn edge_faces(&self, edge_id: usize) -> Vec<&LatticeFace> {
        if let Some(edge) = self.get_edge(edge_id) {
            edge.faces.iter()
                .filter_map(|&face_id| self.get_face(face_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all edges of a face
    pub fn face_edges(&self, face_id: usize) -> Vec<&LatticeEdge> {
        if let Some(face) = self.get_face(face_id) {
            face.edges.iter()
                .filter_map(|&edge_id| self.get_edge(edge_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get lattice statistics
    pub fn statistics(&self) -> LatticeStatistics {
        LatticeStatistics {
            num_vertices: self.vertices.len(),
            num_edges: self.edges.len(),
            num_faces: self.faces.len(),
            num_qubits: self.edges.iter().filter_map(|e| e.qubit_id).count(),
            average_vertex_degree: self.vertices.iter()
                .map(|v| v.edges.len())
                .sum::<usize>() as f64 / self.vertices.len() as f64,
            boundary_vertices: self.vertices.iter()
                .filter(|v| v.vertex_type.is_boundary())
                .count(),
        }
    }
}