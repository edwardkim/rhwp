//! #8153: 재조판한 TAC 글상자 줄의 점유와 음수 간격을 보존한다.
//! 두 원본 289쪽의 Native/fresh DEV WASM raw 100%·미대응 0 및 독립 Print는
//! `mydocs/plans/issue_8153_reflowed_inline_shape_line_20261027.md`에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn collect<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    let selected = match &node.node_type {
        RenderNodeType::TextLine(line) => {
            line.section_index == Some(10) && line.para_index == Some(26)
        }
        RenderNodeType::Rectangle(shape) => {
            shape.section_index == Some(10)
                && shape.para_index == Some(26)
                && shape.control_index == Some(0)
        }
        RenderNodeType::Table(table) => {
            table.section_index == Some(10)
                && table.para_index == Some(27)
                && table.cell_context.is_none()
        }
        _ => false,
    };
    if selected {
        found.push(node);
    }
    for child in &node.children {
        collect(child, found);
    }
}

fn verify(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let source = &core.document().sections[10].paragraphs[26];
    assert!(
        source.text.is_empty(),
        "보이는 글자 없이 TAC가 점유하는 호스트"
    );
    let [Control::Shape(shape)] = source.controls.as_slice() else {
        panic!("원본의 단독 글상자");
    };
    assert!(shape.common().treat_as_char);
    let before = core.export_hwp_native().unwrap();

    let tree = core.build_page_render_tree(288).unwrap();
    let mut found = Vec::new();
    collect(&tree.root, &mut found);
    assert_eq!(
        found.len(),
        3,
        "같은 쪽의 호스트 줄·글상자·뒤 표 누락/중복 금지"
    );
    let line = found
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .unwrap()
        .bbox;
    let frame = found
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Rectangle(_)))
        .unwrap()
        .bbox;
    let table = found
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table(_)))
        .unwrap()
        .bbox;
    let hu = 96.0 / 7200.0;
    assert!(
        line.y <= frame.y + hu && line.y + line.height + hu >= frame.y + frame.height,
        "같은 호스트 줄이 실제 글상자의 위·아래를 포함한다"
    );
    // 원본 charPr 340: 1250HU, paraPr 712: Percent 90·뒤 간격 1000HU.
    // HWP5 표 62의 줄 높이/간격을 구분하며 구현의 resolver를 기대값에 호출하지 않는다.
    let signed_gap = 1250.0 * (90.0 - 100.0) / 100.0 * hu;
    let after = 1000.0 * hu;
    assert!(
        (table.y - (line.y + line.height + signed_gap + after)).abs() <= 96.0 / 600.0,
        "뒤 표는 같은 구성 줄의 음수 간격과 문단 뒤 간격을 한 번 소비한다"
    );
    assert!(
        (table.y - 191.02132).abs() <= 2.0,
        "독립 2024 Print 물리 289쪽·96dpi의 표 상단: {}",
        table.y
    );
    assert_eq!(
        before,
        core.export_hwp_native().unwrap(),
        "원본 저장 내용 보존"
    );
}

#[test]
fn hwp_reflowed_shape_row_owns_frame_and_signed_spacing() {
    verify("hwp");
}

#[test]
fn hwpx_reflowed_shape_row_owns_frame_and_signed_spacing() {
    verify("hwpx");
}
