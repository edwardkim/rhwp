//! M06-f 광고 vs 실지원 정직성.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::render_backend::{
    expected_honesty_table, honesty_table_holds, observe_svg, replay_page, BackendFeature,
    NullBackend, PageSize, PngBackend, RenderBackend, RenderBackendError, SceneOp, SceneSpec,
    SkiaBackend, SvgBackend, TraceBackend, ALL_FEATURES, HONESTY_TEXT, PNG_SIGNATURE,
};

#[test]
fn honesty_table_matches_live_capabilities() {
    honesty_table_holds().unwrap();
    let table = expected_honesty_table();
    assert_eq!(table.len(), 5);
    let names: Vec<_> = table.iter().map(|r| r.name).collect();
    assert_eq!(names, vec!["svg", "null", "trace", "png", "skia"]);
}

#[test]
fn all_features_have_stable_names() {
    let names: Vec<_> = ALL_FEATURES.iter().map(|f| f.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "vectorText",
            "embeddedFonts",
            "gradients",
            "clipping",
            "images",
            "multiPage",
            "deterministic",
        ]
    );
}

#[test]
fn honesty_row_svg_is_consistent() {
    let row = expected_honesty_table()
        .into_iter()
        .find(|r| r.name == "svg")
        .unwrap();
    assert!(row.is_consistent());
    assert!(!row.note.is_empty());
}

#[test]
fn honesty_row_null_is_consistent() {
    let row = expected_honesty_table()
        .into_iter()
        .find(|r| r.name == "null")
        .unwrap();
    assert!(row.is_consistent());
    assert!(!row.note.is_empty());
}

#[test]
fn honesty_row_trace_is_consistent() {
    let row = expected_honesty_table()
        .into_iter()
        .find(|r| r.name == "trace")
        .unwrap();
    assert!(row.is_consistent());
    assert!(!row.note.is_empty());
}

#[test]
fn honesty_row_png_is_consistent() {
    let row = expected_honesty_table()
        .into_iter()
        .find(|r| r.name == "png")
        .unwrap();
    assert!(row.is_consistent());
    assert!(!row.note.is_empty());
}

#[test]
fn honesty_row_skia_is_consistent() {
    let row = expected_honesty_table()
        .into_iter()
        .find(|r| r.name == "skia")
        .unwrap();
    assert!(row.is_consistent());
    assert!(!row.note.is_empty());
}

fn assert_multi_page<B: RenderBackend<Error = RenderBackendError>>(mut backend: B) {
    let caps = backend.capabilities();
    let name = caps.name;
    backend.begin_page(PageSize::new(40.0, 30.0)).unwrap();
    backend.end_page().unwrap();
    let second = backend.begin_page(PageSize::new(40.0, 30.0));
    if caps.supports(BackendFeature::MultiPage) {
        second.unwrap();
        backend.end_page().unwrap();
        backend.finish().unwrap();
    } else {
        assert_eq!(
            second.unwrap_err(),
            RenderBackendError::MultiplePagesUnsupported { backend: name }
        );
    }
}

#[test]
fn svg_multi_page_matches_advertisement() {
    assert_multi_page(SvgBackend::new());
}

#[test]
fn null_multi_page_matches_advertisement() {
    assert_multi_page(NullBackend::new());
}

#[test]
fn trace_multi_page_matches_advertisement() {
    assert_multi_page(TraceBackend::new());
}

#[test]
fn png_multi_page_matches_advertisement() {
    assert_multi_page(PngBackend::new());
}

#[test]
fn skia_multi_page_matches_advertisement() {
    assert_multi_page(SkiaBackend::new());
}

#[test]
fn svg_text_observation_matches_vector_text_flag() {
    let scene = SceneSpec::empty("h-text", 400.0, 300.0)
        .push(SceneOp::new("textRun", 10.0, 20.0, 160.0, 16.0).with_text(HONESTY_TEXT));
    let mut backend = SvgBackend::new();
    replay_page(&mut backend, &scene.to_layer_tree()).unwrap();
    let svg = backend.finish().unwrap();
    let obs = observe_svg(&svg, HONESTY_TEXT);
    let caps = SvgBackend::new().capabilities();
    assert_eq!(obs.vector_text, caps.supports(BackendFeature::VectorText));
    assert_eq!(
        obs.embedded_fonts,
        caps.supports(BackendFeature::EmbeddedFonts)
    );
}

#[test]
fn svg_gradient_observation_matches_flag() {
    let scene = SceneSpec::empty("h-grad", 400.0, 300.0)
        .push(SceneOp::new("rectangle", 0.0, 0.0, 80.0, 40.0).with_gradient());
    let mut backend = SvgBackend::new();
    replay_page(&mut backend, &scene.to_layer_tree()).unwrap();
    let svg = backend.finish().unwrap();
    let obs = observe_svg(&svg, HONESTY_TEXT);
    assert_eq!(
        obs.gradients,
        SvgBackend::new()
            .capabilities()
            .supports(BackendFeature::Gradients)
    );
}

#[test]
fn svg_image_observation_matches_flag() {
    let scene = SceneSpec::empty("h-img", 400.0, 300.0)
        .push(SceneOp::new("image", 0.0, 0.0, 8.0, 8.0).with_image());
    let mut backend = SvgBackend::new();
    replay_page(&mut backend, &scene.to_layer_tree()).unwrap();
    let svg = backend.finish().unwrap();
    let obs = observe_svg(&svg, HONESTY_TEXT);
    assert_eq!(
        obs.images,
        SvgBackend::new()
            .capabilities()
            .supports(BackendFeature::Images)
    );
}

#[test]
fn png_signature_constant_is_real() {
    assert_eq!(
        PNG_SIGNATURE,
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
    );
    if PngBackend::raster_available() {
        let scene = SceneSpec::empty("png", 40.0, 30.0).push(SceneOp::new(
            "rectangle",
            0.0,
            0.0,
            10.0,
            10.0,
        ));
        let mut backend = PngBackend::new();
        replay_page(&mut backend, &scene.to_layer_tree()).unwrap();
        let bytes = backend.finish().unwrap();
        assert!(bytes.starts_with(PNG_SIGNATURE));
    }
}

#[test]
fn instrument_backends_have_no_visual_features() {
    for row in expected_honesty_table() {
        if row.name == "null" || row.name == "trace" {
            assert!(!row.vector_text);
            assert!(!row.gradients);
            assert!(!row.images);
            assert!(!row.clipping);
            assert!(!row.embedded_fonts);
            assert!(row.multi_page);
            assert!(row.deterministic);
        }
    }
}
