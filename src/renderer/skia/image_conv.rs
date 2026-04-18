use resvg::{tiny_skia, usvg};
use skia_safe::{
    canvas::SrcRectConstraint, Canvas, Data, FilterMode, Image, MipmapMode, Paint, Rect,
    SamplingOptions,
};

use crate::model::style::ImageFillMode;

pub fn draw_image_bytes(
    canvas: &Canvas,
    bytes: &[u8],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill_mode: Option<ImageFillMode>,
    original_size: Option<(f64, f64)>,
    crop: Option<(i32, i32, i32, i32)>,
) {
    let Some(image) = decode_image(bytes) else {
        return;
    };
    let dst = Rect::from_xywh(x, y, width, height);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let mode = fill_mode.unwrap_or(ImageFillMode::FitToSize);

    let draw_image_rect = |canvas: &Canvas, src: Option<Rect>, dst: Rect| {
        if let Some(src) = src.as_ref() {
            canvas.draw_image_rect_with_sampling_options(
                &image,
                Some((src, SrcRectConstraint::Fast)),
                dst,
                SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
                &paint,
            );
        } else {
            canvas.draw_image_rect_with_sampling_options(
                &image,
                None,
                dst,
                SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
                &paint,
            );
        }
    };

    if matches!(mode, ImageFillMode::FitToSize | ImageFillMode::None) {
        if let Some((left, top, right, bottom)) = crop {
            let image_width = image.width() as f32;
            let image_height = image.height() as f32;
            if image_width > 0.0 && image_height > 0.0 {
                let scale_x = right as f32 / image_width;
                if scale_x > 0.0 {
                    let src_x = left as f32 / scale_x;
                    let src_y = top as f32 / scale_x;
                    let src_w = (right - left) as f32 / scale_x;
                    let src_h = (bottom - top) as f32 / scale_x;
                    let is_cropped = src_x > 0.5
                        || src_y > 0.5
                        || (src_w - image_width).abs() > 1.0
                        || (src_h - image_height).abs() > 1.0;
                    if is_cropped {
                        draw_image_rect(
                            canvas,
                            Some(Rect::from_xywh(src_x, src_y, src_w, src_h)),
                            dst,
                        );
                        return;
                    }
                }
            }
        }

        draw_image_rect(canvas, None, dst);
        return;
    }

    let image_width = original_size
        .map(|(width, _)| width as f32)
        .unwrap_or_else(|| image.width() as f32);
    let image_height = original_size
        .map(|(_, height)| height as f32)
        .unwrap_or_else(|| image.height() as f32);

    canvas.save();
    canvas.clip_rect(dst, None, Some(true));

    if matches!(
        mode,
        ImageFillMode::TileAll
            | ImageFillMode::TileHorzTop
            | ImageFillMode::TileHorzBottom
            | ImageFillMode::TileVertLeft
            | ImageFillMode::TileVertRight
    ) {
        if matches!(mode, ImageFillMode::TileAll) {
            let mut tile_y = y;
            while tile_y < y + height {
                let mut tile_x = x;
                while tile_x < x + width {
                    draw_image_rect(
                        canvas,
                        None,
                        Rect::from_xywh(tile_x, tile_y, image_width, image_height),
                    );
                    tile_x += image_width.max(1.0);
                }
                tile_y += image_height.max(1.0);
            }
        } else if matches!(
            mode,
            ImageFillMode::TileHorzTop | ImageFillMode::TileHorzBottom
        ) {
            let tile_y = if matches!(mode, ImageFillMode::TileHorzTop) {
                y
            } else {
                y + height - image_height
            };
            let mut tile_x = x;
            while tile_x < x + width {
                draw_image_rect(
                    canvas,
                    None,
                    Rect::from_xywh(tile_x, tile_y, image_width, image_height),
                );
                tile_x += image_width.max(1.0);
            }
        } else {
            let tile_x = if matches!(mode, ImageFillMode::TileVertLeft) {
                x
            } else {
                x + width - image_width
            };
            let mut tile_y = y;
            while tile_y < y + height {
                draw_image_rect(
                    canvas,
                    None,
                    Rect::from_xywh(tile_x, tile_y, image_width, image_height),
                );
                tile_y += image_height.max(1.0);
            }
        }
    } else {
        let (image_x, image_y) =
            resolve_image_placement(mode, x, y, width, height, image_width, image_height);
        draw_image_rect(
            canvas,
            None,
            Rect::from_xywh(image_x, image_y, image_width, image_height),
        );
    }

    canvas.restore();
}

pub fn draw_svg_fragment(
    canvas: &Canvas,
    svg_fragment: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let Some(image) = decode_svg_fragment(svg_fragment, width, height) else {
        return;
    };

    let dst = Rect::from_xywh(x, y, width, height);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    canvas.draw_image_rect_with_sampling_options(
        &image,
        None,
        dst,
        SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
        &paint,
    );
}

fn resolve_image_placement(
    fill_mode: ImageFillMode,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    image_width: f32,
    image_height: f32,
) -> (f32, f32) {
    match fill_mode {
        ImageFillMode::LeftTop => (x, y),
        ImageFillMode::CenterTop => (x + (width - image_width) / 2.0, y),
        ImageFillMode::RightTop => (x + width - image_width, y),
        ImageFillMode::LeftCenter => (x, y + (height - image_height) / 2.0),
        ImageFillMode::Center => (
            x + (width - image_width) / 2.0,
            y + (height - image_height) / 2.0,
        ),
        ImageFillMode::RightCenter => (x + width - image_width, y + (height - image_height) / 2.0),
        ImageFillMode::LeftBottom => (x, y + height - image_height),
        ImageFillMode::CenterBottom => (x + (width - image_width) / 2.0, y + height - image_height),
        ImageFillMode::RightBottom => (x + width - image_width, y + height - image_height),
        _ => (x, y),
    }
}

fn decode_image(bytes: &[u8]) -> Option<Image> {
    match detect_image_mime_type(bytes) {
        "image/x-wmf" => {
            let svg = crate::renderer::svg::convert_wmf_to_svg(bytes)?;
            let mut options = usvg::Options::default();
            options.fontdb_mut().load_system_fonts();
            let tree = usvg::Tree::from_data(&svg, &options).ok()?;
            let size = tree.size().to_int_size();
            let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())?;
            resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
            let png = pixmap.encode_png().ok()?;
            Image::from_encoded(Data::new_copy(&png))
        }
        _ => Image::from_encoded(Data::new_copy(bytes)),
    }
}

fn decode_svg_fragment(svg_fragment: &str, width: f32, height: f32) -> Option<Image> {
    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.2}\" height=\"{height:.2}\" viewBox=\"0 0 {width:.2} {height:.2}\">{svg_fragment}</svg>"
    );
    let mut options = usvg::Options::default();
    let fontdb = options.fontdb_mut();
    fontdb.load_system_fonts();
    fontdb.set_sans_serif_family("Noto Sans CJK KR");
    fontdb.set_serif_family("Noto Serif CJK KR");
    fontdb.set_monospace_family("D2Coding");

    let tree = usvg::Tree::from_str(&svg, &options).ok()?;
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let png = pixmap.encode_png().ok()?;
    Image::from_encoded(Data::new_copy(&png))
}

fn detect_image_mime_type(data: &[u8]) -> &'static str {
    if data.len() >= 8 {
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return "image/png";
        }
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return "image/jpeg";
        }
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return "image/gif";
        }
        if data.starts_with(&[0x42, 0x4D]) {
            return "image/bmp";
        }
        if data.starts_with(&[0xD7, 0xCD, 0xC6, 0x9A])
            || data.starts_with(&[0x01, 0x00, 0x09, 0x00])
        {
            return "image/x-wmf";
        }
        if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
            || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        {
            return "image/tiff";
        }
    }

    "application/octet-stream"
}
