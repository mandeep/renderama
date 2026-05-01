use f32::consts::PI;

use glam::Vec3;
use image;

#[derive(Clone)]
pub enum Texture {
    SolidColor(SolidColor),
    ImageTexture(ImageTexture),
    EnvironmentMap(EnvironmentMap),
}

impl Texture {
    pub fn value(&self, u: f32, v: f32, w: &Vec3) -> Vec3 {
        match self {
            Texture::SolidColor(texture) => texture.value(u, v, w),
            Texture::ImageTexture(texture) => texture.value(u, v, w),
            Texture::EnvironmentMap(texture) => texture.value(u, v, w),
        }
    }
}

macro_rules! impl_from_texture {
    ($($t:ty => $v:ident),*) => {
        $(
            impl From<$t> for Texture {
                fn from(m: $t) -> Self {
                    Texture::$v(m)
                }
            }
        )*
    };
}

impl_from_texture!(
    SolidColor => SolidColor,
    ImageTexture => ImageTexture,
    EnvironmentMap => EnvironmentMap
);

#[derive(Clone)]
/// SolidColor is just a wrapping for a Vec3 of RGB values
pub struct SolidColor {
    color: Vec3,
}

/// Create a new SolidColor
impl SolidColor {
    pub fn new(r: f32, g: f32, b: f32) -> SolidColor {
        SolidColor { color: Vec3::new(r, g, b) }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn value(&self, _u: f32, _v: f32, _p: &Vec3) -> Vec3 {
        self.color
    }
}

#[derive(Clone)]
/// ImageTexture is a struct for textures loaded from file
pub struct ImageTexture {
    im: image::RgbImage,
    scale: f32,
}

/// Create a new texture from the given data and image dimensions
impl ImageTexture {
    pub fn new(filename: &str, scale: f32) -> ImageTexture {
        ImageTexture { im: image::open(filename).unwrap().flipv().to_rgb8(), scale}
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn value(&self, u: f32, v: f32, _p: &Vec3) -> Vec3 {
        let u_scaled = (u * self.scale) % 1.0;
        let v_scaled = (v * self.scale) % 1.0;

        let i = 0.0f32.max((u_scaled * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v_scaled * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}

/// EnvironmentMap is a struct for loading an environment map from an EXR file
#[derive(Clone)]
pub struct EnvironmentMap {
    im: image::Rgb32FImage,
}

/// Create a new map from the given data and image dimensions
impl EnvironmentMap {
    pub fn new(filename: &str) -> EnvironmentMap {
        EnvironmentMap { im: image::open(filename).unwrap().flipv().to_rgb32f() }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn value(&self, _u: f32, _v: f32, direction: &Vec3) -> Vec3 {
        let u = 0.5 + direction.z.atan2(direction.x) / (2.0 * PI);
        let v = 0.5 - direction.y.asin() / PI;

        let i = 0.0f32.max((u * self.im.width() as f32).min(self.im.width() as f32 - 1.0));
        let j = 0.0f32.max((v * self.im.height() as f32).min(self.im.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.im.get_pixel(i as u32, j as u32);

        Vec3::new(r, g, b)
    }
}
