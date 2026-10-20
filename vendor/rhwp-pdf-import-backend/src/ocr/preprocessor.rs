//! Image preprocessing for OCR models.
//!
//! This module handles image preprocessing for both detection (DBNet++)
//! and recognition (SVTR) models, including resizing, normalization,
//! and tensor conversion.

use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use ndarray::Array4;

use super::config::DetResizeStrategy;
use super::error::{OcrError, OcrResult};

/// Mean values for ImageNet normalization (RGB order).
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];

/// Standard deviation values for ImageNet normalization (RGB order).
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Preprocess an image for text detection (DBNet++).
///
/// Performs the following operations:
/// 1. Resize according to the given strategy while maintaining aspect ratio
/// 2. Pad to multiple of 32 (required by DBNet++)
/// 3. Normalize using ImageNet mean/std
/// 4. Convert to CHW tensor format with batch dimension
///
/// # Arguments
///
/// * `image` - Input image
/// * `strategy` - Resize strategy (MaxSide for v3/v4, MinSide for v5)
///
/// # Returns
///
/// A 4D tensor of shape `[1, 3, H, W]` (NCHW format) and the applied scale ratio.
pub fn preprocess_for_detection(
    image: &DynamicImage,
    strategy: &DetResizeStrategy,
) -> OcrResult<(Array4<f32>, f32)> {
    let (orig_w, orig_h) = image.dimensions();

    if orig_w == 0 || orig_h == 0 {
        return Err(OcrError::InvalidImage("Image has zero dimensions".to_string()));
    }

    // Calculate resize ratio based on strategy
    let ratio = match strategy {
        DetResizeStrategy::MaxSide { max_side } => {
            // PP-OCRv3/v4: scale DOWN so longest side fits within max_side
            let max_dim = orig_w.max(orig_h);
            if max_dim > *max_side {
                *max_side as f32 / max_dim as f32
            } else {
                1.0
            }
        },
        DetResizeStrategy::MinSide {
            min_side,
            max_side_limit,
        } => {
            // PP-OCRv5: scale UP so shortest side is at least min_side,
            // but cap at max_side_limit
            let min_dim = orig_w.min(orig_h);
            let mut r = if min_dim < *min_side {
                *min_side as f32 / min_dim as f32
            } else {
                1.0
            };
            // Cap: don't exceed max_side_limit on the longest side
            let max_dim_after = (orig_w.max(orig_h) as f32 * r) as u32;
            if max_dim_after > *max_side_limit {
                r = *max_side_limit as f32 / orig_w.max(orig_h) as f32;
            }
            r
        },
    };

    let new_w = ((orig_w as f32 * ratio) as u32).max(1);
    let new_h = ((orig_h as f32 * ratio) as u32).max(1);

    // Resize image
    let resized = image.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);

    // Pad to multiple of 32
    let pad_w = (32 - new_w % 32) % 32;
    let pad_h = (32 - new_h % 32) % 32;
    let padded_w = new_w + pad_w;
    let padded_h = new_h + pad_h;

    // Create padded image with black background
    let mut padded = RgbImage::new(padded_w, padded_h);
    let rgb_image = resized.to_rgb8();

    // Copy resized image to padded canvas
    for y in 0..new_h {
        for x in 0..new_w {
            padded.put_pixel(x, y, *rgb_image.get_pixel(x, y));
        }
    }

    // Convert to normalized tensor
    let tensor = image_to_tensor_imagenet(&padded)?;

    Ok((tensor, ratio))
}

/// Preprocess a cropped text region for recognition (SVTR).
///
/// Performs the following operations:
/// 1. Resize to target height while maintaining aspect ratio
/// 2. Pad width to multiple of 4
/// 3. Normalize to [-1, 1] range
/// 4. Convert to CHW tensor format with batch dimension
///
/// # Arguments
///
/// * `crop` - Cropped text region image
/// * `target_height` - Target height (typically 48)
///
/// # Returns
///
/// A 4D tensor of shape `[1, 3, target_height, W]` (NCHW format).
pub fn preprocess_for_recognition(
    crop: &DynamicImage,
    target_height: u32,
) -> OcrResult<Array4<f32>> {
    let (orig_w, orig_h) = crop.dimensions();

    if orig_w == 0 || orig_h == 0 {
        return Err(OcrError::InvalidImage("Crop has zero dimensions".to_string()));
    }

    // Calculate new width maintaining aspect ratio
    let ratio = target_height as f32 / orig_h as f32;
    let new_w = ((orig_w as f32 * ratio) as u32).max(1);

    // Resize image
    let resized = crop.resize_exact(new_w, target_height, image::imageops::FilterType::Lanczos3);

    // Pad width to multiple of 4
    let pad_w = (4 - new_w % 4) % 4;
    let padded_w = new_w + pad_w;

    // Create padded image
    let mut padded = RgbImage::new(padded_w, target_height);
    let rgb_image = resized.to_rgb8();

    // Copy resized image to padded canvas
    for y in 0..target_height {
        for x in 0..new_w {
            padded.put_pixel(x, y, *rgb_image.get_pixel(x, y));
        }
    }

    // Convert to normalized tensor (different normalization for recognition)
    image_to_tensor_symmetric(&padded)
}

/// Convert RGB image to normalized tensor using ImageNet mean/std.
///
/// Output range approximately [-2.5, 2.5] for typical images.
fn image_to_tensor_imagenet(image: &RgbImage) -> OcrResult<Array4<f32>> {
    let (width, height) = image.dimensions();
    let mut tensor = Array4::<f32>::zeros((1, 3, height as usize, width as usize));

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let Rgb([r, g, b]) = *pixel;

            // Normalize: (pixel / 255.0 - mean) / std
            tensor[[0, 0, y as usize, x as usize]] =
                (r as f32 / 255.0 - IMAGENET_MEAN[0]) / IMAGENET_STD[0];
            tensor[[0, 1, y as usize, x as usize]] =
                (g as f32 / 255.0 - IMAGENET_MEAN[1]) / IMAGENET_STD[1];
            tensor[[0, 2, y as usize, x as usize]] =
                (b as f32 / 255.0 - IMAGENET_MEAN[2]) / IMAGENET_STD[2];
        }
    }

    Ok(tensor)
}

/// Convert RGB image to normalized tensor with symmetric [-1, 1] range.
///
/// Used for recognition model which expects (pixel / 255.0 - 0.5) / 0.5.
fn image_to_tensor_symmetric(image: &RgbImage) -> OcrResult<Array4<f32>> {
    let (width, height) = image.dimensions();
    let mut tensor = Array4::<f32>::zeros((1, 3, height as usize, width as usize));

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let Rgb([r, g, b]) = *pixel;

            // Normalize: (pixel / 255.0 - 0.5) / 0.5 = pixel / 127.5 - 1.0
            tensor[[0, 0, y as usize, x as usize]] = r as f32 / 127.5 - 1.0;
            tensor[[0, 1, y as usize, x as usize]] = g as f32 / 127.5 - 1.0;
            tensor[[0, 2, y as usize, x as usize]] = b as f32 / 127.5 - 1.0;
        }
    }

    Ok(tensor)
}

/// Crop a region from an image given a quadrilateral (4 points).
///
/// For simplicity, this uses the axis-aligned bounding box of the quadrilateral.
/// A more sophisticated implementation could use perspective transformation.
pub fn crop_text_region(image: &DynamicImage, polygon: &[[f32; 2]; 4]) -> OcrResult<DynamicImage> {
    let (img_w, img_h) = image.dimensions();

    // Find bounding box of the polygon
    let min_x = polygon.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
    let max_x = polygon.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
    let min_y = polygon.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    let max_y = polygon.iter().map(|p| p[1]).fold(f32::MIN, f32::max);

    // Clamp to image bounds
    let x = (min_x.max(0.0) as u32).min(img_w.saturating_sub(1));
    let y = (min_y.max(0.0) as u32).min(img_h.saturating_sub(1));
    let w = ((max_x - min_x).max(1.0) as u32).min(img_w - x);
    let h = ((max_y - min_y).max(1.0) as u32).min(img_h - y);

    if w == 0 || h == 0 {
        return Err(OcrError::InvalidImage("Crop region has zero size".to_string()));
    }

    Ok(image.crop_imm(x, y, w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            Rgb([(x % 256) as u8, (y % 256) as u8, 128u8])
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn test_preprocess_for_detection_max_side() {
        let img = create_test_image(800, 600);
        let strategy = DetResizeStrategy::MaxSide { max_side: 640 };
        let (tensor, ratio) = preprocess_for_detection(&img, &strategy).unwrap();

        // Check shape: [1, 3, H, W]
        assert_eq!(tensor.shape()[0], 1);
        assert_eq!(tensor.shape()[1], 3);

        // Height and width should be padded to multiple of 32
        assert!(tensor.shape()[2] % 32 == 0);
        assert!(tensor.shape()[3] % 32 == 0);

        // Ratio should be < 1 since we're downscaling
        assert!(ratio <= 1.0);
    }

    #[test]
    fn test_preprocess_for_detection_small_image_max_side() {
        let img = create_test_image(100, 100);
        let strategy = DetResizeStrategy::MaxSide { max_side: 640 };
        let (tensor, ratio) = preprocess_for_detection(&img, &strategy).unwrap();

        // Small image should not be scaled up, ratio = 1.0
        assert!((ratio - 1.0).abs() < f32::EPSILON);

        // Should still be padded to multiple of 32
        assert!(tensor.shape()[2] % 32 == 0);
        assert!(tensor.shape()[3] % 32 == 0);
    }

    #[test]
    fn test_preprocess_for_detection_min_side_upscale() {
        let img = create_test_image(30, 20);
        let strategy = DetResizeStrategy::MinSide {
            min_side: 64,
            max_side_limit: 4000,
        };
        let (tensor, ratio) = preprocess_for_detection(&img, &strategy).unwrap();

        // Small image should be scaled UP so min side >= 64
        assert!(ratio > 1.0);
        assert!(tensor.shape()[2] % 32 == 0);
        assert!(tensor.shape()[3] % 32 == 0);
    }

    #[test]
    fn test_preprocess_for_detection_min_side_passthrough() {
        let img = create_test_image(2480, 3508);
        let strategy = DetResizeStrategy::MinSide {
            min_side: 64,
            max_side_limit: 4000,
        };
        let (tensor, ratio) = preprocess_for_detection(&img, &strategy).unwrap();

        // Large image should pass through at original resolution (ratio = 1.0)
        assert!((ratio - 1.0).abs() < f32::EPSILON);
        assert!(tensor.shape()[2] % 32 == 0);
        assert!(tensor.shape()[3] % 32 == 0);
        // Should be close to original dimensions (padded to 32)
        assert!(tensor.shape()[2] >= 3508);
        assert!(tensor.shape()[3] >= 2480);
    }

    #[test]
    fn test_preprocess_for_recognition() {
        let img = create_test_image(200, 50);
        let tensor = preprocess_for_recognition(&img, 48).unwrap();

        // Check shape
        assert_eq!(tensor.shape()[0], 1);
        assert_eq!(tensor.shape()[1], 3);
        assert_eq!(tensor.shape()[2], 48); // Target height

        // Width should be padded to multiple of 4
        assert!(tensor.shape()[3].is_multiple_of(4));
    }

    #[test]
    fn test_normalize_values_detection() {
        let img = create_test_image(64, 64);
        let strategy = DetResizeStrategy::MaxSide { max_side: 640 };
        let (tensor, _) = preprocess_for_detection(&img, &strategy).unwrap();

        // Check that values are in reasonable range for ImageNet normalization
        let min_val = tensor.iter().cloned().fold(f32::MAX, f32::min);
        let max_val = tensor.iter().cloned().fold(f32::MIN, f32::max);

        // ImageNet normalization typically produces values in [-3, 3] range
        assert!(min_val >= -5.0);
        assert!(max_val <= 5.0);
    }

    #[test]
    fn test_normalize_values_recognition() {
        let img = create_test_image(200, 50);
        let tensor = preprocess_for_recognition(&img, 48).unwrap();

        // Check that values are in [-1, 1] range (symmetric normalization)
        for val in tensor.iter() {
            assert!(*val >= -1.0 && *val <= 1.0, "Value {} out of range", val);
        }
    }

    #[test]
    fn test_crop_text_region() {
        let img = create_test_image(100, 100);
        let polygon = [[10.0, 10.0], [50.0, 10.0], [50.0, 30.0], [10.0, 30.0]];

        let crop = crop_text_region(&img, &polygon).unwrap();
        let (w, h) = crop.dimensions();

        assert_eq!(w, 40); // 50 - 10
        assert_eq!(h, 20); // 30 - 10
    }

    #[test]
    fn test_crop_text_region_clamped() {
        let img = create_test_image(100, 100);
        // Polygon extends beyond image bounds
        let polygon = [
            [-10.0, -10.0],
            [150.0, -10.0],
            [150.0, 150.0],
            [-10.0, 150.0],
        ];

        let crop = crop_text_region(&img, &polygon).unwrap();
        let (w, h) = crop.dimensions();

        // Should be clamped to image size
        assert!(w <= 100);
        assert!(h <= 100);
    }
}
