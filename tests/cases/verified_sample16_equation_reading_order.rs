//! 원본/한컴 변환본 sample16의 수식 소유·글줄 참여·읽기 순서를 보호한다.
//! 전체64쪽의 Native/fresh WASM 독립 Print 증거는 sample16 원본 계약 기록을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

struct Drawn<'a> {
    node: &'a RenderNode,
    equation: &'a EquationNode,
    line: &'a RenderNode,
    body: BoundingBox,
}

fn equations<'a>(
    node: &'a RenderNode,
    line: Option<&'a RenderNode>,
    body: Option<BoundingBox>,
    out: &mut Vec<Drawn<'a>>,
) {
    let line = if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        Some(node)
    } else {
        line
    };
    let body = if matches!(node.node_type, RenderNodeType::Body { .. }) {
        Some(node.bbox)
    } else {
        body
    };
    if let RenderNodeType::Equation(equation) = &node.node_type {
        out.push(Drawn {
            node,
            equation,
            line: line.expect("원본 TAC 수식의 글줄 흐름"),
            body: body.expect("원본 본문 수식 소속"),
        });
    }
    for child in &node.children {
        equations(child, line, body, out);
    }
}

fn assert_price_equations(sample: &str, compat2024: bool) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(sample);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(compat2024);
    assert_eq!(core.page_count(), 64, "독립 Print 전체 쪽수");
    let tree = core.build_page_render_tree(5).unwrap();
    let mut drawn = Vec::new();
    equations(&tree.root, None, None, &mut drawn);
    assert_eq!(drawn.len(), 2, "독립 Print6쪽 입찰가격 수식 두 개");
    let scripts: Vec<_> = drawn
        .iter()
        .map(|value| compact(&value.equation.script))
        .collect();
    assert_eq!(scripts[0].chars().count(), 49, "첫 산식 보존");
    assert_eq!(scripts[1].chars().count(), 81, "둘째 산식 보존");
    let mut owners = BTreeSet::new();
    for value in &drawn {
        let eq = value.equation;
        assert_eq!(eq.section_index, Some(0));
        let owner = (eq.para_index.unwrap(), eq.control_index.unwrap());
        assert!(owners.insert(owner), "원본 수식 제어의 중복 없음");
        let source = &core.document().sections[0].paragraphs[owner.0].controls[owner.1];
        let Control::Equation(source) = source else {
            panic!("원본 수식 제어 소속");
        };
        assert!(source.common.treat_as_char, "원본 두 산식의 TAC 속성");
        assert_eq!(compact(&eq.script), compact(&source.script));
        assert_eq!(eq.cell_index, None, "본문 수식을 표 셀로 이동하지 않는다");
        let RenderNodeType::TextLine(line) = &value.line.node_type else {
            unreachable!();
        };
        assert_eq!(line.section_index, eq.section_index);
        assert_eq!(
            line.para_index, eq.para_index,
            "수식이 원본 문단의 줄에 참여"
        );
        let actual = value.node.bbox;
        let bounds = value.line.bbox;
        assert!(actual.y >= bounds.y && actual.y + actual.height <= bounds.y + bounds.height);
        assert!(actual.x >= value.body.x);
        assert!(actual.x + actual.width <= value.body.x + value.body.width);
    }
    assert!(drawn[0].equation.para_index < drawn[1].equation.para_index);
    assert!(drawn[0].node.bbox.y + drawn[0].node.bbox.height <= drawn[1].node.bbox.y);

    let text = compact(&core.extract_page_text_native(5).unwrap());
    let mut previous = text
        .find("주)입찰가격평점산식")
        .expect("독립 Print의 산식 제목");
    for script in &scripts {
        assert_eq!(
            text.matches(script).count(),
            1,
            "수식 추출의 누락·중복 없음"
        );
        let position = text.find(script).unwrap();
        assert!(previous < position, "제목과 두 산식의 읽기 순서");
        previous = position;
    }
    assert!(
        previous
            < text
                .find("다만,입찰가격이")
                .expect("독립 Print의 후속 주의문")
    );
}

#[test]
fn hwp3_price_equations_preserve_source_flow_and_reading_order() {
    assert_price_equations("hwp3-sample16.hwp", false);
}

#[test]
fn hwpx_price_equations_preserve_source_flow_and_reading_order() {
    assert_price_equations("hwp3-sample16-hwp5.hwpx", true);
}
