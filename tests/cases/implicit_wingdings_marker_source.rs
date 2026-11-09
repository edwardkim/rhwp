//! 관계형: 원 PUA 기호의 소유·전진과 표시 대체 설정 사이의 독립성을 검사한다.
//! 주석 작성: 2026-11-07T13:58:21+09:00. 새 파일이므로 직전 변경 커밋은 없다.
//! 측정 2026-11-07, 제품 ef77d740e1, Print·96 DPI·보정 전 2px raw.
//! samples/basic/sungeo.hwp ↔ pdf/basic/sungeo-hwp-2020.pdf, 양쪽 모두 전체 94쪽 일치.
//! 전쪽 최저 Native 99.98% / fresh WASM 99.99%, 검사 5쪽 Native 100.00% / WASM 99.99%.
//! 같은 5쪽의 review·overlay·확대 판독과 SHA·전쪽 TSV는 다음 경로에 있다.
//! mydocs/pr/assets/planet6897_20261106/pr8611/r33-source-symbol/README.md.
//! 독립 Print의 Wingdings-Regular glyph 3은 UPEM 2048·advance 937이다.
//! 설치 글꼴·절대 배치 px 핀 없이 원본 주소·메트릭 소유와 Unicode 대조군을 보호한다.

#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn load() -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/sungeo.hwp"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn marker(core: &DocumentCore) -> (String, f64, f64) {
    fn find(node: &RenderNode) -> Option<(String, f64, f64)> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.section_index == Some(1)
                && run.para_index == Some(63)
                && run.char_start.is_none()
            {
                return Some((run.text.clone(), node.bbox.width, run.style.font_size));
            }
        }
        node.children.iter().find_map(find)
    }
    find(&core.build_page_render_tree(4).unwrap().root).unwrap()
}
fn marker_width_source(core: &DocumentCore) -> String {
    let value: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(4, r#"{"maxCharacters":8}"#)
            .unwrap(),
    )
    .unwrap();
    value["records"][0]["layoutMetric"]["widthSource"]
        .as_str()
        .unwrap()
        .to_string()
}
#[test]
fn implicit_wingdings_keeps_source_identity_and_a_separate_advance() {
    let core = load();
    let (text, width, size) = marker(&core);
    assert_eq!(text, "\u{f09f}", "일대일 표시 투영 전의 원 기호 주소");
    assert!(
        width > 0.0 && width < size,
        "원 기호의 반각 칸을 대체 face의 전각과 구별"
    );
    assert_eq!(marker_width_source(&core), "implicitSymbolProgram");
}
#[test]
fn declared_display_fallback_does_not_replace_implicit_program_metrics() {
    let core = load();
    let mut other = load();
    other.set_font_environment(Some(FontEnvironment::from_json(r#"{"id":"marker-source-display-control","substitutions":{"한양신명조":"Gulim"},"preserveSourceMetrics":true}"#).unwrap())).unwrap();
    assert_eq!(
        marker(&core),
        marker(&other),
        "설치 여부를 가정하지 않는 원 프로그램 소유"
    );
    assert_eq!(marker_width_source(&other), "implicitSymbolProgram");
    assert_eq!(
        core.document().sections[1].paragraphs[63].text,
        other.document().sections[1].paragraphs[63].text
    );
}
#[test]
fn normal_unicode_bullet_keeps_its_own_font_contract() {
    let mut core = load();
    let mut document = core.document().clone();
    document.doc_info.bullets[0].bullet_char = '•';
    core.set_document(document);
    assert_eq!(marker(&core).0, "•");
    assert_ne!(marker_width_source(&core), "implicitSymbolProgram");
}
