//! 관계형·고정형: Print의 소수 원점과 Screen 격자를 합성 glyph로 대조한다.
//! 주석 갱신: 2026-11-22T07:47:52+09:00; 직전 파일 커밋: 5cd54d764bb455ef92155fbc5ac00e9ebbb8b270 2026-11-15T08:41:26+09:00.
//! #7754 분할 원 glyph·표시 전진폭 검사는 MIT CFF의 합성 font-only·래스터 관계 계약이다.
//! 이 합성 입력의 독립 Print raw는 비해당이며 실제 문서 좌표를 고정하지 않는다.
//! 근거: mydocs/pr/assets/issues7754_7765/source35/metric-provenance/README.md.
//! 기존 Native CI의 이 검사 묶음에서 세 계약을 실행한다. workflow 호출은 추가하지 않는다.
//! #9075 추가 검사는 공개 TTF의 상대 이동 계약이며 사설 설치 글꼴에 의존하지 않는다.
//! 관련 86036 HWPX 전 14쪽: 2026-11-14, 96 DPI, 독립 한컴 Print PDFium, 보정 전 2px raw.
//! Native 최저 99.72%(12쪽), fresh WASM 최저 99.99%(2·12쪽). 절대 px 기대값을 추가하지 않는다.
//! 근거: mydocs/pr/assets/issue_9060/review-measurement.json 및 native-residual-diagnosis/README.md.
//! [#8986] Print 장치 clip 은 인쇄 세대와 무관하다 — 기준 raster(PDFium)가 2020 Print PDF 의 축 정렬
//! clip 도 바깥 화소로 넓힌다(36325899 2쪽 칸 위끝 중첩 표 테두리). 글자 기하만 세대별로 남는다.
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
        assert!(
            print.uses_raster_clip_coverage(),
            "Print 장치 clip 은 세대와 무관"
        );
        let exported: serde_json::Value = serde_json::from_str(&print.to_json()).unwrap();
        assert_eq!(exported["rasterTextGeometry"], enabled);
        assert_eq!(exported["rasterClipCoverage"], true);
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
        assert!(tree.uses_raster_clip_coverage());
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
    generation: Option<bool>,
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
    if let Some(enabled) = generation {
        let mut core = rhwp::DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.set_hangul2024_compat(enabled);
        tree.resources = core
            .build_page_layer_tree_with_profile(0, profile)
            .unwrap()
            .resources;
        assert_eq!(tree.preserves_raster_text_geometry(), enabled);
    }
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

fn check(renderer: &SkiaLayerRenderer, profile: RenderProfile, generation: Option<bool>) {
    let original = ink_center(&glyph_at(renderer, 4.1, false, profile, generation));
    let quarter = ink_center(&glyph_at(renderer, 4.35, false, profile, generation));
    let whole = ink_center(&glyph_at(renderer, 5.1, false, profile, generation));
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
    check(&renderer(), RenderProfile::Print, None);
}
#[test]
fn screen_ordinary_ttf_keeps_existing_baseline_grid() {
    check(&renderer(), RenderProfile::Screen, None);
}
#[test]
fn print_screen_print_roundtrip_keeps_each_raster_contract() {
    let renderer = renderer();
    for profile in [
        RenderProfile::Print,
        RenderProfile::Screen,
        RenderProfile::Print,
    ] {
        check(&renderer, profile, None);
    }
}

#[test]
fn legacy_print_ttf_preserves_fractional_baseline_across_generation_changes() {
    let renderer = renderer();
    for enabled in [false, true, false] {
        // 합성 입력의 동일한 1/4px 이동 관계다. 실문서 좌표를 핀으로 고정하지 않는다.
        check(&renderer, RenderProfile::Print, Some(enabled));
        check(&renderer, RenderProfile::Screen, None);
    }
}

mod split_hft_original_programs {
    use rhwp::renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        native_pua_fonts::NativePuaFonts,
        TextStyle,
    };
    use std::sync::Arc;

    const CLUSTER: &str = "ᄒᆞᆫ";
    const COMPATIBLE: char = '\u{f53a}';

    fn table(bytes: &[u8], tag: &[u8; 4]) -> usize {
        (0..usize::from(u16::from_be_bytes([bytes[4], bytes[5]])))
            .find_map(|i| {
                let p = 12 + i * 16;
                (&bytes[p..p + 4] == tag)
                    .then(|| u32::from_be_bytes(bytes[p + 8..p + 12].try_into().unwrap()) as usize)
            })
            .expect("licensed fixture table")
    }

    // The generated fixture has a format-4 cmap with one non-sentinel segment.
    // No installed/private font or outline bytes are added to the repository.
    fn font(character: char, advance: u16) -> Vec<u8> {
        let mut bytes = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf").to_vec();
        let cmap = table(&bytes, b"cmap");
        let sub =
            cmap + u32::from_be_bytes(bytes[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
        assert_eq!(
            u16::from_be_bytes(bytes[sub..sub + 2].try_into().unwrap()),
            4
        );
        for offset in [sub + 14, sub + 20] {
            bytes[offset..offset + 2].copy_from_slice(&(character as u16).to_be_bytes());
        }
        bytes[sub + 24..sub + 26]
            .copy_from_slice(&1u16.wrapping_sub(character as u16).to_be_bytes());
        let hmtx = table(&bytes, b"hmtx");
        bytes[hmtx + 4..hmtx + 6].copy_from_slice(&advance.to_be_bytes());
        bytes
    }

    fn original_style() -> TextStyle {
        TextStyle {
            font_family: "HY Sinmyeongjo".into(),
            font_size: 20.0,
            source_font_is_hft: true,
            hft_font_face: true,
            hft_hangul_face: true,
            original_glyph_metric_family: Some(Arc::new("한양신명조".into())),
            ..Default::default()
        }
    }

    fn supply(style: &mut TextStyle, programs: &[Vec<u8>]) {
        let mut fonts = NativePuaFonts::default();
        for program in programs {
            fonts
                .register_otf("display", false, false, program, 0)
                .expect("licensed supplied program");
        }
        style.native_otf_metrics = fonts.get_otf("display", false, false);
    }

    fn width(style: &TextStyle) -> f64 {
        *EmbeddedTextMeasurer
            .compute_char_positions(CLUSTER, style)
            .last()
            .expect("cluster end")
    }

    #[test]
    fn display_program_preserves_original_legacy_advance() {
        let mut style = original_style();
        let original = width(&style);
        supply(&mut style, &[font(COMPATIBLE, 900)]);
        assert_eq!(
            width(&style),
            original,
            "display CFF must not override the original font advance"
        );

        style.natural_program_metrics = true;
        let natural = width(&style);
        assert!(
            natural < original,
            "explicit natural metrics consume supplied hmtx"
        );
        style.source_font_is_hft = false;
        assert_eq!(width(&style), natural);
        style.natural_program_metrics = false;
        assert_eq!(
            width(&style),
            natural,
            "other source types retain the supplied program metrics"
        );
    }

    #[test]
    fn earlier_original_jamo_is_not_replaced_by_later_compatible_program() {
        let first = font('\u{1112}', 700);
        let later = font(COMPATIBLE, 900);
        let mut style = original_style();
        style.natural_program_metrics = true;
        supply(&mut style, std::slice::from_ref(&first));
        let before = width(&style);
        supply(&mut style, &[first, later]);
        assert_eq!(
            width(&style),
            before,
            "original cmap keeps ordered ownership"
        );
    }

    #[test]
    fn native_paint_reaches_visible_compatible_glyph_in_later_fragment() {
        use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
        use rhwp::renderer::{
            layer_renderer::RasterRenderOptions,
            render_tree::{BoundingBox, TextRunNode},
            skia::SkiaLayerRenderer,
        };

        let render = |text: &str, glyph: char| {
            let programs = [font('X', 1000), font(glyph, 1000)];
            let mut style = original_style();
            style.font_family = "RHWP HFT Outline Fixture".into();
            style.font_size = 32.0;
            supply(&mut style, &programs);
            let run = TextRunNode {
                text: text.into(),
                style,
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
                    vec![PaintOp::text_run(
                        BoundingBox::new(8.0, 8.0, 40.0, 40.0),
                        run,
                    )],
                ),
            );
            tree.profile = RenderProfile::Print;
            SkiaLayerRenderer::new()
                .try_with_font_data(&programs)
                .expect("explicit licensed font bytes")
                .render_pixels_with_options(&tree, RasterRenderOptions::default())
                .expect("actual Native text replay")
        };
        let actual = render(CLUSTER, COMPATIBLE);
        let control = render("가", '가');
        assert!(control
            .bytes
            .chunks_exact(4)
            .any(|p| p[3] > 0 && p[..3] != [255; 3]));
        assert!(
            actual == control,
            "same original outline must paint through split lookup"
        );
    }
}
