use resvg::{tiny_skia, usvg};
use skia_safe::{
    canvas::SrcRectConstraint, color_filters, image::RequiredProperties, Color, Data, FilterMode,
    IRect, Image, Matrix, MipmapMode, Paint, Rect, SamplingOptions, TileMode,
};

use crate::model::image::ImageEffect;
use crate::model::style::ImageFillMode;

const MAX_SVG_FRAGMENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_SVG_RASTER_PIXELS: u64 = 67_108_864;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSampling {
    filter_mode: FilterMode,
    mipmap_mode: MipmapMode,
}

impl ImageSampling {
    pub fn linear() -> Self {
        Self {
            filter_mode: FilterMode::Linear,
            mipmap_mode: MipmapMode::None,
        }
    }

    fn options(self) -> SamplingOptions {
        SamplingOptions::new(self.filter_mode, self.mipmap_mode)
    }
}

pub fn draw_svg_fragment(
    canvas: &skia_safe::Canvas,
    svg_fragment: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    sampling: ImageSampling,
) -> bool {
    let Some(png) = rasterize_svg_fragment_to_png(svg_fragment, width, height) else {
        return false;
    };
    draw_image_bytes(
        canvas,
        &png,
        x,
        y,
        width,
        height,
        Some(ImageFillMode::FitToSize),
        None,
        None,
        ImageEffect::RealPic,
        sampling,
    );
    true
}

pub fn draw_image_bytes(
    canvas: &skia_safe::Canvas,
    bytes: &[u8],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill_mode: Option<ImageFillMode>,
    original_size: Option<(f64, f64)>,
    crop: Option<(i32, i32, i32, i32)>,
    effect: ImageEffect,
    sampling: ImageSampling,
) {
    let is_valid_destination_rect = |x: f32, y: f32, width: f32, height: f32| {
        x.is_finite()
            && y.is_finite()
            && width.is_finite()
            && height.is_finite()
            && width > 0.0
            && height > 0.0
    };
    let is_valid_image_size = |width: f32, height: f32| {
        width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0
    };
    let grayscale_filter = |scale: f32, translate: f32| {
        let r = 0.299 * scale;
        let g = 0.587 * scale;
        let b = 0.114 * scale;
        color_filters::matrix_row_major(
            &[
                r, g, b, 0.0, translate, r, g, b, 0.0, translate, r, g, b, 0.0, translate, 0.0,
                0.0, 0.0, 1.0, 0.0,
            ],
            None,
        )
    };
    let image_effect_filter = |effect: ImageEffect| match effect {
        ImageEffect::RealPic => None,
        ImageEffect::GrayScale => Some(grayscale_filter(1.0, 0.0)),
        ImageEffect::BlackWhite => Some(grayscale_filter(255.0, -127.5)),
        ImageEffect::Pattern8x8 => Some(grayscale_filter(1.0, 0.0)),
    };
    let resolve_image_placement = |fill_mode: ImageFillMode,
                                   x: f32,
                                   y: f32,
                                   width: f32,
                                   height: f32,
                                   image_width: f32,
                                   image_height: f32| {
        match fill_mode {
            ImageFillMode::LeftTop => (x, y),
            ImageFillMode::CenterTop => (x + (width - image_width) / 2.0, y),
            ImageFillMode::RightTop => (x + width - image_width, y),
            ImageFillMode::LeftCenter => (x, y + (height - image_height) / 2.0),
            ImageFillMode::Center => (
                x + (width - image_width) / 2.0,
                y + (height - image_height) / 2.0,
            ),
            ImageFillMode::RightCenter => {
                (x + width - image_width, y + (height - image_height) / 2.0)
            }
            ImageFillMode::LeftBottom => (x, y + height - image_height),
            ImageFillMode::CenterBottom => {
                (x + (width - image_width) / 2.0, y + height - image_height)
            }
            ImageFillMode::RightBottom => (x + width - image_width, y + height - image_height),
            _ => (x, y),
        }
    };
    let draw_missing_image_placeholder = |x: f32, y: f32, width: f32, height: f32| {
        let rect = Rect::from_xywh(x, y, width, height);
        let mut fill = Paint::default();
        fill.set_anti_alias(true);
        fill.set_style(skia_safe::paint::Style::Fill);
        fill.set_color(Color::from_argb(48, 96, 96, 96));
        canvas.draw_rect(rect, &fill);

        let mut stroke = Paint::default();
        stroke.set_anti_alias(true);
        stroke.set_style(skia_safe::paint::Style::Stroke);
        stroke.set_stroke_width(1.0);
        stroke.set_color(Color::from_argb(160, 96, 96, 96));
        canvas.draw_rect(rect, &stroke);
    };

    if !is_valid_destination_rect(x, y, width, height) {
        return;
    }
    let Some(image) = Image::from_encoded(Data::new_copy(bytes)) else {
        draw_missing_image_placeholder(x, y, width, height);
        return;
    };

    let dst = Rect::from_xywh(x, y, width, height);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    if let Some(color_filter) = image_effect_filter(effect) {
        paint.set_color_filter(color_filter);
    }

    let mode = fill_mode.unwrap_or(ImageFillMode::FitToSize);
    let decoded_width = image.width() as f32;
    let decoded_height = image.height() as f32;
    let crop_src = crop.and_then(|(left, top, right, bottom)| {
        if decoded_width <= 0.0 || decoded_height <= 0.0 {
            return None;
        }
        let scale_x = right as f32 / decoded_width;
        let scale_y = bottom as f32 / decoded_height;
        if scale_x <= 0.0 || scale_y <= 0.0 {
            return None;
        }
        let src_x = left as f32 / scale_x;
        let src_y = top as f32 / scale_y;
        let src_w = (right - left) as f32 / scale_x;
        let src_h = (bottom - top) as f32 / scale_y;
        let is_cropped = src_x > 0.5
            || src_y > 0.5
            || (src_w - decoded_width).abs() > 1.0
            || (src_h - decoded_height).abs() > 1.0;
        if is_cropped && src_w > 0.0 && src_h > 0.0 {
            Some(Rect::from_xywh(src_x, src_y, src_w, src_h))
        } else {
            None
        }
    });

    let draw_image_rect = |src: Option<Rect>, dst: Rect| {
        if let Some(src) = src.as_ref() {
            canvas.draw_image_rect_with_sampling_options(
                &image,
                Some((src, SrcRectConstraint::Strict)),
                dst,
                sampling.options(),
                &paint,
            );
        } else {
            canvas.draw_image_rect_with_sampling_options(
                &image,
                None,
                dst,
                sampling.options(),
                &paint,
            );
        }
    };

    if matches!(mode, ImageFillMode::FitToSize | ImageFillMode::None) {
        draw_image_rect(crop_src, dst);
        return;
    }

    let image_width = original_size
        .map(|(width, _)| width as f32)
        .unwrap_or_else(|| image.width() as f32);
    let image_height = original_size
        .map(|(_, height)| height as f32)
        .unwrap_or_else(|| image.height() as f32);
    if !is_valid_image_size(image_width, image_height) {
        draw_missing_image_placeholder(x, y, width, height);
        return;
    }

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
        let shader_image = crop_src
            .and_then(|src| {
                let left = src.left.floor().max(0.0) as i32;
                let top = src.top.floor().max(0.0) as i32;
                let right = src.right.ceil().min(decoded_width) as i32;
                let bottom = src.bottom.ceil().min(decoded_height) as i32;
                if right <= left || bottom <= top {
                    return None;
                }
                image.make_subset(
                    None,
                    IRect::from_xywh(left, top, right - left, bottom - top),
                    RequiredProperties::default(),
                )
            })
            .unwrap_or_else(|| image.clone());
        let shader_source_width = shader_image.width() as f32;
        let shader_source_height = shader_image.height() as f32;
        let draw_tiled_shader = |tile_rect: Rect, origin_x: f32, origin_y: f32| -> bool {
            if shader_source_width <= 0.0 || shader_source_height <= 0.0 {
                return false;
            }
            let scale_x = shader_source_width / image_width;
            let scale_y = shader_source_height / image_height;
            if !scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0 {
                return false;
            }
            let local_matrix = Matrix::scale_translate(
                (scale_x, scale_y),
                (-origin_x * scale_x, -origin_y * scale_y),
            );
            let Some(shader) = shader_image.to_shader(
                Some((TileMode::Repeat, TileMode::Repeat)),
                sampling.options(),
                Some(&local_matrix),
            ) else {
                return false;
            };
            let mut shader_paint = paint.clone();
            shader_paint.set_shader(shader);
            canvas.draw_rect(tile_rect, &shader_paint);
            true
        };

        if matches!(mode, ImageFillMode::TileAll) && draw_tiled_shader(dst, x, y) {
            canvas.restore();
            return;
        }
        if matches!(
            mode,
            ImageFillMode::TileHorzTop | ImageFillMode::TileHorzBottom
        ) {
            let tile_y = if matches!(mode, ImageFillMode::TileHorzTop) {
                y
            } else {
                y + height - image_height
            };
            if draw_tiled_shader(Rect::from_xywh(x, tile_y, width, image_height), x, tile_y) {
                canvas.restore();
                return;
            }
        }
        if matches!(
            mode,
            ImageFillMode::TileVertLeft | ImageFillMode::TileVertRight
        ) {
            let tile_x = if matches!(mode, ImageFillMode::TileVertLeft) {
                x
            } else {
                x + width - image_width
            };
            if draw_tiled_shader(Rect::from_xywh(tile_x, y, image_width, height), tile_x, y) {
                canvas.restore();
                return;
            }
        }
    } else {
        let (image_x, image_y) =
            resolve_image_placement(mode, x, y, width, height, image_width, image_height);
        draw_image_rect(
            crop_src,
            Rect::from_xywh(image_x, image_y, image_width, image_height),
        );
    }

    canvas.restore();
}

fn rasterize_svg_fragment_to_png(svg_fragment: &str, width: f32, height: f32) -> Option<Vec<u8>> {
    if svg_fragment.is_empty()
        || svg_fragment.len() > MAX_SVG_FRAGMENT_BYTES
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
    {
        return None;
    }
    let raster_width = width.ceil() as u64;
    let raster_height = height.ceil() as u64;
    if raster_width
        .checked_mul(raster_height)
        .is_none_or(|pixels| pixels > MAX_SVG_RASTER_PIXELS)
    {
        return None;
    }

    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.2}\" height=\"{height:.2}\" viewBox=\"0 0 {width:.2} {height:.2}\">{svg_fragment}</svg>"
    );
    let options = svg_parse_options();
    let tree = usvg::Tree::from_str(&svg, &options).ok()?;
    let size = tree.size().to_int_size();
    let pixels = u64::from(size.width()).checked_mul(u64::from(size.height()))?;
    if pixels == 0 || pixels > MAX_SVG_RASTER_PIXELS {
        return None;
    }

    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.encode_png().ok()
}

fn svg_parse_options() -> usvg::Options<'static> {
    let mut options = usvg::Options::default();
    options.resources_dir = None;
    options.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
        resolve_string: Box::new(|_, _| None),
    };
    let fontdb = options.fontdb_mut();
    fontdb.load_system_fonts();
    fontdb.set_sans_serif_family("Noto Sans CJK KR");
    fontdb.set_serif_family("Noto Serif CJK KR");
    fontdb.set_monospace_family("D2Coding");
    options
}
