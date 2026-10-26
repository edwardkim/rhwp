//! HTML JPEG 해독을 제공하고 공통 WMF 장치 재생 결과만 Canvas에 그린다.

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

pub(super) fn draw(context: &CanvasRenderingContext2d, bytes: &[u8]) -> bool {
    if !crate::paint::metafile_print::is_prepared(bytes) {
        return false;
    }
    let Ok(matrix) = context.get_transform() else {
        return false;
    };
    if matrix.b() != 0.0
        || matrix.c() != 0.0
        || matrix.e().fract() != 0.0
        || matrix.f().fract() != 0.0
    {
        return false;
    }
    let mut pending = false;
    let raster = crate::paint::metafile_print::rasterize(
        bytes,
        [matrix.a(), matrix.d(), matrix.e(), matrix.f()],
        |jpeg| {
            let Some(canvas) = super::cached_decoded_image_canvas(jpeg, super::hash_bytes(jpeg))
            else {
                pending = true;
                return None;
            };
            let source = canvas
                .get_context("2d")
                .ok()??
                .dyn_into::<CanvasRenderingContext2d>()
                .ok()?;
            let pixels = source
                .get_image_data(
                    0.0,
                    0.0,
                    f64::from(canvas.width()),
                    f64::from(canvas.height()),
                )
                .ok()?;
            image::RgbaImage::from_raw(canvas.width(), canvas.height(), pixels.data().0)
        },
    );
    // 브라우저 decode 대기는 실패나 legacy 출력으로 캐시하지 않는다.
    if pending {
        return true;
    }
    let Some(()) = (|| {
        let raster = raster?;
        let canvas = web_sys::window()?
            .document()?
            .create_element("canvas")
            .ok()?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        canvas.set_width(raster.width);
        canvas.set_height(raster.height);
        let target = canvas
            .get_context("2d")
            .ok()??
            .dyn_into::<CanvasRenderingContext2d>()
            .ok()?;
        let pixels = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&raster.rgba),
            raster.width,
            raster.height,
        )
        .ok()?;
        target.put_image_data(&pixels, 0.0, 0.0).ok()?;
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
    })() else {
        return false;
    };
    true
}
