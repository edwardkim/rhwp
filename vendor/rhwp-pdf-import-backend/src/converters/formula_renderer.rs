//! Formula rendering support for HTML output.
//!
//! This module provides functionality to extract formula regions from rendered
//! PDF page images and embed them as base64 data URIs in HTML output.

use crate::layout::TextSpan;
use crate::structure::{StructChild, StructElem};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

/// Formula rendering context for a document.
pub struct FormulaRenderer {
    /// Pre-rendered page images
    page_images: Vec<DynamicImage>,
    /// Page dimensions in PDF points (width, height)
    page_dimensions: (f32, f32),
    /// MCID to Y coordinate mapping per page: page -> mcid -> (min_y, max_y)
    mcid_y_maps: HashMap<u32, HashMap<u32, (f32, f32)>>,
    /// Formula counter for logging
    formula_count: usize,
}

/// Rendered formula with base64 data URI
#[derive(Debug, Clone)]
pub struct RenderedFormula {
    /// Base64 data URI (data:image/png;base64,...)
    pub data_uri: String,
    /// Alt text if available
    pub alt_text: Option<String>,
}

impl FormulaRenderer {
    /// Create a new formula renderer with page images.
    ///
    /// # Arguments
    ///
    /// * `page_image_paths` - Paths to pre-rendered page images (PNG format)
    /// * `page_dimensions` - Page dimensions in PDF points (width, height)
    ///
    /// # Returns
    ///
    /// A new FormulaRenderer or an error if images cannot be loaded.
    pub fn new<P: AsRef<Path>>(
        page_image_paths: &[P],
        page_dimensions: (f32, f32),
    ) -> Result<Self, String> {
        let mut page_images = Vec::new();

        for path in page_image_paths {
            let img = image::open(path.as_ref())
                .map_err(|e| format!("Failed to load page image {:?}: {}", path.as_ref(), e))?;
            page_images.push(img);
        }

        Ok(Self {
            page_images,
            page_dimensions,
            mcid_y_maps: HashMap::new(),
            formula_count: 0,
        })
    }

    /// Build MCID to Y coordinate mappings from extracted spans.
    ///
    /// This must be called before rendering formulas to establish the
    /// coordinate mapping for formula region estimation.
    pub fn build_mcid_map(&mut self, page: u32, spans: &[TextSpan]) {
        let mut mcid_y: HashMap<u32, (f32, f32)> = HashMap::new();

        for span in spans {
            if let Some(mcid) = span.mcid {
                let entry = mcid_y
                    .entry(mcid)
                    .or_insert((span.bbox.y, span.bbox.y + span.bbox.height));
                entry.0 = entry.0.min(span.bbox.y);
                entry.1 = entry.1.max(span.bbox.y + span.bbox.height);
            }
        }

        self.mcid_y_maps.insert(page, mcid_y);
    }

    /// Render a formula element as a base64 image.
    ///
    /// Returns None if the formula cannot be rendered (e.g., no valid bounds).
    pub fn render_formula(&mut self, elem: &StructElem, page: u32) -> Option<RenderedFormula> {
        let bounds = self.estimate_formula_bounds(elem, page)?;
        let (top_y, bot_y) = bounds;

        let page_image = self.page_images.get(page as usize)?;
        let data_uri = self.crop_formula_region(page_image, top_y, bot_y)?;

        self.formula_count += 1;

        Some(RenderedFormula {
            data_uri,
            alt_text: elem.alt_text.clone(),
        })
    }

    /// Estimate formula bounds from neighboring text MCIDs.
    fn estimate_formula_bounds(&self, elem: &StructElem, page: u32) -> Option<(f32, f32)> {
        let mcid_y_map = self.mcid_y_maps.get(&page)?;

        let mut mcids = Vec::new();
        collect_mcids_recursive(elem, &mut mcids);

        if mcids.is_empty() {
            return None;
        }

        // Safety: mcids.is_empty() is checked above and returns None
        let min_mcid = *mcids.iter().min().expect("mcids verified non-empty above");
        let max_mcid = *mcids.iter().max().expect("mcids verified non-empty above");

        // Find text above (highest MCID < min_mcid)
        let text_above_y = mcid_y_map
            .iter()
            .filter(|(&m, _)| m < min_mcid)
            .max_by_key(|(&m, _)| m)
            .map(|(_, (min_y, _))| *min_y);

        // Find text below (lowest MCID > max_mcid)
        let text_below_y = mcid_y_map
            .iter()
            .filter(|(&m, _)| m > max_mcid)
            .min_by_key(|(&m, _)| m)
            .map(|(_, (_, max_y))| *max_y);

        if let (Some(above_y), Some(below_y)) = (text_above_y, text_below_y) {
            let gap_height = above_y - below_y;
            if gap_height > 30.0 {
                // Take middle 40% of gap (30% margin on each side)
                let margin = gap_height * 0.30;
                let top_y = above_y - margin;
                let bot_y = below_y + margin;
                if top_y > bot_y {
                    return Some((top_y, bot_y));
                }
            }
        }

        None
    }

    /// Crop a formula region from a page image and return as base64 data URI.
    fn crop_formula_region(
        &self,
        page_image: &DynamicImage,
        top_y: f32,
        bot_y: f32,
    ) -> Option<String> {
        let (img_width, img_height) = page_image.dimensions();
        let pdf_height = self.page_dimensions.1;
        let scale = img_height as f32 / pdf_height;

        // Convert PDF coords to image coords (Y inverted)
        let img_top = ((pdf_height - top_y) * scale) as u32;
        let img_bot = ((pdf_height - bot_y) * scale) as u32;

        if img_bot <= img_top || img_bot > img_height {
            return None;
        }

        let height = img_bot - img_top;

        // Crop the region
        let cropped = page_image.crop_imm(0, img_top, img_width, height);

        // Trim whitespace (find bounding box of non-white pixels)
        let trimmed = trim_whitespace(&cropped);

        // Add small border
        let bordered = add_border(&trimmed, 10, 5);

        // Encode as PNG and convert to base64
        let mut buffer = Cursor::new(Vec::new());
        bordered.write_to(&mut buffer, ImageFormat::Png).ok()?;

        let base64_data = BASE64.encode(buffer.into_inner());
        Some(format!("data:image/png;base64,{}", base64_data))
    }

    /// Get the number of formulas rendered so far.
    pub fn formula_count(&self) -> usize {
        self.formula_count
    }
}

/// Collect MCIDs from a structure element recursively.
fn collect_mcids_recursive(elem: &StructElem, mcids: &mut Vec<u32>) {
    for child in &elem.children {
        match child {
            StructChild::MarkedContentRef { mcid, .. } => {
                mcids.push(*mcid);
            },
            StructChild::StructElem(child_elem) => {
                collect_mcids_recursive(child_elem, mcids);
            },
            _ => {},
        }
    }
}

/// Trim whitespace from an image by finding the bounding box of non-white pixels.
fn trim_whitespace(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();

    // Find bounds of non-white content
    let mut min_x = width;
    let mut max_x = 0u32;
    let mut min_y = height;
    let mut max_y = 0u32;

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            // Check if pixel is not white (threshold)
            if pixel[0] < 250 || pixel[1] < 250 || pixel[2] < 250 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    // If no content found, return original
    if min_x >= max_x || min_y >= max_y {
        return img.clone();
    }

    // Crop to content bounds
    img.crop_imm(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
}

/// Add a white border around an image.
fn add_border(img: &DynamicImage, horizontal: u32, vertical: u32) -> DynamicImage {
    let (width, height) = img.dimensions();
    let new_width = width + 2 * horizontal;
    let new_height = height + 2 * vertical;

    let mut bordered =
        image::RgbaImage::from_pixel(new_width, new_height, image::Rgba([255, 255, 255, 255]));

    // Convert to rgba8 once to avoid repeated conversions
    let rgba = img.to_rgba8();

    // Copy original image into center
    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            bordered.put_pixel(x + horizontal, y + vertical, *pixel);
        }
    }

    DynamicImage::ImageRgba8(bordered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::StructType;

    #[test]
    fn test_collect_mcids_recursive() {
        // Basic test for MCID collection
        let elem = StructElem {
            struct_type: StructType::Formula,
            children: vec![
                StructChild::MarkedContentRef {
                    mcid: 10,
                    page: 0,
                    scope: crate::structure::McidScope::Page(0),
                },
                StructChild::MarkedContentRef {
                    mcid: 11,
                    page: 0,
                    scope: crate::structure::McidScope::Page(0),
                },
            ],
            page: Some(0),
            attributes: HashMap::new(),
            alt_text: None,
            expansion: None,
            actual_text: None,
            source_role: None,
        };

        let mut mcids = Vec::new();
        collect_mcids_recursive(&elem, &mut mcids);
        assert_eq!(mcids, vec![10, 11]);
    }
}
