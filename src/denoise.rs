#[cfg(feature = "denoise")]
/// Denoise the input buffer using albedo and normal auxiliary buffers.
/// Reference: https://github.com/Twinklebear/oidn-rs/blob/master/examples/simple/src/main.rs
pub fn denoise(input: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut filter_output = vec![0.0f32; input.len()];

    let device = oidn::Device::new();

    oidn::RayTracing::new(&device)
        .hdr(true)
        .image_dimensions(width, height)
        .filter(&input[..], &mut filter_output[..])
        .expect("Filter config error!");

    if let Err(e) = device.get_error() {
        println!("Error denoising image: {}", e.1);
    }

    filter_output
}
