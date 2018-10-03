extern crate image;
extern crate nalgebra;
extern crate rand;

mod camera;
mod hitable;
mod ray;
mod sphere;

use camera::Camera;
use hitable::HitableList;
use nalgebra::core::Vector3;
use sphere::Sphere;
use std::fs::File;


fn main() {
    let (width, height): (u32, u32) = (1600, 800);
    let samples: u32 = 100;

    let mut buffer = image::ImageBuffer::new(width, height);
    let camera = Camera::new(Vector3::new(-2.0, -1.0, -1.0),
                             Vector3::new(4.0, 0.0, 0.0),
                             Vector3::new(0.0, 2.0, 0.0),
                             Vector3::new(0.0, 0.0, 0.0));

    let mut world = HitableList::new();
    world.push(Box::new(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)));
    world.push(Box::new(Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0)));

    for x in 0..width {
        for y in 0..height {
            let mut coordinate: Vector3<f64> = Vector3::zeros();

            for sample in 0..samples {
                let u = (x as f64 + rand::random::<f64>()) / width as f64;
                let v = (y as f64 + rand::random::<f64>()) / height as f64;
                let ray = camera.get_ray(u, v);
                coordinate += ray.color(&world);
            }

            coordinate /= samples as f64;
            let red = (255.0 * coordinate.x) as u8;
            let green = (255.0 * coordinate.y) as u8;
            let blue = (255.0 * coordinate.z) as u8;
            buffer.put_pixel(x, y, image::Rgb([red, green, blue]));
        }
    }

    let ref mut render = File::create("render.png").unwrap();
    image::ImageRgb8(buffer).flipv().save(render, image::PNG).unwrap();
}
