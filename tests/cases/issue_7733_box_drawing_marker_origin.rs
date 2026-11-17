//! 검사 유형: 고정형 — 양쪽 98% 이상인 화학 92쪽의 독립 Print 좌표를 ±2px로 보호한다.
//! 주석 갱신: 2026-11-18 05:26 KST.
//! 보정 전 Native 98.49337% / fresh WASM 98.48877%(96 DPI·Print·2px raw).
//! 독립 한컴 2020 Hancom PDF Print의 첫 U+257A 원점은 (120, 152.64), 본문 x는 144다.
//! 올바른 glyph 공급은 이 원점이나 원문·쪽수를 바꾸지 않아야 한다.
//! 프로그램 형식 선언만 공급하며 개인 글꼴 파일 설치에 의존하지 않는다.
//! 원점은 공유 render tree와 글리프 기준선으로 검사하고 공급 프로그램은 별도 관계 검사로 보호한다.

use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn text_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out.push(node);
    }
    for child in &node.children {
        text_runs(child, out);
    }
}

#[test]
fn right_bar_supplier_preserves_the_print_origin_and_following_text() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(
        FontEnvironment::from_json(
            r#"{"id":"right-bar-original-ttc","substitutions":{"바탕체":"BatangChe"},"trueTypeFaces":["BatangChe"],"preserveSourceMetrics":true}"#,
        )
        .unwrap(),
    ))
    .unwrap();
    assert_eq!(core.page_count(), 103);
    let page = core.build_page_render_tree(91).unwrap();
    let mut visible = Vec::new();
    text_runs(&page.root, &mut visible);
    let markers: Vec<_> = visible
        .iter()
        .copied()
        .filter(|n| matches!(&n.node_type, RenderNodeType::TextRun(run) if run.text == "╺"))
        .collect();
    assert_eq!(markers.len(), 3, "세 표식은 같은 쪽에 한 번씩 남는다");
    let baseline = |n: &RenderNode| match &n.node_type {
        RenderNodeType::TextRun(run) => n.bbox.y + run.baseline,
        _ => unreachable!(),
    };
    let first = markers
        .into_iter()
        .min_by(|a, b| baseline(a).total_cmp(&baseline(b)))
        .unwrap();
    assert!((first.bbox.x - 120.0).abs() <= 2.0);
    assert!((baseline(first) - 152.64).abs() <= 2.0);
    let mut body: Vec<_> = visible
        .iter()
        .copied()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextRun(run)
            if run.section_index == Some(4) && run.para_index == Some(224)
                && run.char_start.is_some())
        })
        .collect();
    body.sort_by(|a, b| a.bbox.x.total_cmp(&b.bbox.x));
    let text: String = body
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        text.trim(),
        "안전확인대상생활화학제품",
        "원본의 뒤 본문 소속·수량"
    );
    assert!((body[0].bbox.x - 144.0).abs() <= 2.0);
    assert!((baseline(body[0]) - 152.64).abs() <= 2.0);
}
