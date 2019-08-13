use std::f32::consts::PI;
use std::sync::Arc;

use nalgebra::Vector3;
use rand::Rng;

use basis::OrthonormalBase;
use hitable::Hitable;

pub fn random_cosine_direction(rng: &mut rand::rngs::ThreadRng) -> Vector3<f32> {
    let r1 = rng.gen::<f32>();
    let r2 = rng.gen::<f32>();
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * PI * r1;
    let x = phi.cos() * 2.0 * r2.sqrt();
    let y = phi.sin() * 2.0 * r2.sqrt();
    Vector3::new(x, y, z)
}

pub struct CosinePDF {
    uvw: OrthonormalBase,
}

impl CosinePDF {
    pub fn new(normal: &Vector3<f32>) -> CosinePDF {
        CosinePDF { uvw: OrthonormalBase::new(&normal) }
    }

    pub fn value(&self, direction: &Vector3<f32>) -> f32 {
        let cosine = direction.normalize().dot(&self.uvw.w());

        if cosine > 0.0 {
            cosine / PI
        } else {
            0.0
        }
    }

    pub fn generate(&self, rng: &mut rand::rngs::ThreadRng) -> Vector3<f32> {
        self.uvw.local(&random_cosine_direction(rng))
    }
}

pub struct HitablePDF {
    origin: Vector3<f32>,
    hitable: Arc<dyn Hitable>,
}

impl HitablePDF {
    pub fn new<H: Hitable + 'static>(origin: Vector3<f32>, hitable: H) -> HitablePDF {
        let hitable = Arc::new(hitable);
        HitablePDF { origin, hitable }
    }

    pub fn value(&self, direction: &Vector3<f32>) -> f32 {
        self.hitable.pdf_value(&self.origin, direction)
    }

    pub fn generate(&self, rng: &mut rand::rngs::ThreadRng) -> Vector3<f32> {
        self.hitable.pdf_random(&self.origin, rng)
    }
}

pub struct MixturePDF {
    cosine_pdf: CosinePDF,
    hitable_pdf: HitablePDF,
}

impl MixturePDF {
    pub fn new(cosine_pdf: CosinePDF, hitable_pdf: HitablePDF) -> MixturePDF {
        MixturePDF { cosine_pdf, hitable_pdf }
    }

    pub fn value(&self, direction: &Vector3<f32>) -> f32 {
        0.5 * self.cosine_pdf.value(&direction) + 0.5 * self.hitable_pdf.value(direction)
    }

    pub fn generate(&self, rng: &mut rand::rngs::ThreadRng) -> Vector3<f32> {
        if rng.gen::<f32>() < 0.5 {
            self.cosine_pdf.generate(rng)
        } else {
            self.hitable_pdf.generate(rng)
        }
    }
}
