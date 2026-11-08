//! #5585/#7840: 겹치는 저장 줄의 이어받기, 마지막 괘선과 후속 본문 보존.
//! 실제 저장소 원본과 독립 한컴 Print(7쪽)를 사용한다. 외부 코퍼스 부재로 건너뛰지 않는다.
//! 전쪽 Native/fresh WASM·대표 PNG·독립 좌표 근거:
//! `mydocs/pr/assets/issue_7840/completed_wrapper_frame/README.md`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("커밋된 원본")).expect("문서 로드")
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

fn owned_table(root: &RenderNode, pi: usize) -> Option<&RenderNode> {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    let owned: Vec<_> = nodes
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(pi) && table.cell_context.is_none())
        })
        .collect();
    assert!(owned.len() <= 1, "원본 표 {pi}가 한 쪽에 중복 배치됨");
    owned.first().copied()
}

fn source_table(core: &DocumentCore, pi: usize) -> &Table {
    core.document().sections[0].paragraphs[pi]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table.as_ref())
            } else {
                None
            }
        })
        .expect("원본 소유 표")
}

#[test]
fn overlapping_line_steps_do_not_split_the_cell_into_tiny_pages() {
    let core = core();
    assert_eq!(core.page_count(), 7, "독립 Print의 전체 쪽수");
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page as u32).expect("쪽 배치");
        assert_eq!(
            owned_table(&tree.root, 5).is_some(),
            page < 5,
            "{}쪽: 감싼 표는 앞 다섯 쪽에서 이어지고 참고 본문 앞에서 끝난다",
            page + 1
        );
        let body = compact(&text(&tree.root));
        if page == 5 {
            assert!(body.contains("1.소주시장현황"), "이어받기 뒤 첫 참고 본문");
            assert!(
                owned_table(&tree.root, 24).is_some(),
                "국내시장 수량 표 보존"
            );
        }
        if page == 6 {
            assert!(
                owned_table(&tree.root, 35).is_some(),
                "저도주 판매 현황 보존"
            );
            assert!(
                owned_table(&tree.root, 39).is_some(),
                "마지막 제조사별 표 보존"
            );
        }
    }
}

#[test]
fn fragment_pages_fill_the_body_instead_of_breaking_early() {
    let core = core();
    let tree = core.build_page_render_tree(4).expect("마지막 표 조각");
    let frame = owned_table(&tree.root, 5).expect("5쪽의 원본 wrapper");
    let source = &source_table(&core, 5).cells[0];
    let (last_pi, last_para) = source
        .paragraphs
        .iter()
        .enumerate()
        .rfind(|(_, para)| !para.text.trim().is_empty())
        .expect("마지막 원문");
    assert!(
        compact(&last_para.text).ends_with("제품현황"),
        "독립 Print의 끝 내용"
    );
    let cell = frame.children.iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 0 && cell.col == 0)
    }).expect("원본 칸");
    let lines: Vec<_> = cell.children.iter().filter(|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(last_pi))
    }).collect();
    let actual: String = lines.iter().map(|line| text(line)).collect();
    assert_eq!(
        compact(&actual),
        compact(&last_para.text),
        "마지막 원문 순서·누락·중복"
    );
    let closing = frame
        .children
        .iter()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::Line(_))
                && node.bbox.width > frame.bbox.width / 2.0
        })
        .map(|node| node.bbox.y + node.bbox.height / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(closing.is_finite(), "실제 그리는 닫는 괘선");
    for line in lines {
        for node in &line.children {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                if !run.display_or_text().trim().is_empty() {
                    assert!(
                        node.bbox.y + run.baseline < closing,
                        "마지막 첨부 문구는 실제 괘선 안에 표시되어야 한다"
                    );
                }
            }
        }
    }

    // 같은 핵심 후속 본문 중 공동 raw 최솟값이 가장 높은 6쪽을 선정한다.
    // 독립 PDF의 수량 표 괘선 중심: 72pt에서 96dpi CSS px로 변환한 값이다.
    let tree = core.build_page_render_tree(5).unwrap();
    let table = owned_table(&tree.root, 24).expect("이어받기 뒤 국내시장 표");
    for (actual, expected) in [
        (table.bbox.x, 100.745331),
        (table.bbox.y, 374.630697),
        (table.bbox.x + table.bbox.width, 715.774658),
        (table.bbox.y + table.bbox.height, 687.728027),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 괘선 {expected:.6}px, 실제 {actual:.6}px"
        );
    }
    let original = source_table(&core, 24);
    let cells: Vec<_> = table
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(cells.len(), original.cells.len(), "후속 표의 칸 누락·중복");
    for expected in &original.cells {
        let owned: Vec<_> = cells
            .iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TableCell(cell)
                if cell.row == expected.row && cell.col == expected.col)
            })
            .collect();
        assert_eq!(owned.len(), 1, "원본 행·열 소유는 유일");
        let expected_text: String = expected
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        assert_eq!(
            compact(&text(owned[0])),
            compact(&expected_text),
            "후속 표 원문 보존"
        );
    }
}

/// 독립 Print의 3·4쪽 이어받기 하단과 5쪽 마지막 하단을 보호한다.
/// 좌표는 PDF 괘선 중심을 72pt에서 96dpi CSS px로 환산한 독립 관측값이다.
/// 이 쪽들은 전쪽 양쪽 90%, 해당 쪽 양쪽 98%와 확대 판독을 모두 통과했다.
#[test]
fn stored_frame_bottoms_match_independent_print_at_nonterminal_cuts() {
    let core = core();
    for (page, pdf_bottom) in [
        (2, 1016.968017578125),
        (3, 1010.254638671875),
        (4, 832.3693033854166),
    ] {
        let tree = core
            .build_page_render_tree(page)
            .expect("독립 Print 대응 쪽");
        let frame = owned_table(&tree.root, 5).expect("원본 감싼 표의 유일한 조각");
        let closing = frame
            .children
            .iter()
            .filter(|node| {
                matches!(node.node_type, RenderNodeType::Line(_))
                    && node.bbox.width > frame.bbox.width / 2.0
            })
            .map(|node| node.bbox.y + node.bbox.height / 2.0)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(closing.is_finite(), "실제 그리는 하단 괘선");
        assert!(
            (closing - pdf_bottom).abs() <= 2.0,
            "{}쪽: 독립 Print {pdf_bottom:.6}px, 실제 {closing:.6}px",
            page + 1
        );
    }
}
