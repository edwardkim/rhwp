//! M06-f 생명주기·치수·오류 Display 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::render_backend::{
    error_display_holds, page_size_cases_hold, run_lifecycle, standard_lifecycle_scripts,
    LifecycleExpect, LifecycleStep, NullBackend, PageSize, PngBackend, RenderBackend,
    RenderBackendError, SceneOp, SkiaBackend, SvgBackend, TraceBackend, PAGE_SIZE_CASES,
};

fn draw_rect(
    backend: &mut impl RenderBackend<Error = RenderBackendError>,
) -> Result<(), RenderBackendError> {
    backend.draw(&rhwp::render_backend::materialize_scene_op(&SceneOp::new(
        "rectangle",
        0.0,
        0.0,
        10.0,
        10.0,
    )))
}

#[test]
fn page_size_table_matches_is_valid() {
    page_size_cases_hold().unwrap();
    assert!(PAGE_SIZE_CASES.len() >= 10);
    let valid = PAGE_SIZE_CASES.iter().filter(|c| c.valid).count();
    let invalid = PAGE_SIZE_CASES.iter().filter(|c| !c.valid).count();
    assert!(valid >= 3 && invalid >= 5);
}

#[test]
fn error_display_contains_tokens() {
    let errors = [
        RenderBackendError::NoOpenPage { call: "draw" },
        RenderBackendError::NoOpenPage { call: "end_page" },
        RenderBackendError::PageAlreadyOpen,
        RenderBackendError::UnclosedPage { pages_completed: 0 },
        RenderBackendError::InvalidPageSize {
            width: 0.0,
            height: 1.0,
        },
        RenderBackendError::UnsupportedOp {
            backend: "svg",
            op: "clip",
        },
        RenderBackendError::MultiplePagesUnsupported { backend: "svg" },
        RenderBackendError::Backend("boom".into()),
    ];
    for err in &errors {
        error_display_holds(err).unwrap();
    }
}

fn run_on_null(script_id: &str) {
    let script = standard_lifecycle_scripts()
        .iter()
        .find(|s| s.id == script_id)
        .unwrap();
    let mut backend = NullBackend::new();
    run_lifecycle(&mut backend, script, draw_rect).unwrap();
    if matches!(
        script.rows.last().map(|r| &r.step),
        Some(LifecycleStep::Finish)
    ) {
        match script.rows.last().unwrap().expect {
            LifecycleExpect::Ok => {
                backend.finish().unwrap();
            }
            LifecycleExpect::Err(_) => {
                assert!(backend.finish().is_err());
            }
        }
    }
}

#[test]
fn lifecycle_draw_without_begin() {
    run_on_null("draw-without-begin");
}

#[test]
fn lifecycle_end_without_begin() {
    run_on_null("end-without-begin");
}

#[test]
fn lifecycle_double_begin() {
    run_on_null("double-begin");
}

#[test]
fn lifecycle_finish_while_open() {
    run_on_null("finish-while-open");
}

#[test]
fn lifecycle_empty_page() {
    run_on_null("empty-page");
}

#[test]
fn lifecycle_one_draw() {
    run_on_null("one-draw");
}

#[test]
fn lifecycle_invalid_then_valid() {
    run_on_null("invalid-then-valid");
}

#[test]
fn trace_draw_without_begin_is_no_open_page() {
    let mut backend = TraceBackend::new();
    let err = draw_rect(&mut backend).unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "draw" });
}

#[test]
fn trace_end_without_begin_is_no_open_page() {
    let mut backend = TraceBackend::new();
    let err = backend.end_page().unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "end_page" });
}

#[test]
fn trace_invalid_size_rejected() {
    let mut backend = TraceBackend::new();
    let err = backend.begin_page(PageSize::new(0.0, 10.0)).unwrap_err();
    assert_eq!(
        err,
        RenderBackendError::InvalidPageSize {
            width: 0.0,
            height: 10.0
        }
    );
}

#[test]
fn svg_draw_without_begin_is_no_open_page() {
    let mut backend = SvgBackend::new();
    let err = draw_rect(&mut backend).unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "draw" });
}

#[test]
fn svg_end_without_begin_is_no_open_page() {
    let mut backend = SvgBackend::new();
    let err = backend.end_page().unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "end_page" });
}

#[test]
fn svg_invalid_size_rejected() {
    let mut backend = SvgBackend::new();
    let err = backend.begin_page(PageSize::new(0.0, 10.0)).unwrap_err();
    assert_eq!(
        err,
        RenderBackendError::InvalidPageSize {
            width: 0.0,
            height: 10.0
        }
    );
}

#[test]
fn png_draw_without_begin_is_no_open_page() {
    let mut backend = PngBackend::new();
    let err = draw_rect(&mut backend).unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "draw" });
}

#[test]
fn png_end_without_begin_is_no_open_page() {
    let mut backend = PngBackend::new();
    let err = backend.end_page().unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "end_page" });
}

#[test]
fn png_invalid_size_rejected() {
    let mut backend = PngBackend::new();
    let err = backend.begin_page(PageSize::new(0.0, 10.0)).unwrap_err();
    assert_eq!(
        err,
        RenderBackendError::InvalidPageSize {
            width: 0.0,
            height: 10.0
        }
    );
}

#[test]
fn skia_draw_without_begin_is_no_open_page() {
    let mut backend = SkiaBackend::new();
    let err = draw_rect(&mut backend).unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "draw" });
}

#[test]
fn skia_end_without_begin_is_no_open_page() {
    let mut backend = SkiaBackend::new();
    let err = backend.end_page().unwrap_err();
    assert_eq!(err, RenderBackendError::NoOpenPage { call: "end_page" });
}

#[test]
fn skia_invalid_size_rejected() {
    let mut backend = SkiaBackend::new();
    let err = backend.begin_page(PageSize::new(0.0, 10.0)).unwrap_err();
    assert_eq!(
        err,
        RenderBackendError::InvalidPageSize {
            width: 0.0,
            height: 10.0
        }
    );
}
