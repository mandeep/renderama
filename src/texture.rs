use glam::Vec3;
use image;

/// Texture trait can be implemented so that textures can be applied to materials
pub trait Texture: Send + Sync {
    fn value(&self, u: f32, v: f32, p: &Vec3) -> Vec3;
}

#[derive(Clone)]
/// ConstantTexture is just a wrapping for a Vec3 of RGB values
pub struct ConstantTexture {
    color: Vec3,
}

/// Create a new ConstantTexture
impl ConstantTexture {
    pub fn new(r: f32, g: f32, b: f32) -> ConstantTexture {
        ConstantTexture { color: Vec3::new(r, g, b) }
    }
}

/// Implement the Texture trait for ConstantTexture
/// This allows the ConstantTexture's color to be retrieved
/// as well as the ConstantTexture to be cloned.
impl Texture for ConstantTexture {
    fn value(&self, _u: f32, _v: f32, _p: &Vec3) -> Vec3 {
        self.color
    }
}

#[derive(Clone)]
/// ImageTexture is a struct for textures loaded from file
pub struct ImageTexture {
    im: image::RgbImage,
}

/// Create a new texture from the given data and image dimensions
impl ImageTexture {
    pub fn new(filename: &str) -> ImageTexture {
        ImageTexture { im: image::open(filename).unwrap().flipv().to_rgb() }
    }
}

/// Determine which pixel to retrieve from the image by
/// converting pixel coordinates to UV coordinates
impl Texture for ImageTexture {
    fn value(&self, u: f32, v: f32, _p: &Vec3) -> Vec3 {
        let i = 0.0f32.max((u * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}
