use image;
use nalgebra::core::Vector3;


/// Texture trait can be implemented so that textures can be applied to materials
pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: &Vector3<f64>) -> Vector3<f64>;
    fn box_clone(&self) -> Box<Texture>;
}


/// Implement Clone so that the Texture trait can be used along with the Material trait
impl Clone for Box<Texture> {
    fn clone(&self) -> Box<Texture> {
        self.box_clone()
    }
}


#[derive(Clone)]
/// ConstantTexture is just a wrapping for a Vector3 of RGB values
pub struct ConstantTexture {
    color: Vector3<f64>
}


/// Create a new ConstantTexture
impl ConstantTexture {
    pub fn new(r: f64, g: f64, b: f64) -> ConstantTexture {
        ConstantTexture {color: Vector3::new(r, g, b)}
    }
}


/// Implement the Texture trait for ConstantTexture
/// This allows the ConstantTexture's color to be retrieved
/// as well as the ConstantTexture to be cloned.
impl Texture for ConstantTexture {
    fn value(&self, u: f64, v: f64, p: &Vector3<f64>) -> Vector3<f64> {
        self.color
    }

    fn box_clone(&self) -> Box<Texture> {
        Box::new((*self).clone())
    }
}


#[derive(Clone)]
/// ImageTexture is a struct for textures loaded from file
pub struct ImageTexture {
    im: image::RgbImage
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
    fn value(&self, u: f64, v: f64, p: &Vector3<f64>) -> Vector3<f64> {
        let i = 0.0f64.max((u * self.im.width() as f64).min(self.im.width() as f64 - 1.0));

        let j = 0.0f64.max((v * self.im.height() as f64).min(self.im.height() as f64 - 1.0));

        let [r, g, b] = self.im.get_pixel(i as u32, j as u32).data;

        Vector3::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
    }

    fn box_clone(&self) -> Box<Texture> {
        Box::new((*self).clone())
    }
}
