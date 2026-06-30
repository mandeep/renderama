use glam::{Vec2, Vec3A};
use image::{DynamicImage, RgbImage};

#[derive(Clone, Copy)]
/// TextureId is used to index into the textures Vec that is instantiated
/// at scene creation.
///
/// Keeping track of the index instead of the actual texture
/// saves from allocating memory unnecessarily.
pub struct TextureId(pub u32);

impl TextureId {
    /// Create a new TextureId with material at index
    pub fn new(index: u32) -> TextureId {
        TextureId(index)
    }

    /// Retrieve the TextureId as a usize for indexing purposes
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[macro_export]
macro_rules! tex {
    ($vec:expr, $texture:expr) => {{
        let id = $crate::texture::TextureId::new($vec.len() as u32);
        $vec.push($texture.into());
        id
    }};
}

#[derive(Clone, Copy)]
pub enum TextureEncoding {
    Srgb,
    Linear,
}

#[derive(Clone)]
/// Housing items here such as Color and ImageTexture allows us to pass
/// Texture as the type for the albedo in materials.
pub enum Texture {
    Color(Color),
    ImageTexture(ImageTexture),
}

impl Texture {
    pub fn sample_texture(&self, u: f32, v: f32) -> Vec3A {
        match self {
            Texture::Color(texture) => texture.sample_texture(),
            Texture::ImageTexture(texture) => texture.sample_texture(u, v),
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
    Color => Color,
    ImageTexture => ImageTexture
);

#[derive(Copy, Clone)]
/// Color is just a wrapping for a Vec3A of RGB values
pub struct Color {
    color: Vec3A,
}

/// Create a new Color which is used for pure albedo materials
impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { color: Vec3A::new(r, g, b) }
    }

    pub fn splat(value: f32) -> Color {
        Color { color: Vec3A::splat(value) }
    }

    /// Returning the albedo instead of sampling with u and v
    pub fn sample_texture(&self) -> Vec3A {
        self.color
    }
}

#[derive(Clone)]
/// ImageTexture is a struct for textures loaded from file
pub struct ImageTexture {
    img: image::RgbImage,
    scale: Vec2,
    encoding: TextureEncoding
}

/// Create a new texture from the given data and image dimensions
impl ImageTexture {
    /// Load an image on disk into memory as an RGB8.
    fn load_image(filename: &str) -> RgbImage {
        image::open(filename).unwrap_or_else(|e| {
            eprintln!("Failed to open texture '{}': {}", filename, e);
            DynamicImage::new_rgb8(1, 1)
        }).to_rgb8()
    }

    /// Construct a new ImageTexture.
    ///
    /// This constructor is no longer used but is kept for backwards compatibility.
    /// The srgb or linear constructors should be used depending on which texture
    /// encoding is desired.
    pub fn new(filename: &str, scale: Vec2) -> ImageTexture {
        let img = Self::load_image(filename);

        ImageTexture { img, scale, encoding: TextureEncoding::Srgb }
    }

    /// Create a new ImageTexture with sRGB encoding.
    ///
    /// This constructor is used for diffuse maps and emission maps
    /// and lets the sampler know to convert from sRGB to linear.
    pub fn srgb(filename: &str, scale: Vec2) -> ImageTexture {
        let img = Self::load_image(filename);

        ImageTexture { img, scale, encoding: TextureEncoding::Srgb }
    }

    /// Create a new ImageTexture with Linar encoding.
    ///
    /// This constructor is used for normal maps, roughness maps,
    /// etc. It keeps the texture encoding as is and only casts
    /// from f32 to u8.
    pub fn linear(filename: &str, scale: Vec2) -> ImageTexture {
        let img = Self::load_image(filename);
        ImageTexture { img, scale, encoding: TextureEncoding::Linear }
    }

    /// Determine which pixel to retrieve from the image by
    /// converting pixel coordinates to UV coordinates
    pub fn sample_texture(&self, u: f32, v: f32) -> Vec3A {
        // need to use rem_euclid to properly wrap negative uv coordinates
        // https://doc.rust-lang.org/std/primitive.f32.html#method.rem_euclid
        let inverted_v = 1.0 - v;
        let u_scaled = (u * self.scale.x).rem_euclid(1.0);
        let v_scaled = (inverted_v * self.scale.y).rem_euclid(1.0);

        let i = 0.0f32.max((u_scaled * self.img.width() as f32).min(self.img.width() as f32 - 1.0));
        let j = 0.0f32.max((v_scaled * self.img.height() as f32).min(self.img.height() as f32 - 1.0));

        let image::Rgb([r, g, b]) = *self.img.get_pixel(i as u32, j as u32);

        let (r, g, b) = match self.encoding {
            TextureEncoding::Srgb => {
                // need to convert from srgb color space to linear as the
                // renderer computes color in linear color space
                let linear_r = (r as f32 / 255.0).powf(2.2);
                let linear_g = (g as f32 / 255.0).powf(2.2);
                let linear_b = (b as f32 / 255.0).powf(2.2);

                (linear_r, linear_g, linear_b)
            },
            TextureEncoding::Linear => {
                (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
            }
        };

        Vec3A::new(r, g, b)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splat() {
        let color = Color::splat(0.375);
        let sample = color.sample_texture();

        assert_eq!(sample.x, 0.375);
        assert_eq!(sample.y, 0.375);
        assert_eq!(sample.z, 0.375);
    }
}