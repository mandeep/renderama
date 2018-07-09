extern crate image;
extern crate nalgebra;

use std::fs::File;

fn main() {
    let (width, height): (u32, u32) = (1600, 1600);

    let mut buffer = image::ImageBuffer::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let red = (255.99 * (x as f64 / width as f64)) as u8;
            let green = (255.99 * (y as f64 / height as f64)) as u8;
            let blue = (255.99 * 0.2) as u8;
            buffer.put_pixel(x, y, image::Rgb([red, green, blue]));
        }
    }

    let ref mut render = File::create("output.png").unwrap();
    image::ImageRgb8(buffer).flipv().save(render, image::PNG).unwrap();
}
