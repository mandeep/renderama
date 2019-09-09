use std::f32::consts::PI;
use std::sync::Arc;

use nalgebra::Vector3;
use rand::Rng;
use rand::rngs::ThreadRng;

use basis::OrthonormalBase;
use hitable::Hitable;

pub fn random_cosine_direction(rng: &mut ThreadRng) -> Vector3<f32> {
    let r1 = rng.gen::<f32>();
    let r2 = rng.gen::<f32>();
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * PI * r1;
    let x = phi.cos() * 2.0 * r2.sqrt();
    let y = phi.sin() * 2.0 * r2.sqrt();
    Vector3::new(x, y, z)
}

pub enum PDF<'a> {
    CosinePDF {
        uvw: OrthonormalBase,
    },
    HitablePDF {
        origin: Vector3<f32>,
        hitable: Arc<dyn Hitable>,
    },
    MixturePDF {
        cosine_pdf: &'a PDF<'a>,
        hitable_pdf: &'a PDF<'a>,
    },
}

impl<'a> PDF<'a> {
    pub fn value(&self, direction: &Vector3<f32>) -> f32 {
        match self {
            PDF::CosinePDF { uvw } => {
                let cosine = direction.normalize().dot(&uvw.w());

                if cosine > 0.0 {
                    cosine / PI
                } else {
                    0.0
                }
            }
            PDF::HitablePDF { origin, hitable } => hitable.pdf_value(origin, direction),
            PDF::MixturePDF { cosine_pdf,
                              hitable_pdf, } => {
                0.5 * cosine_pdf.value(&direction) + 0.5 * hitable_pdf.value(direction)
            }
        }
    }

    pub fn generate(&self, rng: &mut ThreadRng) -> Vector3<f32> {
        match self {
            PDF::CosinePDF { uvw } => uvw.local(&random_cosine_direction(rng)),
            PDF::HitablePDF { origin, hitable } => hitable.pdf_random(&origin, rng),
            PDF::MixturePDF { cosine_pdf,
                              hitable_pdf, } => {
                if rng.gen::<f32>() < 0.5 {
                    cosine_pdf.generate(rng)
                } else {
                    hitable_pdf.generate(rng)
                }
            }
        }
    }
}
