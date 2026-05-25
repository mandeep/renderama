use std::f32;
use std::sync::Arc;

use glam::{Mat4, Vec3A, Vec3};
use rand_pcg::Pcg64Mcg;

use aabb::AABB;
use primitive::Primitive;
use ray::Ray;
use results::HitResult;

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
        primitive: Primitive,
    ) -> TransformedMesh {
        let scale_matrix = Mat4::from_scale(scale.into());

        // build rotation matrices for each rotation axis
        let rotation_x = Mat4::from_rotation_x(rotation.x.to_radians());
        let rotation_y = Mat4::from_rotation_y(rotation.y.to_radians());
        let rotation_z = Mat4::from_rotation_z(rotation.z.to_radians());

        let translate_matrix = Mat4::from_translation(translate.into());

        // remember that matrix multiplication applies from right to left
        // so first we apply scale; then rotation x, rotation y, rotation z;
        // and finally the translation matrix 
        let forward_transform = translate_matrix * rotation_z * rotation_y * rotation_x * scale_matrix;
        // the inverse transform is needed for converting back from world space to local space
        let inverse_transform = forward_transform.inverse();

        // transform the local space bbox to world space
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
        let local_ray = Ray::new(local_origin.into(), local_direction_normalized.into());

        if let Some(hit) = self.primitive.hit(&local_ray, local_start_distance, local_end_distance, rng) {
            // transform the hit attributes back into world space before returning them
            // in a new HitResult
            let parameter = hit.parameter / local_direction_length;
            let point = self.forward_transform.transform_point3(hit.point.into()).into();
            let geometric_normal = self.normal_transform.transform_vector3(hit.geometric_normal.into()).normalize().into();
            let shading_normal = self.normal_transform.transform_vector3(hit.shading_normal.into()).normalize().into();

            Some(HitResult::new(parameter, hit.u, hit.v, point, geometric_normal, shading_normal, hit.material_id))
        } else {
            None
        }
    }
    
    /// Return the world space bounding box of the TransformedMesh
    pub fn bounding_box(&self) -> Option<AABB> {
        Some(self.bbox)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec3A, Vec3};
    use materials::MaterialId;
    use rand::SeedableRng;
use ray::Ray;
    use sphere::Sphere;

    fn get_rng() -> Pcg64Mcg {
        Pcg64Mcg::seed_from_u64(0)
    }

    #[test]
    fn test_translation_hit() {
        let sphere = Sphere::new(Vec3A::ZERO, 1.0, MaterialId(0));
        let mut rng = get_rng();

        let transformed = TransformedMesh::new(
            Vec3A::new(5.0, 0.0, 0.0),
            Vec3A::ZERO,
            Vec3A::ONE,
            sphere.into(),
        );

        let ray = Ray::new(Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));

        let hit = transformed.hit(&ray, 0.001, 100.0, &mut rng);
        assert!(hit.is_some());

        let hit_result = hit.unwrap();
        assert_eq!(hit_result.parameter, 4.0);
        assert_eq!(hit_result.point.x, 4.0);
    }

    #[test]
    fn test_scaling_hit() {
        let sphere = Sphere::new(Vec3A::ZERO, 1.0, MaterialId(0));
        let mut rng = get_rng();

        let transformed = TransformedMesh::new(
            Vec3A::ZERO,
            Vec3A::ZERO,
            Vec3A::new(2.0, 2.0, 2.0),
            sphere.into(),
        );

        let ray = Ray::new(Vec3A::new(-5.0, 0.0, 0.0), Vec3A::new(1.0, 0.0, 0.0));

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