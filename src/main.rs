use aethertrace::film::{Film, Tonemap, auto_exposure};
use aethertrace::rng::Rng;
use aethertrace::scenes::{self, from_name};
use aethertrace::tracer::trace_pixel;
use aethertrace::vec3::Color;
use clap::Parser;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(
    name = "aethertrace",
    version,
    about = "AetherTrace — a premium physically-based path tracer in Rust.",
    long_about = None
)]
struct Cli {
    /// Scene to render: cornell | spheres | studio | sunset
    #[arg(short = 'c', long, default_value = "spheres")]
    scene: String,

    /// Image width in pixels.
    #[arg(short = 'w', long, default_value_t = 1280)]
    width: u32,

    /// Image height in pixels.
    #[arg(short = 'H', long, default_value_t = 800)]
    height: u32,

    /// Total samples per pixel.
    #[arg(short, long, default_value_t = 256)]
    samples: u64,

    /// Max path depth.
    #[arg(long, default_value_t = 16)]
    max_depth: usize,

    /// Threads to use (0 = all available).
    #[arg(long, default_value_t = 0)]
    threads: usize,

    /// Photographic exposure in stops (default: auto).
    #[arg(long, default_value_t = f32::NAN)]
    exposure: f32,

    /// Tonemapper: aces | neutral | none.
    #[arg(long, default_value = "aces")]
    tonemap: String,

    /// Output base path (final images written as <out>.png and <out>.hdr).
    #[arg(short, long, default_value = "render")]
    out: String,

    /// Number of progressive checkpoints.
    #[arg(long, default_value_t = 4)]
    passes: usize,

    /// RNG seed for reproducibility.
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Quick quality preset.
    #[arg(long, default_value = "")]
    preset: String,
}

fn main() {
    let cli = Cli::parse();

    // Apply quality presets.
    let (width, height, samples, passes) = match cli.preset.to_ascii_lowercase().as_str() {
        "fast" | "preview" => (640, 400, 64, 2),
        "final" | "high" => (1920, 1080, 2048, 4),
        "ultra" => (2560, 1440, 8192, 6),
        _ => (cli.width, cli.height, cli.samples, cli.passes),
    };

    if cli.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.threads)
            .build_global()
            .expect("failed to build thread pool");
    }

    let tonemap = match cli.tonemap.to_ascii_lowercase().as_str() {
        "aces" => Tonemap::Aces,
        "neutral" => Tonemap::Neutral,
        "none" | "raw" => Tonemap::None,
        other => {
            eprintln!("warning: unknown tonemapper '{other}', using aces");
            Tonemap::Aces
        }
    };

    let scene_name = from_name(&cli.scene).unwrap_or_else(|| {
        eprintln!("error: unknown scene '{}'. Choices: cornell, spheres, studio, sunset", cli.scene);
        std::process::exit(1);
    });

    let aspect = width as f32 / height as f32;
    let built = scenes::build(&scene_name, aspect);
    let total_samples: u64 = samples;
    let spp_per_pass = (total_samples / passes.max(1) as u64).max(1);
    let actual_passes = ((total_samples as usize).div_ceil(spp_per_pass as usize)).max(1);

    let mut film = Film::new(width as usize, height as usize);
    let scene = Arc::new(built.scene);
    let camera = Arc::new(built.camera);

    let width = width as usize;
    let height = height as usize;
    let n_pixels = width * height;

    println!("AetherTrace — premium path tracer");
    println!(
        "  scene={}  {}x{}  spp={}  threads={}  depth={}",
        cli.scene,
        width,
        height,
        total_samples,
        rayon::current_num_threads(),
        cli.max_depth,
    );
    println!("  output: {}.png / {}.hdr", cli.out, cli.out);

    let t0 = Instant::now();
    let pb = indicatif::ProgressBar::new((n_pixels * actual_passes) as u64);
    pb.set_style(
        indicatif::ProgressStyle::with_template(
            "{spinner:.cyan} {msg} [{bar:40.cyan/blue}] {percent:>3}% eta {eta} {per_sec}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let seed = cli.seed;

    for pass in 0..actual_passes {
        let pass_samples = spp_per_pass;
        pb.set_message(format!("pass {}/{}", pass + 1, actual_passes));

        let contributions: Vec<Color> = (0..n_pixels)
            .into_par_iter()
            .map(|idx| {
                let px = (idx % width) as u32;
                let py = (idx / width) as u32;
                let mut acc = Color::zero();
                let mut rng = Rng::from_u64(pixel_seed(px as u64, py as u64, pass as u64, seed));
                let u = (px as f32 + 0.5) / width as f32;
                let v = (py as f32 + 0.5) / height as f32;
                for _ in 0..pass_samples {
                    acc += trace_pixel(&scene, &camera, &mut rng, u, v, cli.max_depth);
                }
                acc
            })
            .map_with(pb.clone(), |pb, c| {
                pb.inc(1);
                c
            })
            .collect();

        for (idx, c) in contributions.iter().enumerate() {
            film.accumulator[idx] += *c;
        }
        film.sample_count += pass_samples;

        let exposure = if cli.exposure.is_nan() {
            auto_exposure(&film)
        } else {
            cli.exposure
        };

        let is_final = pass + 1 == actual_passes;
        if is_final {
            let png_path = format!("{}.png", cli.out);
            let hdr_path = format!("{}.hdr", cli.out);
            aethertrace::film::write_png(&png_path, &film, exposure, tonemap).expect("failed to write PNG");
            aethertrace::film::write_hdr(&hdr_path, &film).expect("failed to write HDR");
            println!("  saved {} (exposure {:+.2} stops)", png_path, exposure);
        } else {
            let path = format!("{}_pass{}.png", cli.out, pass + 1);
            aethertrace::film::write_png(&path, &film, exposure, tonemap).expect("failed to write PNG");
        }
    }

    pb.finish_with_message("done");
    let elapsed = t0.elapsed();
    let rays: f64 = (n_pixels * total_samples as usize) as f64;
    println!(
        "  rendered {} rays in {:.2}s — {:.1}M rays/s",
        rays as u64,
        elapsed.as_secs_f64(),
        rays / elapsed.as_secs_f64() / 1e6,
    );
}

#[inline(always)]
fn pixel_seed(x: u64, y: u64, pass: u64, seed: u64) -> u64 {
    let mut h = seed
        ^ x.wrapping_mul(0x9E3779B97F4A7C15)
        ^ y.wrapping_mul(0xC2B2AE3D27D4EB4F)
        ^ pass.wrapping_mul(0x165667B19E3779F9);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51AFD7ED558CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
    h ^ h >> 33
}
