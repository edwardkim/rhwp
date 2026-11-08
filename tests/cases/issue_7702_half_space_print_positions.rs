//! 독립 Print의 15·17pt 줄바꿈 경계와 다음 구역 혼합 문자 줄의 원점을 보호한다.
//! 독립 Print: pdf/issue7702/half_space_ties-2020.pdf.
//! code2d500a의 Native/fresh WASM 전2쪽 모두100%와 확대 판독을 확인했다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    document_core::DocumentCore,
    renderer::render_tree::{RenderNode, RenderNodeType},
};

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}
fn lines<'a>(node: &'a RenderNode, si: usize, pi: usize, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && node.children.iter().any(|child|
        matches!(&child.node_type, RenderNodeType::TextRun(run) if run.section_index == Some(si) && run.para_index == Some(pi))) {
        out.push(node);
    }
    for child in &node.children {
        lines(child, si, pi, out);
    }
}
fn origin(line: &RenderNode, expected: (f64, f64)) {
    let node = line
        .children
        .iter()
        .find(|child| {
            matches!(&child.node_type,
        RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
        })
        .expect("첫 가시 글자");
    let RenderNodeType::TextRun(run) = &node.node_type else {
        unreachable!()
    };
    assert!(
        run.cell_context.is_none(),
        "본문 줄을 셀에 재소속하지 않는다"
    );
    assert!(
        (node.bbox.x - expected.0).abs() <= 2.0,
        "Print의 글줄 시작 x"
    );
    assert!(
        (node.bbox.y + run.baseline - expected.1).abs() <= 2.0,
        "Print의 글줄 기준선"
    );
}
fn core() -> DocumentCore {
    let core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue7702/half_space_ties.hwp"
    ))
    .expect("원본 입력");
    assert_eq!(
        core.page_count(),
        2,
        "독립 Print와 같은2쪽이며 음절·혼합문자 내용도 검사한다"
    );
    core
}
#[test]
fn half_space_wrapping_keeps_print_line_origins_and_all_syllables() {
    let core = core();
    let tree = core.build_page_render_tree(0).unwrap();
    // 원본 15pt는19+2, 17pt는17+4 음절로 이어진다. 독립 Print의 같은 경계다.
    for (pi, counts, baselines) in [
        (4, [19, 2], [234.4637654622396, 266.4288330078125]),
        (5, [17, 4], [300.63144938151044, 336.75197347005206]),
    ] {
        let mut found = Vec::new();
        lines(&tree.root, 0, pi, &mut found);
        assert_eq!(found.len(), 2, "한 문단의 두 줄을 같은 쪽에 보존한다");
        for ((line, count), baseline) in found.iter().zip(counts).zip(baselines) {
            let visible: String = text(line).chars().filter(|c| !c.is_whitespace()).collect();
            assert_eq!(
                visible,
                "가".repeat(count),
                "음절 누락·중복 없이 Print의 줄 소속 유지"
            );
            origin(line, (113.37855021158853, baseline));
        }
        assert!(
            found[0].bbox.y + found[0].bbox.height <= found[1].bbox.y,
            "이어지는 줄은 겹치지 않는다"
        );
    }
}
#[test]
fn next_section_mixed_script_lines_keep_print_origins_and_content() {
    let core = core();
    let tree = core.build_page_render_tree(1).unwrap();
    let mut found = Vec::new();
    lines(&tree.root, 1, 0, &mut found);
    assert_eq!(
        found.len(),
        2,
        "다음 구역의 혼합 문자 문단을 두 줄로 유지한다"
    );
    for (line, expected, baseline) in [
        (
            found[0],
            "가나다라 English 12345 가,나.다! 마바사아 test 67890 자차카타파하 가나다라 English ",
            143.52315266927084,
        ),
        (
            found[1],
            "12345 가,나.다! 마바사아 test 67890 자차카타파하",
            164.77991739908853,
        ),
    ] {
        assert_eq!(
            text(line),
            expected,
            "한글·영문·숫자·문장부호의 줄 소속과 공백 보존"
        );
        origin(line, (113.37855021158853, baseline));
    }
    assert!(found[0].bbox.y + found[0].bbox.height <= found[1].bbox.y);
}
