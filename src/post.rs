use image::ImageBuffer;

use utils;


pub fn blur_image(image: ImageBuffer<image::Rgb<f32>, Vec<f32>>, n: usize) -> ImageBuffer<image::Rgb<f32>, Vec<f32>> {
    if n == 0 {
        image
    } else {
        let buffer = image::imageops::blur(&image, 8.0);
        blur_image(buffer, n - 1)
    }

}


pub fn bloom_filter(buffer: &Vec<f32>, width: usize, height: usize) -> Vec<f32> {
    let min_luminance = utils::f32_min(&buffer);
    let max_luminance = utils::f32_max(&buffer);

    let mut high_pass: Vec<f32> = vec![0.0f32; buffer.len()];

    for (i, luminance) in buffer.iter().enumerate() {
        if *luminance < max_luminance {
            high_pass[i] = min_luminance;
        } else {
            high_pass[i] = *luminance;
        }
    }

    let high_pass_buffer: ImageBuffer<image::Rgb<f32>, Vec<f32>> =
        ImageBuffer::from_vec(width as u32, height as u32, high_pass).unwrap();

    let blurred_buffer = blur_image(high_pass_buffer, 5);

    let mut bloom = vec![0.0f32; buffer.len()];

    for (i, pixel) in blurred_buffer.into_raw().iter().enumerate() {
        bloom[i] = buffer[i] + *pixel as f32;
    }

    bloom
}
