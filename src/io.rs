use std::collections::HashMap;

use glam::{Vec2, Vec3A};
use tobj;

use crate::lights::Light;
use crate::materials::{Diffuse, Emissive, Material, MaterialId, Plastic, Reflective, Refractive};
use crate::sphere::Sphere;
use crate::texture::{Color, ImageTexture};
use crate::triangle::{Triangle, TriangleMesh};


/// Load an obj file with its related mtl file.
///
/// Moved the following code from the TriangleMesh::from method.
pub fn load_obj(
    filepath: &str,
    materials: &mut Vec<Material>,
    material_overrides: Option<HashMap<String, Material>>,
    default_material: Material,
) -> (Vec<TriangleMesh>, Vec<Light>) {
    let base_directory = std::path::Path::new(filepath)
        .parent()
        .unwrap_or(std::path::Path::new("."));

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
                        map_mtl_to_material(&material, base_directory)
                    }
                },
                None => map_mtl_to_material(&material, base_directory),
            };

            let new_id = MaterialId::new(materials.len() as u32);
            materials.push(new_material);
            material_map.push(new_id);
        }
    }

    let mut meshes: Vec<TriangleMesh> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

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

        // currently adding a sphere light for any emissive material since it
        // seems like a decent fallback. will need to revisit if this is the best
        // approach later.
        if let Material::Emissive(emissive) = &materials[current_mat_id.index()] {
            let intensity = emissive.emissive_texture
                .sample_texture(0.0, 0.0, &Vec3A::ZERO);
            let centroid = positions.iter().fold(Vec3A::ZERO, |a, &b| a + b)
                           / positions.len() as f32;
            let radius = positions.iter()
                .map(|&p| (p - centroid).length())
                .fold(0.0_f32, f32::max);
            let sphere = Sphere::new(centroid, radius, current_mat_id);
            let light = Light::new(sphere.into(), intensity);
            lights.push(light);
        }

        meshes.push(TriangleMesh::new(triangles));
    }

    (meshes, lights)
}

/// Map the material properties in the MTL file to one of our materials.
///
/// References:
/// https://en.wikipedia.org/wiki/Wavefront_.obj_file
/// https://steamcommunity.com/sharedfiles/filedetails/?l=brazilian&id=2005695630
fn map_mtl_to_material(material: &tobj::Material, base_directory: &std::path::Path) -> Material {
    let kd = material.diffuse.unwrap_or([0.8, 0.8, 0.8]);
    let ks = material.specular.unwrap_or([0.0, 0.0, 0.0]);
    let ke = material.emissive.unwrap_or([0.0, 0.0, 0.0]);
    let ns = material.shininess.unwrap_or(0.0);
    let ni = material.optical_density.unwrap_or(1.5);
    let d  = material.dissolve.unwrap_or(1.0);
    let illum = material.illumination_model.unwrap_or(2);
    let map_ke = material.unknown_param.get("map_Ke");

    // emissive material may have any illum # so we need to handle it first
    if ke.iter().sum::<f32>() > f32::EPSILON || map_ke.is_some() {
        let albedo = if let Some(path) = map_ke {
            let full_path = base_directory.join(path);
            ImageTexture::new(full_path.to_str().unwrap(), Vec2::ONE).into()
        } else {
            Color::new(ke[0], ke[1], ke[2]).into()
        };

        return Emissive::new(albedo).into();
    }

    // mapping Ns to a lower roughness range by using powf(13.5)
    let roughness = (1.0 - ns / 1000.0).powf(13.5).clamp(0.025, 1.0);

    let albedo = if let Some(path) = &material.diffuse_texture {
        let full_path = base_directory.join(path);
        ImageTexture::new(full_path.to_str().unwrap(), Vec2::ONE).into()
    } else {
        Color::new(kd[0], kd[1], kd[2]).into()
    };

    if illum <= 1 {
        return Diffuse::new(albedo, roughness).into();
    }

    if illum == 3 {
        let albedo = Color::new(ks[0], ks[1], ks[2]).into();
        return Reflective::new(albedo, roughness).into();
    }

    if matches!(illum, 4..=7) || d < 0.99 {
        let absorption = Color::new(kd[0], kd[1], kd[2]).into();
        return Refractive::new(absorption, ni).into();
    }

    // illum 2 or if illum is not provided, default to Plastic
    Plastic::new(albedo, roughness, ni).into()
}