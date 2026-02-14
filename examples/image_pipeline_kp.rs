//! Image processing pipeline with keypaths and parallel execution (rayon).
//!
//! Loads real images, validates via keypaths, and runs transforms in parallel:
//! - Grayscale, Brightness, Contrast
//! - Threshold (simple + Otsu)
//! - Box blur, Gaussian blur
//! - Sobel edge detection
//!
//! Run: `cargo run --example image_pipeline_kp --features image_pipeline`
//! Or:  `cargo run --example image_pipeline_kp --features image_pipeline -- path/to/image.png`

#![cfg(feature = "image_pipeline")]

use rayon::prelude::*;
use rust_key_paths::{Kp, KpType};
use std::path::Path;
use std::sync::OnceLock;

// ============================================================================
// Image type
// ============================================================================

#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub depth: usize,
    pub data: Vec<u8>,
}

// ============================================================================
// Keypath registry (all field access via Kp)
// ============================================================================

pub struct ImageKpRegistry {
    pub width: KpType<'static, Image, usize>,
    pub height: KpType<'static, Image, usize>,
    pub channels: KpType<'static, Image, usize>,
    pub depth: KpType<'static, Image, usize>,
    pub data: KpType<'static, Image, Vec<u8>>,
}

static KP_REGISTRY: OnceLock<ImageKpRegistry> = OnceLock::new();

impl ImageKpRegistry {
    pub fn get() -> &'static ImageKpRegistry {
        KP_REGISTRY.get_or_init(|| ImageKpRegistry {
            width: Kp::new(
                |img: &Image| Some(&img.width),
                |img: &mut Image| Some(&mut img.width),
            ),
            height: Kp::new(
                |img: &Image| Some(&img.height),
                |img: &mut Image| Some(&mut img.height),
            ),
            channels: Kp::new(
                |img: &Image| Some(&img.channels),
                |img: &mut Image| Some(&mut img.channels),
            ),
            depth: Kp::new(
                |img: &Image| Some(&img.depth),
                |img: &mut Image| Some(&mut img.depth),
            ),
            data: Kp::new(
                |img: &Image| Some(&img.data),
                |img: &mut Image| Some(&mut img.data),
            ),
        })
    }
}

// ============================================================================
// Validation (all reads via keypaths)
// ============================================================================

#[derive(Debug)]
pub enum PipelineError {
    InvalidDimensions { width: usize, height: usize },
    UnsupportedChannels(usize),
    UnsupportedDepth(usize),
    CorruptedBuffer { expected: usize, actual: usize },
    LoadError(String),
}

pub fn validate_image(img: &Image) -> Result<(), PipelineError> {
    let reg = ImageKpRegistry::get();
    let width = *reg.width.get(img).expect("width");
    let height = *reg.height.get(img).expect("height");
    let channels = *reg.channels.get(img).expect("channels");
    let depth = *reg.depth.get(img).expect("depth");
    let data = reg.data.get(img).expect("data");

    if width == 0 || height == 0 {
        return Err(PipelineError::InvalidDimensions { width, height });
    }
    if channels != 1 && channels != 3 {
        return Err(PipelineError::UnsupportedChannels(channels));
    }
    if depth != 8 {
        return Err(PipelineError::UnsupportedDepth(depth));
    }
    let expected = width * height * channels;
    if data.len() != expected {
        return Err(PipelineError::CorruptedBuffer {
            expected,
            actual: data.len(),
        });
    }
    Ok(())
}

// ============================================================================
// Load real image from file
// ============================================================================

#[cfg(feature = "image_pipeline_load")]
fn load_image(path: &Path) -> Result<Image, PipelineError> {
    let img = image::open(path).map_err(|e| PipelineError::LoadError(e.to_string()))?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let data: Vec<u8> = rgb.as_raw().to_vec();
    Ok(Image {
        width: w as usize,
        height: h as usize,
        channels: 3,
        depth: 8,
        data,
    })
}

/// Create a sample image (gradient) when no file is provided
fn create_sample_image() -> Image {
    let w = 128;
    let h = 128;
    let mut data = vec![0u8; w * h * 3];
    data.par_chunks_exact_mut(3)
        .enumerate()
        .for_each(|(i, px)| {
            let x = (i % w) as u8;
            let y = (i / w) as u8;
            px[0] = x.wrapping_add(y);
            px[1] = (255u16.saturating_sub(x as u16) as u8).wrapping_add(y);
            px[2] = y.wrapping_add(x);
        });
    Image {
        width: w,
        height: h,
        channels: 3,
        depth: 8,
        data,
    }
}

// ============================================================================
// Parallel transforms (all read via keypaths, process with rayon)
// ============================================================================

fn clamp(v: f64) -> u8 {
    v.clamp(0.0, 255.0).round() as u8
}

/// Grayscale: 0.299R + 0.587G + 0.114B
pub fn grayscale(img: &Image) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let reg = ImageKpRegistry::get();
    let width = *reg.width.get(img).unwrap();
    let height = *reg.height.get(img).unwrap();
    let channels = *reg.channels.get(img).unwrap();
    let data = reg.data.get(img).unwrap();

    let new_data: Vec<u8> = data
        .par_chunks_exact(channels)
        .map(|px| {
            let gray = if px.len() == 3 {
                (0.299 * px[0] as f64 + 0.587 * px[1] as f64 + 0.114 * px[2] as f64).round() as u8
            } else {
                px[0]
            };
            vec![gray]
        })
        .flat_map(|v| v)
        .collect();

    Ok(Image {
        width,
        height,
        channels: 1,
        depth: 8,
        data: new_data,
    })
}

/// Brightness: clamp(pixel + k, 0, 255)
pub fn brightness(img: &Image, k: i16) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let reg = ImageKpRegistry::get();
    let (width, height, channels, data) = (
        *reg.width.get(img).unwrap(),
        *reg.height.get(img).unwrap(),
        *reg.channels.get(img).unwrap(),
        reg.data.get(img).unwrap().clone(),
    );

    let new_data: Vec<u8> = data
        .par_iter()
        .map(|&c| clamp((c as i16).saturating_add(k) as f64))
        .collect();

    Ok(Image {
        width,
        height,
        channels,
        depth: 8,
        data: new_data,
    })
}

/// Contrast: (pixel - 128) * factor + 128
pub fn contrast(img: &Image, factor: f64) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let reg = ImageKpRegistry::get();
    let (width, height, channels, data) = (
        *reg.width.get(img).unwrap(),
        *reg.height.get(img).unwrap(),
        *reg.channels.get(img).unwrap(),
        reg.data.get(img).unwrap().clone(),
    );

    let new_data: Vec<u8> = data
        .par_iter()
        .map(|&c| clamp((c as f64 - 128.0) * factor + 128.0))
        .collect();

    Ok(Image {
        width,
        height,
        channels,
        depth: 8,
        data: new_data,
    })
}

/// Simple threshold: > T -> 255 else 0
pub fn threshold(img: &Image, t: u8) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let reg = ImageKpRegistry::get();
    let (width, height, channels, data) = (
        *reg.width.get(img).unwrap(),
        *reg.height.get(img).unwrap(),
        *reg.channels.get(img).unwrap(),
        reg.data.get(img).unwrap().clone(),
    );

    let new_data: Vec<u8> = data
        .par_iter()
        .map(|&c| if c > t { 255 } else { 0 })
        .collect();

    Ok(Image {
        width,
        height,
        channels,
        depth: 8,
        data: new_data,
    })
}

/// Otsu threshold: compute optimal T from histogram (parallel bucket count)
fn compute_histogram(data: &[u8]) -> [u32; 256] {
    let chunks: Vec<&[u8]> = data.par_chunks(data.len().max(1) / rayon::current_num_threads().max(1))
        .collect();
    let partials: Vec<[u32; 256]> = chunks
        .par_iter()
        .map(|chunk| {
            let mut h = [0u32; 256];
            for &p in *chunk {
                h[p as usize] += 1;
            }
            h
        })
        .collect();
    let mut hist = [0u32; 256];
    for p in partials {
        for (i, &c) in p.iter().enumerate() {
            hist[i] += c;
        }
    }
    hist
}

fn otsu_threshold_from_histogram(hist: &[u32; 256]) -> u8 {
    let total: u64 = hist.iter().map(|&x| x as u64).sum();
    let sum_total: u64 = hist
        .iter()
        .enumerate()
        .map(|(i, &h)| i as u64 * h as u64)
        .sum();

    let mut sum_bg: u64 = 0;
    let mut weight_bg: u64 = 0;
    let mut max_var = 0f64;
    let mut best_t = 0u8;

    for t in 0..255 {
        weight_bg += hist[t] as u64;
        if weight_bg == 0 {
            continue;
        }
        let weight_fg = total - weight_bg;
        if weight_fg == 0 {
            break;
        }
        sum_bg += t as u64 * hist[t] as u64;
        let mean_bg = sum_bg as f64 / weight_bg as f64;
        let mean_fg = (sum_total - sum_bg) as f64 / weight_fg as f64;
        let var = (weight_bg as f64) * (weight_fg as f64) * (mean_bg - mean_fg).powi(2);
        if var > max_var {
            max_var = var;
            best_t = t as u8;
        }
    }
    best_t
}

pub fn otsu_threshold(img: &Image) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let data = ImageKpRegistry::get().data.get(img).unwrap();
    let hist = compute_histogram(data);
    let t = otsu_threshold_from_histogram(&hist);
    threshold(img, t)
}

/// Box blur 3x3
fn safe_get(data: &[u8], w: usize, h: usize, ch: usize, x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return 0;
    }
    let idx = (y as usize * w + x as usize) * ch;
    data.get(idx).copied().unwrap_or(0)
}

pub fn box_blur(img: &Image) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let reg = ImageKpRegistry::get();
    let (w, h, ch, data) = (
        *reg.width.get(img).unwrap(),
        *reg.height.get(img).unwrap(),
        *reg.channels.get(img).unwrap(),
        reg.data.get(img).unwrap().as_slice(),
    );

    let new_data: Vec<u8> = (0..h)
        .into_par_iter()
        .flat_map(|y| {
            (0..w)
                .map(|x| {
                    let mut sum = [0u32; 3];
                    let mut count = 0u32;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                                for c in 0..ch {
                                    sum[c] += safe_get(data, w, h, ch, nx, ny) as u32;
                                }
                                count += 1;
                            }
                        }
                    }
                    (0..ch)
                        .map(|c| (sum[c] / count.max(1)) as u8)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .flat_map(|v| v)
        .collect();

    Ok(Image {
        width: w,
        height: h,
        channels: ch,
        depth: 8,
        data: new_data,
    })
}

/// Gaussian 3x3 kernel (σ≈1.0), normalized
const GAUSS_3: [[f64; 3]; 3] = [
    [1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0],
    [2.0 / 16.0, 4.0 / 16.0, 2.0 / 16.0],
    [1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0],
];

pub fn gaussian_blur(img: &Image) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let reg = ImageKpRegistry::get();
    let (w, h, ch, data) = (
        *reg.width.get(img).unwrap(),
        *reg.height.get(img).unwrap(),
        *reg.channels.get(img).unwrap(),
        reg.data.get(img).unwrap().as_slice(),
    );

    let new_data: Vec<u8> = (0..h)
        .into_par_iter()
        .flat_map(|y| {
            (0..w)
                .map(|x| {
                    let mut sum = [0f64; 3];
                    for (dy, row) in GAUSS_3.iter().enumerate() {
                        for (dx, &k) in row.iter().enumerate() {
                            let nx = x as i32 + dx as i32 - 1;
                            let ny = y as i32 + dy as i32 - 1;
                            for c in 0..ch {
                                sum[c] += k * safe_get(data, w, h, ch, nx, ny) as f64;
                            }
                        }
                    }
                    (0..ch).map(|c| clamp(sum[c])).collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .flat_map(|v| v)
        .collect();

    Ok(Image {
        width: w,
        height: h,
        channels: ch,
        depth: 8,
        data: new_data,
    })
}

/// Sobel kernels
const SOBEL_X: [[i16; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
const SOBEL_Y: [[i16; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

fn convolve_at(data: &[u8], w: usize, h: usize, x: usize, y: usize, kernel: &[[i16; 3]; 3]) -> f64 {
    let mut g = 0i32;
    for (dy, row) in kernel.iter().enumerate() {
        for (dx, &k) in row.iter().enumerate() {
            let nx = x as i32 + dx as i32 - 1;
            let ny = y as i32 + dy as i32 - 1;
            g += k as i32 * safe_get(data, w, h, 1, nx, ny) as i32;
        }
    }
    g as f64
}

pub fn sobel(img: &Image) -> Result<Image, PipelineError> {
    validate_image(img)?;
    let reg = ImageKpRegistry::get();
    let (w, h, _, data) = (
        *reg.width.get(img).unwrap(),
        *reg.height.get(img).unwrap(),
        *reg.channels.get(img).unwrap(),
        reg.data.get(img).unwrap().as_slice(),
    );
    if *reg.channels.get(img).unwrap() != 1 {
        let gray = grayscale(img)?;
        return sobel(&gray);
    }

    let new_data: Vec<u8> = (0..h)
        .into_par_iter()
        .flat_map(|y| {
            (0..w)
                .map(|x| {
                    let gx = convolve_at(data, w, h, x, y, &SOBEL_X);
                    let gy = convolve_at(data, w, h, x, y, &SOBEL_Y);
                    let mag = (gx * gx + gy * gy).sqrt();
                    clamp(mag)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(Image {
        width: w,
        height: h,
        channels: 1,
        depth: 8,
        data: new_data,
    })
}

// ============================================================================
// Pipeline composition
// ============================================================================

pub fn run_pipeline(img: Image) -> Result<(), PipelineError> {
    println!("  validate...");
    validate_image(&img)?;

    println!("  grayscale...");
    let g = grayscale(&img)?;
    validate_image(&g)?;

    println!("  brightness(+30)...");
    let b = brightness(&g, 30)?;

    println!("  contrast(1.2)...");
    let c = contrast(&b, 1.2)?;

    println!("  threshold(128)...");
    let _t = threshold(&c, 128)?;

    println!("  otsu_threshold...");
    let _o = otsu_threshold(&c)?;

    println!("  box_blur...");
    let _bb = box_blur(&g)?;

    println!("  gaussian_blur...");
    let _gb = gaussian_blur(&g)?;

    println!("  sobel...");
    let _s = sobel(&g)?;

    println!("  pipeline: grayscale -> contrast -> gaussian -> sobel...");
    let _p = grayscale(&img)
        .and_then(|g| contrast(&g, 1.1))
        .and_then(|c| gaussian_blur(&c))
        .and_then(|gb| sobel(&gb))?;

    Ok(())
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Result<(), PipelineError> {
    println!("=== Image Pipeline (Keypaths + Rayon) ===\n");

    let img = {
        #[cfg(feature = "image_pipeline_load")]
        {
            match std::env::args().nth(1) {
                Some(path) => {
                    let p = Path::new(&path);
                    println!("Loading: {}", p.display());
                    load_image(p)?
                }
                None => {
                    println!("No path given, using sample image (128x128 gradient)");
                    create_sample_image()
                }
            }
        }
        #[cfg(not(feature = "image_pipeline_load"))]
        {
            println!("Using sample image (128x128 gradient). For file loading, use --features image_pipeline_load");
            create_sample_image()
        }
    };

    println!("Image: {}x{}, {} channels\n", img.width, img.height, img.channels);
    println!("Running pipeline (parallel transforms via rayon):");
    run_pipeline(img)?;
    println!("\n✓ Pipeline complete");
    Ok(())
}
