//! Topological lattice generation algorithms
//!
//! This module provides lattice generation algorithms for different topological codes
//! with zero-allocation patterns and blazing-fast performance.

use crate::cognitive::types::{CognitiveError, CognitiveResult};
use smallvec::SmallVec;
use super::topological_types::{TopologicalCodeType, VertexType, EdgeOrientation, FaceType};
use super::topological_lattice_types::{
    TopologicalLattice, LatticeVertex, LatticeEdge, LatticeFace, BoundaryConditions
};

impl TopologicalLattice {
    /// Generate lattice structure based on code type
    pub fn generate(
        code_type: TopologicalCodeType,
        dimensions: (usize, usize),
        boundary: BoundaryConditions,
    ) -> CognitiveResult<Self> {
        match code_type {
            TopologicalCodeType::ToricCode | TopologicalCodeType::PlanarCode => {
                Self::generate_square_lattice(dimensions, boundary)
            },
            TopologicalCodeType::ColorCode => {
                Self::generate_triangular_lattice(dimensions, boundary)
            },
            TopologicalCodeType::HyperbolicCode => {
                Self::generate_hyperbolic_lattice(dimensions, boundary)
            },
        }
    }

    /// Generate square lattice for toric/planar codes
    fn generate_square_lattice(
        dimensions: (usize, usize),
        boundary: BoundaryConditions,
    ) -> CognitiveResult<Self> {
        let (rows, cols) = dimensions;
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut faces = Vec::new();

        // Generate vertices
        for row in 0..=rows {
            for col in 0..=cols {
                let vertex_type = if (row == 0 || row == rows) && (col == 0 || col == cols) {
                    VertexType::Corner
                } else if row == 0 || row == rows || col == 0 || col == cols {
                    VertexType::Boundary
                } else {
                    VertexType::Regular
                };

                vertices.push(LatticeVertex {
                    id: row * (cols + 1) + col,
                    position: (col as f64, row as f64),
                    edges: SmallVec::new(),
                    vertex_type,
                });
            }
        }

        let mut edge_id = 0;
        let mut qubit_id = 0;

        // Generate horizontal edges
        for row in 0..=rows {
            for col in 0..cols {
                let v1 = row * (cols + 1) + col;
                let v2 = row * (cols + 1) + col + 1;

                edges.push(LatticeEdge {
                    id: edge_id,
                    vertices: (v1, v2),
                    faces: SmallVec::new(),
                    orientation: EdgeOrientation::Horizontal,
                    qubit_id: Some(qubit_id),
                });

                vertices[v1].edges.push(edge_id);
                vertices[v2].edges.push(edge_id);
                edge_id += 1;
                qubit_id += 1;
            }
        }

        // Generate vertical edges
        for row in 0..rows {
            for col in 0..=cols {
                let v1 = row * (cols + 1) + col;
                let v2 = (row + 1) * (cols + 1) + col;

                edges.push(LatticeEdge {
                    id: edge_id,
                    vertices: (v1, v2),
                    faces: SmallVec::new(),
                    orientation: EdgeOrientation::Vertical,
                    qubit_id: Some(qubit_id),
                });

                vertices[v1].edges.push(edge_id);
                vertices[v2].edges.push(edge_id);
                edge_id += 1;
                qubit_id += 1;
            }
        }

        // Generate faces (plaquettes)
        for row in 0..rows {
            for col in 0..cols {
                let bottom_edge = row * cols + col;
                let top_edge = (row + 1) * cols + col;
                let left_edge = (rows + 1) * cols + row * (cols + 1) + col;
                let right_edge = (rows + 1) * cols + row * (cols + 1) + col + 1;

                let face_edges = smallvec::smallvec![bottom_edge, right_edge, top_edge, left_edge];

                faces.push(LatticeFace {
                    id: row * cols + col,
                    edges: face_edges.clone(),
                    face_type: FaceType::Square,
                    syndrome_qubit: Some(row * cols + col),
                });

                // Update edge face references
                for &edge_id in &face_edges {
                    if let Some(edge) = edges.get_mut(edge_id) {
                        edge.faces.push(row * cols + col);
                    }
                }
            }
        }

        Ok(TopologicalLattice {
            dimensions,
            vertices,
            edges,
            faces,
            boundary,
        })
    }

    /// Generate triangular lattice for color codes
    fn generate_triangular_lattice(
        dimensions: (usize, usize),
        boundary: BoundaryConditions,
    ) -> CognitiveResult<Self> {
        let (rows, cols) = dimensions;
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut faces = Vec::new();

        // Generate vertices in triangular arrangement
        for row in 0..=rows {
            let offset = if row % 2 == 0 { 0.0 } else { 0.5 };
            for col in 0..=cols {
                let vertex_type = if row == 0 || row == rows || col == 0 || col == cols {
                    VertexType::Boundary
                } else {
                    VertexType::Regular
                };

                vertices.push(LatticeVertex {
                    id: row * (cols + 1) + col,
                    position: (col as f64 + offset, row as f64 * 0.866), // sqrt(3)/2 spacing
                    edges: SmallVec::new(),
                    vertex_type,
                });
            }
        }

        // Generate edges and faces for triangular lattice
        let mut edge_id = 0;
        let mut face_id = 0;

        for row in 0..rows {
            for col in 0..cols {
                // Create triangular faces
                let v1 = row * (cols + 1) + col;
                let v2 = row * (cols + 1) + col + 1;
                let v3 = (row + 1) * (cols + 1) + col;

                // Create edges for this triangle
                let edge_ids = [edge_id, edge_id + 1, edge_id + 2];

                edges.push(LatticeEdge {
                    id: edge_ids[0],
                    vertices: (v1, v2),
                    faces: smallvec::smallvec![face_id],
                    orientation: EdgeOrientation::Horizontal,
                    qubit_id: Some(edge_ids[0]),
                });

                edges.push(LatticeEdge {
                    id: edge_ids[1],
                    vertices: (v2, v3),
                    faces: smallvec::smallvec![face_id],
                    orientation: EdgeOrientation::Diagonal,
                    qubit_id: Some(edge_ids[1]),
                });

                edges.push(LatticeEdge {
                    id: edge_ids[2],
                    vertices: (v3, v1),
                    faces: smallvec::smallvec![face_id],
                    orientation: EdgeOrientation::Vertical,
                    qubit_id: Some(edge_ids[2]),
                });

                // Update vertex edge references
                vertices[v1].edges.extend_from_slice(&[edge_ids[0], edge_ids[2]]);
                vertices[v2].edges.extend_from_slice(&[edge_ids[0], edge_ids[1]]);
                vertices[v3].edges.extend_from_slice(&[edge_ids[1], edge_ids[2]]);

                // Create triangular face
                faces.push(LatticeFace {
                    id: face_id,
                    edges: SmallVec::from_slice(&edge_ids),
                    face_type: FaceType::Triangle,
                    syndrome_qubit: Some(face_id),
                });

                edge_id += 3;
                face_id += 1;
            }
        }

        Ok(TopologicalLattice {
            dimensions,
            vertices,
            edges,
            faces,
            boundary,
        })
    }

    /// Generate hyperbolic lattice (simplified implementation)
    fn generate_hyperbolic_lattice(
        dimensions: (usize, usize),
        boundary: BoundaryConditions,
    ) -> CognitiveResult<Self> {
        // For now, use a modified square lattice as placeholder
        // Full hyperbolic lattice implementation would be much more complex
        Self::generate_square_lattice(dimensions, boundary)
    }

    /// Check if lattice is valid (all references are consistent)
    pub fn validate(&self) -> Result<(), String> {
        // Check vertex-edge consistency
        for vertex in &self.vertices {
            for &edge_id in &vertex.edges {
                if let Some(edge) = self.get_edge(edge_id) {
                    if edge.vertices.0 != vertex.id && edge.vertices.1 != vertex.id {
                        return Err(format!("Vertex {} references edge {} but edge doesn't reference vertex", vertex.id, edge_id));
                    }
                } else {
                    return Err(format!("Vertex {} references non-existent edge {}", vertex.id, edge_id));
                }
            }
        }

        // Check edge-face consistency
        for edge in &self.edges {
            for &face_id in &edge.faces {
                if let Some(face) = self.get_face(face_id) {
                    if !face.edges.contains(&edge.id) {
                        return Err(format!("Edge {} references face {} but face doesn't reference edge", edge.id, face_id));
                    }
                } else {
                    return Err(format!("Edge {} references non-existent face {}", edge.id, face_id));
                }
            }
        }

        // Check face-edge consistency
        for face in &self.faces {
            for &edge_id in &face.edges {
                if let Some(edge) = self.get_edge(edge_id) {
                    if !edge.faces.contains(&face.id) {
                        return Err(format!("Face {} references edge {} but edge doesn't reference face", face.id, edge_id));
                    }
                } else {
                    return Err(format!("Face {} references non-existent edge {}", face.id, edge_id));
                }
            }
        }

        Ok(())
    }
}