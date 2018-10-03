extern crate image;
extern crate nalgebra;
extern crate rand;
extern crate rayon;

mod camera;
mod hitable;
mod ray;
mod sphere;

use camera::Camera;
use hitable::{Hitable, HitableList, HitRecord};
use nalgebra::core::Vector3;
use ray::Ray;
use rayon::prelude::*;
use sphere::Sphere;
use std::f64;


fn color(ray: &Ray, world: &Hitable) -> Vector3<f64> {
    let mut hit_record = HitRecord::new(f64::MAX);

    let x = rand::random::<f64>();
    let y = rand::random::<f64>();
    let z = rand::random::<f64>();

    let distribution = 1.0 / (x * x + y * y + z * z).sqrt();
    let random_unit_sphere_point = distribution * Vector3::new(x, y, z);

    if world.hit(ray, 0.001, f64::MAX, &mut hit_record) {
        let target: Vector3<f64> = hit_record.point + hit_record.normal + random_unit_sphere_point;
        return 0.5 * color(&Ray::new(hit_record.point, target - hit_record.point), world);
    }

    let unit_direction: Vector3<f64> = ray.direction.normalize();
    let point: f64 = 0.5 * (unit_direction.y + 1.0);

    (1.0 - point) * Vector3::new(1.0, 1.0, 1.0) + point * Vector3::new(0.5, 0.7, 1.0)
}


fn main() {
    let (width, height): (u32, u32) = (1920, 960);
    let samples: u32 = 100;

    let mut buffer = image::ImageBuffer::new(width, height);
    let camera = Camera::new(Vector3::new(-2.0, -1.0, -1.0),
                             Vector3::new(4.0, 0.0, 0.0),
                             Vector3::new(0.0, 2.0, 0.0),
                             Vector3::new(0.0, 0.0, 0.0));

    let mut world = HitableList::new();
    world.push(Box::new(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)));
    world.push(Box::new(Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0)));

    let mut pixels = vec![image::Rgb([0, 0, 0]); (width * height) as usize];
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let mut coordinate: Vector3<f64> = Vector3::zeros();
        let x = i % width as usize;
        let y = i / width as usize;

        (0..samples).for_each(|_| {
            let u = (x as f64 + rand::random::<f64>()) / width as f64;
            let v = (y as f64 + rand::random::<f64>()) / height as f64;
            let ray = camera.get_ray(u, v);
            coordinate += color(&ray, &world);
        });

        coordinate /= samples as f64;
        let red = (255.0 * coordinate.x.sqrt()) as u8;
        let green = (255.0 * coordinate.y.sqrt()) as u8;
        let blue = (255.0 * coordinate.z.sqrt()) as u8;
        *pixel = image::Rgb([red, green, blue]);
    });


    buffer.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        let index = (y * width + x) as usize;
        *pixel = pixels[index];
    });

    image::ImageRgb8(buffer).flipv().save("render.png").unwrap();
}
