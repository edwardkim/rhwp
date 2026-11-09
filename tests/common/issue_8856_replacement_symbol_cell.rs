//! 정식 CI 진입점은 skia_native_glyph_replay의 단일 #[test]이며 이 파일은 공용 검사 본문이다.
//! 관계형: 명시 대체 프로그램의 기호 잉크와 원본 전진 칸을 분리한다.
//! 주석 갱신: 2026-11-10T04:10:11+09:00. 실물 좌표를 고정하지 않는 합성 Native paint 검사다.
//! 실물 근거: issue1994 4쪽 독립 Print의 U+2219는 512/1024 전진 칸이며,
//! 표시 TTF의 같은 윤곽은 1024/1024 칸에 있다. 음수 자간은 다음 원점만 옮긴다.
//! 과거 Native 99.94357%, 현재 재현 99.80186%; 새 head 전쪽 수용을 뜻하지 않는다.
//! MIT 합성 CFF만 메모리에서 공급하며 개인·한컴 글꼴 설치에 의존하지 않는다.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use super::symbol_font_program;
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::{
    layer_renderer::LayerRasterRenderer,
    render_tree::{BoundingBox, TextRunNode},
    skia::SkiaLayerRenderer,
    TextStyle,
};

fn render(renderer: &SkiaLayerRenderer, spacing: f64, selected: bool, original: bool) -> Vec<u8> {
    let run = TextRunNode {
        text: "∙".into(),
        style: TextStyle {
            font_family: "RHWP HFT Outline Fixture".into(),
            metric_font_family: Some("Haansoft Batang".into()),
            font_size: 24.0,
            color: 0,
            letter_spacing: spacing,
            preserve_glyph_ink: selected,
            source_font_is_ttf: original,
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
        baseline: 24.0,
        field_marker: Default::default(),
        layout_positions: Some(vec![0.0, 12.0 + spacing]),
        display_text: None,
    };
    let tree = PageLayerTree::new(
        64.0,
        40.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 64.0, 40.0),
            None,
            vec![PaintOp::text_run(
                BoundingBox::new(24.0, 4.0, 24.0, 24.0),
                run,
            )],
        ),
    );
    renderer.render_png(&tree).unwrap()
}

pub(crate) fn assert_replacement_symbol_ink_keeps_the_intrinsic_cell_under_negative_tracking() {
    let program = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPHftOutline.otf"),
        &[('∙', 1)],
    );
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[program])
        .unwrap();
    let reference = render(&renderer, 0.0, false, false);
    for spacing in [-0.4, -1.0, -2.0] {
        assert!(
            render(&renderer, spacing, true, false) == reference,
            "음수 자간 {spacing}은 원점 뒤 전진만 바꾸며 같은 반각 기호 잉크는 보존한다"
        );
    }
    assert_ne!(
        render(&renderer, -1.0, true, true),
        reference,
        "실제 원본 TTF의 자연 프로그램은 표시 대체의 반각 정렬로 바꾸지 않는다"
    );
}
