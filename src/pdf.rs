use std::f32::consts::PI;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::Rng;

use basis::OrthonormalBasis;
use geometry::Geometry;
use sampling::uniform_sample_hemisphere;

pub enum PDF<'a> {
    MaterialPDF {
        uvw: OrthonormalBasis,
    },
    ImportancePDF {
        origin: Vec3,
        geometry: Geometry,
    },
    HybridPDF {
        material_pdf: &'a PDF<'a>,
        importance_pdf: &'a PDF<'a>,
    },
    FallbackPDF {
        uvw: OrthonormalBasis
    }
}

impl<'a> PDF<'a> {
    pub fn value(&self, direction: Vec3) -> f32 {
        match self {
            PDF::MaterialPDF { uvw } => {
                let cosine = direction.normalize().dot(uvw.w());

                if cosine > 0.0 {
                    cosine / PI
                } else {
                    0.0
                }
            }
            PDF::ImportancePDF { origin, geometry } => geometry.pdf_value(*origin, direction),
            PDF::HybridPDF { material_pdf, importance_pdf, } => {
                0.5 * material_pdf.value(direction) + 0.5 * importance_pdf.value(direction)
            },
            PDF::FallbackPDF { uvw } => {
                let cosine = direction.normalize().dot(uvw.w());

                if cosine > 0.0 {
                    cosine / PI
                } else {
                    0.0
                }
            }
        }
    }

    pub fn generate(&self, rng: &mut ThreadRng) -> Vec3 {
        match self {
            PDF::MaterialPDF { uvw } => uvw.local(&uniform_sample_hemisphere(rng)),
            PDF::ImportancePDF { origin, geometry } => geometry.pdf_random(*origin, rng),
            PDF::HybridPDF { material_pdf,
                              importance_pdf, } => {
                if rng.gen::<f32>() < 0.5 {
                    material_pdf.generate(rng)
                } else {
                    importance_pdf.generate(rng)
                }
            },
            PDF::FallbackPDF { uvw } => uvw.local(&uniform_sample_hemisphere(rng)),
        }
    }
}
