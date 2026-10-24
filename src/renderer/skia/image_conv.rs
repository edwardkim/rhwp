use resvg::{tiny_skia, usvg};
use skia_safe::{
    canvas::SrcRectConstraint, color_filters, image::RequiredProperties, Color, ColorSpace,
    CubicResampler, Data, FilterMode, IRect, Image, Matrix, MipmapMode, Paint, Rect,
    SamplingOptions, TileMode,
};
use std::sync::{Arc, OnceLock};

use super::cache::byte_key;
use super::image_cache::{CachedImageResource, ImageResourceCache};
use super::image_effect::binary_effect_png;

use crate::model::image::ImageEffect;
use crate::model::style::ImageFillMode;
use crate::renderer::image_resolver::{detect_image_mime_type, grayscale_jpeg_bytes_to_png_bytes};

const MAX_SVG_FRAGMENT_BYTES: usize = 4 * 1024 * 1024;
// 완결된 WMF/EMF/EPS 그림은 내장 래스터를 data URI로 포함한다.
// 조각과 별도 예산을 사용하며 EPS 변환기의 16MiB 출력 범위도 수용한다.
// 조각 상한과 래스터 픽셀 상한은 유지한다.
const MAX_SVG_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
const MAX_SVG_RASTER_PIXELS: u64 = 67_108_864;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSampling {
    filter_mode: FilterMode,
    mipmap_mode: MipmapMode,
    profile: crate::paint::RenderProfile,
    paper_grid: Option<crate::renderer::bitmap_raster::PrintPaperGrid>,
}

impl ImageSampling {
    pub(super) fn nearest() -> Self {
        Self {
            filter_mode: FilterMode::Nearest,
            mipmap_mode: MipmapMode::None,
            profile: crate::paint::RenderProfile::Screen,
            paper_grid: None,
        }
    }

    pub(super) fn linear_mipmap() -> Self {
        Self {
            filter_mode: FilterMode::Linear,
            mipmap_mode: MipmapMode::Linear,
            profile: crate::paint::RenderProfile::Screen,
            paper_grid: None,
        }
    }

    pub fn linear() -> Self {
        Self {
            filter_mode: FilterMode::Linear,
            mipmap_mode: MipmapMode::None,
            profile: crate::paint::RenderProfile::Screen,
            paper_grid: None,
        }
    }

    pub fn for_profile(profile: crate::paint::RenderProfile) -> Self {
        Self {
            profile,
            ..Self::linear()
        }
    }

    pub(super) fn with_profile(mut self, profile: crate::paint::RenderProfile) -> Self {
        self.profile = profile;
        self
    }

    pub(super) fn with_paper_grid(
        mut self,
        grid: Option<crate::renderer::bitmap_raster::PrintPaperGrid>,
    ) -> Self {
        self.paper_grid = grid;
        self
    }

    pub(super) fn options(self) -> SamplingOptions {
        SamplingOptions::new(self.filter_mode, self.mipmap_mode)
    }
}

pub(super) fn draw_svg_fragment(
    canvas: &skia_safe::Canvas,
    cache: &ImageResourceCache,
    svg_fragment: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    sampling: ImageSampling,
    raster_scale: f32,
) -> bool {
    // [Issue #2292] RawSvg 조각은 페이지 절대 좌표로 방출된다(SVG 백엔드
    // 직접 삽입·web_canvas 와 동일 계약). viewBox 원점에 조각의 페이지
    // 위치(x, y)를 넘겨 bbox 창만 래스터한다 — (0,0) 가정 시 창 밖 콘텐츠
    // 전부 클리핑 + bbox 재배치 이중 오프셋으로 차트가 잘렸다.
    let key = byte_key(
        b"raw-svg",
        svg_fragment.as_bytes(),
        &(x, y, width, height, raster_scale),
    );
    let cached = cache.get(key, false);
    let png = if let Some(CachedImageResource::SvgPng(png)) = cached {
        png
    } else {
        let Some(png) =
            rasterize_svg_fragment_to_png(svg_fragment, x, y, width, height, raster_scale)
        else {
            return false;
        };
        let png = Arc::new(png);
        if let Some(key) = key {
            cache.insert(
                key,
                CachedImageResource::SvgPng(png.clone()),
                png.capacity(),
            );
        }
        png
    };
    draw_image_bytes(
        canvas,
        cache,
        &png,
        x,
        y,
        width,
        height,
        Some(ImageFillMode::FitToSize),
        None,
        None,
        None,
        ImageEffect::RealPic,
        0,
        0,
        sampling,
    )
}

pub(super) fn draw_image_bytes(
    canvas: &skia_safe::Canvas,
    cache: &ImageResourceCache,
    bytes: &[u8],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill_mode: Option<ImageFillMode>,
    original_size: Option<(f64, f64)>,
    crop: Option<(i32, i32, i32, i32)>,
    crop_reference_size: Option<(u32, u32)>,
    effect: ImageEffect,
    brightness: i8,
    contrast: i8,
    sampling: ImageSampling,
) -> bool {
    // 빈 자르기 선택은 정상적인 빈 출력이며 누락 그림 대체 표시 대상이 아니다.
    if crop.is_some_and(|(l, t, r, b)| r <= l || b <= t) {
        return true;
    }
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
    let brightness_contrast_filter = |brightness: i8, contrast: i8| {
        let brightness = brightness.clamp(-100, 100) as f32 / 100.0;
        let slope = (100.0 + contrast.clamp(-100, 100) as f32) / 100.0;
        // Skia의 matrix_4x5는 정규화된 RGBA에 적용된 뒤 0..1로 제한된다.
        // SVG와 같은 절편을 전달한다. 255를 곱하면 양수 밝기의 그림이 흰색으로 소실된다.
        let intercept = (0.5 - 0.5 * slope) + brightness;
        color_filters::matrix_row_major(
            &[
                slope, 0.0, 0.0, 0.0, intercept, 0.0, slope, 0.0, 0.0, intercept, 0.0, 0.0, slope,
                0.0, intercept, 0.0, 0.0, 0.0, 1.0, 0.0,
            ],
            None,
        )
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
        return false;
    }
    if effect == ImageEffect::RealPic
        && brightness == 0
        && contrast == 0
        && super::metafile_print::draw(canvas, bytes)
    {
        return true;
    }
    // [#3460] HWPX BinData 는 SVG 를 그대로 담는다(`<hc:img>` → `Format="svg"`). skia 디코더는
    // SVG 를 읽지 못해 종전에는 회색 자리표시자만 남았다. 같은 모듈의 resvg 로 목적지 크기에
    // 맞춰 먼저 래스터화하면 이후 크롭·채움·효과 경로를 그대로 탈 수 있다.
    let svg_size = crate::renderer::svg_fragment::is_svg_prefix(bytes)
        .then(|| (width, height, canvas_raster_scale(canvas)));
    let binary_effect =
        matches!(effect, ImageEffect::BlackWhite | ImageEffect::Pattern8x8).then_some(effect);
    // 같은 원본의 중립 캐시가 색조 변환을 우회하지 않도록 변환 조건도 구분한다.
    let tone = (brightness != 0 || contrast != 0).then_some((effect, brightness, contrast));
    let key = byte_key(b"decoded-image", bytes, &(svg_size, binary_effect, tone));
    let cached = cache.get(key, binary_effect.is_some());
    let (image, binary_preprocessed, encoded) = if let Some(CachedImageResource::Decoded {
        image,
        encoded,
        binary_preprocessed,
        ..
    }) = cached
    {
        (image, binary_preprocessed, encoded)
    } else {
        let svg_raster = svg_size.and_then(|(width, height, scale)| {
            rasterize_svg_document_to_png(bytes, width, height, scale)
        });
        let bytes = svg_raster.as_deref().unwrap_or(bytes);
        let normalized_bytes = if detect_image_mime_type(bytes) == "image/jpeg" {
            grayscale_jpeg_bytes_to_png_bytes(bytes)
        } else {
            None
        };
        let encoded_bytes = normalized_bytes.as_deref().unwrap_or(bytes);
        // 한컴 색조를 원본 픽셀에 먼저 확정하고 동일 캐시·배치 경로로 재생한다.
        // 재귀 입력은 보정 PNG와 중립 효과이므로 색조를 두 번 적용하지 않는다.
        if let Some(adjusted) = crate::renderer::image_resolver::adjusted_picture_bytes(
            encoded_bytes,
            effect,
            brightness,
            contrast,
        ) {
            return draw_image_bytes(
                canvas,
                cache,
                &adjusted,
                x,
                y,
                width,
                height,
                fill_mode,
                original_size,
                crop,
                crop_reference_size,
                ImageEffect::RealPic,
                0,
                0,
                sampling,
            );
        }
        // Binary effects are applied in source pixels before placement/filtering.
        // Embedded SVG is keyed by destination and device scale above, so its
        // raster-specific dither cannot leak to a differently sized placement.
        let binary_png = binary_effect.and_then(|effect| binary_effect_png(encoded_bytes, effect));
        let encoded_bytes = binary_png.as_deref().unwrap_or(encoded_bytes);
        // 디코더와 캐시가 같은 immutable Data를 공유한다. 캐시 hit도 실제
        // 정규화·전처리된 바이트를 전달해 원본/색조 전 바이트로 되돌리지 않는다.
        let encoded = Data::new_copy(encoded_bytes);
        let Some(image) = Image::from_encoded(encoded.clone()) else {
            if binary_effect.is_some() {
                cache.record_preprocessing(None);
            }
            draw_missing_image_placeholder(x, y, width, height);
            return false;
        };
        if binary_effect.is_some() {
            cache.record_preprocessing(binary_png.as_ref().map(|_| {
                (image.width().max(0) as usize)
                    .saturating_mul(image.height().max(0) as usize)
                    .saturating_mul(4)
            }));
        }
        if let Some(key) = key {
            let cost = (image.width().max(0) as usize)
                .saturating_mul(image.height().max(0) as usize)
                .saturating_mul(4)
                .saturating_add(encoded_bytes.len());
            cache.insert(
                key,
                CachedImageResource::Decoded {
                    image: image.clone(),
                    encoded: encoded.clone(),
                    binary_requested: binary_effect.is_some(),
                    binary_preprocessed: binary_png.is_some(),
                },
                cost,
            );
        }
        (image, binary_png.is_some(), encoded)
    };

    // Interpolation after source-pixel thresholding would reintroduce gray
    // pixels and shift ordered-dither phase. Failed preprocessing retains the
    // old filter/sampling fallback, including on a decoded-image cache hit.
    let (filter_effect, sampling) = if binary_preprocessed {
        (ImageEffect::RealPic, ImageSampling::nearest())
    } else {
        if binary_effect.is_some() {
            cache.record_filter_fallback();
        }
        (effect, sampling)
    };

    let dst = Rect::from_xywh(x, y, width, height);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    let has_adjustments = brightness != 0 || contrast != 0;
    let use_linear_adjustments =
        has_adjustments && matches!(effect, ImageEffect::RealPic | ImageEffect::GrayScale);
    let effect_key = byte_key(b"image-effect", &[], &(filter_effect, brightness, contrast));
    let cached_effect = cache.get(effect_key, false);
    let color_filter = if let Some(CachedImageResource::Effect(filter)) = cached_effect {
        filter
    } else {
        let effect_filter = image_effect_filter(filter_effect);
        let adjustment_filter = (brightness != 0 || contrast != 0)
            .then(|| brightness_contrast_filter(brightness, contrast));
        let filter = match (effect_filter, adjustment_filter) {
            (Some(effect), Some(adjustment)) if use_linear_adjustments => {
                color_filters::compose(effect, adjustment)
            }
            (Some(effect), Some(adjustment)) => color_filters::compose(adjustment, effect),
            (Some(effect), None) => Some(effect),
            (None, Some(adjustment)) => Some(adjustment),
            (None, None) => None,
        };
        if let Some(key) = effect_key {
            cache.insert(key, CachedImageResource::Effect(filter.clone()), 256);
        }
        filter
    };
    if let Some(color_filter) = color_filter {
        // SVG 필터의 기본 색 공간은 linearRGB다. 검증한 일반 그림·회색조의
        // 보정 체인 전체를 같은 공간에서 계산한다.
        let color_filter = if use_linear_adjustments {
            let Some(filter) = color_filter.with_working_color_space(ColorSpace::new_srgb_linear())
            else {
                return false;
            };
            filter
        } else {
            color_filter
        };
        paint.set_color_filter(color_filter);
    }

    let mode = fill_mode.unwrap_or(ImageFillMode::FitToSize);
    let decoded_width = image.width() as f32;
    let decoded_height = image.height() as f32;
    let crop_src = crop.and_then(|crop_rect| {
        if decoded_width <= 0.0 || decoded_height <= 0.0 {
            return None;
        }
        let (src_x, src_y, src_w, src_h) = crate::renderer::svg::compute_image_crop_src(
            crop_rect,
            crop_reference_size,
            decoded_width as f64,
            decoded_height as f64,
        );
        let (src_x, src_y, src_w, src_h) = (src_x as f32, src_y as f32, src_w as f32, src_h as f32);
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
        if sampling.mipmap_mode == MipmapMode::Linear && src.is_none() && !binary_preprocessed {
            if let Some(frame) = super::super::print_raster::PrintRasterFrame::read(bytes) {
                if super::anisotropic_image::draw_print_frame(canvas, &image, frame, &paint) {
                    return;
                }
            }
        }
        let matrix = canvas.local_to_device_as_3x3();
        let scale_x = matrix.scale_x().hypot(matrix.skew_y());
        let scale_y = matrix.skew_x().hypot(matrix.scale_y());
        let source = src.unwrap_or(Rect::from_xywh(0., 0., decoded_width, decoded_height));
        let interpolation = super::super::image_resolver::png_sampling::magnification(
            super::super::image_resolver::png_sampling::interpolation(bytes),
            (source.width() as f64, source.height() as f64),
            (
                (dst.width() * scale_x) as f64,
                (dst.height() * scale_y) as f64,
            ),
        );
        // 확대 선언과 binary effect의 우선순위를 보존한다.
        let sampling = if binary_preprocessed {
            sampling
        } else {
            match interpolation {
                Some(false) => ImageSampling::nearest().with_profile(sampling.profile),
                Some(true) => ImageSampling::linear().with_profile(sampling.profile),
                None => sampling,
            }
        };
        if sampling.mipmap_mode == MipmapMode::Linear
            && src.is_none()
            && !binary_preprocessed
            && super::super::image_resolver::png_sampling::print_jpeg_transfer(bytes)
            && super::anisotropic_image::draw(canvas, &image, dst, &paint)
        {
            return;
        }
        let transform = canvas.local_to_device_as_3x3();
        // SVG는 위에서 이미 실제 장치 크기로 raster했다. 이를 다시 bitmap
        // 축소 경로에 넣으면 벡터의 가는 선까지 두 번 필터링하게 된다.
        let prepared = (src.is_none()
            && svg_size.is_none()
            && !binary_preprocessed
            && !matrix.has_perspective()
            && sampling.filter_mode != FilterMode::Nearest)
            .then(|| {
                crate::renderer::bitmap_raster::prepare_bitmap(
                    encoded.as_bytes(),
                    crate::renderer::render_tree::BoundingBox::new(
                        dst.x() as f64,
                        dst.y() as f64,
                        dst.width() as f64,
                        dst.height() as f64,
                    ),
                    [
                        transform.scale_x() as f64,
                        transform.skew_y() as f64,
                        transform.skew_x() as f64,
                        transform.scale_y() as f64,
                        transform.translate_x() as f64,
                        transform.translate_y() as f64,
                    ],
                    sampling.profile,
                    sampling.paper_grid,
                )
            })
            .flatten();
        let filtered = prepared
            .as_ref()
            .and_then(|prepared| Image::from_encoded(Data::new_copy(prepared.bytes.as_slice())));
        let (image, dst) = match (filtered.as_ref(), prepared.as_ref()) {
            (Some(image), Some(prepared)) => (
                image,
                Rect::from_xywh(
                    prepared.bbox.x as f32,
                    prepared.bbox.y as f32,
                    prepared.bbox.width as f32,
                    prepared.bbox.height as f32,
                ),
            ),
            _ => (&image, dst),
        };
        // 전체 원본의 축소에는 가는 선을 보존하는 저역 통과를 적용한다.
        // crop의 strict 경계와 binary·빠른 미리보기의 명시적 nearest를 유지한다.
        let cubic = filtered.is_none()
            && sampling.mipmap_mode == MipmapMode::Linear
            && src.is_none()
            && !binary_preprocessed
            && super::super::image_resolver::png_sampling::print_bitmap_reconstruction(bytes)
            && !matrix.has_perspective()
            && matrix.skew_x() == 0.0
            && matrix.skew_y() == 0.0
            && matrix.scale_x() > 0.0
            && matrix.scale_y() > 0.0
            && super::super::cubic_sampling::modest_reduction(
                (f64::from(image.width()), f64::from(image.height())),
                (
                    f64::from(dst.width() * scale_x),
                    f64::from(dst.height() * scale_y),
                ),
            );
        let reduced = (filtered.is_none()
            && !cubic
            && sampling.filter_mode != FilterMode::Nearest
            && src.is_none()
            && !binary_preprocessed
            && !matrix.has_perspective()
            && scale_x.is_finite()
            && scale_y.is_finite()
            && (image.width() as f32 > dst.width() * scale_x
                || image.height() as f32 > dst.height() * scale_y))
            .then(|| image.with_default_mipmaps())
            .flatten();
        let actual_image = reduced.as_ref().unwrap_or(image);
        let options = if filtered.is_some() {
            SamplingOptions::new(FilterMode::Nearest, MipmapMode::None)
        } else if cubic {
            SamplingOptions::from(CubicResampler::catmull_rom())
        } else if reduced.is_some() {
            SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear)
        } else {
            sampling.options()
        };
        if let Some(src) = src.as_ref() {
            canvas.draw_image_rect_with_sampling_options(
                actual_image,
                Some((src, SrcRectConstraint::Strict)),
                dst,
                options,
                &paint,
            );
        } else {
            canvas.draw_image_rect_with_sampling_options(actual_image, None, dst, options, &paint);
        }
    };

    if matches!(mode, ImageFillMode::FitToSize | ImageFillMode::Total) {
        draw_image_rect(crop_src, dst);
        return true;
    }
    if matches!(mode, ImageFillMode::None | ImageFillMode::Zoom) {
        // [#7235] 채우기 유형 15(NONE): 종횡비를 지켜 영역 가운데에 맞춘다.
        let (source_width, source_height) = crop_src
            .map(|src| (src.width(), src.height()))
            .unwrap_or((decoded_width, decoded_height));
        if !is_valid_image_size(source_width, source_height) {
            draw_missing_image_placeholder(x, y, width, height);
            return false;
        }
        let scale = (width / source_width).min(height / source_height);
        let fit_width = source_width * scale;
        let fit_height = source_height * scale;
        draw_image_rect(
            crop_src,
            Rect::from_xywh(
                x + (width - fit_width) / 2.0,
                y + (height - fit_height) / 2.0,
                fit_width,
                fit_height,
            ),
        );
        return true;
    }

    let image_width = original_size
        .map(|(width, _)| width as f32)
        .unwrap_or_else(|| image.width() as f32);
    let image_height = original_size
        .map(|(_, height)| height as f32)
        .unwrap_or_else(|| image.height() as f32);
    if !is_valid_image_size(image_width, image_height) {
        draw_missing_image_placeholder(x, y, width, height);
        return false;
    }

    canvas.save();
    canvas.clip_rect(dst, None, Some(true));

    if matches!(
        mode,
        ImageFillMode::TileAll
            | ImageFillMode::Total
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
            let scale_x = image_width / shader_source_width;
            let scale_y = image_height / shader_source_height;
            if !scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0 {
                return false;
            }
            // Skia's local matrix maps source pixels into logical canvas
            // coordinates; shader lookup performs the inverse internally.
            let local_matrix = Matrix::scale_translate((scale_x, scale_y), (origin_x, origin_y));
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

        if matches!(mode, ImageFillMode::TileAll | ImageFillMode::Total)
            && draw_tiled_shader(dst, x, y)
        {
            canvas.restore();
            return true;
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
                return true;
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
                return true;
            }
        }
        // A valid decoded image can still lack a usable shader matrix. Retain
        // the committed bounded draw fallback instead of dropping that image.
        let step_x = image_width.max(1.0);
        let step_y = image_height.max(1.0);
        let horizontal = matches!(
            mode,
            ImageFillMode::TileHorzTop | ImageFillMode::TileHorzBottom
        );
        let vertical = matches!(
            mode,
            ImageFillMode::TileVertLeft | ImageFillMode::TileVertRight
        );
        let columns = if vertical {
            1
        } else {
            (width / step_x).ceil() as usize
        };
        let rows = if horizontal {
            1
        } else {
            (height / step_y).ceil() as usize
        };
        let draws = columns.saturating_mul(rows);
        let origin_x = if mode == ImageFillMode::TileVertRight {
            x + width - image_width
        } else {
            x
        };
        let origin_y = if mode == ImageFillMode::TileHorzBottom {
            y + height - image_height
        } else {
            y
        };
        const MAX_TILE_DRAWS: usize = 4096;
        for index in 0..draws.min(MAX_TILE_DRAWS) {
            let tile_x = origin_x + (index % columns) as f32 * step_x;
            let tile_y = origin_y + (index / columns) as f32 * step_y;
            draw_image_rect(
                crop_src,
                Rect::from_xywh(tile_x, tile_y, image_width, image_height),
            );
        }
        if draws > MAX_TILE_DRAWS {
            cache.record_tile_cap();
        }
        canvas.restore();
        return true;
    } else {
        let (image_x, image_y) =
            resolve_image_placement(mode, x, y, width, height, image_width, image_height);
        draw_image_rect(
            crop_src,
            Rect::from_xywh(image_x, image_y, image_width, image_height),
        );
    }

    canvas.restore();
    true
}

/// [#3460] 캔버스 변환에서 래스터 배율을 얻는다 (확대 출력에서 SVG 가 뭉개지지 않게).
fn canvas_raster_scale(canvas: &skia_safe::Canvas) -> f32 {
    let matrix = canvas.local_to_device_as_3x3();
    // Host canvases can rotate or shear the page. Diagonal entries alone lose
    // that scale (a quarter turn has zero diagonals); use the largest stretch
    // while preserving the existing bounded SVG document rasterization policy.
    let scale = matrix.max_scale();
    if scale.is_finite() && scale > 0.0 {
        scale.clamp(1.0, 4.0)
    } else {
        1.0
    }
}

/// [#3460] 완결된 SVG 문서(BinData 그림)를 목적지 크기에 맞춰 PNG 로 래스터화한다.
///
/// 조각(fragment) 경로와 달리 원본이 자체 `viewBox`/크기를 가진 문서이므로, 트리 크기를
/// 목적지 픽셀 크기로 맞추는 스케일 변환만 적용한다.
fn rasterize_svg_document_to_png(
    bytes: &[u8],
    width: f32,
    height: f32,
    raster_scale: f32,
) -> Option<Vec<u8>> {
    if bytes.is_empty()
        || bytes.len() > MAX_SVG_DOCUMENT_BYTES
        || !width.is_finite()
        || !height.is_finite()
        || !raster_scale.is_finite()
        || width <= 0.0
        || height <= 0.0
        || raster_scale <= 0.0
    {
        return None;
    }
    let raster_width = (width * raster_scale).ceil().max(1.0) as u64;
    let raster_height = (height * raster_scale).ceil().max(1.0) as u64;
    if raster_width
        .checked_mul(raster_height)
        .is_none_or(|pixels| pixels > MAX_SVG_RASTER_PIXELS)
    {
        return None;
    }

    let options = svg_parse_options();
    let tree = usvg::Tree::from_data(bytes, &options).ok()?;
    let size = tree.size();
    if !(size.width() > 0.0 && size.height() > 0.0) {
        return None;
    }

    if let Some(png) = super::svg_bitmap_mask::rasterize_before_reduction(
        &tree,
        raster_width as u32,
        raster_height as u32,
        MAX_SVG_RASTER_PIXELS,
    ) {
        return Some(png);
    }

    let transform = tiny_skia::Transform::from_scale(
        raster_width as f32 / size.width(),
        raster_height as f32 / size.height(),
    );
    rasterize_svg_with_mask_coverage(
        &tree,
        raster_width as u32,
        raster_height as u32,
        transform,
        raster_scale,
    )
}

fn rasterize_svg_fragment_to_png(
    svg_fragment: &str,
    src_x: f32,
    src_y: f32,
    width: f32,
    height: f32,
    raster_scale: f32,
) -> Option<Vec<u8>> {
    if svg_fragment.is_empty()
        || svg_fragment.len() > MAX_SVG_FRAGMENT_BYTES
        || !src_x.is_finite()
        || !src_y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || !raster_scale.is_finite()
        || width <= 0.0
        || height <= 0.0
        || raster_scale <= 0.0
    {
        return None;
    }
    let output_width = width * raster_scale;
    let output_height = height * raster_scale;
    let raster_width = output_width.ceil() as u64;
    let raster_height = output_height.ceil() as u64;
    if raster_width
        .checked_mul(raster_height)
        .is_none_or(|pixels| pixels > MAX_SVG_RASTER_PIXELS)
    {
        return None;
    }

    // [Issue #2292] 조각은 페이지 절대 좌표 — viewBox 를 조각의 페이지 좌표
    // 창(src_x, src_y 원점)으로 지정해 bbox 영역만 (0,0) 래스터로 사상한다.
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{output_width:.2}\" height=\"{output_height:.2}\" viewBox=\"{src_x:.2} {src_y:.2} {width:.2} {height:.2}\">{svg_fragment}</svg>"
    );
    let options = svg_parse_options();
    let tree = usvg::Tree::from_str(&svg, &options).ok()?;
    let size = tree.size().to_int_size();
    let pixels = u64::from(size.width()).checked_mul(u64::from(size.height()))?;
    if pixels == 0 || pixels > MAX_SVG_RASTER_PIXELS {
        return None;
    }

    rasterize_svg_with_mask_coverage(
        &tree,
        size.width(),
        size.height(),
        tiny_skia::Transform::default(),
        raster_scale,
    )
}

/// 축소된 패턴 마스크의 피복률을 면적으로 적분한다. 위치·viewport·최종 크기는 보존한다.
fn rasterize_svg_with_mask_coverage(
    tree: &usvg::Tree,
    width: u32,
    height: u32,
    transform: tiny_skia::Transform,
    raster_scale: f32,
) -> Option<Vec<u8>> {
    // WMF의 미세한 흑백 마스크를 목적지 픽셀에서 한 번만 샘플링하면
    // 검정/흰색 중 한쪽만 선택되어 칠과 글자가 사라진다. 마스크가 있는
    // SVG에만 축마다 최대 4배 표본을 두고 최종 출력 크기로 저역 통과한다.
    // 이미 고배율인 출력과 마스크 없는 SVG는 기존 경로를 유지한다.
    let mut samples = if tree.masks().is_empty() {
        1
    } else {
        (4.0 / raster_scale.max(1.0)).floor().clamp(1.0, 4.0) as u32
    };
    let pixels = u64::from(width).checked_mul(u64::from(height))?;
    if pixels == 0 || pixels > MAX_SVG_RASTER_PIXELS {
        return None;
    }
    while pixels.checked_mul(u64::from(samples).pow(2))? > MAX_SVG_RASTER_PIXELS {
        samples -= 1;
    }
    let render_width = width.checked_mul(samples)?;
    let render_height = height.checked_mul(samples)?;
    let mut pixmap = tiny_skia::Pixmap::new(render_width, render_height)?;
    let transform =
        tiny_skia::Transform::from_scale(samples as f32, samples as f32).post_concat(transform);
    resvg::render(tree, transform, &mut pixmap.as_mut());
    if samples == 1 {
        return pixmap.encode_png().ok();
    }

    // 투명 가장자리의 색 번짐을 막기 위해 premultiplied RGBA 상태에서 적분한다.
    // PNG 인코더가 마지막에만 알파를 되돌린다. Lanczos의 작은 overshoot도
    // premultiplied 불변식(R/G/B <= A) 안으로 제한한다.
    super::svg_bitmap_mask::encode_reduced(&pixmap, width, height)
}

fn svg_parse_options() -> usvg::Options<'static> {
    let mut options = usvg::Options::default();
    options.resources_dir = None;
    options.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
        resolve_string: Box::new(|_, _| None),
    };
    options.fontdb = svg_fontdb();
    options
}

fn svg_fontdb() -> Arc<usvg::fontdb::Database> {
    static SVG_FONTDB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();

    SVG_FONTDB
        .get_or_init(|| {
            // [Issue #2293] PDF 경로(renderer/pdf.rs::create_fontdb)와 동일
            // 규약: 시스템 폰트 + 프로젝트 ttfs/(재귀) + WSL 윈도우 폰트.
            // 종전에는 시스템 폰트만 로드하고 generic 폴백을 존재 확인 없이
            // 하드 고정("Noto Sans CJK KR")해, 해당 폰트가 없는 환경에서
            // resvg 가 조각의 텍스트를 통째로 드롭했다.
            let mut fontdb = usvg::fontdb::Database::new();
            fontdb.load_system_fonts();
            // [#2864] 조달 순서는 renderer::font_paths 가 단일 정의한다.
            // ttfs/opensource 번들이 최후 폴백으로 남아 한국어 드롭을 막는다(#2293).
            crate::renderer::font_paths::load_into_fontdb(&mut fontdb, &[]);

            // generic 폴백은 실존하는 첫 후보로 (매칭 실패 = 텍스트 드롭 방지).
            let sans = first_existing_family(
                &fontdb,
                &[
                    "Noto Sans CJK KR",
                    "Noto Sans KR",
                    "함초롬돋움",
                    "HCR Dotum",
                    "맑은 고딕",
                    "Malgun Gothic",
                    "NanumGothic",
                    "나눔고딕",
                    "DejaVu Sans",
                ],
            );
            // [작업지시자 권고] 폴백은 한국어 가용 폰트를 우선한다 — 스타일
            // (serif/mono) 정합보다 한글이 보이는 것이 우선이므로, 라틴 전용
            // 최후 폴백(DejaVu) 앞에 한국어 sans 를 둔다. 저장소 체크아웃에는
            // ttfs/opensource/NotoSansKR 이 항상 있어 한국어 폴백이 보장된다.
            let serif = first_existing_family(
                &fontdb,
                &[
                    "Noto Serif CJK KR",
                    "Noto Serif KR",
                    "함초롬바탕",
                    "HCR Batang",
                    "바탕",
                    "Batang",
                    "NanumMyeongjo",
                    "나눔명조",
                    "Noto Sans KR",
                    "DejaVu Serif",
                ],
            );
            let mono = first_existing_family(
                &fontdb,
                &[
                    "D2Coding",
                    "D2Coding ligature",
                    "Noto Sans KR",
                    "DejaVu Sans Mono",
                ],
            );
            if let Some(f) = sans {
                fontdb.set_sans_serif_family(f);
            }
            if let Some(f) = serif {
                fontdb.set_serif_family(f);
            }
            if let Some(f) = mono {
                fontdb.set_monospace_family(f);
            }
            Arc::new(fontdb)
        })
        .clone()
}

/// [Issue #2293] fontdb 에 실존하는 첫 패밀리 — 후보가 전부 없으면 None
/// (usvg 기본 generic 매핑 유지, 폴백 지정으로 오히려 드롭되는 것을 방지).
fn first_existing_family(fontdb: &usvg::fontdb::Database, candidates: &[&str]) -> Option<String> {
    let mut families = std::collections::HashSet::new();
    for face in fontdb.faces() {
        for (name, _) in &face.families {
            families.insert(name.clone());
        }
    }
    candidates
        .iter()
        .find(|c| families.contains(**c))
        .map(|c| c.to_string())
}
