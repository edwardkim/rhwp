//! 고정형(px): 완료된 서식 1쪽의 첫 줄·어절·이어지는 줄 원점을 독립 PDF ±2px로 보호한다.
//! 주석 갱신: 2026-11-10T02:25:03+09:00 (Asia/Seoul).
//! 최소 글꼴 전1쪽 Native 99.99813%, fresh WASM 99.99259%; 96 DPI·PDFium·2px raw 실루엣.
//! 같은 쪽의 review/overlay를 직접 판독한 뒤 현재 지침의 98% 조건으로 추가했다.
//! 독립 기대값: pdf/issue8639/policy_research_review_request-trackchange0-print-2024.pdf,
//! SHA ad8de57af837d16c45b0bd01f3c5e26e0fdf50f223d3d433e922e930a9e60eeb.
//! 한컴 PDF의 KoPubDotumLight 문자 원점(pt)에 96/72를 곱한 값이며 후보 좌표를 복사하지 않았다.
//! 근거: mydocs/pr/assets/issue8856/stage4-policy-fields/validation.json.
//! 원문 소유·누락·중복과 미확인 기호 반례는 issue_8856_policy_field_line_ownership에서 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

const SOURCE: &[u8] =
    include_bytes!("../../samples/issue8639/policy_research_review_request-trackchange0.hwpx");

fn origins(node: &RenderNode, phrase: &str, out: &mut Vec<[f64; 2]>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let Some(byte) = run.text.find(phrase) {
            let scalar = run.text[..byte].chars().count();
            let positions = run.layout_positions.clone().unwrap_or_else(|| {
                EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style)
            });
            out.push([node.bbox.x + positions[scalar], node.bbox.y + run.baseline]);
        }
    }
    for child in &node.children {
        origins(child, phrase, out);
    }
}

#[test]
fn completed_policy_field_origins_stay_on_the_independent_print_rows() {
    let mut core = DocumentCore::from_bytes(SOURCE).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1, "독립 Print의 전체 쪽수");
    let tree = core.build_page_render_tree(0).unwrap();
    // 첫 줄 시작, 앞줄로 복구한 어절, 다음 줄 시작을 각각 보호한다.
    for (phrase, point) in [
        ("근무성적평정", [163.95114135742188, 655.0841064453125]),
        ("포상금", [502.76751708984375, 655.0841064453125]),
        ("지급(본청", [159.15374755859375, 668.1498413085938]),
    ] {
        let mut actual = Vec::new();
        origins(&tree.root, phrase, &mut actual);
        assert_eq!(actual.len(), 1, "중복되지 않은 원문 어절: {phrase}");
        for axis in 0..2 {
            let expected = point[axis] * 96.0 / 72.0;
            assert!(
                (actual[0][axis] - expected).abs() <= 2.0,
                "{phrase} axis={axis}: 독립 Print {expected:.5}px, 후보 {:.5}px",
                actual[0][axis]
            );
        }
    }
}
