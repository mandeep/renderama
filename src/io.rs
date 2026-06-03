use std::collections::HashMap;

use glam::{Vec2, Vec3A};
use tobj;

use materials::{Material, MaterialId};
use triangle::{Triangle, TriangleMesh};


/// Load an obj file with its related mtl file.
///
/// Moved the following code from the TriangleMesh::from method.
pub fn load_obj(
    filepath: &str,
    materials: &mut Vec<Material>,
    material_overrides: &HashMap<String, Material>,
    default_material: Material,
) -> Vec<TriangleMesh> {
    let load_options = tobj::LoadOptions {
        single_index: true,
        triangulate: true,
        ignore_points: true,
        ignore_lines: true,
    };

    let (models, obj_material_list) =
        tobj::load_obj(filepath, &load_options).expect("Failed to load OBJ");

    let mut material_map: Vec<MaterialId> = Vec::new();

    let default_id = MaterialId::new(materials.len() as u32);
    materials.push(default_material.clone());

    if let Ok(obj_materials) = obj_material_list {
        for material in obj_materials {
            let new_material = if let Some(override_material) = material_overrides.get(&material.name) {
                override_material.clone()
            } else {
                default_material.clone()
            };

            let new_id = MaterialId::new(materials.len() as u32);
            materials.push(new_material);
            material_map.push(new_id);
        }
    }

    let mut meshes: Vec<TriangleMesh> = Vec::new();

    for model in models {
        let mesh = &model.mesh;

        let current_mat_id = match mesh.material_id {
            Some(id) if id < material_map.len() => material_map[id],
            _ => default_id, 
        };

        
        let positions: Vec<Vec3A> = mesh.positions
                                        .chunks(3)
                                        .map(|i| Vec3A::new(i[0], i[1], i[2]))
                                        .collect();

        let uvs: Vec<Vec2> = if !mesh.texcoords.is_empty() {
            mesh.texcoords.chunks(2).map(|c| Vec2::new(c[0], c[1])).collect()
        } else {
            vec![Vec2::ZERO; positions.len()]
        };

        let normals: Vec<Vec3A> = if !mesh.normals.is_empty() {
            mesh.normals.chunks(3).map(|i| Vec3A::new(i[0], i[1], i[2])).collect()
        } else {
            let mut computed = vec![Vec3A::ZERO; positions.len()];
            for i in 0..mesh.indices.len() / 3 {
                let (a, b, c) = (mesh.indices[3 * i] as usize,
                    mesh.indices[3 * i + 1] as usize,
                    mesh.indices[3 * i + 2] as usize);
                let edge1 = positions[b] - positions[a];
                let edge2 = positions[c] - positions[a];
                let face_normal = edge1.cross(edge2);
                computed[a] += face_normal;
                computed[b] += face_normal;
                computed[c] += face_normal;
            }
            for n in computed.iter_mut() {
                if n.length_squared() > 1e-20 {
                    *n = n.normalize();
                } else {
                    *n = Vec3A::new(0.0, 1.0, 0.0);
                }
            }
            computed
        };

        let mut triangles: Vec<Triangle> = Vec::with_capacity(mesh.indices.len() / 3);
        for i in 0..mesh.indices.len() / 3 {
            let (a, b, c) = (mesh.indices[3 * i] as usize,
                                mesh.indices[3 * i + 1] as usize,
                                mesh.indices[3 * i + 2] as usize);
            let (v0, v1, v2) = (positions[a], positions[b], positions[c]);
            let (n0, n1, n2) = (normals[a], normals[b], normals[c]);
            let (uv0, uv1, uv2) = (uvs[a], uvs[b], uvs[c]);

            let triangle = Triangle::new(v0, v1, v2, n0, n1, n2, uv0, uv1, uv2, current_mat_id);
            triangles.push(triangle);
        }
        meshes.push(TriangleMesh::new(triangles));
    }

    meshes
}