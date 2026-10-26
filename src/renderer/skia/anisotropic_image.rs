//! 공통 축별 필터를 실제 Skia 장치 변환에서 재생한다.
use skia_safe::{
    image::CachingHint, images, AlphaType, Canvas, ColorType, Data, Image, ImageInfo, Paint, Rect,
};

pub(super) fn draw_print_frame(
    canvas: &Canvas,
    image: &Image,
    frame: super::super::print_raster::PrintRasterFrame,
    paint: &Paint,
) -> bool {
    let m = canvas.local_to_device_as_3x3();
    if m.has_perspective()
        || m.skew_x() != 0.0
        || m.skew_y() != 0.0
        || m.translate_x().fract() != 0.0
        || m.translate_y().fract() != 0.0
    {
        return false;
    }
    let info = ImageInfo::new(
        (image.width(), image.height()),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut rgba = vec![0; image.width() as usize * image.height() as usize * 4];
    if !image.read_pixels(
        &info,
        &mut rgba,
        image.width() as usize * 4,
        (0, 0),
        CachingHint::Disallow,
    ) {
        return false;
    }
    let Some(raster) = frame.rasterize(
        &rgba,
        (image.width() as u32, image.height() as u32),
        [f64::from(m.scale_x()), f64::from(m.scale_y()), 0.0, 0.0],
    ) else {
        return false;
    };
    let output_info = ImageInfo::new(
        (raster.width as i32, raster.height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(output) = images::raster_from_data(
        &output_info,
        Data::new_copy(&raster.rgba),
        raster.width as usize * 4,
    ) else {
        return false;
    };
    canvas.save();
    canvas.reset_matrix();
    canvas.draw_image(
        &output,
        (
            raster.left as f32 + m.translate_x(),
            raster.top as f32 + m.translate_y(),
        ),
        Some(paint),
    );
    canvas.restore();
    true
}

pub(super) fn draw(canvas: &Canvas, image: &Image, destination: Rect, paint: &Paint) -> bool {
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
        f64::from(destination.left() * matrix.scale_x() + matrix.translate_x()),
        f64::from(destination.top() * matrix.scale_y() + matrix.translate_y()),
        f64::from(destination.width() * matrix.scale_x()),
        f64::from(destination.height() * matrix.scale_y()),
    );
    if !super::super::anisotropic_sampling::mixed_reduction(
        (f64::from(image.width()), f64::from(image.height())),
        (rectangle.2, rectangle.3),
    ) {
        return false;
    }
    let info = ImageInfo::new(
        (image.width(), image.height()),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let mut rgba = vec![0; image.width() as usize * image.height() as usize * 4];
    if !image.read_pixels(
        &info,
        &mut rgba,
        image.width() as usize * 4,
        (0, 0),
        CachingHint::Disallow,
    ) {
        return false;
    }
    let Some(raster) = super::super::anisotropic_sampling::rasterize(
        &rgba,
        (image.width() as u32, image.height() as u32),
        rectangle,
    ) else {
        return false;
    };
    let output_info = ImageInfo::new(
        (raster.width as i32, raster.height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(output) = images::raster_from_data(
        &output_info,
        Data::new_copy(&raster.rgba),
        raster.width as usize * 4,
    ) else {
        return false;
    };
    canvas.save();
    canvas.reset_matrix();
    canvas.draw_image(
        &output,
        (rectangle.0.floor() as f32, rectangle.1.floor() as f32),
        Some(paint),
    );
    canvas.restore();
    true
}
