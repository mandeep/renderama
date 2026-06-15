use std::f32;

use glam::{Vec2, Vec3A};
use rand_pcg::Pcg64Mcg;
use tobj;

use crate::aabb::AABB;
use crate::bvh::BVH;
use crate::materials::MaterialId;
use crate::primitive::Primitive;
use crate::ray::Ray;
use crate::results::HitResult;
use crate::texture::TextureId;

#[derive(Clone)]
pub struct Triangle {
    v0: Vec3A,
    v1: Vec3A,
    v2: Vec3A,
    n0: Vec3A,
    n1: Vec3A,
    n2: Vec3A,
    uv0: Vec2,
    uv1: Vec2,
    uv2: Vec2,
    material_id: MaterialId,
    texture_id: TextureId,
}

#[derive(Clone)]
pub struct TriangleMesh {
    accelerator: BVH,
}

impl Triangle {
    /// Create a new triangle with vertices v0, v1, and v2,
    /// normals n0, n1, and n2, and uvs uv0, uv1, and uv2
    /// These are typically loaded from a file such as OBJ.
    pub fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A,
               n0: Vec3A, n1: Vec3A, n2: Vec3A,
               uv0: Vec2, uv1: Vec2, uv2: Vec2,
               material_id: MaterialId, texture_id: TextureId) -> Triangle {

        Triangle { v0, v1, v2, n0, n1, n2, uv0, uv1, uv2, material_id, texture_id }
    }

    pub fn minimum(&self) -> Vec3A {
        self.v0.min(self.v1.min(self.v2))
    }

    pub fn maximum(&self) -> Vec3A {
        self.v0.max(self.v1.max(self.v2))
    }

    /// Determine whether or not a ray hits the triangle
    ///
    /// Reference:
    /// Tomas Moller, Ben Trumbore
    /// Fast, Minimum Storage Ray/Triangle Intersection
    /// Journal of Graphics Tools Vol. 2 Issue 1, 1997
    /// https://dl.acm.org/doi/abs/10.1145/1198555.1198746
    ///
    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitResult> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;

        let pvec = ray.direction.cross(edge2);
        let determinant = edge1.dot(pvec);

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

        let point = ray.point_at_parameter(t);
        let geometric_normal = edge1.cross(edge2).normalize();
        let shading_normal = ((1.0 - u - v) * self.n0 + u * self.n1 + v * self.n2).normalize();

        let w = 1.0 - u - v;
        let interpolated_uv = w * self.uv0 + u * self.uv1 + v * self.uv2;

        Some(HitResult::new(t,
                            interpolated_uv.x,
                            interpolated_uv.y,
                            point,
                            geometric_normal,
                            shading_normal,
                            self.material_id,
                            self.texture_id,
                        )
                    )
    }

    /// Create a bounding box around the triangle
    ///
    /// The bounding box is created using the minimum
    /// and maximum points of all of the vertices
    pub fn bounding_box(&self) -> Option<AABB> {
        Some(AABB::from(self.minimum(), self.maximum()))
    }
}

impl TriangleMesh {
    pub fn new(triangles: Vec<Triangle>) -> TriangleMesh {
        let mut geometries: Vec<Primitive> = triangles
            .into_iter()
            .map(Primitive::Triangle)
            .collect();

        let accelerator = BVH::new(&mut geometries);

        TriangleMesh { accelerator }
    }

    pub fn from(filepath: &str, material_id: MaterialId, texture_id: TextureId) -> TriangleMesh {
        let load_options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        };

        let (models, _materials) =
            tobj::load_obj(filepath, &load_options).expect("Failed to load OBJ");

        let mut triangles: Vec<Triangle> = Vec::new();
        for model in models {
            let mesh = &model.mesh;

            let positions: Vec<Vec3A> = mesh.positions
                                           .chunks(3)
                                           .map(|i| Vec3A::new(i[0], i[1], i[2]))
                                           .collect();

            let uvs: Vec<Vec2> = if !mesh.texcoords.is_empty() {
                                            mesh.texcoords.chunks(2)
                                                        .map(|c| Vec2::new(c[0], c[1]))
                                                        .collect()
                                        } else {
                                            vec![Vec2::ZERO; positions.len()]
                                        };

            let normals: Vec<Vec3A> = if !mesh.normals.is_empty() {
                mesh.normals.chunks(3)
                            .map(|i| Vec3A::new(i[0], i[1], i[2]))
                            .collect()
            } else {
                let mut computed = vec![Vec3A::ZERO; positions.len()];
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
                        *n = Vec3A::new(0.0, 1.0, 0.0);
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

                let triangle = Triangle::new(v0, v1, v2, n0, n1, n2, uv0, uv1, uv2, material_id, texture_id);
                triangles.push(triangle);
            }
        }

        TriangleMesh::new(triangles)
    }

    pub fn hit(&self, ray: &Ray, position_min: f32, position_max: f32, rng: &mut Pcg64Mcg) -> Option<HitResult> {
        self.accelerator.hit(ray, position_min, position_max, rng)
    }

    pub fn hits_anything(&self, ray: &Ray, position_min: f32, position_max: f32, rng: &mut Pcg64Mcg) -> bool {
        self.accelerator.hits_anything(ray, position_min, position_max, rng)
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        self.accelerator.bounding_box()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3A};
    use crate::materials::MaterialId;
    use crate::ray::Ray;

    fn create_test_triangle() -> Triangle {
        Triangle::new(
            Vec3A::new(-1.0, -1.0, 0.0),
            Vec3A::new(1.0, -1.0, 0.0),
            Vec3A::new(0.0, 1.0, 0.0),
            Vec3A::Z, Vec3A::Z, Vec3A::Z,
            Vec2::ZERO, Vec2::X, Vec2::Y,
            MaterialId(0),
            TextureId(0),
        )
    }

    #[test]
    fn test_triangle_hit() {
        let triangle = create_test_triangle();

        let ray = Ray::new(Vec3A::new(0.0, 0.0, 2.0), Vec3A::new(0.0, 0.0, -1.0), 0.0);

        let hit = triangle.hit(&ray, 0.001, 100.0);
        assert!(hit.is_some());

        let result = hit.unwrap();
        assert!((result.parameter - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_miss() {
        let triangle = create_test_triangle();

        let ray_parallel = Ray::new(Vec3A::new(0.0, 0.0, 2.0), Vec3A::new(1.0, 0.0, 0.0), 0.0);
        assert!(triangle.hit(&ray_parallel, 0.001, 100.0).is_none());

        let ray_away = Ray::new(Vec3A::new(0.0, 0.0, 2.0), Vec3A::new(0.0, 0.0, 1.0), 0.0);
        assert!(triangle.hit(&ray_away, 0.001, 100.0).is_none());
    }
}