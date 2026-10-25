//! #8153: 동일한 저장 TAC 줄의 pen·본체와 비흐름 장식 소유를 보존한다.
//! 독립 2024 Print 116쪽의 내부 표 좌표는 ±2px로 확인한다.
//! 동일 원본 HU의 형식 간 원점 동등성도 검사해 여백 중복을 보호한다.
//! 근거: mydocs/plans/issue_8153_decorated_tac_margin_20261025.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn visit<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    found.push(node);
    for child in &node.children {
        visit(child, found);
    }
}

fn text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    visit(node, &mut nodes);
    nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn origins(extension: &str) -> Vec<BoundingBox> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383);
    let tree = core.build_page_render_tree(115).expect("원본 116쪽");
    let mut nodes = Vec::new();
    visit(&tree.root, &mut nodes);

    [884, 885]
        .into_iter()
        .map(|paragraph| {
            let source = &core.document().sections[2].paragraphs[paragraph];
            let Control::Table(table) = &source.controls[1] else {
                panic!("장식 뒤 TAC 표의 원래 소유");
            };
            assert!(table.common.treat_as_char);
            assert_eq!(source.line_segs.len(), 1);
            assert_eq!(
                i64::from(source.line_segs[0].line_height),
                i64::from(table.common.height)
                    + i64::from(table.outer_margin_top)
                    + i64::from(table.outer_margin_bottom),
                "독립 TAC 줄은 본체와 바깥 여백을 한 번 소유한다"
            );
            let found: Vec<_> = nodes
                .iter()
                .filter(|node| {
                    matches!(&node.node_type, RenderNodeType::Table(meta)
                        if meta.section_index == Some(2)
                            && meta.para_index == Some(paragraph)
                            && meta.control_index == Some(1)
                            && meta.cell_context.is_none())
                })
                .collect();
            assert_eq!(found.len(), 1, "TAC 표의 쪽·문단 소유와 수량 보존");
            let node = found[0];
            let content = text(node);
            let words: String = content.chars().filter(|ch| !ch.is_whitespace()).collect();
            let title = if paragraph == 884 {
                "관인생략"
            } else {
                "서명생략"
            };
            assert_eq!(
                words.matches(title).count(),
                1,
                "문단 {paragraph}: {content}"
            );
            assert_eq!(
                content.matches('3').count(),
                1,
                "문단 {paragraph}: {content}"
            );
            assert_eq!(
                content.matches('㎝').count(),
                2,
                "문단 {paragraph}: {content}"
            );
            if paragraph == 884 {
                let mut children = Vec::new();
                visit(node, &mut children);
                let nested: Vec<_> = children
                    .into_iter()
                    .filter(|child| {
                        matches!(&child.node_type, RenderNodeType::Table(meta)
                            if meta.cell_context.is_some())
                    })
                    .collect();
                assert_eq!(nested.len(), 1, "관인 생략의 내부 표 소유");
                let inner = nested[0].bbox;
                for (actual, point) in [
                    (inner.y, 180.9420166),
                    (inner.y + inner.height, 211.6589966),
                ] {
                    assert!(
                        (actual - point * 96.0 / 72.0).abs() <= 2.0,
                        "독립 Print의 내부 표 경계: {actual}, 기준 {point}pt, ±2px"
                    );
                }
                assert!(inner.y >= node.bbox.y);
                assert!(inner.y + inner.height <= node.bbox.y + node.bbox.height);
            }
            node.bbox
        })
        .collect()
}

#[test]
fn same_saved_tac_rows_keep_their_pen_across_hwp_and_hwpx() {
    let hwp = origins("hwp");
    let hwpx = origins("hwpx");
    assert!(hwp[0].y + hwp[0].height < hwp[1].y);
    assert!(hwpx[0].y + hwpx[0].height < hwpx[1].y);
    for (binary, xml) in hwp.iter().zip(&hwpx) {
        // 같은 원본 HU를 가진 두 형식의 상호 배치는 1HU 반올림 범위에서 같다.
        let unit = 96.0 / 7200.0;
        assert!(
            (binary.y - xml.y).abs() <= unit,
            "같은 저장 줄의 본체 원점: HWP {}, HWPX {}",
            binary.y,
            xml.y
        );
        assert!((binary.x - xml.x).abs() <= unit);
        assert!((binary.width - xml.width).abs() <= unit);
        assert!((binary.height - xml.height).abs() <= unit);
    }
}
