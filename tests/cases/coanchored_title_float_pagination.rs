//! 검사 유형: 관계형·고정형(독립 Print의 전쪽 쪽수).
//! 주석 갱신: 2026-11-13T18:45:00+09:00.
//! 직전 파일 변경: 637410dbfa8deb060d1c8739b0bf5e7ae8286c89, 2026-11-13T15:49:10+09:00.
//! 측정: 2026-11-13, 96 DPI·PDFium·색상 경계 보정 전 2px raw 실루엣, Native Skia/fresh 최적화 WASM Canvas.
//! 원 Mac Print 사례 전 2쪽 최저: Native 94.20% / WASM 94.38%.
//! 무변경 저장·빈 머리말·빈 꼬리말 각각 전 2쪽 최저: Native 96.04% / WASM 96.72%.
//! 실제 한컴 재저장 원 사례와 중첩 한 문단·바깥 여백·두 문단 각각 전 2쪽 최저: Native 100.00% / WASM 99.59%.
//! 제품 소스 fedc716cbe810985c798254b370962e24aa4ffcf; 전쪽 TSV: mydocs/pr/assets/pr9042/final-visual/.
//! 독립 기준·직접 판독·입력 생성 구분: mydocs/pr/assets/pr9042/native-nested-validation/README.md.
//! 원 Mac 사례의 기존 글줄/괘선 차이는 위 수치로 면제하지 않는다. 이 파일은 그 픽셀을 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, table::Table};
use rhwp::renderer::height_measurer::fit_stored_declared_tail_row_heights;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SOURCE: &[u8] =
    include_bytes!("../../mydocs/pr/assets/coanchored-table-pagination/float-pagination.hwp");

fn top_level_tables<'a>(node: &'a RenderNode, tables: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::Table(_)) {
        tables.push(node);
        return;
    }
    for child in &node.children {
        top_level_tables(child, tables);
    }
}

fn text(node: &RenderNode, result: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push_str(run.display_or_text());
    }
    for child in &node.children {
        text(child, result);
    }
}

#[test]
fn coanchored_title_and_body_stay_on_the_first_page() {
    assert_coanchored_tables(SOURCE);
}

fn assert_coanchored_tables(bytes: &[u8]) {
    let core = DocumentCore::from_bytes(bytes).unwrap();
    assert_eq!(core.page_count(), 2);

    let first = core.build_page_render_tree(0).unwrap();
    let second = core.build_page_render_tree(1).unwrap();
    let mut first_tables = Vec::new();
    let mut second_tables = Vec::new();
    top_level_tables(&first.root, &mut first_tables);
    top_level_tables(&second.root, &mut second_tables);
    let [title, body] = first_tables.as_slice() else {
        panic!("Expected the title and body tables on page 1");
    };
    let [next] = second_tables.as_slice() else {
        panic!("Expected only the following table on page 2");
    };
    assert!(title.bbox.y + title.bbox.height <= body.bbox.y);
    for (table, expected) in [
        (title, "Title"),
        (body, "BodyEnd of bodyBody end"),
        (next, "Next page"),
    ] {
        let mut actual = String::new();
        text(table, &mut actual);
        assert_eq!(actual, expected);
    }
}

#[test]
fn coanchored_empty_header_footer_and_saved_copy_preserve_table_ownership() {
    for path in [
        "samples/pr9042/metadata-unchanged.hwp",
        "samples/pr9042/metadata-empty-header.hwp",
        "samples/pr9042/metadata-empty-footer.hwp",
        "samples/pr9042/pr9042-hancom-saved.hwp",
    ] {
        assert_coanchored_tables(&std::fs::read(path).unwrap());
    }
}

fn inside(node: &RenderNode, parent: &RenderNode) {
    // Border rounding tolerance, not an absolute placement pin.
    assert!(node.bbox.x >= parent.bbox.x - 0.5);
    assert!(node.bbox.y >= parent.bbox.y - 0.5);
    assert!(node.bbox.x + node.bbox.width <= parent.bbox.x + parent.bbox.width + 0.5);
    assert!(node.bbox.y + node.bbox.height <= parent.bbox.y + parent.bbox.height + 0.5);
}

fn contents_stay_in_cells<'a>(node: &'a RenderNode, cell: Option<&'a RenderNode>) {
    if matches!(
        node.node_type,
        RenderNodeType::TextLine(_) | RenderNodeType::Table(_)
    ) {
        if let Some(cell) = cell {
            inside(node, cell);
        }
    }
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    for child in &node.children {
        contents_stay_in_cells(child, cell);
    }
}

#[test]
fn coanchored_saved_nested_boxes_keep_content_inside_the_recovered_tail() {
    for (kind, expected) in [
        ("single-line", "BodyEnd of bodyBody end"),
        ("margins", "BodyEnd of bodyBody end"),
        ("two-paragraphs", "BodyEnd of bodyBody endBody end"),
    ] {
        let bytes = std::fs::read(format!("samples/pr9042/nested-tail-{kind}-hancom.hwp")).unwrap();
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(core.page_count(), 2, "{kind}: independent Print page count");
        for (page, expected) in [(0, expected), (1, "Next page")] {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut tables = Vec::new();
            top_level_tables(&tree.root, &mut tables);
            let [table] = tables.as_slice() else {
                panic!("{kind}: page {page} must own exactly one outer table");
            };
            let mut actual = String::new();
            text(table, &mut actual);
            assert_eq!(
                actual, expected,
                "{kind}: content order, omissions and duplication"
            );
            contents_stay_in_cells(table, None);
            for cell in &table.children {
                if matches!(cell.node_type, RenderNodeType::TableCell(_)) {
                    inside(cell, table);
                }
            }
        }
    }
}

fn saved_nested_table() -> Table {
    let bytes = include_bytes!("../../samples/pr9042/nested-tail-two-paragraphs-hancom.hwp");
    let core = DocumentCore::from_bytes(bytes).unwrap();
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 2 => Some(table.as_ref().clone()),
            _ => None,
        })
        .expect("actual Hancom saved two-row parent")
}

fn child_table_mut(table: &mut Table) -> &mut Table {
    table.cells[1].paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|control| match control {
            Control::Table(child) => Some(child.as_mut()),
            _ => None,
        })
        .expect("saved child table")
}

#[test]
fn coanchored_nested_box_admission_rejects_unproved_occupancy() {
    let table = saved_nested_table();
    // Independent source contract: the first cell's stored content end and padding
    // own its grown height; the untouched tail retains its cellSz declaration.
    let first = &table.cells[0];
    let end = first
        .paragraphs
        .iter()
        .flat_map(|p| &p.line_segs)
        .map(|line| line.vertical_pos + line.line_height)
        .max()
        .unwrap();
    let first_height = f64::from(end + first.effective_vertical_padding_hu(&table.padding)) / 75.0;
    let rows = [first_height, f64::from(table.cells[1].height) / 75.0];
    let spacing = f64::from(table.cell_spacing) / 75.0;
    let fitted = fit_stored_declared_tail_row_heights(&rows, spacing, &table, 96.0)
        .expect("closed saved boxes allow recovery of an empty tail band");
    assert_eq!(fitted[0], rows[0]);
    assert!(fitted[1] < rows[1]);
    for variant in 0..4 {
        let mut invalid = table.clone();
        match variant {
            0 => child_table_mut(&mut invalid).common.width += 1,
            1 => child_table_mut(&mut invalid).outer_margin_top = -1,
            2 => {
                let paragraph = &mut child_table_mut(&mut invalid).cells[0].paragraphs[1];
                paragraph.line_segs[0].vertical_pos = 0;
                paragraph.source_line_seg_vertical_pos = Some(vec![0]);
            }
            3 => invalid.cells[0].row_span = 2,
            _ => unreachable!(),
        }
        assert!(
            fit_stored_declared_tail_row_heights(&rows, spacing, &invalid, 96.0).is_none(),
            "synthetic contract {variant}: no whole-box proof; do not reclaim the band"
        );
    }
}
