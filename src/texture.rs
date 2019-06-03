use nalgebra::core::Vector3;


pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: &Vector3<f64>) -> Vector3<f64>;
    fn box_clone(&self) -> Box<Texture>;
}


impl Clone for Box<Texture> {
    fn clone(&self) -> Box<Texture> {
        self.box_clone()
    }
}


#[derive(Clone)]
pub struct ConstantTexture {
    color: Vector3<f64>
}


impl ConstantTexture {
    pub fn new(r: f64, g: f64, b: f64) -> ConstantTexture {
        ConstantTexture {color: Vector3::new(r, g, b)}
    }
}


impl Texture for ConstantTexture {
    fn value(&self, u: f64, v: f64, p: &Vector3<f64>) -> Vector3<f64> {
        self.color
    }

    fn box_clone(&self) -> Box<Texture> {
        Box::new((*self).clone())
    }
}


#[derive(Clone)]
pub struct ImageTexture {
    data: Vec<u8>,
    nx: u32,
    ny: u32
}


impl ImageTexture {
    pub fn new(data: Vec<u8>, nx: u32, ny: u32) -> ImageTexture {
        ImageTexture {data: data, nx: nx, ny: ny}
    }
}


impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, p: &Vector3<f64>) -> Vector3<f64> {
        let i = 0.0f64.max((u * self.nx as f64).min(self.nx as f64 - 1.0));
        let j = 0.0f64.max((v * self.ny as f64).min(self.ny as f64 - 1.0));

        let index = 3.0 * i + 3.0 * self.nx as f64 * j;
        let r = self.data[index as usize] as f64 / 255.0;
        let g = self.data[index as usize + 1] as f64 / 255.0;
        let b = self.data[index as usize + 2] as f64 / 255.0;

        Vector3::new(r, g, b)
    }

    fn box_clone(&self) -> Box<Texture> {
        Box::new((*self).clone())
    }
}
