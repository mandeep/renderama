#[cfg(feature = "denoise")]
/// Denoise the input buffer and return a denoised buffer
/// Reference: https://github.com/Twinklebear/oidn-rs/blob/master/examples/simple/src/main.rs
pub fn denoise(input: &Vec<f32>, width: usize, height: usize) -> Vec<u8> {
    let mut filter_output = vec![0.0f32; input.len()];

    let mut device = oidn::Device::new();
    let mut filter = oidn::RayTracing::new(&mut device);
    filter.set_srgb(true).set_img_dims(width, height);
    filter.execute(&input[..], &mut filter_output[..]);

    if let Err(e) = device.get_error() {
        println!("Error denosing image: {}", e.1);
    }

    let output_buffer = filter_output.iter()
                                     .map(|&color| (255.0 * color) as u8)
                                     .collect();

    output_buffer
}
