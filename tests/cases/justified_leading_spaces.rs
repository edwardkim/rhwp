//! 양쪽 정렬의 선행 공백은 자연 폭을 유지한다.
//! 원본 가맹사업 보고서 66쪽 Print의 선행 세 공백은 각각 8.32px이고,
//! 기호 뒤 낱말 공백에만 배분 여분이 있다. 절대 좌표 대신 같은 글꼴·크기의
//! 자연 공백 대조군과 실제 SVG 기호 원점을 비교한다.
//! 증거: 해당 원본 전142쪽 Native/fresh WASM TSV와 66쪽 확대 판독.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    renderer::{
        render_tree::{PageRenderTree, RenderNode, RenderNodeType},
        svg::SvgRenderer,
    },
};

fn content(node: &RenderNode) -> String {
    let mut text = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        text.push_str(&content(child));
    }
    text
}

fn selected_line(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::TextLine(_))
        && content(node).starts_with("   · 부당성 판단의 방법과 판단 요소")
    {
        return Some(node);
    }
    node.children.iter().find_map(selected_line)
}

fn circle_x(tree: &PageRenderTree) -> f64 {
    let mut renderer = SvgRenderer::new();
    renderer.render_tree(tree);
    let circles: Vec<_> = renderer.output().split("<circle ").skip(1).collect();
    assert_eq!(circles.len(), 1, "소유 글줄의 항목 기호만 비교한다");
    let tag = circles[0].split('>').next().unwrap();
    tag.split("cx=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn justified_source_indent_keeps_the_natural_leading_space_advance() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5966/1130000-202100008_franchise_review_report.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let mut actual = core.build_page_render_tree(65).unwrap();
    let line = selected_line(&actual.root)
        .expect("독립 Print의 해당 소유 글줄")
        .clone();
    let original = content(&line);
    let runs: Vec<_> = line
        .children
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextRun(_)))
        .collect();
    let dot = runs
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TextRun(run) if run.text.contains('·')))
        .unwrap();
    let mut control = (**dot).clone();
    control.bbox.x = runs[0].bbox.x;
    let RenderNodeType::TextRun(run) = &mut control.node_type else {
        unreachable!()
    };
    // 줄바꿈이나 저장 메타데이터를 바꾸지 않는 자연 공백 대조군이다.
    // 기대 원점은 현재 좌표나 구현의 간격 계산식을 다시 인용하지 않는다.
    run.text = "   ·".into();
    run.display_text = None;
    run.layout_positions = None;
    run.style.extra_word_spacing = 0.0;
    let mut natural = actual.clone();
    natural.root.children = vec![control];
    actual.root.children = vec![line];
    assert_eq!(
        original
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>(),
        "·부당성판단의방법과판단요소에대해서는가맹사업법의목적과",
        "독립 Print의 첫 줄 내용과 순서를 보존한다"
    );
    let difference = circle_x(&actual) - circle_x(&natural);
    assert!(
        difference.abs() < 1e-3,
        "양쪽 정렬이 들여쓰기 공백을 늘렸다: 자연 대조군과 기호 원점 차 {difference}"
    );
}
