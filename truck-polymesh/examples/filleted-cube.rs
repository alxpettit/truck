use truck_polymesh::prelude::*;

fn main() {
    let file = std::fs::File::open("tests/data/filleted_cube.obj").unwrap();
    let mut mesh = obj::read(file).unwrap();
    mesh
        .put_together_same_attrs()
        .quadrangulate(0.1, 1.0)
        .add_smooth_normals(std::f64::consts::PI / 3.0, true);

    let (planes, others) = mesh.extract_planes(0.01);
    let file = std::fs::File::create("planes.obj").unwrap();
    obj::write(&mesh.create_mesh_by_face_indices(&planes), file).unwrap();

    let mesh = mesh.create_mesh_by_face_indices(&others);
    let (upper, lower) = mesh.clustering_faces_by_gcurvature(0.1, false);
    let file = std::fs::File::create("lower.obj").unwrap();
    obj::write(&mesh.create_mesh_by_face_indices(&lower), file).unwrap();
    let file = std::fs::File::create("upper.obj").unwrap();
    obj::write(&mesh.create_mesh_by_face_indices(&upper), file).unwrap();
}
