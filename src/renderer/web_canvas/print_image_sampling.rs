//! Canvas2D는 Native와 같은 공통 정수 표본을 장치 좌표에 재생한다.

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[path = "print_jpeg_replay.rs"]
mod jpeg_replay;

/// 일반 bitmap 전처리 전에 실제 원본 canvas를 공통 표본에 전달한다.
pub(super) fn draw_encoded(
    context: &CanvasRenderingContext2d,
    bytes: &[u8],
    rectangle: (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> bool {
    if crate::renderer::image_resolver::png_sampling::print_separate_alpha(bytes) {
        return device_rectangle(context, rectangle)
            .and_then(|rectangle| {
                crate::renderer::print_image_sampling::rasterize_masked(
                    bytes, rectangle, page, scale,
                )
            })
            .and_then(|raster| replay_raster(context, &raster))
            .is_some();
    }
    let source = if bytes.starts_with(&[0xff, 0xd8]) {
        jpeg_replay::canvas(bytes)
    } else {
        super::cached_image_canvas(bytes)
    };
    source.is_some_and(|source| draw(context, &source, rectangle, page, scale))
}

pub(super) fn draw(
    context: &CanvasRenderingContext2d,
    source: &HtmlCanvasElement,
    rectangle: (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> bool {
    draw_inner(context, source, rectangle, page, scale).is_some()
}

fn device_rectangle(
    context: &CanvasRenderingContext2d,
    (x, y, w, h): (f64, f64, f64, f64),
) -> Option<(f64, f64, f64, f64)> {
    let matrix = context.get_transform().ok()?;
    if matrix.b() != 0.0 || matrix.c() != 0.0 || matrix.a() <= 0.0 || matrix.d() <= 0.0 {
        return None;
    }
    Some((
        matrix.a() * x + matrix.e(),
        matrix.d() * y + matrix.f(),
        matrix.a() * w,
        matrix.d() * h,
    ))
}

fn draw_inner(
    context: &CanvasRenderingContext2d,
    source: &HtmlCanvasElement,
    rectangle: (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> Option<()> {
    let rectangle = device_rectangle(context, rectangle)?;
    let source_context = source
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let pixels = source_context
        .get_image_data(
            0.0,
            0.0,
            f64::from(source.width()),
            f64::from(source.height()),
        )
        .ok()?;
    let raster = crate::renderer::print_image_sampling::rasterize(
        &pixels.data().0,
        (source.width(), source.height()),
        rectangle,
        page,
        scale,
    )?;
    replay_raster(context, &raster)
}

fn replay_raster(
    context: &CanvasRenderingContext2d,
    raster: &crate::renderer::print_image_sampling::DeviceRaster,
) -> Option<()> {
    let canvas = web_sys::window()?
        .document()?
        .create_element("canvas")
        .ok()?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    canvas.set_width(raster.width);
    canvas.set_height(raster.height);
    let destination = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let image = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(&raster.rgba),
        raster.width,
        raster.height,
    )
    .ok()?;
    destination.put_image_data(&image, 0.0, 0.0).ok()?;
    context.save();
    let result = context
        .set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
        .and_then(|()| {
            context.draw_image_with_html_canvas_element(
                &canvas,
                f64::from(raster.left),
                f64::from(raster.top),
            )
        });
    context.restore();
    result.ok()
}
