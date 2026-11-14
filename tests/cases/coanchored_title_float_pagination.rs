//! 검사 유형: 관계형·고정형(독립 Print의 전쪽 쪽수).
//! 주석 갱신: 2026-11-13T22:39:25+09:00.
//! 직전 파일 변경: 62742884777cd8a07b431ead5bb4338520f590f7 2026-11-13T21:04:19+09:00.
//! 측정: 2026-11-13, 96 DPI·PDFium·색상 경계 보정 전 2px raw 실루엣, Native Skia/fresh 최적화 WASM Canvas.
//! 원 Mac Print 사례 전 2쪽 최저: Native 100.00% / WASM 100.00%.
//! 무변경 저장·빈 머리말·빈 꼬리말 각각 전 2쪽 최저: Native 100.00% / WASM 100.00%.
//! 실제 한컴 재저장 원 사례와 중첩 한 문단·바깥 여백·두 문단 각각 전 2쪽 최저: Native 100.00% / WASM 100.00%.
//! 실제 BodyX 편집 전 2쪽 최저: Native 100.00% / WASM 100.00%; 양쪽 한컴 2020 Print 쪽수와 일치.
//! 제품 소스 1506a1c34672931c4f5aa5cb6363fba776225949; 전쪽 TSV·PNG: mydocs/pr/assets/pr9042/final-validation/.
//! 양쪽에 같은 HCRDotum/HCRBatang 프로그램을 공급하고 384 DPI 재출력으로 괘선·글자 하강부를 직접 판독했다.
//! 독립 기준·실제/합성 저장 입력 구분: mydocs/pr/assets/pr9042/native-nested-validation/README.md 및 final-validation/live-edit/README.md.

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
        contents_stay_in_cells(table, None);
    }
}

fn source_body(document: &rhwp::model::document::Document) -> &Table {
    document.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 2 => Some(table.as_ref()),
            _ => None,
        })
        .unwrap()
}

fn rendered_body(core: &DocumentCore) -> RenderNode {
    let tree = core.build_page_render_tree(0).unwrap();
    let mut tables = Vec::new();
    top_level_tables(&tree.root, &mut tables);
    let body = tables
        .into_iter()
        .find(|table| {
            let mut content = String::new();
            text(table, &mut content);
            content.starts_with("BodyEnd of body")
        })
        .unwrap();
    (*body).clone()
}

fn count_lines(node: &RenderNode) -> usize {
    usize::from(matches!(node.node_type, RenderNodeType::TextLine(_)))
        + node.children.iter().map(count_lines).sum::<usize>()
}

#[test]
fn coanchored_stale_metrics_keep_source_and_all_empty_paragraphs() {
    let source = rhwp::parser::parse_hwp(SOURCE).unwrap();
    let core = DocumentCore::from_bytes(SOURCE).unwrap();
    let body = rendered_body(&core);
    let first_cell = body
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .unwrap();
    let paragraphs = &source_body(&source).cells[0].paragraphs;
    assert_eq!(
        count_lines(first_cell),
        paragraphs.len(),
        "empty paragraphs retain their real lines"
    );
    for (original, current) in paragraphs
        .iter()
        .zip(&source_body(core.document()).cells[0].paragraphs)
    {
        assert_eq!(original.text, current.text);
        assert_eq!(original.line_segs.len(), current.line_segs.len());
        for (original, current) in original.line_segs.iter().zip(&current.line_segs) {
            assert_eq!(
                (
                    original.vertical_pos,
                    original.line_height,
                    original.text_height,
                    original.baseline_distance,
                    original.line_spacing
                ),
                (
                    current.vertical_pos,
                    current.line_height,
                    current.text_height,
                    current.baseline_distance,
                    current.line_spacing
                ),
                "the renderer projects current geometry without rewriting source IR"
            );
        }
    }
}

#[test]
fn coanchored_managed_cache_survives_public_save_and_reopen_paths() {
    let core = DocumentCore::from_bytes(SOURCE).unwrap();
    let before = rendered_body(&core);
    let original = source_body(core.document()).cells[0].paragraphs.clone();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwp_with_adapter().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_coanchored_tables(&bytes);
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let after = rendered_body(&reopened);
        if bytes.starts_with(b"PK") {
            use rhwp::serializer::hwpx::roundtrip::{
                diff_exported_hwpx_documents, hwpx_export_expected_document,
                strip_hwp_to_hwpx_noise,
            };
            let expected = hwpx_export_expected_document(&core.hwpx_export_expected_document());
            let actual = hwpx_export_expected_document(reopened.document());
            assert!(
                strip_hwp_to_hwpx_noise(diff_exported_hwpx_documents(&expected, &actual))
                    .is_empty()
            );
            let mut corrupted = actual.clone();
            let table = corrupted.sections[0].paragraphs[0]
                .controls
                .iter_mut()
                .find_map(|control| match control {
                    Control::Table(table) if table.row_count == 2 => Some(table),
                    _ => None,
                })
                .unwrap();
            table.cells[0].paragraphs[0].line_segs[0].baseline_distance += 1;
            assert!(
                !strip_hwp_to_hwpx_noise(diff_exported_hwpx_documents(&expected, &corrupted))
                    .is_empty(),
                "verification still detects a real saved baseline corruption"
            );
        }
        assert_eq!(count_lines(&before), count_lines(&after));
        assert!(
            (last_text_baseline(&before) - last_text_baseline(&after)).abs() < 1e-6,
            "the saved cache retains the same current baseline"
        );
    }
    for (before, after) in original
        .iter()
        .zip(&source_body(core.document()).cells[0].paragraphs)
    {
        assert_eq!(
            before.line_segs[0].baseline_distance,
            after.line_segs[0].baseline_distance
        );
        assert_eq!(
            before.line_segs[0].vertical_pos,
            after.line_segs[0].vertical_pos
        );
    }
}

fn last_text_baseline(node: &RenderNode) -> f64 {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == "End of body" {
            return run.baseline;
        }
    }
    node.children
        .iter()
        .map(last_text_baseline)
        .fold(0.0, f64::max)
}

#[test]
fn coanchored_cache_refresh_requires_the_current_section_receipt() {
    let current = DocumentCore::from_bytes(SOURCE).unwrap();
    let mut foreign = current.document().clone();
    let marker = &mut foreign
        .extra_streams
        .iter_mut()
        .find(|(name, _)| name == "/RhwpLineSegTags")
        .unwrap()
        .1;
    *marker.last_mut().unwrap() ^= 1;
    let mut core = DocumentCore::from_bytes(SOURCE).unwrap();
    core.set_document(foreign);
    let stored = source_body(core.document()).cells[0]
        .paragraphs
        .last()
        .unwrap()
        .line_segs[0]
        .baseline_distance;
    let before = last_text_baseline(&rendered_body(&core));
    assert!(
        (before - f64::from(stored) / 75.0).abs() < 1e-6,
        "an unauthenticated cache follows the original stored-frame contract"
    );
    assert!(
        last_text_baseline(&rendered_body(&current)) < before,
        "current styles replace the old baseline only in the managed cache"
    );
}

#[test]
fn coanchored_large_empty_struts_with_current_fixed_advance_are_retained() {
    let mut core = DocumentCore::from_bytes(SOURCE).unwrap();
    let mut document = core.document().clone();
    let body = document.sections[0].paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 2 => Some(table.as_mut()),
            _ => None,
        })
        .unwrap();
    let style_id = usize::from(body.cells[0].paragraphs[0].para_shape_id);
    // Independent synthetic contract: larger blank em boxes can be intentional.
    // The current fixed advance agrees with their cached height + trailing gap.
    let advance = body.cells[0].paragraphs[1].line_segs[0].line_height
        + body.cells[0].paragraphs[1].line_segs[0].line_spacing;
    let last = body.cells[0].paragraphs.last_mut().unwrap();
    last.line_segs[0].line_spacing = advance - last.line_segs[0].line_height;
    let style = &mut document.doc_info.para_shapes[style_id];
    assert_eq!(
        style.line_spacing_type,
        rhwp::model::style::LineSpacingType::Fixed
    );
    style.line_spacing = advance * 2;
    style.line_spacing_v2 = (advance * 2) as u32;
    style.raw_data = None;
    document.doc_info.raw_stream = None;
    document.sections[0].raw_stream = None;
    core.set_document(document);
    let bytes = core.export_hwp_native().unwrap();
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    let paragraphs = &source_body(reopened.document()).cells[0].paragraphs;
    let stored = paragraphs.last().unwrap().line_segs[0].baseline_distance;
    assert!(
        (last_text_baseline(&rendered_body(&reopened)) - f64::from(stored) / 75.0).abs() < 1e-6,
        "a larger cache is preserved when current paragraph properties still own its advance"
    );
    assert_eq!(
        count_lines(&rendered_body(&reopened)),
        source_body(reopened.document())
            .cells
            .iter()
            .map(|cell| cell.paragraphs.len())
            .sum::<usize>()
    );
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

#[test]
fn coanchored_same_line_edit_keeps_the_live_and_saved_physical_frame() {
    let normalized = DocumentCore::from_bytes(SOURCE)
        .unwrap()
        .export_hwp_native()
        .unwrap();
    for input in [SOURCE, normalized.as_slice()] {
        let mut core = DocumentCore::from_bytes(input).unwrap();
        let before = rendered_body(&core);
        let original_lines = count_lines(&before);
        let original_height = source_body(core.document()).common.height;
        core.insert_text_in_cell_native(0, 0, 3, 0, 0, 4, "X")
            .unwrap();
        assert_edited_frame(&core, &before, original_lines, original_height);
        for format in ["hwp", "hwpx"] {
            let mut current = None;
            // Use the original live edited document for the first save in each format.
            for _cycle in 0..2 {
                let source = current.as_ref().unwrap_or(&core);
                let bytes = if format == "hwp" {
                    source.export_hwp_native().unwrap()
                } else {
                    source.export_hwpx_native().unwrap()
                };
                current = Some(DocumentCore::from_bytes(&bytes).unwrap());
                assert_edited_frame(
                    current.as_ref().unwrap(),
                    &before,
                    original_lines,
                    original_height,
                );
            }
        }
    }
}

fn assert_edited_frame(core: &DocumentCore, before: &RenderNode, lines: usize, height: u32) {
    // Independent Print: adding X to the first short line retains both table frames
    // and the explicit page break; it does not remove the 32 intentional empty lines.
    assert_eq!(core.page_count(), 2);
    assert_eq!(source_body(core.document()).common.height, height);
    let first = core.build_page_render_tree(0).unwrap();
    let second = core.build_page_render_tree(1).unwrap();
    let mut tables = Vec::new();
    top_level_tables(&first.root, &mut tables);
    let [title, body] = tables.as_slice() else {
        panic!("title and edited body share page 1")
    };
    let mut body_text = String::new();
    text(body, &mut body_text);
    assert_eq!(body_text, "BodyXEnd of bodyBody end");
    assert_eq!(count_lines(body), lines);
    contents_stay_in_cells(body, None);
    assert!(title.bbox.y + title.bbox.height <= body.bbox.y);
    assert!((body.bbox.y - before.bbox.y).abs() < 1.0);
    assert!((body.bbox.height - before.bbox.height).abs() < 1.0);
    let mut following = Vec::new();
    top_level_tables(&second.root, &mut following);
    assert_eq!(following.len(), 1);
    let mut next = String::new();
    text(following[0], &mut next);
    assert_eq!(next, "Next page");
}
