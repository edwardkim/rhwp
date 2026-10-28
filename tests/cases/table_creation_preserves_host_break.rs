//! 빈 본문 문단을 표 호스트로 바꿀 때 그 자리에 있던 명시적 나눔을 보존한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::ColumnBreakType;
use serde_json::Value;

fn document(kind: ColumnBreakType) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.set_column_def_native(0, 2, 0, true, 2268).unwrap();
    // 새 빈 문서의 단 설정 저장과 분리해, 양쪽 형식에 저장 가능한 단 정의로 시작한다.
    let mut core = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let page: Value = serde_json::from_str(&reopened.get_page_info_native(0).unwrap()).unwrap();
        assert_eq!(page["columns"].as_array().unwrap().len(), 2);
    }
    core.insert_text_native(0, 0, 0, "LEFT").unwrap();
    match kind {
        ColumnBreakType::Column => core.insert_column_break_native(0, 0, 4).unwrap(),
        ColumnBreakType::Page => core.insert_page_break_native(0, 0, 4).unwrap(),
        _ => core.split_paragraph_native(0, 0, 4, None).unwrap(),
    };
    core
}

fn cursor(core: &DocumentCore) -> Value {
    serde_json::from_str(&core.get_cursor_rect_native(0, 1, 0).unwrap()).unwrap()
}

fn insert_table(core: &mut DocumentCore, offset: usize) -> usize {
    let result: Value =
        serde_json::from_str(&core.create_table_native(0, 1, offset, 1, 1).unwrap()).unwrap();
    let host = result["paraIdx"].as_u64().unwrap() as usize;
    // 나눔 메타데이터를 표 너비에 따른 넘침과 분리해 검사한다.
    core.set_cell_properties_native(0, host, 0, 0, r#"{"width":10000}"#)
        .unwrap();
    core.insert_text_in_cell_native(0, host, 0, 0, 0, 0, "TABLE")
        .unwrap();
    host
}

fn check_rendered_location(core: &DocumentCore, host: usize, before: &Value) {
    let cell: Value = serde_json::from_str(
        &core
            .get_cursor_rect_in_cell_native(0, host, 0, 0, 0, 0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        cell["pageIndex"], before["pageIndex"],
        "표는 나누기 뒤의 쪽에 남아야 한다"
    );
    let page = cell["pageIndex"].as_u64().unwrap() as u32;
    let controls: Value =
        serde_json::from_str(&core.get_page_control_layout_native(page).unwrap()).unwrap();
    let table = controls["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["type"] == "table")
        .unwrap();
    assert!(
        table["x"].as_f64().unwrap() >= before["x"].as_f64().unwrap() - 0.5,
        "표가 앞 단으로 돌아가면 안 된다: {table}"
    );
    let text: Value =
        serde_json::from_str(&core.get_page_text_layout_native(page).unwrap()).unwrap();
    let words: Vec<_> = text["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["text"] == "TABLE")
        .collect();
    assert_eq!(words.len(), 1, "표 글은 해당 쪽에 한 번만 그린다");
    assert!(words[0]["x"].as_f64().unwrap() >= table["x"].as_f64().unwrap());
    assert!(words[0]["y"].as_f64().unwrap() >= table["y"].as_f64().unwrap());
    assert!(
        words[0]["y"].as_f64().unwrap()
            < table["y"].as_f64().unwrap() + table["h"].as_f64().unwrap()
    );
    assert_eq!(core.document().sections[0].paragraphs[0].text, "LEFT");
}

fn check_empty_host(kind: ColumnBreakType, raw_break: u8) {
    let mut core = document(kind);
    let before = cursor(&core);
    let snapshot = core.save_snapshot_native();
    let host = insert_table(&mut core, 0);
    assert_eq!(host, 1);
    let paragraph = &core.document().sections[0].paragraphs[host];
    assert_eq!(paragraph.column_type, kind);
    assert_eq!(paragraph.raw_break_type, raw_break);
    assert!(!paragraph.page_break_synthesized);
    check_rendered_location(&core, host, &before);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            reopened.document().sections[0].paragraphs[host].column_type,
            kind
        );
        check_rendered_location(&reopened, host, &before);
    }
    let with_table = core.save_snapshot_native();
    core.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(core.document().sections[0].paragraphs[1].column_type, kind);
    assert!(core.document().sections[0].paragraphs[1]
        .controls
        .is_empty());
    assert_eq!(cursor(&core), before);
    core.restore_snapshot_native(with_table).unwrap();
    check_rendered_location(&core, host, &before);
    core.discard_snapshot_native(snapshot);
    core.discard_snapshot_native(with_table);
}

#[test]
fn empty_column_break_host_keeps_the_table_in_its_column() {
    check_empty_host(ColumnBreakType::Column, 0x08);
}

#[test]
fn empty_page_break_host_keeps_the_table_on_its_page() {
    check_empty_host(ColumnBreakType::Page, 0x04);
}

#[test]
fn ordinary_empty_host_and_split_insertion_do_not_acquire_a_break() {
    check_empty_host(ColumnBreakType::None, 0);
    let mut core = document(ColumnBreakType::Page);
    core.insert_text_native(0, 1, 0, "AB").unwrap();
    let host = insert_table(&mut core, 1);
    assert_eq!(
        core.document().sections[0].paragraphs[1].column_type,
        ColumnBreakType::Page
    );
    assert_eq!(
        core.document().sections[0].paragraphs[host].column_type,
        ColumnBreakType::None
    );
    assert_eq!(
        core.document().sections[0].paragraphs[host].raw_break_type,
        0
    );
    assert_eq!(
        core.document().sections[0]
            .paragraphs
            .iter()
            .filter(|p| p.column_type == ColumnBreakType::Page)
            .count(),
        1
    );
}

#[test]
fn replacing_a_synthesized_break_host_keeps_its_export_provenance() {
    let mut core = document(ColumnBreakType::Page);
    let mut source = core.document().clone();
    source.sections[0].paragraphs[1].page_break_synthesized = true;
    core.set_document(source);
    let host = insert_table(&mut core, 0);
    let paragraph = &core.document().sections[0].paragraphs[host];
    assert_eq!(paragraph.column_type, ColumnBreakType::Page);
    assert_eq!(paragraph.raw_break_type, 0x04);
    assert!(
        paragraph.page_break_synthesized,
        "자연 경계를 명시적인 파일 나눔으로 바꾸면 안 된다"
    );
}

/// 한컴2020 Print: 폭 0 개체 앵커의 표는 새 단 본문 원점에 바깥 위여백을 더한다.
/// 절대 픽셀값 대신 원본 HWPUNIT과 본문 폭의 무차원 비율을 비교한다.
#[test]
fn saved_empty_table_anchor_preserves_its_outer_box_in_the_new_column() {
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn find(node: &RenderNode, table: bool) -> Option<&RenderNode> {
        let selected = match &node.node_type {
            RenderNodeType::Table(_) => table,
            RenderNodeType::Body { .. } => !table,
            _ => false,
        };
        if selected {
            Some(node)
        } else {
            node.children.iter().find_map(|n| find(n, table))
        }
    }
    let core = DocumentCore::from_bytes(include_bytes!(
        "../fixtures/issue7571/column-table-outer-box.hwp"
    ))
    .unwrap();
    assert_eq!(core.page_count(), 1);
    let section = &core.document().sections[0];
    let Control::Table(source) = &section.paragraphs[1].controls[0] else {
        panic!("표")
    };
    let page = &section.section_def.page_def;
    let source_width = (page.width - page.margin_left - page.margin_right) as f64;
    let tree = core.build_page_render_tree(0).unwrap();
    let body = &find(&tree.root, false).unwrap().bbox;
    let table = &find(&tree.root, true).unwrap().bbox;
    let actual = (table.y - body.y) / body.width;
    let expected = f64::from(source.outer_margin_top) / source_width;
    assert!(
        (actual - expected).abs() < 0.00001,
        "바깥 위여백 비율: {actual} / {expected}"
    );
    assert!(table.x > body.x + body.width / 2.0, "표는 오른쪽 단 소유");
    let before = cursor(&document(ColumnBreakType::Column));
    check_rendered_location(&core, 1, &before);
}

#[test]
fn production_hwp_save_preserves_the_empty_floating_table_object_anchor() {
    let mut core = document(ColumnBreakType::Column);
    let host = insert_table(&mut core, 0);
    let reopened =
        DocumentCore::from_bytes(&core.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
    assert_eq!(
        reopened.document().sections[0].paragraphs[host].line_segs[0].segment_width,
        0,
        "빈 단일 floating table의 폭 0 앵커는 글줄 폭으로 재작성하지 않는다"
    );
}
