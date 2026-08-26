//! Page raster fingerprints. Compare uses ink, histogram, and bbox — not filenames.

use super::ResidualClass;

pub const HIST_BINS: usize = 32;
pub const EDGE_SLICES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaPage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl RgbaPage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, String> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| "rgba page size overflow".to_string())?;
        if pixels.len() != expected {
            return Err(format!(
                "rgba length {} != {width}x{height}x4 ({expected})",
                pixels.len()
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn blank(width: u32, height: u32, fill: [u8; 4]) -> Result<Self, String> {
        let len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| "rgba page size overflow".to_string())?;
        let mut pixels = vec![0u8; len];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&fill);
        }
        Self::new(width, height, pixels)
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = ((y as usize * self.width as usize) + x as usize) * 4;
        Some([
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ])
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = x.saturating_add(w).min(self.width);
        let y2 = y.saturating_add(h).min(self.height);
        for yy in y1..y2 {
            for xx in x1..x2 {
                let i = ((yy as usize * self.width as usize) + xx as usize) * 4;
                self.pixels[i..i + 4].copy_from_slice(&color);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterFingerprint {
    pub width: u32,
    pub height: u32,
    pub ink_pixels: u32,
    pub ink_ppm: u32,
    pub hist: [u32; HIST_BINS],
    pub bbox: Option<[u32; 4]>,
    pub top_edge: [u32; EDGE_SLICES],
    pub bottom_edge: [u32; EDGE_SLICES],
    pub left_edge: [u32; EDGE_SLICES],
    pub right_edge: [u32; EDGE_SLICES],
    pub left_strip_ink: u32,
    pub right_band_ink: u32,
}

impl RasterFingerprint {
    pub fn from_rgba(page: &RgbaPage) -> Self {
        let mut hist = [0u32; HIST_BINS];
        let mut ink_pixels = 0u32;
        let mut min_x = page.width;
        let mut min_y = page.height;
        let mut max_x = 0u32;
        let mut max_y = 0u32;
        let mut top_edge = [0u32; EDGE_SLICES];
        let mut bottom_edge = [0u32; EDGE_SLICES];
        let mut left_edge = [0u32; EDGE_SLICES];
        let mut right_edge = [0u32; EDGE_SLICES];
        let mut left_strip_ink = 0u32;
        let mut right_band_ink = 0u32;
        let left_cut = (page.width * 45) / 100;
        let right_cut = (page.width * 55) / 100;
        let edge_h = (page.height / 8).max(1);
        let edge_w = (page.width / 8).max(1);

        for y in 0..page.height {
            for x in 0..page.width {
                let px = page.pixel(x, y).unwrap_or([255, 255, 255, 255]);
                let luma = luma8(px);
                let bin = ((luma as u32 * HIST_BINS as u32) / 256) as usize;
                hist[bin.min(HIST_BINS - 1)] += 1;
                if !is_ink(px) {
                    continue;
                }
                ink_pixels += 1;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                if x < left_cut {
                    left_strip_ink += 1;
                }
                if x >= right_cut {
                    right_band_ink += 1;
                }
                let sx = slice_index(x, page.width);
                let sy = slice_index(y, page.height);
                if y < edge_h {
                    top_edge[sx] += 1;
                }
                if y + edge_h >= page.height {
                    bottom_edge[sx] += 1;
                }
                if x < edge_w {
                    left_edge[sy] += 1;
                }
                if x + edge_w >= page.width {
                    right_edge[sy] += 1;
                }
            }
        }

        let area = page.width.saturating_mul(page.height).max(1);
        let ink_ppm = ((ink_pixels as u64) * 1_000_000 / area as u64) as u32;
        let bbox = if ink_pixels == 0 {
            None
        } else {
            Some([
                min_x,
                min_y,
                max_x.saturating_add(1),
                max_y.saturating_add(1),
            ])
        };
        Self {
            width: page.width,
            height: page.height,
            ink_pixels,
            ink_ppm,
            hist,
            bbox,
            top_edge,
            bottom_edge,
            left_edge,
            right_edge,
            left_strip_ink,
            right_band_ink,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FingerprintDelta {
    pub size_mismatch: bool,
    pub ink_ppm_abs: u32,
    pub hist_l1: u32,
    pub bbox_l1: u32,
    pub left_strip_ratio_milli: u32,
    pub right_band_ratio_milli: u32,
    pub top_edge_l1: u32,
    pub hint: ResidualClass,
}

pub fn compare_fingerprints(
    oracle: &RasterFingerprint,
    candidate: &RasterFingerprint,
) -> FingerprintDelta {
    let size_mismatch = oracle.width != candidate.width || oracle.height != candidate.height;
    let ink_ppm_abs = oracle.ink_ppm.abs_diff(candidate.ink_ppm);
    let hist_l1 = oracle
        .hist
        .iter()
        .zip(candidate.hist.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .sum();
    let bbox_l1 = match (oracle.bbox, candidate.bbox) {
        (Some(a), Some(b)) => a.iter().zip(b.iter()).map(|(l, r)| l.abs_diff(*r)).sum(),
        (None, None) => 0,
        _ => u32::MAX / 4,
    };
    let left_strip_ratio_milli = ratio_milli(candidate.left_strip_ink, oracle.left_strip_ink);
    let right_band_ratio_milli = ratio_milli(candidate.right_band_ink, oracle.right_band_ink);
    let top_edge_l1 = oracle
        .top_edge
        .iter()
        .zip(candidate.top_edge.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .sum();
    let hint = hint_class(
        size_mismatch,
        ink_ppm_abs,
        hist_l1,
        bbox_l1,
        left_strip_ratio_milli,
        right_band_ratio_milli,
        top_edge_l1,
        oracle,
        candidate,
    );
    FingerprintDelta {
        size_mismatch,
        ink_ppm_abs,
        hist_l1,
        bbox_l1,
        left_strip_ratio_milli,
        right_band_ratio_milli,
        top_edge_l1,
        hint,
    }
}

fn hint_class(
    size_mismatch: bool,
    ink_ppm_abs: u32,
    hist_l1: u32,
    bbox_l1: u32,
    left_strip_ratio_milli: u32,
    right_band_ratio_milli: u32,
    top_edge_l1: u32,
    oracle: &RasterFingerprint,
    candidate: &RasterFingerprint,
) -> ResidualClass {
    if size_mismatch {
        return ResidualClass::TablePlace;
    }
    if oracle.left_strip_ink > 80 && left_strip_ratio_milli <= 150 && right_band_ratio_milli >= 700
    {
        return ResidualClass::WrapFlow;
    }
    if bbox_l1 > 24 && top_edge_l1 > 40 {
        return ResidualClass::TablePlace;
    }
    let dark_shift = dark_bin_shift(&oracle.hist, &candidate.hist);
    if dark_shift.abs() >= 12 && ink_ppm_abs < 8_000 && bbox_l1 < 12 {
        return if dark_shift > 0 {
            ResidualClass::FontWeight
        } else {
            ResidualClass::FontWidth
        };
    }
    if hist_l1 > 0 && ink_ppm_abs < 4_000 && bbox_l1 < 8 {
        return ResidualClass::Glyph;
    }
    if ink_ppm_abs >= 4_000 {
        return ResidualClass::Paint;
    }
    ResidualClass::None
}

fn dark_bin_shift(oracle: &[u32; HIST_BINS], candidate: &[u32; HIST_BINS]) -> i32 {
    let o = centroid(oracle);
    let c = centroid(candidate);
    o as i32 - c as i32
}

fn centroid(hist: &[u32; HIST_BINS]) -> f32 {
    let mut acc = 0.0f32;
    let mut w = 0.0f32;
    for (i, count) in hist.iter().enumerate() {
        acc += (i as f32) * (*count as f32);
        w += *count as f32;
    }
    if w == 0.0 {
        0.0
    } else {
        acc / w
    }
}

fn ratio_milli(numer: u32, denom: u32) -> u32 {
    if denom == 0 {
        return if numer == 0 { 1000 } else { 0 };
    }
    ((numer as u64 * 1000) / denom as u64) as u32
}

pub fn is_ink(px: [u8; 4]) -> bool {
    if px[3] < 16 {
        return false;
    }
    luma8(px) < 245
}

pub fn luma8(px: [u8; 4]) -> u8 {
    let r = px[0] as u32;
    let g = px[1] as u32;
    let b = px[2] as u32;
    ((r * 30 + g * 59 + b * 11) / 100) as u8
}

fn slice_index(value: u32, extent: u32) -> usize {
    if extent == 0 {
        return 0;
    }
    let idx = (value as u64 * EDGE_SLICES as u64) / extent as u64;
    idx.min(EDGE_SLICES as u64 - 1) as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterPrimitive {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterPageSpec {
    pub width: u32,
    pub height: u32,
    pub background: [u8; 4],
    pub primitives: Vec<RasterPrimitive>,
}

pub fn render_synthetic_page(spec: &RasterPageSpec) -> Result<RgbaPage, String> {
    let mut page = RgbaPage::blank(spec.width, spec.height, spec.background)?;
    for prim in &spec.primitives {
        page.fill_rect(prim.x, prim.y, prim.w, prim.h, prim.color);
    }
    Ok(page)
}
