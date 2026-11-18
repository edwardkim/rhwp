//! 그림의 실제 선택 영역을 축소할 때 Canvas의 저역 통과 품질을 선택한다.

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

/// 원본 장치에서 합성된 WMF 표면도 실제 출력이 더 작으면 Native와 같은 mip를 쓴다.
/// Canvas의 quality 문자열은 실제 mip 레벨 선택을 보장하지 않으므로 그 결과에 맡기지 않는다.
pub(super) fn draw_metafile_surface(
    context: &CanvasRenderingContext2d,
    bytes: &[u8],
    (x, y, width, height): (f64, f64, f64, f64),
) -> bool {
    if !crate::renderer::image_resolver::png_sampling::metafile_surface(bytes) {
        return false;
    }
    let draw = || {
        let matrix = context.get_transform().ok()?;
        if matrix.b() != 0.0 || matrix.c() != 0.0 || matrix.a() <= 0.0 || matrix.d() <= 0.0 {
            return None;
        }
        let header = crate::renderer::image_header::canvaskit_encoded_image_header(bytes)
            .filter(|header| header.is_within_decode_limits())?;
        if u64::from(header.width) * u64::from(header.height) > 32 * 1024 * 1024 {
            return None;
        }
        let source = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .ok()?
            .into_rgba8();
        let raster = crate::renderer::mipmap_sampling::rasterize(
            source.as_raw(),
            source.dimensions(),
            (
                matrix.a() * x + matrix.e(),
                matrix.d() * y + matrix.f(),
                matrix.a() * width,
                matrix.d() * height,
            ),
        )?;
        super::print_image_sampling::replay_raster(
            context,
            &crate::renderer::print_image_sampling::DeviceRaster {
                left: raster.left,
                top: raster.top,
                width: raster.width,
                height: raster.height,
                rgba: raster.rgba,
            },
        )
    };
    draw().is_some()
}

pub(super) struct ReductionQuality<'a> {
    context: &'a CanvasRenderingContext2d,
    previous: Option<JsValue>,
}

impl<'a> ReductionQuality<'a> {
    pub(super) fn for_source_window(
        context: &'a CanvasRenderingContext2d,
        source_width: f64,
        source_height: f64,
        width: f64,
        height: f64,
    ) -> Self {
        let needs_wider_filter = context.get_transform().is_ok_and(|matrix| {
            let scale_x = matrix.a().hypot(matrix.b());
            let scale_y = matrix.c().hypot(matrix.d());
            // 선형 필터의 축별 두 source 표본 범위 안에서는 기존 필터를 쓴다.
            // 이 범위를 넘는 축소에만 넓은 저역 통과 필터가 필요하다.
            let linear_source_support = 2.0;
            scale_x.is_finite()
                && scale_y.is_finite()
                && (source_width > width * scale_x * linear_source_support
                    || source_height > height * scale_y * linear_source_support)
        });
        // 현재 web-sys는 imageSmoothingQuality 접근자를 제공하지 않는다.
        // 브라우저가 실제로 제공하는 속성만 바꾸고 원래 값을 보관한다.
        let key = JsValue::from_str("imageSmoothingQuality");
        let previous = needs_wider_filter
            .then(|| js_sys::Reflect::get(context.as_ref(), &key).ok())
            .flatten()
            .filter(JsValue::is_string)
            .filter(|_| {
                js_sys::Reflect::set(context.as_ref(), &key, &JsValue::from_str("high"))
                    .unwrap_or(false)
            });
        Self { context, previous }
    }
}

impl Drop for ReductionQuality<'_> {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_ref() {
            let _ = js_sys::Reflect::set(
                self.context.as_ref(),
                &JsValue::from_str("imageSmoothingQuality"),
                previous,
            );
        }
    }
}
