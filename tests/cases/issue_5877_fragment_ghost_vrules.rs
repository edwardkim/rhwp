//! [#5877] 이어받은 표의 괘선은 실제 셀 경계와 일치해야 한다.
//!
//! 정본은 원본 31쪽 HWP와 `pdf/fragment_ghost_vrules-2020.pdf`다.
//! 균등 보간 격자가 실제 열을 대신한 결함을 셀·괘선의 관계로 검사한다.
//! 특정 x/y 좌표나 과거의 잘못된 열 폭을 기대값으로 고정하지 않는다.
//! 가스 표의 마지막 병합 블록은 두 라벨을 10쪽에 남기고, 반복 제목행 뒤의
//! 물리 빈 밴드와 11쪽 의견 행의 독립 최소를 함께 보존해야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::{document_core::DocumentCore, model::control::Control};
use std::path::Path;

const SAMPLE: &str = "samples/issue5877/fragment_ghost_vrules.hwp";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 열기")
}

fn owned_table(node: &RenderNode, para: usize, control: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t)
        if t.para_index == Some(para) && t.control_index == Some(control))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| owned_table(child, para, control))
}

fn cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TableCell(c) if c.row == row && c.col == col)
        })
        .unwrap_or_else(|| panic!("셀 row={row}, col={col} 없음"))
}

fn text(node: &RenderNode) -> String {
    let mut value = if let RenderNodeType::TextRun(run) = &node.node_type {
        run.text.clone()
    } else {
        String::new()
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

#[test]
fn issue_5877_continuation_fragment_uses_its_own_column_grid() {
    let core = core();
    let page = core.build_page_render_tree(14).expect("15쪽 렌더 트리");
    let table = owned_table(&page.root, 20, 0).expect("이어받은 본문 표");
    let cells: Vec<_> = table
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert!(!cells.is_empty(), "실제 셀이 있어야 검증이 유효하다");
    let epsilon = table.bbox.width * 1e-8;
    let top = cells
        .iter()
        .map(|node| node.bbox.y)
        .fold(f64::INFINITY, f64::min);
    let opening: Vec<_> = cells
        .iter()
        .filter(|node| (node.bbox.y - top).abs() <= epsilon)
        .collect();
    let bottom = opening
        .iter()
        .map(|node| node.bbox.y + node.bbox.height)
        .fold(f64::INFINITY, f64::min);
    let mut boundaries = Vec::new();
    for node in opening {
        boundaries.push(node.bbox.x);
        boundaries.push(node.bbox.x + node.bbox.width);
    }
    boundaries.sort_by(f64::total_cmp);
    boundaries.dedup_by(|a, b| (*a - *b).abs() <= epsilon);
    assert!(
        boundaries.len() > 2,
        "전폭 제목 셀을 본문 격자로 검사하지 않는다"
    );

    let svg = core.render_page_svg_native(14).expect("15쪽 SVG");
    let mut drawn = Vec::new();
    for segment in svg.split("<line ").skip(1) {
        let head = &segment[..segment.find('>').unwrap_or(segment.len())];
        let (Some(x1), Some(x2), Some(y1), Some(y2)) = (
            attr(head, "x1=\""),
            attr(head, "x2=\""),
            attr(head, "y1=\""),
            attr(head, "y2=\""),
        ) else {
            continue;
        };
        if (x1 - x2).abs() <= epsilon
            && y1.min(y2) < bottom
            && y1.max(y2) > top
            && x1 >= table.bbox.x - epsilon
            && x1 <= table.bbox.x + table.bbox.width + epsilon
        {
            drawn.push(x1);
        }
    }
    assert!(
        !drawn.is_empty(),
        "표의 첫 물리 밴드에 세로 괘선이 있어야 한다"
    );
    for x in &drawn {
        assert!(
            boundaries.iter().any(|edge| (x - edge).abs() <= epsilon),
            "실제 셀 경계 밖 유령 괘선: x={x}, 셀 경계={boundaries:?}"
        );
    }
    for edge in boundaries {
        assert!(
            drawn.iter().any(|x| (x - edge).abs() <= epsilon),
            "실제 셀 경계의 괘선 누락: 경계={edge}, 그린 괘선={drawn:?}"
        );
    }
}

#[test]
fn issue_5877_gas_rowspan_keeps_label_owner_and_next_row_minimum() {
    let core = core();
    assert_eq!(core.page_count(), 31, "정본 전체 쪽 소유를 보존해야 한다");
    let first_page = core.build_page_render_tree(9).expect("10쪽");
    let next_page = core.build_page_render_tree(10).expect("11쪽");
    let first = owned_table(&first_page.root, 19, 0).expect("가스 표 앞 조각");
    let next = owned_table(&next_page.root, 19, 0).expect("가스 표 뒤 조각");
    assert_eq!(
        text(cell(first, 23, 1)),
        "연소기(공통)",
        "두 라벨은 같은 원본 쪽의 소유다"
    );
    assert!(
        text(cell(next, 23, 1)).is_empty(),
        "소비한 라벨을 다음 쪽에 중복하지 않는다"
    );
    let source = &core.document().sections[0];
    let Control::Table(table) = &source.paragraphs[19].controls[0] else {
        panic!("가스 표 원본")
    };
    let opening_source = table
        .cells
        .iter()
        .find(|c| c.row == 23 && c.col == 2)
        .unwrap();
    let next_source = table
        .cells
        .iter()
        .find(|c| c.row == 24 && c.col == 2)
        .unwrap();
    let scale = first_page.root.bbox.width / f64::from(source.section_def.page_def.width);
    assert!(
        cell(first, 23, 2).bbox.height > f64::from(opening_source.height) * scale,
        "미완료 병합 블록의 쪽 상자는 마지막 내용 행 최소 밖의 빈 밴드도 소유한다"
    );
    let epsilon = next.bbox.height * 1e-8;
    assert!(
        cell(next, 24, 2).bbox.height + epsilon >= f64::from(next_source.height) * scale,
        "앞 조각의 빈 밴드를 이유로 다음 독립 의견 행의 최소를 축소하지 않는다"
    );
    let compact = |value: String| {
        value
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    let expected = opening_source
        .paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect::<String>();
    assert_eq!(
        compact(text(cell(first, 23, 2))),
        compact(expected),
        "마지막 행 본문 누락·중복 금지"
    );
}
fn attr(head: &str, key: &str) -> Option<f64> {
    let rest = head.split_once(key)?.1;
    rest[..rest.find('"')?].parse().ok()
}
