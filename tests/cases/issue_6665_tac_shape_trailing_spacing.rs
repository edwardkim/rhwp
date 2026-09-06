//! #6665: 원 HWP5의 도형 전용 줄 뒤에서 저장 꼬리 줄간격을 누락하지 않는다.
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text_baselines(node: &RenderNode, needle: &str, out: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.replace(' ', "").contains(needle) {
            out.push(node.bbox.y + run.baseline);
        }
    }
    for child in &node.children {
        text_baselines(child, needle, out);
    }
}

fn check_variant(name: &str) {
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/3-09월_교육_통합_2024-{name}.hwp"));
    let core = DocumentCore::from_bytes(&std::fs::read(sample).expect("공개 HWP 회귀 문서"))
        .expect("문서 파싱");
    // 기존 pdf/3-09월_교육_통합_2024-{name}-2024.pdf의 Haansoft Dotum
    // 한글 span 기준선이다. 같은 줄의 작은 문항 번호나 수식 글꼴 원점과 섞지 않는다.
    // PDF MediaBox 595x841pt를 원본 쪽 높이에 맞춰 균일 배율로 비교한다.
    for (page, needle, pdf_baseline_pt) in [
        (4u32, "개의중량이", 493.91998291015625),
        (4, "일정한간격을두고", 669.719970703125),
        (5, "수직인평면으로자른단면", 633.239990234375),
        (7, "좌표공간의두점", 105.83999633789062),
    ] {
        let tree = core.build_page_render_tree(page - 1).expect("대조 쪽 렌더");
        let mut actual = Vec::new();
        text_baselines(&tree.root, needle, &mut actual);
        assert_eq!(
            actual.len(),
            1,
            "{name} {page}쪽의 {needle}을 유일하게 찾는다"
        );
        let expected = pdf_baseline_pt * tree.root.bbox.height / 841.0;
        assert!(
            (actual[0] - expected).abs() < 1.0,
            "{name} {page}쪽 {needle}: 실제 {:.3}px, 한글 {:.3}px. 도형 전용 줄 뒤 ls 누락",
            actual[0],
            expected,
        );
    }
}

#[test]
fn below_separator_variant_preserves_shape_line_spacing() {
    check_variant("구분선아래20");
}

#[test]
fn between_endnotes_variant_preserves_shape_line_spacing() {
    check_variant("미주사이20");
}
