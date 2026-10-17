//! 80168의 완료 페이지: 원본 소유·줄 구성·절대 중첩 폭과 독립 Print 좌표.
//!
//! 원본 SHA-256: c8ad10fe9f07be5119cd804278017aefa46e555bbee4f05f0f5132fe4f591a22.
//! pdf/80168_regulatory_analysis-hwp-2024.pdf:
//! b233df7098547df43455f49c13ba5b63f1f06113c853a4773f86a94d6b34a982.
//! f7ddf477e의96dpi/print/compat2024 전157쪽 Native·fresh WASM 최저는
//! 94.75686/94.95563%. 90쪽은 양쪽100%,133쪽은99.90063/99.97526%,
//! 139쪽은100/99.95091%다. 직접 review/overlay와 TSV는 통합 계획서에 연결한다.
//! 좌표는 PDF 글자 원점·기준선·괘선을72pt→96dpi로 환산하며 허용치는±2px다.
//! 다른 글꼴의 nominal bbox 상단을 PDF glyph bbox 상단과 동일시하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/80168_regulatory_analysis.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 157, "독립 Print 전쪽과 내용 쪽수");
    core
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn select(root: &RenderNode, predicate: impl Fn(&RenderNode) -> bool) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    let selected: Vec<_> = nodes.into_iter().filter(|node| predicate(node)).collect();
    assert_eq!(selected.len(), 1, "원본 소유/내용으로 단일 대상 선택");
    selected[0]
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn compact(value: &str) -> String {
    value.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn table(root: &RenderNode, paragraph: usize) -> &RenderNode {
    select(
        root,
        |node| matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(paragraph)),
    )
}

fn cell(root: &RenderNode, row: u16, col: u16) -> &RenderNode {
    select(
        root,
        |node| matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row == row && c.col == col),
    )
}

fn lines(root: &RenderNode, paragraph: usize) -> Vec<&RenderNode> {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(paragraph))
        })
        .collect()
}

fn near(actual: f64, independent: f64) {
    assert!(
        (actual - independent).abs() <= 2.0,
        "실제 {actual:.6}, 독립 Print {independent:.6}, 허용±2px"
    );
}

fn contained(child: &RenderNode, owner: &RenderNode) {
    let a = child.bbox;
    let b = owner.bbox;
    assert!(a.width > 0.0 && a.height > 0.0);
    assert!(
        a.x >= b.x - 0.05
            && a.y >= b.y - 0.05
            && a.x + a.width <= b.x + b.width + 0.05
            && a.y + a.height <= b.y + b.height + 0.05,
        "자식 {a:?}는 원본 소유 상자 {b:?} 안에 있어야 한다"
    );
}

fn first_run(line: &RenderNode) -> &RenderNode {
    line.children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.display_or_text().trim().is_empty())
        })
        .expect("가시 글자 run")
}

fn baseline(line: &RenderNode, expected: f64) {
    for node in &line.children {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if !run.display_or_text().trim().is_empty() {
                near(node.bbox.y + run.baseline, expected);
            }
        }
    }
}

fn source_table(core: &DocumentCore, paragraph: usize) -> &Table {
    core.document().sections[0].paragraphs[paragraph]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("원본 표")
}

#[test]
fn continuation_keeps_final_row_frame_and_plan_on_completed_page90() {
    let core = load();
    let page = core.build_page_render_tree(89).unwrap();
    let frame = table(&page.root, 760);
    let owner = cell(frame, 30, 1);
    let line = select(owner, |node| {
        matches!(node.node_type, RenderNodeType::TextLine(_)) && compact(&text(node)) == "계획"
    });
    assert_eq!(compact(&text(frame)), "계획", "이전 쪽 본문 중복 없음");
    contained(line, owner);
    near(first_run(line).bbox.x, 144.48);
    baseline(line, 99.36);
    // 독립 PDF90쪽의 실제 네 괘선. 소유는 좌표 검색으로 결정하지 않는다.
    for (actual, independent) in [
        (frame.bbox.x, 77.3987),
        (frame.bbox.y, 77.5146),
        (frame.bbox.x + frame.bbox.width, 733.0453),
        (frame.bbox.y + frame.bbox.height, 108.6813),
    ] {
        near(actual, independent);
    }
}

#[test]
fn bold_decoration_preserves_hancom_line_partition_on_completed_page133() {
    let core = load();
    let page = core.build_page_render_tree(132).unwrap();
    let owner = cell(table(&page.root, 1146), 0, 0);
    let rows = lines(owner, 0);
    assert_eq!(rows.len(), 13, "독립 Print 첫 법 조항의13줄");
    assert!(compact(&text(rows[0])).ends_with("④사업시"));
    assert!(compact(&text(rows[1])).starts_with("행자는"));
    let rendered: String = rows.iter().map(|line| text(line)).collect();
    let original = &source_table(&core, 1146).cells[0].paragraphs[0].text;
    assert_eq!(compact(&rendered), compact(original), "원문 순서·누락·중복");
    for line in &rows {
        contained(line, owner);
    }
    let first = first_run(rows[0]);
    assert!(matches!(&first.node_type, RenderNodeType::TextRun(run) if run.style.bold));
    assert!(rows[0].children.iter().any(|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.display_or_text().contains("사업시") && !run.style.bold)
    }));
    near(first.bbox.x, 103.04);
    baseline(rows[0], 408.8);
    baseline(rows[1], 428.0);
}

#[test]
fn absolute_nested_table_keeps_width_and_text_inside_owner_on_completed_page139() {
    let core = load();
    let page = core.build_page_render_tree(138).unwrap();
    let owner = cell(table(&page.root, 1232), 4, 1);
    let nested = select(owner, |node| {
        matches!(node.node_type, RenderNodeType::Table(_))
    });
    let source_owner = source_table(&core, 1232)
        .cells
        .iter()
        .find(|c| c.row == 4 && c.col == 1)
        .unwrap();
    let source_nested = source_owner.paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .unwrap();
    contained(nested, owner);
    near(
        nested.bbox.width,
        f64::from(source_nested.common.width) * 96.0 / 7200.0,
    );
    let rows = lines(nested, 0);
    assert_eq!(rows.len(), 4, "독립 Print의4줄");
    assert!(compact(&text(rows[0])).ends_with("임대"));
    assert!(compact(&text(rows[1])).starts_with("주택의공급량"));
    let rendered: String = rows.iter().map(|line| text(line)).collect();
    assert_eq!(
        compact(&rendered),
        compact(&source_nested.cells[0].paragraphs[0].text)
    );
    for line in &rows {
        contained(line, nested);
    }
    near(first_run(rows[0]).bbox.x, 209.28);
    baseline(rows[0], 133.76);
}
