//! 관계형: 원본 HFT 기호의 전진과 표시용 별칭 적용 전후의 내용 소유를 검사한다.
//! 주석 갱신: 2026-11-07T02:11:45+09:00.
//! 2026-11-07 raw(192DPI, PDFium, 2px): 21쪽 Native99.97728% / fresh WASM99.97745%.
//! 전197쪽 최저94.83034% /94.98570%; 44·129·130쪽은98% 미만으로 보정 중이다.
//! 독립 Type3의1000/1000em은 원본 글리프 비율이며 페이지 절대 px 핀이 아니다.
//! 기호의 원본 비율 근거: mydocs/pr/assets/issue4138/body-relation-symbol-20261107/README.md.
//! 현재 출력·좌표 근거: mydocs/pr/assets/issue4138/interword-space-20261107/README.md.

use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn symbol(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.text == "≤" && run.para_index == Some(279))
    {
        return Some(node);
    }
    node.children.iter().find_map(symbol)
}

fn check(alias: bool) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let source_fonts: Vec<_> = core
        .document()
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .map(|f| f.name.clone())
        .collect();
    if alias {
        core.set_font_environment(Some(FontEnvironment::from_json(
            r#"{"id":"original-hft-family-alias","substitutions":{"한양신명조":"HY Sinmyeongjo"},"preserveSourceMetrics":true}"#,
        ).unwrap())).unwrap();
    }
    let tree = core.build_page_render_tree(20).unwrap();
    let node = symbol(&tree.root).expect("독립 Print21쪽의 본문 관계 기호 소속");
    let RenderNodeType::TextRun(run) = &node.node_type else {
        unreachable!()
    };
    assert!(
        !run.style.source_font_is_ttf,
        "원본 HFT 선언을 대체 TTF로 승격하지 않는다"
    );
    // 독립 PDF의 T4 U+2264 /Widths·CharProc d1=1000, FontMatrix=.001이다.
    assert!(
        (node.bbox.width / (run.style.font_size * run.style.ratio) - 1.0).abs() < 1e-6,
        "표시 별칭이 원본의1em 점유를 반각 추정으로 덮지 않는다: {:?}",
        node.bbox
    );
    assert!(
        core.extract_page_text_native(20).unwrap().contains("f≤1"),
        "기호와 앞뒤 본문의 원문·소속 보존"
    );
    let after_fonts: Vec<_> = core
        .document()
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .map(|f| f.name.clone())
        .collect();
    assert_eq!(
        source_fonts, after_fonts,
        "표시용 환경은 원본 글꼴 선언을 변경하지 않는다"
    );
}

#[test]
fn original_hft_relation_symbol_owns_a_full_em() {
    check(false);
}

#[test]
fn display_family_alias_keeps_original_relation_symbol_advance() {
    check(true);
}
