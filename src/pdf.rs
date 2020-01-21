use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::Rng;

use basis::OrthonormalBasis;
use hitable::Hitable;
use sampling::uniform_sample_hemisphere;

pub enum PDF<'a> {
    CosinePDF {
        uvw: OrthonormalBasis,
    },
    HitablePDF {
        origin: Vec3,
        hitable: Arc<dyn Hitable>,
    },
    MixturePDF {
        cosine_pdf: &'a PDF<'a>,
        hitable_pdf: &'a PDF<'a>,
    },
}

impl<'a> PDF<'a> {
    pub fn value(&self, direction: Vec3) -> f32 {
        match self {
            PDF::CosinePDF { uvw } => {
                let cosine = direction.normalize().dot(uvw.w());

                if cosine > 0.0 {
                    cosine / PI
                } else {
                    0.0
                }
            }
            PDF::HitablePDF { origin, hitable } => hitable.pdf_value(*origin, direction),
            PDF::MixturePDF { cosine_pdf,
                              hitable_pdf, } => {
                0.5 * cosine_pdf.value(direction) + 0.5 * hitable_pdf.value(direction)
            }
        }
    }

    pub fn generate(&self, rng: &mut ThreadRng) -> Vec3 {
        match self {
            PDF::CosinePDF { uvw } => uvw.local(&uniform_sample_hemisphere(rng)),
            PDF::HitablePDF { origin, hitable } => hitable.pdf_random(*origin, rng),
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
