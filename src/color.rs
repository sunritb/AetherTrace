use crate::vec3::Color;

/// Linear-sRGB to luminance (Rec. 709).
#[inline(always)]
pub fn luminance(c: Color) -> f32 {
    0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z
}

/// ACES Filmic tone mapping (Narkowicz).
#[inline(always)]
pub fn aces(x: f32) -> f32 {
    const A: f32 = 2.51;
    const B: f32 = 0.03;
    const C: f32 = 2.43;
    const D: f32 = 0.59;
    const E: f32 = 0.14;
    ((x * (A * x + B)) / (x * (C * x + D) + E)).clamp(0.0, 1.0)
}

/// Khronos PBR-neutral tonemapper — filmic, keeps grays linear.
#[inline(always)]
pub fn neutral(x: f32) -> f32 {
    const A: f32 = 0.2;
    const B: f32 = 0.29;
    const C: f32 = 0.24;
    const D: f32 = 0.272;
    const E: f32 = 0.02;
    const F: f32 = 0.3;
    const W: f32 = 11.2;
    let w = (W * (A * W + C * B) + D * E) / (W * (A * W + B) + C * W + D * F) - E / F;
    let base = (x * (A * x + C * B) + D * E) / (x * (A * x + B) + C * x + D * F);
    ((base - E / F) / w).clamp(0.0, 1.0)
}

/// Gamma-encode a linear value (sRGB transfer function).
#[inline(always)]
pub fn gamma_encode(x: f32) -> f32 {
    if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

/// Exposure control: photographic stops, scaled so ~1.0 is mid-grey.
#[inline(always)]
pub fn expose(c: Color, stops: f32) -> Color {
    c * 2f32.powf(stops)
}
