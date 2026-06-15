use std::f32;
use std::sync::Arc;

use glam::{Mat4, Vec3A, Vec3};
use rand_pcg::Pcg64Mcg;

use crate::aabb::AABB;
use crate::primitive::Primitive;
use crate::ray::Ray;
use crate::results::HitResult;
use crate::triangle::TriangleMesh;

/// TransformedMesh provides a way to transform a mesh.
///
/// Instead of changing the geometry of the mesh, rays are
/// transformed into the primitive's local space and the
/// ray-object intersection test is performed there. Once complete,
/// the result is transformed back into world space.
///
/// inverse_transform: world-to-local space transform matrix
/// forward_transform: local-to-world space transform matrix
/// normal_transform: surface normal transform matrix
/// bbox: bounding box in world space
/// primitive: the primitive being transformed
#[derive(Clone)]
pub struct TransformedMesh {
    inverse_transform: Mat4,
    forward_transform: Mat4,
    normal_transform: Mat4,
    bbox: AABB,
    primitive: Arc<Primitive>,
}

impl TransformedMesh {
    /// Create a new TransformedMesh.
    ///
    /// translate: the translation vector
    /// rotation: the rotation vector where each position correlates to that same axis
    /// scale: scale factor
    /// primitive: the primitive to transform
    pub fn new(
        translate: Vec3A,
        rotation: Vec3A,
        scale: Vec3A,
        primitive: impl Into<Primitive>,
    ) -> TransformedMesh {
        let primitive = primitive.into();
        let forward_transform = build_transform_matrix(translate, rotation, scale);

        // the inverse transform is needed for converting back from world space to local space
        let inverse_transform = forward_transform.inverse();

        // transform the local space bbox to world space
        let local_bbox = primitive.bounding_box().unwrap();
        let bbox = transform_aabb(&local_bbox, &forward_transform);

        let primitive = Arc::new(primitive.into());
        
        TransformedMesh {
            inverse_transform,
            forward_transform,
            normal_transform: inverse_transform.transpose(),
            bbox,
            primitive,
        }
    }

    /// Create a TransformedMesh from the given matrix and primitive.
    ///
    /// forward_transform: a homogeneous matrix
    /// primitive: the primitive to transform
    pub fn from_matrix(forward_transform: Mat4, primitive: Primitive) -> TransformedMesh {
        let inverse_transform = forward_transform.inverse();

        let local_bbox = primitive.bounding_box().unwrap();
        let bbox = transform_aabb(&local_bbox, &forward_transform);

        let primitive = Arc::new(primitive);

        TransformedMesh {
            inverse_transform,
            forward_transform,
            normal_transform: inverse_transform.transpose(),
            bbox,
            primitive,
        }
    }

    /// Determine whether the TransformedMesh has been hit by the given ray.
    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut Pcg64Mcg) -> Option<HitResult> {
        // transform the ray from world space into local space using the inverse transform
        let local_origin = self.inverse_transform.transform_point3(ray.origin.into());
        let local_direction = self.inverse_transform.transform_vector3(ray.direction.into());

        let local_direction_length = local_direction.length();
        let local_direction_normalized = local_direction / local_direction_length;

        // scale the distance range into local space using the stretched direction vector
        let local_start_distance = start_distance * local_direction_length;
        let local_end_distance = end_distance * local_direction_length;

        // construct the local space ray
        let local_ray = Ray::new(local_origin.into(), local_direction_normalized.into(), ray.time);

        if let Some(hit) = self.primitive.hit(&local_ray, local_start_distance, local_end_distance, rng) {
            // transform the hit attributes back into world space before returning them
            // in a new HitResult
            let parameter = hit.parameter / local_direction_length;
            let point = self.forward_transform.transform_point3(hit.point.into()).into();
            let geometric_normal = self.normal_transform.transform_vector3(hit.geometric_normal.into()).normalize().into();
            let shading_normal = self.normal_transform.transform_vector3(hit.shading_normal.into()).normalize().into();

            Some(HitResult::new(parameter, hit.u, hit.v, point, geometric_normal, shading_normal, hit.material_id, hit.texture_id))
        } else {
            None
        }
    }

    /// Determine if the given ray hits anything inside the mesh
    ///
    /// Used in the BVH for early completion.
    pub fn hits_anything(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut Pcg64Mcg) -> bool {
        let local_origin = self.inverse_transform.transform_point3(ray.origin.into());
        let local_direction = self.inverse_transform.transform_vector3(ray.direction.into());

        let local_direction_length = local_direction.length();
        let local_direction_normalized = local_direction / local_direction_length;

        let local_start_distance = start_distance * local_direction_length;
        let local_end_distance = end_distance * local_direction_length;

        let local_ray = Ray::new(local_origin.into(), local_direction_normalized.into(), ray.time);

        self.primitive.hits_anything(&local_ray, local_start_distance, local_end_distance, rng)
    }
    
    /// Return the world space bounding box of the TransformedMesh
    pub fn bounding_box(&self) -> Option<AABB> {
        Some(self.bbox)
    }

    /// Convert this TransformedMesh into a MotionMeshBuilder to start the
    /// process to build the mesh into a MotionMesh.
    pub fn into_motion(self) -> MotionMeshBuilder {
        MotionMeshBuilder::new(self)
    }
}

impl From<Primitive> for TransformedMesh {
    /// Create a TransformedMesh from a Primitive
    fn from(primitive: Primitive) -> Self {
        TransformedMesh::new(
            Vec3A::ZERO,
            Vec3A::ZERO,
            Vec3A::ONE,
            primitive,
        )
    }
}

impl From<TriangleMesh> for TransformedMesh {
    /// Create a TransformedMesh directly from a TriangleMesh
    fn from(mesh: TriangleMesh) -> Self {
        let primitive = Primitive::from(mesh);

        TransformedMesh::from(primitive)
    }
}

#[derive(Clone)]
pub struct MotionMesh {
    translate0: Vec3A, translate1: Vec3A,
    rotation0: Vec3A, rotation1: Vec3A,
    scale0: Vec3A, scale1: Vec3A,
    time0: f32, time1: f32,
    bbox: AABB,
    primitive: Arc<Primitive>,
}

impl MotionMesh {
    /// Create a new mesh that moves over time.
    pub fn new(
        translate0: Vec3A, translate1: Vec3A,
        rotation0: Vec3A, rotation1: Vec3A,
        scale0: Vec3A, scale1: Vec3A,
        time0: f32, time1: f32,
        primitive: Primitive,
    ) -> MotionMesh {
        let transform0 = build_transform_matrix(translate0, rotation0, scale0);
        let transform1 = build_transform_matrix(translate1, rotation1, scale1);

        let local_bbox = primitive.bounding_box().unwrap();

        let bbox0 = transform_aabb(&local_bbox, &transform0);
        let bbox1 = transform_aabb(&local_bbox, &transform1);
        let bbox = bbox0.surrounding_box(&bbox1);

        let primitive = Arc::new(primitive);

        MotionMesh {
            translate0, translate1,
            rotation0, rotation1,
            scale0, scale1,
            time0, time1,
            bbox,
            primitive,
        }
    }

    pub fn hit(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut Pcg64Mcg) -> Option<HitResult> {
        let time = if self.time1 > self.time0 {
            ((ray.time - self.time0) / (self.time1 - self.time0)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let current_translate = self.translate0.lerp(self.translate1, time);
        let current_rotation = self.rotation0.lerp(self.rotation1, time);
        let current_scale = self.scale0.lerp(self.scale1, time);

        let forward_transform = build_transform_matrix(current_translate, current_rotation, current_scale);
        let inverse_transform = forward_transform.inverse();
        let normal_transform = inverse_transform.transpose();

        let local_origin = inverse_transform.transform_point3(ray.origin.into());
        let local_direction = inverse_transform.transform_vector3(ray.direction.into());

        let local_direction_length = local_direction.length();
        let local_direction_normalized = local_direction / local_direction_length;

        let local_start_distance = start_distance * local_direction_length;
        let local_end_distance = end_distance * local_direction_length;

        let local_ray = Ray::new(local_origin.into(), local_direction_normalized.into(), ray.time);

        if let Some(hit) = self.primitive.hit(&local_ray, local_start_distance, local_end_distance, rng) {
            let parameter = hit.parameter / local_direction_length;
            let point = forward_transform.transform_point3(hit.point.into()).into();
            let geometric_normal = normal_transform.transform_vector3(hit.geometric_normal.into()).normalize().into();
            let shading_normal = normal_transform.transform_vector3(hit.shading_normal.into()).normalize().into();

            Some(HitResult::new(parameter, hit.u, hit.v, point, geometric_normal, shading_normal, hit.material_id, hit.texture_id))
        } else {
            None
        }
    }

    pub fn hits_anything(&self, ray: &Ray, start_distance: f32, end_distance: f32, rng: &mut Pcg64Mcg) -> bool {
        let time = if self.time1 > self.time0 {
            ((ray.time - self.time0) / (self.time1 - self.time0)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let current_translate = self.translate0.lerp(self.translate1, time);
        let current_rotation = self.rotation0.lerp(self.rotation1, time);
        let current_scale = self.scale0.lerp(self.scale1, time);

        let forward_transform = build_transform_matrix(current_translate, current_rotation, current_scale);
        let inverse_transform = forward_transform.inverse();

        let local_origin = inverse_transform.transform_point3(ray.origin.into());
        let local_direction = inverse_transform.transform_vector3(ray.direction.into());

        let local_direction_length = local_direction.length();
        let local_direction_normalized = local_direction / local_direction_length;

        let local_start_distance = start_distance * local_direction_length;
        let local_end_distance = end_distance * local_direction_length;

        let local_ray = Ray::new(local_origin.into(), local_direction_normalized.into(), ray.time);

        self.primitive.hits_anything(&local_ray, local_start_distance, local_end_distance, rng)
    }

    pub fn bounding_box(&self) -> Option<AABB> {
        Some(self.bbox)
    }
}

/// A builder that is used to create a MotionMesh
pub struct MotionMeshBuilder {
    translate0: Vec3A, translate1: Vec3A,
    rotation0: Vec3A, rotation1: Vec3A,
    scale0: Vec3A, scale1: Vec3A,
    time0: f32, time1: f32,
    primitive: Primitive,
}

impl MotionMeshBuilder {
    /// Start the MotionMesh build process for the given TransformedMesh
    pub fn new(mesh: TransformedMesh) -> Self {
        let (scale, rotation, translate) = mesh.forward_transform.to_scale_rotation_translation();

        // since we multiplied Z * Y * X in build_transform, we need to reverse that here
        let (rotation_z, rotation_y, rotation_x) = rotation.to_euler(glam::EulerRot::ZYX);
        let rotation = Vec3A::new(rotation_x.to_degrees(), rotation_y.to_degrees(), rotation_z.to_degrees());

        let primitive = Arc::unwrap_or_clone(mesh.primitive);

        Self {
            translate0: Vec3A::from(translate), translate1: Vec3A::from(translate),
            rotation0: rotation, rotation1: rotation,
            scale0: Vec3A::from(scale), scale1: Vec3A::from(scale),
            time0: 0.0, time1: 1.0,
            primitive,
        }
    }

    /// Transform the TransformedMesh from it's static translation in world space to the
    /// given translation.
    pub fn with_translation(mut self, translation: Vec3A) -> Self {
        self.translate1 = translation;
        self
    }

    /// Transform the TransformedMesh from it's static rotation in world space to the
    /// given rotation.
    pub fn with_rotation(mut self, rotation: Vec3A) -> Self {
        self.rotation1 = rotation;
        self
    }

    /// Transform the TransformedMesh from it's static scale in world space to the
    /// given scale.
    pub fn with_scale(mut self, scale: Vec3A) -> Self {
        self.scale1 = scale;
        self
    }

    /// Define the time range for the MotionMesh
    pub fn with_time_range(mut self, time0: f32, time1: f32) -> Self {
        self.time0 = time0;
        self.time1 = time1;
        self
    }

    /// Build the MotionMeshBuilder into a MotionMesh
    pub fn build(self) -> MotionMesh {
        MotionMesh::new(
            self.translate0, self.translate1,
            self.rotation0, self.rotation1,
            self.scale0, self.scale1,
            self.time0, self.time1,
            self.primitive
        )
    }
}

/// Transform the local space AABB into world space with the given transform matrix.
fn transform_aabb(bbox: &AABB, transform: &Mat4) -> AABB {
    let min = bbox.minimum;
    let max = bbox.maximum;

    // after a rotation or non-uniform transform, the old min and max are
    // no longer relevant, so we have to compute a new min and max for the bbox
    // by testing all eight vertices.
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ];

    // first select the first corner as min and max
    let first = transform.transform_point3(corners[0]);
    let mut new_min = first;
    let mut new_max = first;

    // iterate through the rest of the corners and find the new min and max
    for i in 1..8 {
        let p = transform.transform_point3(corners[i]);
        new_min = new_min.min(p);
        new_max = new_max.max(p);
    }

    AABB::from(new_min.into(), new_max.into())
}

/// Build a transform matrix from the given vectors.
///
/// translate: the translation vector
/// rotation: the rotation vector with each axis represented by angles (in degrees)
/// scale: the scale vector
fn build_transform_matrix(translate: Vec3A, rotation: Vec3A, scale: Vec3A) -> Mat4 {
    let scale_matrix = Mat4::from_scale(scale.into());

    // build rotation matrices for each rotation axis
    let rotation_x = Mat4::from_rotation_x(rotation.x.to_radians());
    let rotation_y = Mat4::from_rotation_y(rotation.y.to_radians());
    let rotation_z = Mat4::from_rotation_z(rotation.z.to_radians());
    let translate_matrix = Mat4::from_translation(translate.into());

    // remember that matrix multiplication applies from right to left
    // so first we apply scale; then rotation x, rotation y, rotation z;
    // and finally the translation matrix
    translate_matrix * rotation_z * rotation_y * rotation_x * scale_matrix
}


#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec3A, Vec3};
    use crate::materials::MaterialId;
    use rand::SeedableRng;
    use crate::ray::Ray;
    use crate::sphere::Sphere;
    use crate::texture::TextureId;

    fn get_rng() -> Pcg64Mcg {
        Pcg64Mcg::seed_from_u64(0)
    }

    #[test]
    fn test_translation_hit() {
        let sphere = Sphere::new(Vec3A::ZERO, 1.0, MaterialId(0), TextureId(0));
        let mut rng = get_rng();

        let transformed = TransformedMesh::new(
            Vec3A::new(5.0, 0.0, 0.0),
            Vec3A::ZERO,
            Vec3A::ONE,
            sphere,
        );

        let ray = Ray::new(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0), 0.0);

        let hit = transformed.hit(&ray, 0.001, 100.0, &mut rng);
        assert!(hit.is_some());

        let hit_result = hit.unwrap();
        assert_eq!(hit_result.parameter, 4.0);
        assert_eq!(hit_result.point.x, 4.0);
    }

    #[test]
    fn test_scaling_hit() {
        let sphere = Sphere::new(Vec3A::ZERO, 1.0, MaterialId(0), TextureId(0));
        let mut rng = get_rng();

        let transformed = TransformedMesh::new(
            Vec3A::ZERO,
            Vec3A::ZERO,
            Vec3A::new(2.0, 2.0, 2.0),
            sphere,
        );

        let ray = Ray::new(Vec3A::new(-5.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0), 0.0);

        let hit = transformed.hit(&ray, 0.001, 100.0, &mut rng);
        assert!(hit.is_some());

        let hit_result = hit.unwrap();
        assert_eq!(hit_result.parameter, 3.0);
        assert_eq!(hit_result.geometric_normal.x, -1.0);
    }

    #[test]
    fn test_aabb_transformation() {
        let original_bbox = AABB::from(Vec3A::new(-1.0, -1.0, -1.0), Vec3A::ONE);
        let translate = Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0));
        let transformed_bbox = transform_aabb(&original_bbox, &translate);

        assert_eq!(transformed_bbox.minimum.x, -1.0);
        assert_eq!(transformed_bbox.minimum.y, 0.0);
        assert_eq!(transformed_bbox.maximum.y, 2.0);
    }
}