use std::f32::consts::PI;

use glam::Vec3;
use rand::rngs::ThreadRng;
use rand::Rng;

use basis::OrthonormalBasis;
use geometry::Geometry;
use sampling::uniform_sample_hemisphere;

pub enum MaterialPDF {
    Cosine { uvw: OrthonormalBasis },
    Importance { origin: Vec3, geometry: Geometry },
    Fallback { uvw: OrthonormalBasis }
}

impl MaterialPDF {
    pub fn value(&self, direction: Vec3) -> f32 {
        match self {
            MaterialPDF::Cosine { uvw } => {
                let cosine = direction.normalize().dot(uvw.w());
                if cosine > 0.0 { cosine / PI } else { 0.0 }
            }
            MaterialPDF::Importance { origin, geometry } => {
                geometry.pdf_value(*origin, direction)
            }
            MaterialPDF::Fallback { uvw } => {
                let cosine = direction.normalize().dot(uvw.w());
                if cosine > 0.0 { cosine / PI } else { 0.0 }
            }
        }
    }

    pub fn generate(&self, rng: &mut ThreadRng) -> Vec3 {
        match self {
            MaterialPDF::Cosine { uvw } => {
                uvw.local(&uniform_sample_hemisphere(rng))
            }
            MaterialPDF::Importance { origin, geometry } => {
                geometry.pdf_random(*origin, rng)
            }
            MaterialPDF::Fallback { uvw } => {
                uvw.local(&uniform_sample_hemisphere(rng))
            }
        }
    }
}

pub struct HybridPDF<'a> {
        material_pdf: &'a MaterialPDF,
        importance_pdf: &'a MaterialPDF,
}

impl<'a> HybridPDF<'a> {
    pub fn new(material_pdf: &'a MaterialPDF, importance_pdf: &'a MaterialPDF) -> HybridPDF<'a> {
        HybridPDF { material_pdf, importance_pdf }
    }

    pub fn value(&self, direction: Vec3) -> f32 {
        0.5 * self.material_pdf.value(direction) + 0.5 * self.importance_pdf.value(direction)
    }

    pub fn generate(&self, rng: &mut ThreadRng) -> Vec3 {
        if rng.gen::<f32>() < 0.5 {
            self.material_pdf.generate(rng)
        } else {
            self.importance_pdf.generate(rng)
        }
    }
}