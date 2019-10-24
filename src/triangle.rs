use std::f32;
use std::path::Path;
use std::sync::Arc;

use nalgebra::Vector3;
use tobj;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::Material;
use ray::Ray;
use world::World;

#[derive(Clone)]
pub struct Triangle {
    v0: Vector3<f32>,
    v1: Vector3<f32>,
    v2: Vector3<f32>,
    material: Arc<dyn Material>,
}

pub struct TriangleMesh {
    triangles: Vec<Triangle>,
    hitables: World,
    material: Arc<dyn Material>
}

impl Triangle {
    /// Create a new triangle with vertices v0, v1, and v2
    pub fn new<M: Material + 'static>(v0: Vector3<f32>,
                                      v1: Vector3<f32>,
                                      v2: Vector3<f32>,
                                      material: M)
                                      -> Triangle {
        let material = Arc::new(material);
        Triangle { v0: v0,
                   v1: v1,
                   v2: v2,
                   material: material }
    }

    pub fn from_box(v0: Vector3<f32>,
                                      v1: Vector3<f32>,
                                      v2: Vector3<f32>,
                                      material: Arc<dyn Material>)
                                      -> Triangle {
        Triangle { v0: v0,
                   v1: v1,
                   v2: v2,
                   material: material }
    }

    pub fn minimum(&self) -> Vector3<f32> {
        self.v0
            .zip_map(&self.v1, |a, b| a.min(b))
            .zip_map(&self.v2, |c, d| c.min(d))
    }

    pub fn maximum(&self) -> Vector3<f32> {
        self.v0
            .zip_map(&self.v1, |a, b| a.max(b))
            .zip_map(&self.v2, |c, d| c.max(d))
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

        let pvec = ray.direction.cross(&edge2);
        let determinant = edge1.dot(&pvec);

        if determinant < position_min {
            return None;
        }

        let tvec = ray.origin - self.v0;
        let mut u = tvec.dot(&pvec);

        if u < 0.0 || u > determinant {
            return None;
        }

        let qvec = tvec.cross(&edge1);
        let mut v = ray.direction.dot(&qvec);

        if v < 0.0 || u + v > determinant {
            return None;
        }

        let mut t = edge2.dot(&qvec);

        let inverse_determinant = 1.0 / determinant;
        t *= inverse_determinant;
        u *= inverse_determinant;
        v *= inverse_determinant;

        let point = ray.point_at_parameter(t);
        let normal = edge1.cross(&edge2);

        Some(HitRecord::new(t, u, v, point, normal, self.material.clone()))
    }

    /// Create a bounding box around the triangle
    ///
    /// The bounding box is created using the minimum
    /// and maximum points of all of the vertices
    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        Some(AABB::new(self.minimum(), self.maximum()))
    }
}

impl TriangleMesh {
    pub fn new(triangles: Vec<Triangle>, material: Arc<dyn Material>) -> TriangleMesh {
    let mut world = World::new();

    for triangle in &triangles {
        world.add(triangle.clone());
    }

    TriangleMesh { triangles: triangles, hitables: world, material: material }
    }

    pub fn from(filepath: &str, material: Arc<dyn Material>) -> TriangleMesh {
        let obj = tobj::load_obj(&Path::new(&filepath));
        let (models, _) = obj.unwrap();

        let mut triangles: Vec<Triangle> = Vec::new();
        for model in models {
            let mesh = &model.mesh;

        let positions: Vec<Vector3<f32>> = mesh.positions
            .chunks(3)
            .map(|i| Vector3::new(i[0], i[1], i[2]))
            .collect();

        for i in 0..positions.len() / 3 {
            let (v0, v1, v2) = (positions[3*i], positions[3*i+1], positions[3*i+2]);
            let triangle = Triangle::from_box(v0, v1, v2, material.clone());
            triangles.push(triangle);
        }

        TriangleMesh::new(triangles, material)
    }
}

impl Hitable for TriangleMesh {
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        self.hitables.hit(&ray, position_min, position_max)
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        let mut minimum = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut maximum = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for triangle in &self.triangles {
            minimum = minimum.zip_map(&triangle.minimum(), |a, b| a.min(b));
            maximum = maximum.zip_map(&triangle.maximum(), |a, b| a.max(b));
        }

        Some(AABB::new(minimum, maximum))
    }
}
