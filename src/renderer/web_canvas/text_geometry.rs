//! Native와 같은 Print 기하 정책을 Canvas의 표준 래스터 힌트로 전달한다.
use crate::paint::RenderProfile;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub(super) fn configure(context: &CanvasRenderingContext2d, profile: RenderProfile) {
    let property = JsValue::from_str("textRendering");
    if !js_sys::Reflect::has(context.as_ref(), &property).unwrap_or(false) {
        return;
    }
    let mode = if profile.preserves_raster_text_geometry() {
        "geometricPrecision"
    } else {
        "auto"
    };
    let _ = js_sys::Reflect::set(context.as_ref(), &property, &JsValue::from_str(mode));
}
