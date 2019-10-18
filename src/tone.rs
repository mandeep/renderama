/// Tone map the given luminance globally with the adaptive logarithmic mapping algorithm
///
/// luminance is the pixel to map, max_luminance
/// is the smallest luminance that will be mapped to pure
/// white, display_luminance is the maximum luminance capability of the display,
/// and bias is term that adjusts compression of high values and visibility details
/// in dark ares.
///
/// Generally, max_luminance is set to the maximum luminance in the scene,
/// display_luminance is between 100-230 cd/m^2, and bias by default can be 0.73.
///
/// The tone mapping derivation can be found in Drago et al.
/// Adaptive Logarithmic Mapping for Displaying High Constrast Scenes.
pub fn adaptive_log_map(luminance: f32,
                        max_luminance: f32,
                        display_luminance: f32,
                        bias: f32)
                        -> f32 {
    let bias_term = bias.ln() / 0.5f32.ln();

    let first_term_numerator = display_luminance * 0.01;
    let first_term_denominator = (max_luminance + 1.0).log10();
    let first_term = first_term_numerator / first_term_denominator;

    let second_term_numerator = (luminance + 1.0).ln();
    let second_term_denominator = (2.0 + (luminance / max_luminance).powf(bias_term) * 8.0).ln();
    let second_term = second_term_numerator / second_term_denominator;

    first_term * second_term
}

/// Tone map the given luminance globally with the Stockham equation
///
/// luminance is the pixel to map and max_luminance
/// is the smallest luminance that will be mapped to pure
/// white. Generally, this luminance is set to the
/// maximum luminance in the scene.
///
/// The tone mapping derivation can be found in Stockham's paper
/// Image Processing in the Context of a Visual Model.
pub fn stockham_tone_map(luminance: f32, max_luminance: f32) -> f32 {
    (luminance + 1.0).ln() / (max_luminance + 1.0).ln()
}

/// Tone map the given luminance globally with the Reinhard equation
///
/// luminance is the pixel to map and max_luminance
/// is the smallest luminance that will be mapped to pure
/// white. Generally, this luminance is set to the
/// maximum luminance in the scene.
///
/// The tone mapping derivation can be found in the paper:
/// Photographic Tone Reproduction for Digital Images by
/// Reinhard et al.
pub fn reinhard_tone_map(luminance: f32, max_luminance: f32) -> f32 {
    if max_luminance > 1e20 {
        luminance / (1.0 + luminance)
    } else {
        (luminance * (1.0 + (luminance / max_luminance.powf(2.0)))) / (1.0 + luminance)
    }
}
