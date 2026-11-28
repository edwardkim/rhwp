//! The picture and its shadow share the cropped source coverage. The shadow
//! expands only the paint extent, so it does not move text or resize the image.

use std::io::Cursor;

use super::{decode_image_with_format_limited, image_crop_reference, memoized, Conversion};
use crate::paint::{PaintOp, ResolvedImageKind, ResolvedImagePayload};
use crate::renderer::render_tree::{BoundingBox, ImageNode};

pub(crate) fn prepare_picture_shadow(
    image: &ImageNode,
    frame: BoundingBox,
    dpi: f64,
) -> Option<PaintOp> {
    let shadow = image.outer_shadow?;
    if image.transform.has_transform()
        || image.content_inset.is_some()
        || image.source_rect.is_some()
        || image.fill_mode.is_some()
        || !dpi.is_finite()
        || dpi <= 0.0
        || ![frame.x, frame.y, frame.width, frame.height]
            .into_iter()
            .all(f64::is_finite)
        || frame.width <= 0.0
        || frame.height <= 0.0
    {
        return None;
    }

    let source = image.data.as_deref()?;
    let format = image::guess_format(source).ok()?;
    let header =
        super::canvaskit_encoded_image_header(source).filter(|h| h.is_within_decode_limits())?;
    let source_size = (f64::from(header.width), f64::from(header.height));
    let crop = match image.crop {
        Some(crop) => crate::renderer::svg::compute_image_crop_src(
            crop,
            image_crop_reference(image),
            source_size.0,
            source_size.1,
        ),
        None => (0.0, 0.0, source_size.0, source_size.1),
    };
    if ![crop.0, crop.1, crop.2, crop.3]
        .into_iter()
        .all(f64::is_finite)
        || crop.0 < 0.0
        || crop.1 < 0.0
        || crop.2 <= 0.0
        || crop.3 <= 0.0
        || crop.0 + crop.2 > source_size.0
        || crop.1 + crop.3 > source_size.1
    {
        return None;
    }

    // Work in a DPI-independent coverage surface. The 400 HU radius in
    // independent HWP Print output has a roughly 2.7 px Gaussian sigma at 96 dpi.
    let base_radius = shadow.radius_hu / 75.0;
    let sample_scale = (255.0 / base_radius).min(1.0);
    let work_dpi = 96.0 * sample_scale;
    let radius = base_radius * sample_scale;
    let pad = radius.ceil() as u32;
    let width = (frame.width * work_dpi / dpi).round().max(1.0) as u32;
    let height = (frame.height * work_dpi / dpi).round().max(1.0) as u32;
    let padded = [width.checked_add(2 * pad)?, height.checked_add(2 * pad)?];
    if padded
        .iter()
        .any(|v| *v > super::CANVASKIT_MAX_IMAGE_DIMENSION)
        || u64::from(padded[0]) * u64::from(padded[1]) > super::CANVASKIT_MAX_IMAGE_PIXELS
    {
        return None;
    }
    let opacity = shadow.opacity * image.opacity;
    if !opacity.is_finite() || opacity <= 0.0 || opacity > 1.0 {
        return None;
    }
    let key = Conversion::PictureShadow {
        radius: radius.to_bits(),
        dimensions: [width, height, pad],
        crop: [crop.0, crop.1, crop.2, crop.3].map(f64::to_bits),
        rgb: shadow.rgb,
        opacity: opacity.to_bits(),
    };
    let bytes = memoized(key, source, || {
        let source = decode_image_with_format_limited(source, format)?.to_rgba8();
        let window = super::picture_soft_edge::sample_window(&source, crop, width, height);
        let mut coverage = image::GrayImage::new(padded[0], padded[1]);
        for (x, y, rgba) in window.enumerate_pixels() {
            coverage.put_pixel(x + pad, y + pad, image::Luma([rgba[3]]));
        }
        let alpha = if radius > 0.0 {
            image::imageops::blur(&coverage, (radius / 2.0) as f32)
        } else {
            coverage
        };
        let [r, g, b] = shadow.rgb;
        let rgba = image::RgbaImage::from_fn(padded[0], padded[1], |x, y| {
            image::Rgba([
                r,
                g,
                b,
                (f64::from(alpha.get_pixel(x, y)[0]) * opacity).round() as u8,
            ])
        });
        let mut out = Cursor::new(Vec::new());
        rgba.write_to(&mut out, image::ImageFormat::Png).ok()?;
        Some(out.into_inner())
    })?;

    let pad_x = f64::from(pad) * frame.width / f64::from(width);
    let pad_y = f64::from(pad) * frame.height / f64::from(height);
    let bounds = BoundingBox::new(
        frame.x + shadow.offset_hu.0 * dpi / 7200.0 - pad_x,
        frame.y + shadow.offset_hu.1 * dpi / 7200.0 - pad_y,
        frame.width + 2.0 * pad_x,
        frame.height + 2.0 * pad_y,
    );
    Some(PaintOp::image(
        bounds,
        ImageNode::new(0, None),
        Some(ResolvedImagePayload {
            data: bytes,
            mime: "image/png",
            kind: ResolvedImageKind::SourceWindow,
            suppress_effects: true,
        }),
    ))
}
