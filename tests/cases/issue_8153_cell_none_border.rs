//! 관계형: 표의 기본 테두리가 명시적 NONE인 칸에 선을 만들어서는 안 된다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 148쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 148쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 Print 148쪽 처리 흐름 표와 cellzone 후속 #6619의 구분을 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::style::BorderLineType;
use rhwp::renderer::render_tree::{LineNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn tables<'a>(node: &'a RenderNode, in_cell: bool, found: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::Table(meta) = &node.node_type {
        if in_cell && meta.row_count == 2 && meta.col_count == 11 {
            found.push(node);
        }
    }
    let in_cell = in_cell || matches!(node.node_type, RenderNodeType::TableCell(_));
    for child in &node.children {
        tables(child, in_cell, found);
    }
}

fn lines<'a>(node: &'a RenderNode, found: &mut Vec<&'a LineNode>) {
    if let RenderNodeType::Line(line) = &node.node_type {
        found.push(line);
    }
    for child in &node.children {
        lines(child, found);
    }
}

fn text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        text(child, out);
    }
}

fn verify(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let page = core.build_page_render_tree(147).unwrap();
    let mut found = Vec::new();
    tables(&page.root, false, &mut found);
    assert_eq!(found.len(), 1, "처리 흐름 중첩 표의 누락·중복 금지");
    let table = found[0];
    let cell = |row, col| {
        table.children.iter().find(|n| {
            matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == row && c.col == col)
        }).unwrap()
    };
    let mut painted = Vec::new();
    lines(table, &mut painted);
    let epsilon = 1e-6;
    for (col, side) in [(0, 0), (10, 1)] {
        let node = cell(1, col);
        let RenderNodeType::TableCell(meta) = &node.node_type else {
            unreachable!()
        };
        let source = &core.document().doc_info.border_fills[usize::from(meta.border_fill_id) - 1];
        assert_eq!(source.borders[side].line_type, BorderLineType::None);
        let x = if side == 0 {
            node.bbox.x
        } else {
            node.bbox.x + node.bbox.width
        };
        let overlaps = painted.iter().any(|line| {
            (line.x1 - x).abs() < epsilon
                && (line.x2 - x).abs() < epsilon
                && line.y1.min(line.y2) < node.bbox.y + node.bbox.height - epsilon
                && line.y1.max(line.y2) > node.bbox.y + epsilon
        });
        assert!(
            !overlaps,
            "명시적 NONE인 둘째 행 바깥 변에 선을 보충하면 안 된다"
        );
    }
    let arrow = cell(0, 1);
    let RenderNodeType::TableCell(meta) = &arrow.node_type else {
        unreachable!()
    };
    let source = &core.document().doc_info.border_fills[usize::from(meta.border_fill_id) - 1];
    assert_eq!(source.borders[2].line_type, BorderLineType::None);
    assert!(
        !painted.iter().any(|line| {
            (line.y1 - arrow.bbox.y).abs() < epsilon
                && (line.y2 - arrow.bbox.y).abs() < epsilon
                && line.x1.min(line.x2) < arrow.bbox.x + arrow.bbox.width - epsilon
                && line.x1.max(line.x2) > arrow.bbox.x + epsilon
        }),
        "명시적 NONE인 화살표 칸 윗변을 채우면 안 된다"
    );
    let mut contents = String::new();
    text(table, &mut contents);
    let contents: String = contents.chars().filter(|c| !c.is_whitespace()).collect();
    let mut previous_end = 0;
    for expected in ["신청서작성", "접수", "결재", "인증서발급", "통보"] {
        assert_eq!(
            contents.matches(expected).count(),
            1,
            "흐름 내용 누락·중복 금지"
        );
        let start = contents.find(expected).unwrap();
        assert!(
            start >= previous_end,
            "처리 흐름의 내용 순서 보존: {expected}"
        );
        previous_end = start + expected.len();
    }
    for expected in ["신청인", "처리기관"] {
        assert!(
            contents.contains(expected),
            "처리 흐름 내용 보존: {expected}"
        );
    }
}

#[test]
fn hwp_explicit_none_preserves_flow_diagram_open_edges() {
    verify("hwp");
}

#[test]
fn hwpx_explicit_none_preserves_flow_diagram_open_edges() {
    verify("hwpx");
}
