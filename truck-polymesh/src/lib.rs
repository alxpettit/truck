//! Defines polyline-polygon data structure and some algorithms handling mesh.

#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use serde::{Deserialize, Serialize};

/// re-export `truck_base`.
pub mod base {
    pub use truck_base::{bounding_box::*, cgmath64::*, geom_traits::*, tolerance::*};
}
pub use base::*;

/// Index vertex of a face of the polygon mesh
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Vertex {
    /// index of vertex's position
    pub pos: usize,
    /// index of vertex's texture coordinate
    pub uv: Option<usize>,
    /// index of vertex's normal
    pub nor: Option<usize>,
}

/// Faces of polygon mesh
///
/// To optimize for the case where the polygon mesh consists only triangles and quadrangle,
/// there are vectors which consist by each triangles and quadrilaterals, internally.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Faces {
    tri_faces: Vec<[Vertex; 3]>,
    quad_faces: Vec<[Vertex; 4]>,
    other_faces: Vec<Vec<Vertex>>,
}

/// Polygon mesh
/// 
/// The polygon data is held in a method compliant with wavefront obj.
/// Position, uv (texture) coordinates, and normal vectors are held in separate arrays,
/// and each face vertex accesses those values by an indices triple.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolygonMesh {
    positions: Vec<Point3>,
    uv_coords: Vec<Vector2>,
    normals: Vec<Vector3>,
    faces: Faces,
}

/// structured quadrangle mesh
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructuredMesh {
    positions: Vec<Vec<Point3>>,
    uv_division: Option<(Vec<f64>, Vec<f64>)>,
    normals: Option<Vec<Vec<Vector3>>>,
}

/// Error handler for [`Error`](./errors/enum.Error.html)
pub type Result<T> = std::result::Result<T, errors::Error>;

/// Re-exports root structs and all mesh filter traits.
pub mod prelude;
/// Defines errors
pub mod errors;
mod eliminate_waste;
mod meshing_shape;
/// I/O of wavefront obj
pub mod obj;
/// Defines [`PolygonMeshEditor`](./polygon_mesh/struct.PolygonMeshEditor.html).
pub mod polygon_mesh;
mod normal_filters;
mod splitting;
mod structured_mesh;
mod structuring;

#[inline(always)]
fn get_tri<T: Clone>(face: &[T], idx0: usize, idx1: usize, idx2: usize) -> [T; 3] {
    [face[idx0].clone(), face[idx1].clone(), face[idx2].clone()]
}

trait CosAngle {
    fn cos_angle(&self, other: &Self) -> f64;
}

impl CosAngle for Vector3 {
    fn cos_angle(&self, other: &Self) -> f64 {
        self.dot(*other) / (self.magnitude() * other.magnitude())
    }
}
