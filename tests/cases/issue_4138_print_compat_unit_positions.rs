//! 고정형: 독립 Print의 CJK 단위·뒤 줄의 원점과 기준선을192DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-07T00:18:18+09:00.
//! 2026-11-07 raw(192DPI, PDFium, 2px): 120쪽 Native/fresh WASM 각100.00000%.
//! 전197쪽 최저94.83034% / 94.98570%; 양쪽21·44·129·130쪽은98% 미만으로 보정 중이다.
//! 부모fc038e4d54의 공통 경계 보정 source이며 줄 소속·원문 보존은 별도 관계 검사로 유지한다.
//! 근거: mydocs/pr/assets/issue4138/compatibility-unit-20261107/README.md.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn find<'a>(node: &'a RenderNode, prefix: &str) -> Option<&'a RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.para_index == Some(1575) && run.display_or_text().starts_with(prefix))
    {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, prefix))
}

#[test]
fn compatibility_unit_and_next_line_match_the_print_origins() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let page = core.build_page_render_tree(119).unwrap();
    // 독립 Hancom Print120쪽의 문자 원점·기준선(pt). 구현의 계산값과 독립이다.
    for (prefix, pdf_x, pdf_baseline) in [
        ("㎥", 269.853759765625, 512.0803833007812),
        ("/kg", 94.86858367919922, 531.2593994140625),
    ] {
        let node = find(&page.root, prefix).expect("같은 원문 문단의 단위 글자");
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        for (label, actual, expected) in [
            ("x", node.bbox.x * 2.0, pdf_x * 192.0 / 72.0),
            (
                "baseline",
                (node.bbox.y + run.baseline) * 2.0,
                pdf_baseline * 192.0 / 72.0,
            ),
        ] {
            assert!(
                (actual - expected).abs() <= 2.0,
                "{prefix} {label}: 독립 Print ±2px, actual={actual}, expected={expected}"
            );
        }
    }
}
