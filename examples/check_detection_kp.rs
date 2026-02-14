//! Check detection using keypaths: static initialization and validation.
//!
//! Demonstrates:
//! 1. **Static keypath init** — OnceLock<KpType> for fast repeated access
//! 2. **Check/validation** — validate struct fields through keypaths
//! 3. **Registry pattern** — group keypaths for one-shot init
//!
//! Run: `cargo run --example check_detection_kp`

use rust_key_paths::{Kp, KpType};
use std::sync::OnceLock;

// ============================================================================
// Image type (validation target)
// ============================================================================

#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub depth: usize,
    pub data: Vec<u8>,
    /// Optional: if present, must be non-empty
    pub color_profile: Option<String>,
    /// Optional: if present, must be in 1..=1200
    pub dpi: Option<u32>,
    /// Optional: if present, each tag must be non-empty, max 64 tags
    pub tags: Option<Vec<String>>,
}

// ============================================================================
// Static keypaths via OnceLock
//
// First call: initializes and caches the keypath (heap alloc once)
// Subsequent calls: one atomic load + fn-ptr call — no allocation
// ============================================================================

static KP_WIDTH: OnceLock<KpType<'static, Image, usize>> = OnceLock::new();
static KP_HEIGHT: OnceLock<KpType<'static, Image, usize>> = OnceLock::new();
static KP_CHANNELS: OnceLock<KpType<'static, Image, usize>> = OnceLock::new();
static KP_DEPTH: OnceLock<KpType<'static, Image, usize>> = OnceLock::new();
static KP_DATA: OnceLock<KpType<'static, Image, Vec<u8>>> = OnceLock::new();
static KP_COLOR_PROFILE: OnceLock<KpType<'static, Image, Option<String>>> = OnceLock::new();
static KP_DPI: OnceLock<KpType<'static, Image, Option<u32>>> = OnceLock::new();
static KP_TAGS: OnceLock<KpType<'static, Image, Option<Vec<String>>>> = OnceLock::new();

pub fn kp_width() -> &'static KpType<'static, Image, usize> {
    KP_WIDTH.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.width),
            |img: &mut Image| Some(&mut img.width),
        )
    })
}

pub fn kp_height() -> &'static KpType<'static, Image, usize> {
    KP_HEIGHT.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.height),
            |img: &mut Image| Some(&mut img.height),
        )
    })
}

pub fn kp_channels() -> &'static KpType<'static, Image, usize> {
    KP_CHANNELS.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.channels),
            |img: &mut Image| Some(&mut img.channels),
        )
    })
}

pub fn kp_depth() -> &'static KpType<'static, Image, usize> {
    KP_DEPTH.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.depth),
            |img: &mut Image| Some(&mut img.depth),
        )
    })
}

pub fn kp_data() -> &'static KpType<'static, Image, Vec<u8>> {
    KP_DATA.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.data),
            |img: &mut Image| Some(&mut img.data),
        )
    })
}

pub fn kp_color_profile() -> &'static KpType<'static, Image, Option<String>> {
    KP_COLOR_PROFILE.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.color_profile),
            |img: &mut Image| Some(&mut img.color_profile),
        )
    })
}

pub fn kp_dpi() -> &'static KpType<'static, Image, Option<u32>> {
    KP_DPI.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.dpi),
            |img: &mut Image| Some(&mut img.dpi),
        )
    })
}

pub fn kp_tags() -> &'static KpType<'static, Image, Option<Vec<String>>> {
    KP_TAGS.get_or_init(|| {
        Kp::new(
            |img: &Image| Some(&img.tags),
            |img: &mut Image| Some(&mut img.tags),
        )
    })
}

// ============================================================================
// Registry: group all keypaths — one atomic load for all fields
// ============================================================================

pub struct ImageKpRegistry {
    pub width: KpType<'static, Image, usize>,
    pub height: KpType<'static, Image, usize>,
    pub channels: KpType<'static, Image, usize>,
    pub depth: KpType<'static, Image, usize>,
    pub data: KpType<'static, Image, Vec<u8>>,
    pub color_profile: KpType<'static, Image, Option<String>>,
    pub dpi: KpType<'static, Image, Option<u32>>,
    pub tags: KpType<'static, Image, Option<Vec<String>>>,
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
            color_profile: Kp::new(
                |img: &Image| Some(&img.color_profile),
                |img: &mut Image| Some(&mut img.color_profile),
            ),
            dpi: Kp::new(
                |img: &Image| Some(&img.dpi),
                |img: &mut Image| Some(&mut img.dpi),
            ),
            tags: Kp::new(
                |img: &Image| Some(&img.tags),
                |img: &mut Image| Some(&mut img.tags),
            ),
        })
    }
}

// ============================================================================
// Check detection: validation errors
// ============================================================================

#[derive(Debug)]
pub enum CheckError {
    InvalidDimensions { width: usize, height: usize },
    UnsupportedChannels(usize),
    UnsupportedDepth(usize),
    CorruptedBuffer { expected: usize, actual: usize },
    /// Optional color_profile present but empty
    EmptyColorProfile,
    /// Optional dpi out of range (1..=1200)
    InvalidDpi(u32),
    /// Optional tags: empty tag or too many (max 64)
    InvalidTags { index: usize },
    TooManyTags { count: usize },
}

// ============================================================================
// Check detection: validate Image via static keypaths
//
// All field reads go through statically cached keypaths — zero allocation
// after the first call to ImageKpRegistry::get().
// ============================================================================

pub fn check_image(img: &Image) -> Result<(), CheckError> {
    let reg = ImageKpRegistry::get();

    let width = *reg.width.get(img).expect("width is a required field");
    let height = *reg.height.get(img).expect("height is a required field");
    let channels = *reg.channels.get(img).expect("channels is a required field");
    let depth = *reg.depth.get(img).expect("depth is a required field");
    let data = reg.data.get(img).expect("data is a required field");

    if width == 0 || height == 0 {
        return Err(CheckError::InvalidDimensions { width, height });
    }
    if channels != 1 && channels != 3 {
        return Err(CheckError::UnsupportedChannels(channels));
    }
    if depth != 8 {
        return Err(CheckError::UnsupportedDepth(depth));
    }

    let expected = width * height * channels;
    if data.len() != expected {
        return Err(CheckError::CorruptedBuffer {
            expected,
            actual: data.len(),
        });
    }

    // Validate optional fields when present
    if let Some(cp) = reg.color_profile.get(img).and_then(|o| o.as_ref()) {
        if cp.is_empty() {
            return Err(CheckError::EmptyColorProfile);
        }
    }
    if let Some(&d) = reg.dpi.get(img).and_then(|o| o.as_ref()) {
        if !(1..=1200).contains(&d) {
            return Err(CheckError::InvalidDpi(d));
        }
    }
    if let Some(tags) = reg.tags.get(img).and_then(|o| o.as_ref()) {
        if tags.len() > 64 {
            return Err(CheckError::TooManyTags { count: tags.len() });
        }
        for (i, t) in tags.iter().enumerate() {
            if t.is_empty() {
                return Err(CheckError::InvalidTags { index: i });
            }
        }
    }

    Ok(())
}

// ============================================================================
// Main: demonstrate check detection
// ============================================================================

fn main() {
    println!("=== Check Detection with Keypaths ===\n");

    // Valid image (no optional fields)
    let valid = Image {
        width: 4,
        height: 4,
        channels: 3,
        depth: 8,
        data: vec![0u8; 4 * 4 * 3],
        color_profile: None,
        dpi: None,
        tags: None,
    };
    match check_image(&valid) {
        Ok(()) => println!("✓ Valid image passed check"),
        Err(e) => println!("✗ Valid image failed: {:?}", e),
    }

    // Valid image with optional fields
    let valid_with_opts = Image {
        width: 4,
        height: 4,
        channels: 3,
        depth: 8,
        data: vec![0u8; 4 * 4 * 3],
        color_profile: Some("sRGB".to_string()),
        dpi: Some(300),
        tags: Some(vec!["photo".to_string(), "landscape".to_string()]),
    };
    match check_image(&valid_with_opts) {
        Ok(()) => println!("✓ Valid image with optional fields passed check"),
        Err(e) => println!("✗ Valid+opts failed: {:?}", e),
    }

    // Invalid: wrong dimensions
    let bad_dims = Image {
        width: 0,
        height: 4,
        channels: 3,
        depth: 8,
        data: vec![0u8; 48],
        color_profile: None,
        dpi: None,
        tags: None,
    };
    match check_image(&bad_dims) {
        Ok(()) => println!("✗ Bad dims should have failed"),
        Err(CheckError::InvalidDimensions { width, height }) => {
            println!("✓ Detected invalid dimensions: {}x{}", width, height)
        }
        Err(e) => println!("  Unexpected: {:?}", e),
    }

    // Invalid: unsupported channels
    let bad_channels = Image {
        width: 4,
        height: 4,
        channels: 4,
        depth: 8,
        data: vec![0u8; 64],
        color_profile: None,
        dpi: None,
        tags: None,
    };
    match check_image(&bad_channels) {
        Ok(()) => println!("✗ Bad channels should have failed"),
        Err(CheckError::UnsupportedChannels(c)) => {
            println!("✓ Detected unsupported channels: {}", c)
        }
        Err(e) => println!("  Unexpected: {:?}", e),
    }

    // Invalid: corrupted buffer
    let bad_buffer = Image {
        width: 4,
        height: 4,
        channels: 3,
        depth: 8,
        data: vec![0u8; 10], // expected 48
        color_profile: None,
        dpi: None,
        tags: None,
    };
    match check_image(&bad_buffer) {
        Ok(()) => println!("✗ Corrupted buffer should have failed"),
        Err(CheckError::CorruptedBuffer { expected, actual }) => {
            println!("✓ Detected corrupted buffer: expected {}, got {}", expected, actual)
        }
        Err(e) => println!("  Unexpected: {:?}", e),
    }

    // Invalid optional: empty color_profile
    let bad_color_profile = Image {
        width: 4,
        height: 4,
        channels: 3,
        depth: 8,
        data: vec![0u8; 48],
        color_profile: Some(String::new()),
        dpi: None,
        tags: None,
    };
    match check_image(&bad_color_profile) {
        Ok(()) => println!("✗ Empty color_profile should have failed"),
        Err(CheckError::EmptyColorProfile) => println!("✓ Detected empty color_profile"),
        Err(e) => println!("  Unexpected: {:?}", e),
    }

    // Invalid optional: dpi out of range
    let bad_dpi = Image {
        width: 4,
        height: 4,
        channels: 3,
        depth: 8,
        data: vec![0u8; 48],
        color_profile: None,
        dpi: Some(0),
        tags: None,
    };
    match check_image(&bad_dpi) {
        Ok(()) => println!("✗ Invalid dpi should have failed"),
        Err(CheckError::InvalidDpi(d)) => println!("✓ Detected invalid dpi: {}", d),
        Err(e) => println!("  Unexpected: {:?}", e),
    }

    // Invalid optional: empty tag
    let bad_tags = Image {
        width: 4,
        height: 4,
        channels: 3,
        depth: 8,
        data: vec![0u8; 48],
        color_profile: None,
        dpi: None,
        tags: Some(vec!["ok".to_string(), String::new(), "also_ok".to_string()]),
    };
    match check_image(&bad_tags) {
        Ok(()) => println!("✗ Empty tag should have failed"),
        Err(CheckError::InvalidTags { index }) => println!("✓ Detected invalid tag at index {}", index),
        Err(e) => println!("  Unexpected: {:?}", e),
    }

    // Demonstrate static keypath read/write
    println!("\n--- Static keypath access ---");
    let mut img = valid.clone();
    *kp_width().get_mut(&mut img).unwrap() = 8;
    assert_eq!(*kp_width().get(&img).unwrap(), 8);
    println!("✓ Static kp read/write works");

    println!("\n=== Check detection complete ===");
}
