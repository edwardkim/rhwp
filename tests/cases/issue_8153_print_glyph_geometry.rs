//! 관계형·고정형: Print의 소수 원점과 Screen 격자를 합성 glyph로 대조한다.
//! 주석 갱신: 2026-10-30T14:56:45+09:00; 직전 파일 커밋: 2026-10-30T14:55:31+09:00 (80ca89e99f).
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! 관련 실사례 HWP 79쪽 Native 100.00% / fresh WASM 100.00%.
//! 관련 실사례 HWPX 79쪽 Native 100.00% / fresh WASM 100.00%.
//! HWP 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! HWPX 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 실사례와 합성 검사의 관계: mydocs/plans/issue_8153_print_glyph_precision_20261027.md.
#![cfg(feature = "native-skia")]

//! #8153: Print의 소수 원점과 Screen의 기존 격자를 실제 Native raster로 대조한다.
//! 합성 이동 계약이며 한컴 실물 문서 시각 정확도의 증거로 대신하지 않는다.

use rhwp::paint::{CacheHint, GroupKind, LayerNode, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::RasterPixels;
use rhwp::renderer::render_tree::{BoundingBox, RectangleNode, TextRunNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::{ShapeStyle, TextStyle};

#[test]
fn explicit_print_generation_survives_layer_cache_transitions() {
    let root = LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, Vec::new());
    let direct = PageLayerTree::with_profile(10.0, 10.0, root, RenderProfile::Print);
    assert!(direct.preserves_raster_text_geometry());
    assert!(direct.uses_raster_clip_coverage());
    let mut core = rhwp::DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "H").unwrap();
    for enabled in [false, true, false, true] {
        core.set_hangul2024_compat(enabled);
        let print = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        assert_eq!(print.preserves_raster_text_geometry(), enabled);
        assert_eq!(print.uses_raster_clip_coverage(), enabled);
        let exported: serde_json::Value = serde_json::from_str(&print.to_json()).unwrap();
        assert_eq!(exported["rasterTextGeometry"], enabled);
        assert_eq!(exported["rasterClipCoverage"], enabled);
        let screen = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Screen)
            .unwrap();
        assert!(!screen.preserves_raster_text_geometry());
        assert!(!screen.uses_raster_clip_coverage());
    }
}

#[test]
fn print_generation_uses_distinct_native_picture_cache_entries() {
    let mut core = rhwp::DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut trees = Vec::new();
    for enabled in [false, true] {
        core.set_hangul2024_compat(enabled);
        let mut tree = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        let bounds = BoundingBox::new(0.0, 0.0, 40.0, 40.0);
        tree.page_width = 40.0;
        tree.page_height = 40.0;
        tree.root = LayerNode::group(
            bounds,
            None,
            vec![LayerNode::leaf(
                bounds,
                None,
                vec![PaintOp::rectangle(
                    bounds,
                    RectangleNode::new(
                        0.0,
                        ShapeStyle {
                            fill_color: Some(0),
                            ..Default::default()
                        },
                        None,
                    ),
                )],
            )],
            CacheHint::StaticSubtree,
            GroupKind::Generic,
        );
        tree.text_sources = Default::default();
        assert_eq!(tree.preserves_raster_text_geometry(), enabled);
        assert_eq!(tree.uses_raster_clip_coverage(), enabled);
        trees.push(tree);
    }
    let renderer = SkiaLayerRenderer::new();
    for tree in &trees {
        let before = renderer.cache_stats().pictures;
        renderer
            .render_pixels_with_options(tree, RasterRenderOptions::default())
            .unwrap();
        let after = renderer.cache_stats().pictures;
        assert!(
            after.misses > before.misses,
            "서로 다른 인쇄 정책은 별도 기록"
        );
        renderer
            .render_pixels_with_options(tree, RasterRenderOptions::default())
            .unwrap();
        assert!(
            renderer.cache_stats().pictures.hits > after.hits,
            "같은 정책은 재사용"
        );
    }
}

fn glyph_at(
    renderer: &SkiaLayerRenderer,
    y: f64,
    fixed_glyph_advances: bool,
    profile: RenderProfile,
) -> RasterPixels {
    let run = TextRunNode {
        text: "H".into(),
        style: TextStyle {
            font_family: "Noto Sans KR".into(),
            font_size: 32.0,
            color: 0,
            fixed_glyph_advances,
            ..Default::default()
        },
        char_shape_id: None,
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 32.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    };
    let mut tree = PageLayerTree::new(
        64.0,
        64.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 64.0, 64.0),
            None,
            vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
        ),
    );
    tree.profile = profile;
    renderer
        .render_pixels_with_options(&tree, RasterRenderOptions::default())
        .expect("글리프를 투명 RGBA에 재생한다")
}

fn ink_center(image: &RasterPixels) -> (f64, f64) {
    let mut mass = 0.0;
    let mut x_sum = 0.0;
    let mut y_sum = 0.0;
    for (y, row) in image.bytes.chunks_exact(image.stride).enumerate() {
        for (x, pixel) in row[..image.width as usize * 4].chunks_exact(4).enumerate() {
            let weight = f64::from(pixel[3]);
            mass += weight;
            x_sum += x as f64 * weight;
            y_sum += y as f64 * weight;
        }
    }
    assert!(mass > 0.0, "공백/누락을 같은 위치의 잉크로 판정하지 않는다");
    (x_sum / mass, y_sum / mass)
}

fn check(renderer: &SkiaLayerRenderer, profile: RenderProfile) {
    let original = ink_center(&glyph_at(renderer, 4.1, false, profile));
    let quarter = ink_center(&glyph_at(renderer, 4.35, false, profile));
    let whole = ink_center(&glyph_at(renderer, 5.1, false, profile));
    let shift = quarter.1 - original.1;
    if profile == RenderProfile::Print {
        assert!(
            (shift - 0.25).abs() <= 0.08,
            "Print는 원본 1/4px 이동을 보존해야 한다: {shift}"
        );
    } else {
        assert!(shift.abs() <= 0.02, "Screen의 기존 격자 계약: {shift}");
    }
    assert!((whole.1 - original.1 - 1.0).abs() <= 0.02);
    assert!((quarter.0 - original.0).abs() <= 0.05);
}
fn renderer() -> SkiaLayerRenderer {
    SkiaLayerRenderer::new().with_font_paths(&[
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ttfs/opensource")
    ])
}
#[test]
fn print_ordinary_ttf_preserves_fractional_baseline() {
    check(&renderer(), RenderProfile::Print);
}
#[test]
fn screen_ordinary_ttf_keeps_existing_baseline_grid() {
    check(&renderer(), RenderProfile::Screen);
}
#[test]
fn print_screen_print_roundtrip_keeps_each_raster_contract() {
    let renderer = renderer();
    for profile in [
        RenderProfile::Print,
        RenderProfile::Screen,
        RenderProfile::Print,
    ] {
        check(&renderer, profile);
    }
}
