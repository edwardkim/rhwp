//! Canvas2D는 Native와 같은 공통 정수 표본을 장치 좌표에 재생한다.

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

pub(super) fn draw(
    context: &CanvasRenderingContext2d,
    source: &HtmlCanvasElement,
    rectangle: (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> bool {
    draw_inner(context, source, rectangle, page, scale).is_some()
}

fn draw_inner(
    context: &CanvasRenderingContext2d,
    source: &HtmlCanvasElement,
    (x, y, w, h): (f64, f64, f64, f64),
    page: (f64, f64),
    scale: f64,
) -> Option<()> {
    let matrix = context.get_transform().ok()?;
    if matrix.b() != 0.0 || matrix.c() != 0.0 || matrix.a() <= 0.0 || matrix.d() <= 0.0 {
        return None;
    }
    let rectangle = (
        matrix.a() * x + matrix.e(),
        matrix.d() * y + matrix.f(),
        matrix.a() * w,
        matrix.d() * h,
    );
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
