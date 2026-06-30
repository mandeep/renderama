use std::collections::HashMap;

use glam::{Vec2, Vec3A};
use tobj;

use crate::lights::{MeshLight, Light};
use crate::materials::{Diffuse, Emissive, Material, MaterialId, Plastic, Reflective, Refractive, TextureMap};
use crate::texture::{Color, ImageTexture, Texture};
use crate::triangle::{Triangle, TriangleMesh};

use crate::tex;

/// Options to be used for load_obj so that we can keep the API clean
pub struct LoadObjOptions {
    pub emissive_scale: f32,
    pub material_overrides: Option<HashMap<String, Material>>,
    pub default_material: Option<Material>,
}

impl LoadObjOptions {
    pub fn new() -> LoadObjOptions {
        LoadObjOptions::default()
    }

    pub fn with_emissive_scale(mut self, scale: f32) -> Self {
        self.emissive_scale = scale;
        self
    }

    pub fn with_overrides(mut self, material_overrides: Option<HashMap<String, Material>>) -> Self {
        self.material_overrides = material_overrides;
        self
    }

    pub fn with_material(mut self, default_material: Option<Material>) -> Self {
        self.default_material = default_material;
        self
    }
}

impl Default for LoadObjOptions {
    fn default() -> LoadObjOptions {
        LoadObjOptions { emissive_scale: 1.0, material_overrides: None, default_material: None}
    }
}

/// Load an obj file with default options.
pub fn load_obj(filepath: &str, materials: &mut Vec<Material>, textures: &mut Vec<Texture>) -> (Vec<TriangleMesh>, Vec<Light>) {
    load_obj_with_options(filepath, materials, textures, LoadObjOptions::default())
}

/// Load an obj file with its related mtl file.
///
/// Moved the following code from the TriangleMesh::from method.
pub fn load_obj_with_options(
    filepath: &str,
    materials: &mut Vec<Material>,
    textures: &mut Vec<Texture>,
    options: LoadObjOptions,
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

    let default_material_id = MaterialId::new(materials.len() as u32);
    let default_material = options.default_material.unwrap_or({
        let texture_id = tex!(textures, Color::new(0.8, 0.8, 0.8));
        Diffuse::new(texture_id, 1.0).into()
    });
    materials.push(default_material);

    if let Ok(obj_materials) = obj_material_list {
        for material in obj_materials {
            let new_material = match &options.material_overrides {
                Some(overrides) => {
                    if let Some(override_material) = overrides.get(&material.name) {
                        override_material.clone()
                    } else{
                        map_mtl_to_material(&material, textures, base_directory)
                    }
                },
                None => map_mtl_to_material(&material, textures, base_directory),
            };

            let mat_id = MaterialId::new(materials.len() as u32);
            materials.push(new_material);
            material_map.push(mat_id);
        }
    }

    let mut meshes: Vec<TriangleMesh> = Vec::new();
    let mut lights: Vec<Light> = Vec::new();

    for model in models {
        let mesh = &model.mesh;

        let current_material_id = match mesh.material_id {
            Some(id) if id < material_map.len() => material_map[id],
            _ => default_material_id,
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
        let mut light_triangles: Vec<Triangle> = Vec::new();

        for i in 0..mesh.indices.len() / 3 {
            let (a, b, c) = (mesh.indices[3 * i] as usize,
                                mesh.indices[3 * i + 1] as usize,
                                mesh.indices[3 * i + 2] as usize);
            let (v0, v1, v2) = (positions[a], positions[b], positions[c]);
            let (n0, n1, n2) = (normals[a], normals[b], normals[c]);
            let (uv0, uv1, uv2) = (uvs[a], uvs[b], uvs[c]);

            let triangle = Triangle::new(v0, v1, v2, n0, n1, n2, uv0, uv1, uv2, current_material_id);

            if matches!(&materials[current_material_id.index()], Material::Emissive(_)) {
                light_triangles.push(triangle);
            }

            triangles.push(triangle);
        }

        if let Material::Emissive(material) = &materials[current_material_id.index()] {
            if !light_triangles.is_empty() {
                let emissive_scale = options.emissive_scale;
                let intensity = textures[material.emissive_color.index()].sample_texture(0.5, 0.5) * emissive_scale;
                let mesh_light = MeshLight::new(light_triangles);
                lights.push(Light::new(mesh_light, intensity));
            }
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
/// https://docs.omniverse.nvidia.com/usd/latest/technical_reference/conceptual_data_mapping/obj-usd-concept-mapping.html
fn map_mtl_to_material(material: &tobj::Material, textures: &mut Vec<Texture>, base_directory: &std::path::Path) -> Material {
    let kd = material.diffuse.unwrap_or([0.8, 0.8, 0.8]);
    let _ks = material.specular.unwrap_or([0.0, 0.0, 0.0]);
    let ke = material.emissive.unwrap_or([0.0, 0.0, 0.0]);
    let ns = material.shininess.unwrap_or(0.0);
    let ni = material.optical_density.unwrap_or(1.5);
    let d  = material.dissolve.unwrap_or(1.0);
    let illum = material.illumination_model.unwrap_or(2);
    let map_ke = material.unknown_param.get("map_Ke");
    let map_bump = &material.normal_texture;
    let map_ns = &material.shininess_texture;
    let map_pm = material.unknown_param.get("map_Pm");

    // emissive material may have any illum # so we need to handle it first
    if ke.iter().sum::<f32>() > f32::EPSILON || map_ke.is_some() {
        let emissive_color: Texture = if let Some(path) = map_ke {
            let full_path = base_directory.join(path);
            // emissive texture map should be converted from srgb to linear
            ImageTexture::srgb(full_path.to_str().unwrap(), Vec2::ONE).into()
        } else {
            Color::new(ke[0], ke[1], ke[2]).into()
        };

        let emissive_texture_id = tex!(textures, emissive_color);

        return Emissive::new(emissive_texture_id).into();
    }

    // mapping Ns to a lower roughness range by using powf(13.5)
    // 0	1.0
    // 10	0.873
    // 25	0.711
    // 50	0.500
    // 100	0.241
    // 150	0.112
    // 200	0.049
    // 300	0.008
    let roughness = (1.0 - ns / 1000.0).powf(13.5).clamp(0.025, 1.0);

    let albedo: Texture = if let Some(path) = &material.diffuse_texture {
        let full_path = base_directory.join(path);
        ImageTexture::srgb(full_path.to_str().unwrap(), Vec2::ONE).into()
    } else {
        Color::new(kd[0], kd[1], kd[2]).into()
    };

    let albedo_texture_id = tex!(textures, albedo);

    let mut texture_map = TextureMap::new(albedo_texture_id);

    if map_bump.is_some() {
        let normal_path = map_bump.as_ref().unwrap();
        let full_path = base_directory.join(normal_path);
        let normal_map_texture: Texture = ImageTexture::linear(full_path.to_str().unwrap(), Vec2::ONE).into();
        let normal_map_id = tex!(textures, normal_map_texture);
        texture_map = texture_map.with_normal(normal_map_id);
    }

    if map_ns.is_some() {
        let roughness_path = map_ns.as_ref().unwrap();
        let full_path = base_directory.join(roughness_path);
        let roughness_map_texture: Texture = ImageTexture::linear(full_path.to_str().unwrap(), Vec2::ONE).into();
        let roughness_map_id = tex!(textures, roughness_map_texture);
        texture_map = texture_map.with_roughness(roughness_map_id);
    }

    if map_pm.is_some() {
        let metallic_roughness_path = map_pm.as_ref().unwrap();
        let full_path = base_directory.join(metallic_roughness_path);
        let metallic_roughness_map_texture: Texture = ImageTexture::linear(full_path.to_str().unwrap(), Vec2::ONE).into();
        let metallic_roughness_map_id = tex!(textures, metallic_roughness_map_texture);
        texture_map = texture_map.with_metallic_roughness(metallic_roughness_map_id);
    }

    if illum <= 1 {
        return Diffuse::new(albedo_texture_id, roughness).into();
    }

    if illum == 3 {
        return Reflective::new(albedo_texture_id, roughness).into();
    }

    if matches!(illum, 4..=7) || d < 0.99 {
        return Refractive::new(albedo_texture_id, ni).into();
    }

    // illum 2 or if illum is not provided, default to Plastic
    Plastic::new(albedo_texture_id, roughness, ni).with_textures(texture_map).into()
}