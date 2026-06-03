use std::collections::HashMap;

use glam::{Vec2, Vec3A};
use tobj;

use materials::{Diffuse, Emissive, Material, MaterialId, Plastic, Reflective, Refractive};
use texture::{Color, ImageTexture};
use triangle::{Triangle, TriangleMesh};


/// Load an obj file with its related mtl file.
///
/// Moved the following code from the TriangleMesh::from method.
pub fn load_obj(
    filepath: &str,
    materials: &mut Vec<Material>,
    material_overrides: Option<HashMap<String, Material>>,
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
            let new_material = match &material_overrides {
                Some(overrides) => {
                    if let Some(override_material) = overrides.get(&material.name) {
                        override_material.clone()
                    } else{
                        map_mtl_to_material(&material)
                    }
                },
                None => map_mtl_to_material(&material),
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

/// Map the material properties in the MTL file to one of our materials.
///
/// References:
/// https://en.wikipedia.org/wiki/Wavefront_.obj_file
/// https://steamcommunity.com/sharedfiles/filedetails/?l=brazilian&id=2005695630
fn map_mtl_to_material(mat: &tobj::Material) -> Material {
    let kd = mat.diffuse.unwrap_or([0.8, 0.8, 0.8]);
    let ks = mat.specular.unwrap_or([0.0, 0.0, 0.0]);
    let ns = mat.shininess.unwrap_or(0.0);
    let ni = mat.optical_density.unwrap_or(1.5);
    let d  = mat.dissolve.unwrap_or(1.0);
    let illum = mat.illumination_model.unwrap_or(2);

    // mapping Ns to a lower roughness range by using powf(13.5)
    let roughness = (1.0 - ns / 1000.0).powf(13.5).clamp(0.025, 1.0);

    let ks_luminance = 0.2126 * ks[0] + 0.7152 * ks[1] + 0.0722 * ks[2];
    let ks_chroma = (ks[0] - ks_luminance).abs()
                  + (ks[1] - ks_luminance).abs()
                  + (ks[2] - ks_luminance).abs();
    let is_metallic = ks_chroma > 0.05 && ks_luminance > 0.1;

    let ke = mat.unknown_param.get("Ke")
        .or_else(|| mat.unknown_param.get("ke"))
        .and_then(|s| {
            let v: Vec<f32> = s.split_whitespace()
                .filter_map(|x| x.parse().ok())
                .collect();
            if v.len() == 3 { Some([v[0], v[1], v[2]]) } else { None }
        })
        .unwrap_or([0.0, 0.0, 0.0]);

    let ke_lum = ke[0] + ke[1] + ke[2];
    if ke_lum > 1e-4 {
        return Material::Emissive(Emissive::new(
            Color::new(ke[0], ke[1], ke[2]).into()
        ));
    }

    if illum == 6 || illum == 7 || d < 0.99 {
        let absorption = Color::new(kd[0], kd[1], kd[2]).into();
        return Material::Refractive(Refractive::new(absorption, ni));
    }

    if illum == 3 || is_metallic {
        let albedo = Color::new(ks[0], ks[1], ks[2]).into();
        return Material::Reflective(Reflective::new(albedo, roughness));
    }

    if illum <= 1 {
        return Material::Diffuse(Diffuse::new(
            Color::new(kd[0], kd[1], kd[2]).into(),
            roughness,
        ));
    }

    let albedo = if let Some(ref path) = mat.diffuse_texture {
        ImageTexture::new(path, Vec2::ONE).into()
    } else {
        Color::new(kd[0], kd[1], kd[2]).into()
    };

    Material::Plastic(Plastic::new(albedo, roughness, ni))
}