//! Canvas2D가 제공하지 않는 고정 네 표본 재구성을 실제 장치 격자에 합성한다.

use std::cell::RefCell;
use std::collections::VecDeque;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

type Key = (u64, [u64; 4]);
struct Entry {
    key: Key,
    canvas: HtmlCanvasElement,
    bytes: usize,
}
thread_local! {
    // 원본 그림 캐시와 분리한다. 확대·장치 배율 변경에 낡은 표본을 쓰지 않는다.
    static CACHE: RefCell<VecDeque<Entry>> = const { RefCell::new(VecDeque::new()) };
}

pub(super) fn draw_print_frame(
    context: &CanvasRenderingContext2d,
    source: &HtmlCanvasElement,
    frame: super::super::print_raster::PrintRasterFrame,
) -> bool {
    let Some(()) = (|| {
        let m = context.get_transform().ok()?;
        if m.b() != 0.0 || m.c() != 0.0 || m.e().fract() != 0.0 || m.f().fract() != 0.0 {
            return None;
        }
        let ctx = source
            .get_context("2d")
            .ok()??
            .dyn_into::<CanvasRenderingContext2d>()
            .ok()?;
        let pixels = ctx
            .get_image_data(
                0.0,
                0.0,
                f64::from(source.width()),
                f64::from(source.height()),
            )
            .ok()?;
        let raster = frame.rasterize(
            &pixels.data().0,
            (source.width(), source.height()),
            [m.a(), m.d(), 0.0, 0.0],
        )?;
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
        let image = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&raster.rgba),
            raster.width,
            raster.height,
        )
        .ok()?;
        target.put_image_data(&image, 0.0, 0.0).ok()?;
        context.save();
        let result = context
            .set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
            .and_then(|()| {
                context.draw_image_with_html_canvas_element(
                    &canvas,
                    f64::from(raster.left) + m.e(),
                    f64::from(raster.top) + m.f(),
                )
            });
        context.restore();
        result.ok()
    })() else {
        return false;
    };
    true
}

fn prepared(
    source: &HtmlCanvasElement,
    key: Key,
    destination: (f64, f64, f64, f64),
) -> Option<HtmlCanvasElement> {
    if let Some(hit) = CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let index = cache.iter().position(|entry| entry.key == key)?;
        let entry = cache.remove(index)?;
        let canvas = entry.canvas.clone();
        cache.push_back(entry);
        Some(canvas)
    }) {
        return Some(hit);
    }
    let context = source
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let pixels = context
        .get_image_data(
            0.0,
            0.0,
            f64::from(source.width()),
            f64::from(source.height()),
        )
        .ok()?;
    let mixed = super::super::anisotropic_sampling::mixed_reduction(
        (f64::from(source.width()), f64::from(source.height())),
        (destination.2, destination.3),
    );
    let (rgba, width, height) = if mixed {
        let raster = super::super::anisotropic_sampling::rasterize(
            &pixels.data().0,
            (source.width(), source.height()),
            destination,
        )?;
        (raster.rgba, raster.width, raster.height)
    } else {
        let raster = super::super::cubic_sampling::rasterize(
            &pixels.data().0,
            (source.width(), source.height()),
            destination,
        )?;
        (raster.rgba, raster.width, raster.height)
    };
    let canvas = web_sys::window()?
        .document()?
        .create_element("canvas")
        .ok()?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    let image = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), width, height).ok()?;
    context.put_image_data(&image, 0.0, 0.0).ok()?;
    let bytes = rgba.len();
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        while cache.len() >= 64
            || cache.iter().map(|entry| entry.bytes).sum::<usize>() + bytes > 16 * 1024 * 1024
        {
            cache.pop_front();
        }
        cache.push_back(Entry {
            key,
            canvas: canvas.clone(),
            bytes,
        });
    });
    Some(canvas)
}

pub(super) fn draw(
    context: &CanvasRenderingContext2d,
    source: &HtmlCanvasElement,
    source_key: u64,
    rectangle: (f64, f64, f64, f64),
    allow_anisotropic: bool,
    allow_cubic: bool,
) -> bool {
    let Ok(matrix) = context.get_transform() else {
        return false;
    };
    if matrix.b() != 0.0 || matrix.c() != 0.0 || matrix.a() <= 0.0 || matrix.d() <= 0.0 {
        return false;
    }
    let (x, y, w, h) = rectangle;
    let rectangle = (
        matrix.a() * x + matrix.e(),
        matrix.d() * y + matrix.f(),
        matrix.a() * w,
        matrix.d() * h,
    );
    if !(allow_anisotropic
        && super::super::anisotropic_sampling::mixed_reduction(
            (f64::from(source.width()), f64::from(source.height())),
            (rectangle.2, rectangle.3),
        ))
        && !(allow_cubic
            && super::super::cubic_sampling::modest_reduction(
                (f64::from(source.width()), f64::from(source.height())),
                (rectangle.2, rectangle.3),
            ))
    {
        return false;
    }
    if !rectangle.0.is_finite()
        || !rectangle.1.is_finite()
        || ((rectangle.0 + rectangle.2).ceil() - rectangle.0.floor())
            * ((rectangle.1 + rectangle.3).ceil() - rectangle.1.floor())
            > 4_194_304.0
    {
        return false;
    }
    // 정수 이동은 표본을 바꾸지 않는다. 소수 원점과 실제 장치 크기만 캐시한다.
    let local = (
        rectangle.0 - rectangle.0.floor(),
        rectangle.1 - rectangle.1.floor(),
        rectangle.2,
        rectangle.3,
    );
    let key = (
        source_key,
        [
            local.0.to_bits(),
            local.1.to_bits(),
            rectangle.2.to_bits(),
            rectangle.3.to_bits(),
        ],
    );
    let Some(canvas) = prepared(source, key, local) else {
        return false;
    };
    context.save();
    // clip·alpha·filter는 유지하고 위치 변환만 장치 원점으로 바꾼다.
    let result = context
        .set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
        .and_then(|()| {
            context.draw_image_with_html_canvas_element(
                &canvas,
                rectangle.0.floor(),
                rectangle.1.floor(),
            )
        });
    context.restore();
    result.is_ok()
}
