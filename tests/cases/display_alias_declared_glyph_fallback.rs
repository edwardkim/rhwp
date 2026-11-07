//! 관계형: 표시용 별칭을 적용해도 문서의 대체 글꼴이 소유한 글리프 전진을 보존한다.
//! 주석 갱신: 2026-11-08T08:08:38.947493+09:00 (새 파일).
//! 측정: 2026-11-08, 96 DPI, 색상 경계 보정 전 2px raw, 정책연구 검토 요청 HWP 전 1쪽.
//! 보정 전 Native 99.73%, fresh WASM 99.75% (대상 쪽과 전쪽 최저값이 같음).
//! 독립 기준: pdf/issue8639/policy_research_review_request-print-{2020,2024}.pdf.
//! 측정·원인 근거: mydocs/pr/assets/planet6897_20261106/rebase-r45-analysis.md.
//! 공개 API로 원본·표시용 세션을 비교하며 시스템에 KoPub·한컴 글꼴이 설치됐다는 가정은 없다.
use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};
use serde_json::json;

fn leader(node: &RenderNode) -> Option<&TextRunNode> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains('․') {
            return Some(run);
        }
    }
    node.children.iter().find_map(leader)
}

#[test]
fn display_alias_preserves_the_documents_missing_glyph_supplier_advance() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue8639/policy_research_review_request.hwp"
    ))
    .unwrap();
    let before = core.build_page_render_tree(0).unwrap();
    let original = leader(&before.root).expect("원본의 한점 리더");
    core.set_font_environment(Some(
        FontEnvironment::from_json(
            &json!({
                "id": "display-alias",
                "preserveSourceMetrics": true,
                "substitutions": {"KoPub돋움체 Light": "KoPubDotum Light"},
                "trueTypeFaces": ["KoPubDotum Light"]
            })
            .to_string(),
        )
        .unwrap(),
    ))
    .unwrap();
    let after = core.build_page_render_tree(0).unwrap();
    let displayed = leader(&after.root).expect("별칭 적용 뒤 같은 한점 리더");
    assert_eq!(
        displayed.text, original.text,
        "원본 글자 주소와 내용을 보존한다"
    );
    assert!(
        displayed.style.font_family.contains("KoPubDotum Light"),
        "표시용 선택은 적용한다"
    );
    assert_eq!(
        EmbeddedTextMeasurer.compute_char_positions(&displayed.text, &displayed.style),
        EmbeddedTextMeasurer.compute_char_positions(&original.text, &original.style),
        "표시용 별칭은 원본 substFont의 결측 글리프 전진을 변경하지 않는다"
    );
}
