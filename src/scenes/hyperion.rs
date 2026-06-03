use std::f32;
use std::sync::Arc;

use glam::{Vec2, Vec3A};

use bvh::BVH;
use camera::Camera;
use environment::EnvironmentMap;
use materials::{Diffuse, Material, Plastic, Reflective, Refractive, Volumetric};
use plane::{Axis, Bounds2D, Plane};
use primitive::Primitive;
use rectangle::Rectangle;
use scene::{Scene, SceneBuilder};
use sphere::Sphere;
use texture::{Color, ImageTexture};
use transformations::TransformedMesh;
use triangle::TriangleMesh;
use volume::Volume;

use mat;


pub fn hyperion_scene(width: Option<usize>, height: Option<usize>) -> Scene {
    let origin = Vec3A::new(0.0, 6.0, 6.0);
    let lookat = Vec3A::new(0.0, 0.0, -1.5);
    let view = Vec3A::new(0.0, 1.0, 0.0);
    let fov = 22.0;
    let aspect_ratio = (width.unwrap_or(1920) as f32, height.unwrap_or(1080) as f32);
    let aperture = 0.01;
    let focus_distance = (lookat - origin).length();

    let camera = Camera::new(origin,
                             lookat,
                             view,
                             fov,
                             aspect_ratio,
                             aperture,
                             focus_distance);

    let mut objects: Vec<Primitive> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();

    let orange_color = Color::new(1.0, 0.32, 0.0);
    let orange_bright_color = Color::new(1.0, 0.16, 0.0);
    let floor_idx = mat!(materials, Diffuse::new(Color::new(0.63, 0.61, 0.59).into(), 0.0));
    let glass_idx = mat!(materials, Refractive::new(Color::new(1.0, 1.0, 1.0).into(), 1.5));
    let metal_idx = mat!(materials, Reflective::new(Color::new(0.93, 0.93, 0.93).into(), 0.2));
    let dark_metal_idx = mat!(materials, Reflective::new(Color::new(0.757, 0.729, 0.694).into(), 0.10));
    let platform_idx = mat!(materials, Diffuse::new(Color::new(0.76, 0.74, 0.72).into(), 0.0));
    let orange_idx = mat!(materials, Plastic::new(orange_color.into(), 0.15, 1.5));
    let orange_rough_idx = mat!(materials, Plastic::new(orange_bright_color.into(), 0.25, 1.5));
    let marble_vol_idx = mat!(materials, Volumetric::new(Color::new(0.60, 0.71, 0.49).into()));
    let cricket_idx = mat!(materials, Plastic::new(ImageTexture::new("extras/textures/cricket_ball_diffuse.jpg", Vec2::splat(1.0)).into(), 0.30, 1.5));
    let pingpong_idx = mat!(materials, Plastic::new(Color::new(0.93, 0.89, 0.85).into(), 0.35, 1.45));
    let white_idx = mat!(materials, Plastic::new(Color::new(1.0, 1.0, 1.0).into(), 0.1, 1.45));
    let cream_idx = mat!(materials, Plastic::new(Color::new(1.0, 0.904, 0.725).into(), 1.0, 1.45));

    let floor_plane = Plane::new(Axis::XZ, Bounds2D::new(-50.0..50.0, -50.0..50.0), 0.0, floor_idx);
    let platform = Rectangle::new(Vec3A::new(-3.5, 0.0, -4.0), Vec3A::new(3.5, 0.2, 0.5), platform_idx);
    let glass_sphere = Sphere::new(Vec3A::new(2.1, 0.60, -1.0), 0.4, glass_idx);
    let orange_sphere = Sphere::new(Vec3A::new(1.0, 0.55, -2.0), 0.35, orange_idx);
    let metal_sphere = Sphere::new(Vec3A::new(-2.25, 0.65, -1.2), 0.45, dark_metal_idx);
    let large_marble = Sphere::new(Vec3A::new(0.35, 0.325, 0.0), 0.125, glass_idx);
    let small_marble = Sphere::new(Vec3A::new(-0.35, 0.30, 0.0), 0.10, glass_idx);
    let large_marble_volume = Volume::new(15.0, large_marble.clone().into(), marble_vol_idx);
    let small_marble_volume = Volume::new(15.0, small_marble.clone().into(), marble_vol_idx);
    let orange_sphere_small = Sphere::new(Vec3A::new(-1.5, 0.35, -2.0), 0.15, orange_rough_idx);

    let ring_mesh = Arc::new(TriangleMesh::from("extras/models/ring.obj", metal_idx));
    let ring_left = TransformedMesh::new(Vec3A::new(-0.5, 0.20, -1.25), Vec3A::new(0.0, 0.0, 0.0), Vec3A::new(0.15, 0.17, 0.15), Primitive::TriangleMesh(Arc::clone(&ring_mesh)));
    let ring_center = TransformedMesh::new(Vec3A::new(0.20, 0.20, -0.80), Vec3A::new(0.0, 45.0, 0.0), Vec3A::new(0.15, 0.17, 0.15), Primitive::TriangleMesh(Arc::clone(&ring_mesh)));
    let ring_right = TransformedMesh::new(Vec3A::new(0.375, 0.25, -1.0), Vec3A::new(-15.0, 0.0, -10.0), Vec3A::new(0.15, 0.17, 0.15), Primitive::TriangleMesh(Arc::clone(&ring_mesh)));

    let cricket_ball_mesh = TriangleMesh::from("extras/models/cricket_ball_no_stitch.obj", cricket_idx);
    let cricket_ball_stitch_bottom_mesh = TriangleMesh::from("extras/models/cricket_ball_stitch_bottom.obj", cream_idx);
    let cricket_ball_stitch_top_mesh = TriangleMesh::from("extras/models/cricket_ball_stitch_top.obj", cream_idx);
    let (translation, rotation, scale) = (Vec3A::new(-0.6, 0.70, -2.5), Vec3A::new(-30.0, 0.0, 15.0), Vec3A::splat(1.75));
    let cricked_ball = TransformedMesh::new(translation, rotation, scale, cricket_ball_mesh.into());
    let cricked_ball_stitch_bottom = TransformedMesh::new(translation, rotation, scale, cricket_ball_stitch_bottom_mesh.into());
    let cricked_ball_stitch_top = TransformedMesh::new(translation, rotation, scale, cricket_ball_stitch_top_mesh.into());

    let pingpong_mesh = TriangleMesh::from("extras/models/pingpong.obj", pingpong_idx);
    let pingpong_sphere = TransformedMesh::new(Vec3A::new(-1.25, 0.475, -0.3), Vec3A::new(0.0, 0.0, 90.0), Vec3A::splat(0.28), pingpong_mesh.into());

    let golf_ball_mesh = TriangleMesh::from("extras/models/golf_ball.obj", white_idx);
    let golf_ball = TransformedMesh::new(Vec3A::new(1.70, 0.67, -0.35), Vec3A::new(-30.0, 0.0, 15.0), Vec3A::splat(0.23), golf_ball_mesh.into());


    objects.push(floor_plane.into());
    objects.push(platform.into());
    objects.push(glass_sphere.into());
    objects.push(metal_sphere.into());
    objects.push(orange_sphere.into());
    objects.push(large_marble.into());
    objects.push(small_marble.into());
    objects.push(large_marble_volume.into());
    objects.push(small_marble_volume.into());
    objects.push(ring_left.into());
    objects.push(ring_center.into());
    objects.push(ring_right.into());
    objects.push(cricked_ball.into());
    objects.push(cricked_ball_stitch_bottom.into());
    objects.push(cricked_ball_stitch_top.into());
    objects.push(pingpong_sphere.into());
    objects.push(golf_ball.into());
    objects.push(orange_sphere_small.into());

    let bvh = BVH::new(&mut objects);

    let environment = EnvironmentMap::new("extras/textures/white_studio_03.exr", 0.6).into();

    SceneBuilder::new("Hyperion")
        .with_accelerator(bvh)
        .with_camera(camera)
        .with_materials(materials)
        .with_environment(environment)
        .build()
        .expect("Failed to build Hyperion scene")
}