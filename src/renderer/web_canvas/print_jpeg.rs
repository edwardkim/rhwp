//! 인쇄용 JPEG는 Native Skia와 같은 브라우저 codec으로 복원한다.
//!
//! 비동기 로딩은 기존 그림 cache·prefetch가 소유한다. 완료된 원본 픽셀만
//! 공통 Print 프레임 재표본기에 전달하며 크기·crop·장치 배율은 바꾸지 않는다.

use base64::Engine;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

pub(super) fn decode(data: &[u8]) -> Option<HtmlCanvasElement> {
    let header = crate::renderer::image_header::canvaskit_encoded_image_header(data)
        .filter(|header| header.is_within_decode_limits())?;
    let key = super::hash_bytes(data);
    let cached = super::IMAGE_CACHE.with(|cache| cache.borrow().get(&key).cloned());
    let image = match cached {
        Some(image) => image,
        None => {
            let image = HtmlImageElement::new().ok()?;
            let url = format!(
                "data:image/jpeg;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(data)
            );
            image.set_src(&url);
            super::IMAGE_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                if cache.len() > 200 {
                    cache.clear();
                }
                cache.insert(key, image.clone());
            });
            image
        }
    };
    if !image.complete()
        || image.natural_width() != header.width
        || image.natural_height() != header.height
    {
        return None;
    }
    let canvas = web_sys::window()?
        .document()?
        .create_element("canvas")
        .ok()?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    canvas.set_width(header.width);
    canvas.set_height(header.height);
    let context = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    context
        .draw_image_with_html_image_element(&image, 0.0, 0.0)
        .ok()?;
    // 원본 canvas를 decoded cache가 소유하므로 임시 HTML image를 중복 보유하지 않는다.
    super::IMAGE_CACHE.with(|cache| cache.borrow_mut().remove(&key));
    Some(canvas)
}
