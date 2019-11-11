use std::f32;
use std::path::Path;
use std::sync::Arc;

use glam::Vec3;
use tobj;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::Material;
use ray::Ray;
use world::World;

#[derive(Clone)]
pub struct Triangle {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    n0: Vec3,
    n1: Vec3,
    n2: Vec3,
    material: Arc<dyn Material>,
}

pub struct TriangleMesh {
    triangles: Vec<Triangle>,
    hitables: World,
    material: Arc<dyn Material>,
}

impl Triangle {
    /// Create a new triangle with vertices v0, v1, and v2
    pub fn new<M: Material + 'static>(v0: Vec3,
                                      v1: Vec3,
                                      v2: Vec3,
                                      n0: Vec3,
                                      n1: Vec3,
                                      n2: Vec3,
                                      material: M)
                                      -> Triangle {
        let material = Arc::new(material);
        Triangle { v0: v0,
                   v1: v1,
                   v2: v2,
                   n0: n0,
                   n1: n1,
                   n2: n2,
                   material: material }
    }

    pub fn from_box(v0: Vec3,
                    v1: Vec3,
                    v2: Vec3,
                    n0: Vec3,
                    n1: Vec3,
                    n2: Vec3,
                    material: Arc<dyn Material>)
                    -> Triangle {
        Triangle { v0: v0,
                   v1: v1,
                   v2: v2,
                   n0: n0,
                   n1: n1,
                   n2: n2,
                   material: material }
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
    fn hit(&self, ray: &Ray, position_min: f32, _position_max: f32) -> Option<HitRecord> {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;

        let pvec = ray.direction.cross(edge2);
        let determinant = edge1.dot(pvec);

        if determinant < position_min {
            return None;
        }

        let tvec = ray.origin - self.v0;
        let mut u = tvec.dot(pvec);

        if u < 0.0 || u > determinant {
            return None;
        }

        let qvec = tvec.cross(edge1);
        let mut v = ray.direction.dot(qvec);

        if v < 0.0 || u + v > determinant {
            return None;
        }

        let mut t = edge2.dot(qvec);

        let inverse_determinant = 1.0 / determinant;
        t *= inverse_determinant;
        u *= inverse_determinant;
        v *= inverse_determinant;

        let point = u * self.v0 + v * self.v1 + (1.0 - u - v) * self.v2;
        let geometric_normal = edge1.cross(edge2).normalize();
        let shading_normal = ((1.0 - u - v) * self.n0 + u * self.n1 + v * self.n2).normalize();

        Some(HitRecord::new(t,
                            u,
                            v,
                            point,
                            geometric_normal,
                            shading_normal,
                            self.material.clone()))
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
    pub fn new(triangles: Vec<Triangle>, material: Arc<dyn Material>) -> TriangleMesh {
        let mut world = World::new();

        for triangle in &triangles {
            world.add(triangle.clone());
        }

        TriangleMesh { triangles: triangles,
                       hitables: world,
                       material: material }
    }

    pub fn from(filepath: &str, material: Arc<dyn Material>) -> TriangleMesh {
        let obj = tobj::load_obj(&Path::new(&filepath));
        let (models, _) = obj.unwrap();

        let mut triangles: Vec<Triangle> = Vec::new();
        for model in models {
            let mesh = &model.mesh;

            let positions: Vec<Vec3> = mesh.positions
                                           .chunks(3)
                                           .map(|i| Vec3::new(i[0], i[1], i[2]))
                                           .collect();

            let normals: Vec<Vec3> = mesh.normals
                                         .chunks(3)
                                         .map(|i| Vec3::new(i[0], i[1], i[2]))
                                         .collect();

            for i in 0..mesh.indices.len() / 3 {
                let (i, j, k) =
                    (mesh.indices[3 * i], mesh.indices[3 * i + 1], mesh.indices[3 * i + 2]);
                let (v0, v1, v2) =
                    (positions[i as usize], positions[j as usize], positions[k as usize]);
                let (n0, n1, n2) = (normals[i as usize], normals[j as usize], normals[k as usize]);

                let triangle = Triangle::from_box(v0, v1, v2, n0, n1, n2, material.clone());
                triangles.push(triangle);
            }
        }

        TriangleMesh::new(triangles, material)
    }
}

impl Hitable for TriangleMesh {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        self.hitables.hit(&ray, position_min, position_max)
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        let mut minimum = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut maximum = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for triangle in &self.triangles {
            minimum = minimum.min(triangle.minimum());
            maximum = maximum.max(triangle.maximum());
        }

        Some(AABB::from(minimum, maximum))
    }
}
