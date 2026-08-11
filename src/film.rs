use crate::color::{aces, expose, gamma_encode, luminance, neutral};
use crate::vec3::{Color, Vec3};

pub struct Film {
    pub width: usize,
    pub height: usize,
    pub accumulator: Vec<Color>,
    pub sample_count: u64,
}

impl Film {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            accumulator: vec![Vec3::zero(); width * height],
            sample_count: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tonemap {
    Aces,
    Neutral,
    None,
}

/// Export the film as a tonemapped sRGB PNG.
pub fn write_png(path: &str, film: &Film, exposure: f32, tonemap: Tonemap) -> std::io::Result<()> {
    let w = film.width;
    let h = film.height;
    let inv = 1.0 / film.sample_count.max(1) as f32;

    let mut data = Vec::with_capacity(w * h * 3);
    for c in &film.accumulator {
        let e = expose(*c * inv, exposure);
        let rgb = match tonemap {
            Tonemap::Aces => Vec3::new(aces(e.x), aces(e.y), aces(e.z)),
            Tonemap::Neutral => Vec3::new(neutral(e.x), neutral(e.y), neutral(e.z)),
            Tonemap::None => e,
        };
        data.push(to_u8(rgb.x));
        data.push(to_u8(rgb.y));
        data.push(to_u8(rgb.z));
    }

    let file = std::fs::File::create(path)?;
    let w_out = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w_out, w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;
    Ok(())
}

/// Export the film as a Radiance (.hdr) RGBE file.
pub fn write_hdr(path: &str, film: &Film) -> std::io::Result<()> {
    use std::io::Write;
    let w = film.width;
    let h = film.height;
    let inv = 1.0 / film.sample_count.max(1) as f32;

    let mut bytes = Vec::with_capacity(w * h * 4);
    for c in &film.accumulator {
        let (r, g, b) = (c.x * inv, c.y * inv, c.z * inv);
        let max = r.max(g).max(b);
        if max < 1e-32 {
            bytes.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        let e = (max.log2().ceil()).clamp(-128.0, 127.0);
        let scale = (2f32.powf(e - 8.0)).recip();
        bytes.push(to_byte8(r * scale));
        bytes.push(to_byte8(g * scale));
        bytes.push(to_byte8(b * scale));
        bytes.push((e + 128.0) as u8);
    }

    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(f, "#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n")?;
    writeln!(f, "-Y {} +X {}", h, w)?;
    f.write_all(&bytes)?;
    Ok(())
}

#[inline(always)]
fn to_u8(x: f32) -> u8 {
    (gamma_encode(x.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8
}

#[inline(always)]
fn to_byte8(x: f32) -> u8 {
    (x.clamp(0.0, 255.0) + 0.5) as u8
}

/// Auto-exposure: map average luminance toward mid-grey.
pub fn auto_exposure(film: &Film) -> f32 {
    let inv = 1.0 / film.sample_count.max(1) as f32;
    let mut sum = 0.0;
    let n = film.accumulator.len();
    let mut i = 0usize;
    while i < n {
        let c = film.accumulator[i] * inv;
        sum += luminance(c);
        i += 1;
    }
    let avg = sum / n.max(1) as f32;
    let log2_avg = (avg.max(1e-6)).log2();
    (0.8 - log2_avg).clamp(-4.0, 6.0)
}
