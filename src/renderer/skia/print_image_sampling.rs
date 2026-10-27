//! Native Skia는 공통 장치 표본을 재생하며 축소 수학을 다시 구현하지 않는다.

use skia_safe::{AlphaType, Canvas, ColorType, Data, Image, ImageInfo, Paint, Rect};

pub(super) fn draw_masked(
    canvas: &Canvas,
    encoded: &[u8],
    destination: Rect,
    page: (f64, f64),
    scale: f64,
) -> bool {
    let matrix = canvas.local_to_device_as_3x3();
    if matrix.has_perspective()
        || matrix.skew_x() != 0.0
        || matrix.skew_y() != 0.0
        || matrix.scale_x() <= 0.0
        || matrix.scale_y() <= 0.0
    {
        return false;
    }
    let rectangle = (
        f64::from(matrix.scale_x()) * f64::from(destination.left())
            + f64::from(matrix.translate_x()),
        f64::from(matrix.scale_y()) * f64::from(destination.top())
            + f64::from(matrix.translate_y()),
        f64::from(matrix.scale_x()) * f64::from(destination.width()),
        f64::from(matrix.scale_y()) * f64::from(destination.height()),
    );
    let Some(raster) =
        crate::renderer::print_image_sampling::rasterize_masked(encoded, rectangle, page, scale)
    else {
        return false;
    };
    paint_raster(canvas, &raster, &Paint::default(), AlphaType::Unpremul)
}

pub(super) fn draw(
    canvas: &Canvas,
    source: &Image,
    destination: Rect,
    paint: &Paint,
    page: (f64, f64),
    scale: f64,
) -> bool {
    let matrix = canvas.local_to_device_as_3x3();
    if matrix.has_perspective()
        || matrix.skew_x() != 0.0
        || matrix.skew_y() != 0.0
        || matrix.scale_x() <= 0.0
        || matrix.scale_y() <= 0.0
    {
        return false;
    }
    let (width, height) = (source.width(), source.height());
    if width <= 0
        || height <= 0
        || width > 8192
        || height > 8192
        || i64::from(width) * i64::from(height) > 32 * 1024 * 1024
    {
        return false;
    }
    let rectangle = (
        f64::from(matrix.scale_x()) * f64::from(destination.left())
            + f64::from(matrix.translate_x()),
        f64::from(matrix.scale_y()) * f64::from(destination.top())
            + f64::from(matrix.translate_y()),
        f64::from(matrix.scale_x()) * f64::from(destination.width()),
        f64::from(matrix.scale_y()) * f64::from(destination.height()),
    );
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let stride = width as usize * 4;
    let mut pixels = vec![0u8; stride * height as usize];
    if !source.read_pixels(
        &info,
        &mut pixels,
        stride,
        (0, 0),
        skia_safe::image::CachingHint::Disallow,
    ) {
        return false;
    }
    let Some(raster) = crate::renderer::print_image_sampling::rasterize(
        &pixels,
        (width as u32, height as u32),
        rectangle,
        page,
        scale,
    ) else {
        return false;
    };
    paint_raster(canvas, &raster, paint, AlphaType::Opaque)
}

fn paint_raster(
    canvas: &Canvas,
    raster: &crate::renderer::print_image_sampling::DeviceRaster,
    paint: &Paint,
    alpha: AlphaType,
) -> bool {
    let info = ImageInfo::new(
        (raster.width as i32, raster.height as i32),
        ColorType::RGBA8888,
        alpha,
        None,
    );
    let Some(image) = skia_safe::images::raster_from_data(
        &info,
        Data::new_copy(&raster.rgba),
        raster.width as usize * 4,
    ) else {
        return false;
    };
    canvas.save();
    canvas.reset_matrix();
    canvas.draw_image(&image, (raster.left as f32, raster.top as f32), Some(paint));
    canvas.restore();
    true
}
