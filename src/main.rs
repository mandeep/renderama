extern crate image;
extern crate nalgebra;

use std::fs::File;

use nalgebra::core::Vector3;

mod ray;


fn main() {
    let (width, height): (u32, u32) = (1600, 800);

    let mut buffer = image::ImageBuffer::new(width, height);

    let lower_left_corner = Vector3::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);
    let origin = Vector3::new(0.0, 0.0, 0.0);

    for x in 0..width {
        for y in 0..height {
            let u = x as f64 / width as f64;
            let v = y as f64 / height as f64;

            let ray = ray::Ray::new(origin, lower_left_corner + u * horizontal + v * vertical);
            let coordinate = ray.color();

            let red = (255.0 * coordinate.x) as u8;
            let green = (255.0 * coordinate.y) as u8;
            let blue = (255.0 * coordinate.z) as u8;
            buffer.put_pixel(x, y, image::Rgb([red, green, blue]));
        }
    }

    let ref mut render = File::create("output.png").unwrap();
    image::ImageRgb8(buffer).flipv().save(render, image::PNG).unwrap();
}
