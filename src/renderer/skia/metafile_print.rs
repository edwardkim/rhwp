//! 준비된 WMF를 실제 Skia CTM과 host JPEG codec으로 공통 재생한다.

use skia_safe::{image::CachingHint, images, AlphaType, Canvas, ColorType, Data, Image, ImageInfo};

pub(super) fn draw(canvas: &Canvas, bytes: &[u8]) -> bool {
    if !crate::paint::metafile_print::is_prepared(bytes) {
        return false;
    }
    let matrix = canvas.local_to_device_as_3x3();
    if matrix.has_perspective()
        || matrix.skew_x() != 0.0
        || matrix.skew_y() != 0.0
        || matrix.translate_x().fract() != 0.0
        || matrix.translate_y().fract() != 0.0
    {
        return false;
    }
    let Some(raster) = crate::paint::metafile_print::rasterize(
        bytes,
        [
            f64::from(matrix.scale_x()),
            f64::from(matrix.scale_y()),
            f64::from(matrix.translate_x()),
            f64::from(matrix.translate_y()),
        ],
        |jpeg| {
            let image = Image::from_encoded(Data::new_copy(jpeg))?;
            let info = ImageInfo::new(
                (image.width(), image.height()),
                ColorType::RGBA8888,
                AlphaType::Unpremul,
                None,
            );
            let header = super::super::image_header::canvaskit_encoded_image_header(jpeg)
                .filter(|header| header.is_within_decode_limits())?;
            if (image.width() as u32, image.height() as u32) != (header.width, header.height) {
                return None;
            }
            let mut rgba = vec![0; image.width() as usize * image.height() as usize * 4];
            image
                .read_pixels(
                    &info,
                    &mut rgba,
                    image.width() as usize * 4,
                    (0, 0),
                    CachingHint::Disallow,
                )
                .then(|| image::RgbaImage::from_raw(header.width, header.height, rgba))
                .flatten()
        },
    ) else {
        return false;
    };
    let info = ImageInfo::new(
        (raster.width as i32, raster.height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let Some(image) = images::raster_from_data(
        &info,
        Data::new_copy(&raster.rgba),
        raster.width as usize * 4,
    ) else {
        return false;
    };
    canvas.save();
    canvas.reset_matrix();
    canvas.draw_image(&image, (raster.left as f32, raster.top as f32), None);
    canvas.restore();
    true
}
