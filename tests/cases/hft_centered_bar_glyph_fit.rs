//! 관계형: 원 HFT의 전각 막대 칸과 좁은 표시 glyph의 중심·전진 소유를 분리한다.
//! 주석 갱신: 2026-11-15 03:35:51 KST. 신규 파일이며 직전 파일 변경 커밋은 없다.
//! MIT 합성 CFF의 실제 hmtx와 1em 원본 칸을 비교하는 font-only 계약이다.
//! 설치 글꼴·실물 페이지 좌표·문서별 PNG golden을 요구하지 않으며 합성 입력의 raw는 비해당이다.
//! 독립 한컴 Print T6/HFT376의 전진1000 및 중심 막대 근거:
//! mydocs/pr/assets/issue_9060/hft-en-dash-analysis.md. 실물 전쪽 비교·확대 판독은 별도다.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use super::symbol_font_program;
use rhwp::{
    paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile},
    renderer::{
        layer_renderer::RasterRenderOptions,
        raster_pixels::RasterPixels,
        render_tree::{BoundingBox, TextRunNode},
        skia::SkiaLayerRenderer,
        TextStyle,
    },
};

fn render(
    renderer: &SkiaLayerRenderer,
    source: Option<&str>,
    x: f64,
    tracking: f64,
) -> RasterPixels {
    let run = TextRunNode {
        text: "–".into(),
        style: TextStyle {
            font_family: "RHWP HFT Outline Fixture".into(),
            metric_font_family: source.map(str::to_owned),
            original_glyph_metric_family: source.map(|face| std::sync::Arc::new(face.to_owned())),
            source_font_is_hft: source.is_some(),
            preserve_glyph_ink: true,
            font_size: 40.0,
            color: 0,
            letter_spacing: tracking,
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
        baseline: 40.0,
        field_marker: Default::default(),
        layout_positions: Some(vec![0.0, 40.0 + tracking]),
        display_text: None,
    };
    let bounds = BoundingBox::new(0.0, 0.0, 96.0, 80.0);
    let tree = PageLayerTree::with_profile(
        96.0,
        80.0,
        LayerNode::leaf(
            bounds,
            None,
            vec![PaintOp::text_run(
                BoundingBox::new(x, 8.0, 40.0 + tracking, 40.0),
                run,
            )],
        ),
        RenderProfile::Print,
    );
    renderer
        .render_pixels_with_options(&tree, RasterRenderOptions::default())
        .unwrap()
}

#[test]
fn narrow_display_glyph_is_centered_in_the_source_hft_bar_cell_without_stretching() {
    let program = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPHftOutline.otf"),
        &[('–', 1)],
    );
    let face = ttf_parser::Face::parse(&program, 0).unwrap();
    let glyph = face.glyph_index('–').unwrap();
    let natural_em =
        f64::from(face.glyph_hor_advance(glyph).unwrap()) / f64::from(face.units_per_em());
    assert!(natural_em > 0.0 && natural_em < 1.0);
    let centered = 40.0 * (1.0 - natural_em) / 2.0;
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[program])
        .unwrap();
    for tracking in [-4.0, 0.0, 4.0] {
        let source = render(&renderer, Some("한양신명조 (HFT)"), 12.0, tracking);
        let reference = render(&renderer, None, 12.0 + centered, tracking);
        assert!(
            source.bytes.iter().any(|value| *value != 0),
            "누락 glyph를 같은 결과로 인정하지 않는다"
        );
        let mismatch = source
            .bytes
            .iter()
            .zip(&reference.bytes)
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            mismatch, 0,
            "자간은 뒤 원점만 바꾸며 source 막대 칸의 glyph는 중심과 모양을 보존한다"
        );
    }
}

#[test]
fn unverified_source_bar_keeps_the_supplied_program_origin() {
    let program = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPHftOutline.otf"),
        &[('–', 1)],
    );
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[program])
        .unwrap();
    let unknown = render(&renderer, Some("검증하지 않은 원본 HFT"), 12.0, 0.0);
    let supplied = render(&renderer, None, 12.0, 0.0);
    let mismatch = unknown
        .bytes
        .iter()
        .zip(&supplied.bytes)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        mismatch, 0,
        "확인하지 않은 원본 칸을 glyph의 새 중심 근거로 쓰지 않는다"
    );
}
