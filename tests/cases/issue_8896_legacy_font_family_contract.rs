//! 관계형: 구형 분수의 실제 글꼴 요청과 독립 준비 도구의 family가 일치한다.
//! 주석 갱신: 2026-11-12T00:52:45+09:00; 실물 px 좌표는 고정하지 않는다.
//! 원본: samples/hwp3-sample16-hwp5-2022.hwp, 독립 한컴 2020 Print PDFium.
//! 2026-11-12 96 DPI·색상 보정 전 2px raw: 6쪽 Native 99.99717%, fresh WASM 100.00000%.
//! 전 64쪽 최저 Native 99.99717%, fresh WASM 100.00000%.
//! 근거: mydocs/pr/assets/pr8896/protection/ 및 font-supply/README.md.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

#[test]
fn legacy_fraction_requests_the_family_published_by_the_independent_font_tool() {
    let tool = include_str!("../../scripts/prepare_legacy_equation_font.py");
    let family = tool
        .lines()
        .find_map(|line| {
            line.strip_prefix("FAMILY = '")
                .and_then(|value| value.strip_suffix('\''))
        })
        .expect("독립 준비 도구의 실제 family 선언");
    let mut core =
        DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample16-hwp5-2022.hwp"))
            .unwrap();
    core.set_hangul2024_compat(false);
    let source = rhwp::serializer::serialize_document(core.document()).unwrap();
    let tree = core.build_page_render_tree(5).unwrap();
    fn collect<'a>(node: &'a RenderNode, fragments: &mut Vec<&'a str>) {
        if let RenderNodeType::Equation(equation) = &node.node_type {
            fragments.push(&equation.svg_content);
        }
        for child in &node.children {
            collect(child, fragments);
        }
    }
    let mut fragments = Vec::new();
    collect(&tree.root, &mut fragments);
    let xml = format!("<svg>{}</svg>", fragments.join("\n"));
    let svg = roxmltree::Document::parse(&xml).unwrap();
    let requests: Vec<_> = svg
        .descendants()
        .filter_map(|node| node.attribute("font-family"))
        .filter(|value| value.contains("RHWP Legacy Equation"))
        .collect();
    assert!(
        !requests.is_empty(),
        "독립 Print와 대조한 복합 구형 분수의 실제 요청"
    );
    for request in requests {
        let candidates: Vec<_> = request.split(',').map(str::trim).collect();
        assert_eq!(
            candidates.first(),
            Some(&family),
            "공급자가 만든 원 프로그램을 먼저 요청한다"
        );
        assert!(
            candidates.contains(&"RHWP Legacy Equation"),
            "이전 로컬 공급을 보존한다"
        );
        assert!(
            candidates.contains(&"HyhwpEQ"),
            "원 프로그램 결측 시 현대 대체를 보존한다"
        );
    }
    assert_eq!(
        rhwp::serializer::serialize_document(core.document()).unwrap(),
        source,
        "조회 가족의 연결이 원 수식·저장 데이터를 바꾸지 않는다"
    );
}
