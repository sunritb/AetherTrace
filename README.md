# AetherTrace

A premium, physically-based path tracer written in Rust. Renders four built-in scenes with
next-event estimation (NEE), multiple importance sampling (MIS), a binned-SAH BVH, PBR metals,
dielectrics, procedural HDR skies, and tone-mapped progressive output — all multi-threaded with
Rayon.

## Features

- **Path tracing core**
  - Next-event estimation for area lights (uniform light sampling + power-heuristic MIS)
  - MIS with the environment map; Russian-roulette path termination
  - Correct MIS-weighted contributions when BSDF-sampled rays hit area lights (specular highlights)
  - Progressive rendering with interactive checkpoints
- **Materials**
  - Lambert, checkerboard, emissive
  - GGX PBR (metallic + roughness, Schlick Fresnel, Smith masking/shadowing)
  - Dielectric (Fresnel reflection/refraction, total internal reflection)
- **Acceleration**
  - Binned surface-area-heuristic (SAH) BVH with in-place primitive partition
  - Verified against brute-force traversal (0 mismatches on all scenes)
- **Camera & environment**
  - Pinhole / depth-of-field, arbitrary look-at
  - Procedural HDR sky with sun disk + analytic sampling, or constant/studio env
- **Output & CLI**
  - PNG (ACES / neutral / none tonemapping, auto or manual exposure) and Radiance `.hdr`
  - Parallel rendering, quality presets, reproducible seeds

## Usage

```
aethertrace [OPTIONS]
```

```
Usage: aethertrace [OPTIONS]

Options:
  -c, --scene <SCENE>          Scene to render: cornell | spheres | studio | sunset [default: spheres]
  -w, --width <WIDTH>          Image width in pixels [default: 1280]
  -H, --height <HEIGHT>        Image height in pixels [default: 800]
  -s, --samples <SAMPLES>      Total samples per pixel [default: 256]
      --max-depth <MAX_DEPTH>  Max path depth [default: 16]
      --threads <THREADS>      Threads to use (0 = all available) [default: 0]
      --exposure <EXPOSURE>    Photographic exposure in stops (default: auto) [default: NaN]
      --tonemap <TONEMAP>      Tonemapper: aces | neutral | none [default: aces]
  -o, --out <OUT>              Output base path (final images written as <out>.png and <out>.hdr) [default: render]
      --passes <PASSES>        Number of progressive checkpoints [default: 4]
      --seed <SEED>            RNG seed for reproducibility [default: 42]
      --preset <PRESET>        Quick quality preset [default: ""]
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

```sh
# Quick preview
aethertrace --preset fast -c cornell

# Final-quality render of the studio scene
aethertrace -c studio -w 1920 -H 1080 -s 2048 --passes 4

# Ultra-quality sunset, custom output, fixed exposure
aethertrace -c sunset --preset ultra -o out/sunset --exposure 1.0

# Reproducible render with a fixed seed
aethertrace -c spheres --seed 12345 -s 4096
```

### Presets

| Preset        | Resolution | Samples | Passes |
|---------------|-----------|---------|--------|
| `fast`        | 640×400   | 64      | 2      |
| `final`/`high`| 1920×1080 | 2048    | 4      |
| `ultra`       | 2560×1440 | 8192    | 6      |

## Scenes

- **cornell** — classic Cornell box: white/red/green room, emissive ceiling panel, gold box,
  rotated gold box, glass sphere.
- **spheres** — checkerboard ground with glass, blue, gold, chrome, and plastic spheres under
  a procedural sky.
- **studio** — dark product studio: near-black PBR floor and backdrop, key softbox, cool rim
  lights, glass/gold/red spheres and a black plastic box.
- **sunset** — wide outdoor scene: warm sun disk, rocky ground, metal box, glass and red spheres.

## Verification

`examples/bvh_verify.rs` compares the BVH against brute-force traversal for both closest-hit and
shadow rays across 20 000 random rays per scene — all four scenes report **0 mismatches**,
including the tie-break on shared quad diagonals (lowest primitive index wins).

```sh
cargo run --release --example bvh_verify
```

## Building

Requires Rust (edition 2024).

```sh
cargo build --release
```

## License

MIT — see [LICENSE](LICENSE).
