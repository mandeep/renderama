use std::f32;
use std::sync::Arc;

use glam::{Vec2, Vec3};
use tobj;

use aabb::AABB;
use bvh::BVH;
use hitable::{HitRecord, Hitable};
use ray::Ray;

#[derive(Clone)]
pub struct Triangle {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    n0: Vec3,
    n1: Vec3,
    n2: Vec3,
    uv0: Vec2,
    uv1: Vec2,
    uv2: Vec2,
    material_id: u32
}

pub struct TriangleMesh {
    triangles: Vec<Triangle>,
    accelerator: BVH,
    material_id: u32,
}

impl Triangle {
    /// Create a new triangle with vertices v0, v1, and v2
    pub fn new(v0: Vec3,
                                      v1: Vec3,
                                      v2: Vec3,
                                      n0: Vec3,
                                      n1: Vec3,
                                      n2: Vec3,
                                      uv0: Vec2,
                                      uv1: Vec2,
                                      uv2: Vec2,
                                      material_id: u32)
                                      -> Triangle {

        Triangle { v0: v0,
                   v1: v1,
                   v2: v2,
                   n0: n0,
                   n1: n1,
                   n2: n2,
                   uv0: uv0,
                   uv1: uv1,
                   uv2: uv2,
                   material_id: material_id }
    }

    pub fn from_box(v0: Vec3,
                    v1: Vec3,
                    v2: Vec3,
                    n0: Vec3,
                    n1: Vec3,
                    n2: Vec3,
                    uv0: Vec2,
                    uv1: Vec2,
                    uv2: Vec2,
                    material_id: u32)
                    -> Triangle {
        Triangle { v0: v0,
                   v1: v1,
                   v2: v2,
                   n0: n0,
                   n1: n1,
                   n2: n2,
                   uv0: uv0,
                   uv1: uv1,
                   uv2: uv2,
                   material_id: material_id
                   }
    }

    pub fn minimum(&self) -> Vec3 {
        self.v0.min(self.v1.min(self.v2))
    }

    pub fn maximum(&self) -> Vec3 {
        self.v0.max(self.v1.max(self.v2))
    }
}

impl Hitable for Triangle {
    /// Determine whether or not a ray hits the triangle
    ///
    /// Reference:
    /// Tomas Moller, Ben Trumbore
    /// Fast, Minimum Storage Ray/Triangle Intersection
    /// Journal of Graphics Tools Vol. 2 Issue 1, 1997
    /// http://www.acm.org/jgt/papers/MollerTrumbore97/
    ///
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;

        let pvec = ray.direction.cross(edge2);
        let determinant = edge1.dot(pvec);

        // Reject rays that are (nearly) parallel to the triangle. We do NOT
        // backface-cull here because a path tracer needs to hit both sides
        // (e.g. refraction exiting a mesh).
        if determinant.abs() < 1e-8 {
            return None;
        }

        let inverse_determinant = 1.0 / determinant;

        let tvec = ray.origin - self.v0;
        let u = tvec.dot(pvec) * inverse_determinant;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(edge1);
        let v = ray.direction.dot(qvec) * inverse_determinant;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = edge2.dot(qvec) * inverse_determinant;
        if t < position_min || t > position_max {
            return None;
        }

        // Möller-Trumbore barycentrics: (1-u-v) weights v0, u weights v1, v weights v2.
        // Use point_at_parameter to get the hit position directly from the ray.
        let point = ray.point_at_parameter(t);
        let geometric_normal = edge1.cross(edge2).normalize();
        let shading_normal = ((1.0 - u - v) * self.n0 + u * self.n1 + v * self.n2).normalize();

        // Interpolate texture coordinates with the same barycentric weights.
        let w = 1.0 - u - v;
        let interpolated_uv = w * self.uv0 + u * self.uv1 + v * self.uv2;

        Some(HitRecord::new(t,
                            interpolated_uv.x,
                            interpolated_uv.y,
                            point,
                            geometric_normal,
                            shading_normal,
                            self.material_id))
    }

    /// Create a bounding box around the triangle
    ///
    /// The bounding box is created using the minimum
    /// and maximum points of all of the vertices
    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(AABB::from(self.minimum(), self.maximum()))
    }
}

impl TriangleMesh {
    pub fn new(triangles: Vec<Triangle>, material_id: u32) -> TriangleMesh {
        let mut hitables: Vec<Arc<dyn Hitable>> = triangles.iter()
                                                           .map(|t| {
                                                               Arc::new(t.clone())
                                                               as Arc<dyn Hitable>
                                                           })
                                                           .collect();

        let accelerator = BVH::new(&mut hitables, 0.0, 1.0);

        TriangleMesh { triangles,
                       accelerator,
                       material_id }
    }

    pub fn from(filepath: &str, material_id: u32) -> TriangleMesh {
        // single_index + triangulate: tobj reindexes so positions and normals
        // are parallel arrays, and quads/ngons are split into triangles.
        let load_options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
            ..Default::default()
        };

        let (models, _materials) =
            tobj::load_obj(filepath, &load_options).expect("Failed to load OBJ");

        let mut triangles: Vec<Triangle> = Vec::new();
        for model in models {
            let mesh = &model.mesh;

            let positions: Vec<Vec3> = mesh.positions
                                           .chunks(3)
                                           .map(|i| Vec3::new(i[0], i[1], i[2]))
                                           .collect();

            let uvs: Vec<Vec2> = if !mesh.texcoords.is_empty() {
                                            mesh.texcoords.chunks(2)
                                                        .map(|c| Vec2::new(c[0], c[1]))
                                                        .collect()
                                        } else {
                                            vec![Vec2::ZERO; positions.len()]
                                        };

            // Use the file's normals if present. Otherwise, compute smooth
            // per-vertex normals by averaging area-weighted face normals.
            let normals: Vec<Vec3> = if !mesh.normals.is_empty() {
                mesh.normals.chunks(3)
                            .map(|i| Vec3::new(i[0], i[1], i[2]))
                            .collect()
            } else {
                let mut computed = vec![Vec3::ZERO; positions.len()];
                for i in 0..mesh.indices.len() / 3 {
                    let (a, b, c) = (mesh.indices[3 * i] as usize,
                                     mesh.indices[3 * i + 1] as usize,
                                     mesh.indices[3 * i + 2] as usize);
                    let edge1 = positions[b] - positions[a];
                    let edge2 = positions[c] - positions[a];
                    let face_normal = edge1.cross(edge2);  // area-weighted
                    computed[a] += face_normal;
                    computed[b] += face_normal;
                    computed[c] += face_normal;
                }
                for n in computed.iter_mut() {
                    if n.length_squared() > 1e-20 {
                        *n = n.normalize();
                    } else {
                        *n = Vec3::new(0.0, 1.0, 0.0);
                    }
                }
                computed
            };

            for i in 0..mesh.indices.len() / 3 {
                let (a, b, c) = (mesh.indices[3 * i] as usize,
                                 mesh.indices[3 * i + 1] as usize,
                                 mesh.indices[3 * i + 2] as usize);
                let (v0, v1, v2) = (positions[a], positions[b], positions[c]);
                let (n0, n1, n2) = (normals[a], normals[b], normals[c]);
                let (uv0, uv1, uv2) = (uvs[a], uvs[b], uvs[c]);

                let triangle = Triangle::from_box(v0, v1, v2, n0, n1, n2, uv0, uv1, uv2, material_id);
                triangles.push(triangle);
            }
        }

        TriangleMesh::new(triangles, material_id)
    }
}

impl Hitable for TriangleMesh {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        self.accelerator.hit(&ray, position_min, position_max)
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        let mut minimum = Vec3::splat(f32::MAX);
        let mut maximum = Vec3::splat(f32::MIN);

        for triangle in &self.triangles {
            minimum = minimum.min(triangle.minimum());
            maximum = maximum.max(triangle.maximum());
        }

        Some(AABB::from(minimum, maximum))
    }
}
