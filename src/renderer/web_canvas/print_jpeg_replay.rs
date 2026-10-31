//! 최종 Print JPEG의 공통 복호화 RGB를 Canvas 장치 표본에 전달한다.

use std::{cell::RefCell, collections::HashMap};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

const MAX_CACHE_BYTES: usize = 16 * 1024 * 1024;
const MAX_CACHE_ENTRIES: usize = 64;

#[derive(Default)]
struct Cache {
    bytes: usize,
    canvases: HashMap<u64, HtmlCanvasElement>,
}

thread_local! {
    // 화면용 decode cache와 분리한다. 동일 JPEG의 서로 다른 정수 변환을 섞지 않는다.
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
}

pub(super) fn canvas(data: &[u8]) -> Option<HtmlCanvasElement> {
    let key = super::super::hash_bytes(data);
    if let Some(cached) = CACHE.with(|cache| cache.borrow().canvases.get(&key).cloned()) {
        return Some(cached);
    }
    let rgb = crate::renderer::image_resolver::decode_print_jpeg_replay(data)?.into_rgb8();
    let (width, height) = rgb.dimensions();
    let length = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)?;
    let mut rgba = Vec::new();
    rgba.try_reserve_exact(length).ok()?;
    for pixel in rgb.as_raw().chunks_exact(3) {
        rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
    }
    drop(rgb);
    let image_data =
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), width, height).ok()?;
    let canvas = web_sys::window()?
        .document()?
        .create_element("canvas")
        .ok()?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?
        .put_image_data(&image_data, 0.0, 0.0)
        .ok()?;
    if length <= MAX_CACHE_BYTES {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.bytes + length > MAX_CACHE_BYTES || cache.canvases.len() >= MAX_CACHE_ENTRIES {
                cache.canvases.clear();
                cache.bytes = 0;
            }
            cache.bytes += length;
            cache.canvases.insert(key, canvas.clone());
        });
    }
    Some(canvas)
}
