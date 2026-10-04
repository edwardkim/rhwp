//! 셀 누름틀 활성 주소는 본문 부모 문단과 중첩 경로의 모든 문단을 구분한다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;

type CellPath = Vec<(usize, usize, usize)>;

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

fn table(core: &mut DocumentCore, columns: u16) -> usize {
    let last = core.document().sections[0].paragraphs.len() - 1;
    core.insert_text_native(0, last, 0, "표 앞").unwrap();
    let result: Value =
        serde_json::from_str(&core.create_table_native(0, last, 3, 1, columns).unwrap()).unwrap();
    result["paraIdx"].as_u64().unwrap() as usize
}

fn field(core: &mut DocumentCore, host: usize, path: &[(usize, usize, usize)], name: &str) {
    core.insert_text_in_cell_by_path(0, host, path, 0, "앞  뒤")
        .unwrap();
    core.insert_click_here_field_at_by_path(0, host, path, 2, name, "", name, true)
        .unwrap();
}

fn value(core: &DocumentCore, name: &str) -> String {
    core.collect_all_fields()
        .into_iter()
        .find(|entry| entry.field.field_name() == Some(name))
        .unwrap()
        .value
}

fn guides(core: &DocumentCore) -> Vec<String> {
    let mut result = Vec::new();
    for page in 0..core.page_count() {
        let layout: Value =
            serde_json::from_str(&core.get_page_text_layout_native(page).unwrap()).unwrap();
        for run in layout["runs"].as_array().unwrap() {
            let text = run["text"].as_str().unwrap();
            if text.ends_with("GUIDE") {
                result.push(text.to_string());
            }
        }
    }
    result
}

fn separate_tables() -> (DocumentCore, usize, usize) {
    let mut core = blank();
    let first = table(&mut core, 1);
    let second = table(&mut core, 1);
    field(&mut core, first, &[(0, 0, 0)], "FIRSTGUIDE");
    field(&mut core, second, &[(0, 0, 0)], "SECONDGUIDE");
    (core, first, second)
}

#[test]
fn separate_table_activation_hides_only_the_selected_guide() {
    let (mut core, first, second) = separate_tables();
    core.insert_click_here_field_at(0, 0, 0, "BODYGUIDE", "", "BODYGUIDE", true)
        .unwrap();
    let original = core.render_page_svg_native(0).unwrap();
    assert_eq!(guides(&core), ["BODYGUIDE", "FIRSTGUIDE", "SECONDGUIDE"]);
    assert!(core.set_active_field_in_cell(0, first, 0, 0, 0, 2, false));
    assert_eq!(guides(&core), ["BODYGUIDE", "SECONDGUIDE"]);
    assert!(core.set_active_field_in_cell(0, second, 0, 0, 0, 2, false));
    assert_eq!(guides(&core), ["BODYGUIDE", "FIRSTGUIDE"]);
    assert!(!core.set_active_field_in_cell(0, second, 0, 0, 0, 2, false));
    assert!(core.set_active_field(0, 0, 0));
    assert_eq!(guides(&core), ["FIRSTGUIDE", "SECONDGUIDE"]);
    core.clear_active_field();
    assert_eq!(core.render_page_svg_native(0).unwrap(), original);
}

#[test]
fn inactive_other_table_keeps_start_and_end_insertions_outside() {
    let (mut core, first, second) = separate_tables();
    core.set_field_value_by_name("FIRSTGUIDE", "AAA").unwrap();
    core.set_field_value_by_name("SECONDGUIDE", "BBB").unwrap();
    assert!(core.set_active_field_in_cell(0, first, 0, 0, 0, 5, false));
    core.insert_text_in_cell_native(0, second, 0, 0, 0, 5, "Z")
        .unwrap();
    assert_eq!(value(&core, "SECONDGUIDE"), "BBB");
    core.insert_text_in_cell_native(0, second, 0, 0, 0, 2, "X")
        .unwrap();
    assert_eq!(value(&core, "SECONDGUIDE"), "BBB");
    assert!(core.set_active_field_in_cell(0, second, 0, 0, 0, 6, false));
    core.insert_text_in_cell_native(0, second, 0, 0, 0, 6, "Y")
        .unwrap();
    assert_eq!(value(&core, "SECONDGUIDE"), "BBBY");
    assert_eq!(value(&core, "FIRSTGUIDE"), "AAA");
}

fn nested_tables(same_host: bool) -> (DocumentCore, usize, CellPath, usize, CellPath) {
    let mut core = blank();
    let source = table(&mut core, 1);
    // 두 번째 셀이 투명 1×1 wrapper 축약을 막아 중첩 주소 자체를 검사한다.
    let first = table(&mut core, if same_host { 1 } else { 2 });
    let second = if same_host {
        first
    } else {
        table(&mut core, 2)
    };
    if same_host {
        core.split_paragraph_in_cell_native(0, first, 0, 0, 0, 0, None)
            .unwrap();
    }
    core.copy_control_native(0, source, &[], 0).unwrap();
    core.paste_internal_in_cell_native(0, first, 0, 0, 0, 0)
        .unwrap();
    let second_para = usize::from(same_host);
    core.paste_internal_in_cell_native(0, second, 0, 0, second_para, 0)
        .unwrap();
    let first_path = vec![(0, 0, 0), (0, 0, 0)];
    let second_path = vec![(0, 0, second_para), (0, 0, 0)];
    field(&mut core, first, &first_path, "FIRSTGUIDE");
    field(&mut core, second, &second_path, "SECONDGUIDE");
    (core, first, first_path, second, second_path)
}

#[test]
fn nested_fields_in_separate_hosts_keep_activation_and_snapshot_isolated() {
    let (mut core, first, first_path, second, second_path) = nested_tables(false);
    assert!(core.set_active_field_by_path(0, first, &first_path, 2));
    assert_eq!(guides(&core), ["SECONDGUIDE"]);
    assert!(core.set_active_field_by_path(0, second, &second_path, 2));
    assert_eq!(guides(&core), ["FIRSTGUIDE"]);
    core.set_field_value_by_name("FIRSTGUIDE", "AAA").unwrap();
    core.set_field_value_by_name("SECONDGUIDE", "BBB").unwrap();
    let snapshot = core.save_snapshot_native();
    core.insert_text_in_cell_by_path(0, second, &second_path, 5, "한")
        .unwrap();
    assert_eq!(value(&core, "SECONDGUIDE"), "BBB한");
    core.restore_snapshot_native(snapshot).unwrap();
    core.discard_snapshot_native(snapshot);
    core.insert_text_in_cell_by_path(0, first, &first_path, 5, "X")
        .unwrap();
    assert_eq!(value(&core, "FIRSTGUIDE"), "AAA");
    core.insert_text_in_cell_by_path(0, second, &second_path, 5, "한")
        .unwrap();
    assert_eq!(value(&core, "SECONDGUIDE"), "BBB한");
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(value(&reopened, "FIRSTGUIDE"), "AAA");
        assert_eq!(value(&reopened, "SECONDGUIDE"), "BBB한");
    }
}

#[test]
fn nested_guide_identity_includes_intermediate_cell_paragraph() {
    let (mut core, first, first_path, second, second_path) = nested_tables(true);
    assert_eq!(guides(&core), ["FIRSTGUIDE", "SECONDGUIDE"]);
    assert!(core.set_active_field_by_path(0, first, &first_path, 2));
    assert_eq!(guides(&core), ["SECONDGUIDE"]);
    assert!(core.set_active_field_by_path(0, second, &second_path, 2));
    assert_eq!(guides(&core), ["FIRSTGUIDE"]);
}
