//! DBNet++ postprocessing for text detection.
//!
//! This module converts the probability map output from DBNet++ into
//! text bounding boxes through binarization, connected components analysis,
//! and polygon extraction.

use ndarray::{Array2, ArrayView2};

use super::error::{OcrError, OcrResult};

/// A detected text box with quadrilateral coordinates and confidence.
#[derive(Debug, Clone)]
pub struct DetectedBox {
    /// Four corner points of the text box [top-left, top-right, bottom-right, bottom-left]
    pub polygon: [[f32; 2]; 4],
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Extract text boxes from DBNet++ probability map.
///
/// # Arguments
///
/// * `prob_map` - 2D probability map from detector (H x W)
/// * `threshold` - Binarization threshold (typically 0.3)
/// * `box_threshold` - Minimum confidence to keep a box (typically 0.5)
/// * `max_candidates` - Maximum number of boxes to return
/// * `unclip_ratio` - Expansion ratio for boxes (typically 1.5)
/// * `scale` - Scale factor to convert back to original image coordinates
///
/// # Returns
///
/// Vector of detected text boxes.
pub fn extract_boxes(
    prob_map: ArrayView2<f32>,
    threshold: f32,
    box_threshold: f32,
    max_candidates: usize,
    unclip_ratio: f32,
    scale: f32,
) -> OcrResult<Vec<DetectedBox>> {
    let (height, width) = prob_map.dim();

    if height == 0 || width == 0 {
        return Err(OcrError::PostprocessingError("Empty probability map".to_string()));
    }

    // Step 1: Binarize the probability map
    let binary = binarize(prob_map, threshold);

    // Step 2: Find connected components (contours)
    let contours = find_contours(&binary);

    // Step 3: Process each contour into a bounding box
    let mut boxes = Vec::new();

    for contour in contours.into_iter().take(max_candidates) {
        if contour.len() < 4 {
            continue;
        }

        // Calculate contour score (mean probability inside)
        let score = calculate_contour_score(prob_map, &contour);
        if score < box_threshold {
            continue;
        }

        // Get minimum bounding box
        let min_rect = min_area_rect(&contour);

        // Expand (unclip) the box
        let expanded = unclip_polygon(&min_rect, unclip_ratio);

        // Scale back to original image coordinates
        let scaled: [[f32; 2]; 4] = [
            [expanded[0][0] / scale, expanded[0][1] / scale],
            [expanded[1][0] / scale, expanded[1][1] / scale],
            [expanded[2][0] / scale, expanded[2][1] / scale],
            [expanded[3][0] / scale, expanded[3][1] / scale],
        ];

        boxes.push(DetectedBox {
            polygon: scaled,
            confidence: score,
        });
    }

    // Sort by confidence (highest first)
    boxes.sort_by(|a, b| crate::utils::safe_float_cmp(b.confidence, a.confidence));

    Ok(boxes)
}

/// Binarize probability map using threshold.
fn binarize(prob_map: ArrayView2<f32>, threshold: f32) -> Array2<bool> {
    prob_map.mapv(|p| p > threshold)
}

/// Simple connected components analysis using flood fill.
///
/// Returns a vector of contours, where each contour is a vector of (x, y) points.
fn find_contours(binary: &Array2<bool>) -> Vec<Vec<[usize; 2]>> {
    let (height, width) = binary.dim();
    let mut visited = Array2::<bool>::default((height, width));
    let mut contours = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if binary[[y, x]] && !visited[[y, x]] {
                // Found a new component - flood fill to get boundary
                let contour = flood_fill_boundary(binary, &mut visited, x, y);
                if !contour.is_empty() {
                    contours.push(contour);
                }
            }
        }
    }

    contours
}

/// Flood fill to find boundary points of a connected component.
fn flood_fill_boundary(
    binary: &Array2<bool>,
    visited: &mut Array2<bool>,
    start_x: usize,
    start_y: usize,
) -> Vec<[usize; 2]> {
    let (height, width) = binary.dim();
    let mut stack = vec![(start_x, start_y)];
    let mut boundary_points = Vec::new();
    let mut min_x = start_x;
    let mut max_x = start_x;
    let mut min_y = start_y;
    let mut max_y = start_y;

    // 4-connectivity directions
    let directions: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    while let Some((x, y)) = stack.pop() {
        if visited[[y, x]] {
            continue;
        }
        visited[[y, x]] = true;

        // Track bounding box
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);

        // Check if this is a boundary pixel
        let mut is_boundary = false;
        for (dx, dy) in &directions {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                is_boundary = true;
            } else {
                let (nx, ny) = (nx as usize, ny as usize);
                if !binary[[ny, nx]] {
                    is_boundary = true;
                } else if !visited[[ny, nx]] {
                    stack.push((nx, ny));
                }
            }
        }

        if is_boundary {
            boundary_points.push([x, y]);
        }
    }

    // If we have a valid region, return simplified boundary
    if max_x > min_x && max_y > min_y {
        // For simplicity, we'll return the bounding box corners
        // A more sophisticated implementation would trace the actual contour
        boundary_points
    } else {
        Vec::new()
    }
}

/// Calculate the mean probability score inside a contour.
fn calculate_contour_score(prob_map: ArrayView2<f32>, contour: &[[usize; 2]]) -> f32 {
    if contour.is_empty() {
        return 0.0;
    }

    // Find bounding box of contour
    let min_x = contour.iter().map(|p| p[0]).min().unwrap_or(0);
    let max_x = contour.iter().map(|p| p[0]).max().unwrap_or(0);
    let min_y = contour.iter().map(|p| p[1]).min().unwrap_or(0);
    let max_y = contour.iter().map(|p| p[1]).max().unwrap_or(0);

    // Calculate mean probability in bounding box
    let mut sum = 0.0;
    let mut count = 0;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if y < prob_map.dim().0 && x < prob_map.dim().1 {
                sum += prob_map[[y, x]];
                count += 1;
            }
        }
    }

    if count > 0 {
        sum / count as f32
    } else {
        0.0
    }
}

/// Get minimum area bounding rectangle from contour points.
fn min_area_rect(contour: &[[usize; 2]]) -> [[f32; 2]; 4] {
    if contour.is_empty() {
        return [[0.0; 2]; 4];
    }

    // Find axis-aligned bounding box
    let min_x = contour.iter().map(|p| p[0]).min().unwrap_or(0) as f32;
    let max_x = contour.iter().map(|p| p[0]).max().unwrap_or(0) as f32;
    let min_y = contour.iter().map(|p| p[1]).min().unwrap_or(0) as f32;
    let max_y = contour.iter().map(|p| p[1]).max().unwrap_or(0) as f32;

    // Return as quadrilateral: top-left, top-right, bottom-right, bottom-left
    [
        [min_x, min_y],
        [max_x, min_y],
        [max_x, max_y],
        [min_x, max_y],
    ]
}

/// Expand (unclip) a detection box back to the true text extent.
///
/// DBNet predicts a **shrunken** text core, so the box must be offset
/// outward. PaddleOCR's `DBPostProcess.unclip` offsets the polygon by a
/// **uniform distance** `D = area * ratio / perimeter` (Vatti /
/// pyclipper). For the axis-aligned rect from [`min_area_rect`] that is
/// an even outset on all four sides.
///
/// The previous implementation instead scaled each corner by a *percent
/// of its own dimension* from the centre (`(ratio-1)/2`). On a wide,
/// short text line that is badly anisotropic: it over-expanded the long
/// axis (shoving x off-image, negative origin) while barely expanding
/// the short axis (box stayed ~one glyph-band tall). The recogniser
/// then received a horizontally-shifted, vertically-clipped sliver and
/// produced garbled text — e.g. "OCR fidelity test hello world 2024"
/// came out "OcR tdenfy test neno woridZoZ4 s" (#524 task 8). A uniform
/// offset expands height and width by the same absolute amount, so a
/// long line is recovered to (about) its true height.
fn unclip_polygon(polygon: &[[f32; 2]; 4], ratio: f32) -> [[f32; 2]; 4] {
    let xs = [polygon[0][0], polygon[1][0], polygon[2][0], polygon[3][0]];
    let ys = [polygon[0][1], polygon[1][1], polygon[2][1], polygon[3][1]];
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    let max_x = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let min_y = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let max_y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let w = (max_x - min_x).max(0.0);
    let h = (max_y - min_y).max(0.0);
    let area = w * h;
    let perimeter = 2.0 * (w + h);
    // D = A * ratio / L  (PaddleOCR). Degenerate boxes → no offset.
    let d = if perimeter > f32::EPSILON {
        area * ratio / perimeter
    } else {
        0.0
    };

    [
        [min_x - d, min_y - d],
        [max_x + d, min_y - d],
        [max_x + d, max_y + d],
        [min_x - d, max_y + d],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    #[test]
    fn test_binarize() {
        let prob_map =
            Array2::from_shape_vec((3, 3), vec![0.1, 0.5, 0.9, 0.2, 0.6, 0.8, 0.3, 0.4, 0.7])
                .unwrap();

        let binary = binarize(prob_map.view(), 0.5);

        assert!(!binary[[0, 0]]); // 0.1 < 0.5
        assert!(!binary[[0, 1]]); // 0.5 not > 0.5
        assert!(binary[[0, 2]]); // 0.9 > 0.5
        assert!(binary[[1, 1]]); // 0.6 > 0.5
    }

    #[test]
    fn test_min_area_rect() {
        let contour = vec![[10, 20], [50, 20], [50, 40], [10, 40]];
        let rect = min_area_rect(&contour);

        assert!((rect[0][0] - 10.0).abs() < f32::EPSILON);
        assert!((rect[0][1] - 20.0).abs() < f32::EPSILON);
        assert!((rect[2][0] - 50.0).abs() < f32::EPSILON);
        assert!((rect[2][1] - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_unclip_polygon_uniform_offset() {
        // 100 x 50 rect. PaddleOCR offset distance:
        //   D = area * ratio / perimeter = (100*50)*1.5 / (2*(100+50))
        //     = 7500 / 300 = 25
        // => an even 25 px outset on every side (NOT a percent-of-
        //    dimension scale — that anisotropy was the #524 garble bug).
        let polygon = [[0.0, 0.0], [100.0, 0.0], [100.0, 50.0], [0.0, 50.0]];
        let e = unclip_polygon(&polygon, 1.5);
        let eps = 1e-3;
        assert!((e[0][0] - -25.0).abs() < eps, "tl.x {}", e[0][0]);
        assert!((e[0][1] - -25.0).abs() < eps, "tl.y {}", e[0][1]);
        assert!((e[2][0] - 125.0).abs() < eps, "br.x {}", e[2][0]);
        assert!((e[2][1] - 75.0).abs() < eps, "br.y {}", e[2][1]);
        // The defining property: equal absolute growth on both axes.
        let grow_x = (e[2][0] - e[0][0]) - 100.0;
        let grow_y = (e[2][1] - e[0][1]) - 50.0;
        assert!(
            (grow_x - grow_y).abs() < eps,
            "offset must be isotropic: dx={grow_x} dy={grow_y}"
        );
    }

    #[test]
    fn test_unclip_recovers_height_of_wide_thin_line() {
        // The exact failure shape: a long, ~1-glyph-band-tall DB core
        // (like a detected text line). The old percent-scale unclip
        // grew width ~hugely and height ~nothing; the uniform offset
        // must grow the *short* axis substantially so the recogniser
        // sees full-height glyphs, and must not push x far negative.
        let w = 700.0;
        let h = 14.0;
        let poly = [
            [20.0, 60.0],
            [20.0 + w, 60.0],
            [20.0 + w, 60.0 + h],
            [20.0, 60.0 + h],
        ];
        let e = unclip_polygon(&poly, 1.5);
        let new_h = e[2][1] - e[0][1];
        let new_w = e[2][0] - e[0][0];
        // Height must clearly more-than-double. Old percent-scale
        // unclip gave new_h = h*(1+2*0.25) = 1.5*h (= 21); the uniform
        // offset gives ~34.6. `> 2*h` (= 28) cleanly separates them.
        assert!(new_h > 2.0 * h, "height barely grew: {h} -> {new_h}");
        // Width grows by the SAME absolute amount, not a % of width.
        assert!(
            ((new_w - w) - (new_h - h)).abs() < 1e-3,
            "anisotropic: dw={} dh={}",
            new_w - w,
            new_h - h
        );
        // Left edge stays near the image (old bug drove it to ~-48).
        assert!(e[0][0] > -40.0, "left edge shoved off-image: {}", e[0][0]);
    }

    #[test]
    fn test_extract_boxes_empty() {
        let prob_map = Array2::<f32>::zeros((100, 100));
        let boxes = extract_boxes(prob_map.view(), 0.3, 0.5, 100, 1.5, 1.0).unwrap();
        assert!(boxes.is_empty());
    }

    #[test]
    fn test_extract_boxes_single_region() {
        // Create a probability map with a high-probability region
        let mut prob_map = Array2::<f32>::zeros((100, 100));
        for y in 20..40 {
            for x in 30..70 {
                prob_map[[y, x]] = 0.9;
            }
        }

        let boxes = extract_boxes(prob_map.view(), 0.3, 0.5, 100, 1.5, 1.0).unwrap();
        assert!(!boxes.is_empty());

        // Check that the box roughly covers the high-probability region
        let box0 = &boxes[0];
        assert!(box0.confidence > 0.5);
    }
}
