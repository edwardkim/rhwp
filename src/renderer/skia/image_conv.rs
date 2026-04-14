use skia_safe::{Data, Image, Paint, Rect};

pub fn draw_image_bytes(
    canvas: &skia_safe::Canvas,
    bytes: &[u8],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let data = Data::new_copy(bytes);
    if let Some(image) = Image::from_encoded(data) {
        let dst = Rect::from_xywh(x, y, width, height);
        let paint = Paint::default();
        canvas.draw_image_rect(image, None, dst, &paint);
    }
}
