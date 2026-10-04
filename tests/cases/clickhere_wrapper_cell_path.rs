//! 화면에서 펼치는 1×1 표도 내부 누름틀의 논리 셀 주소를 보존해야 한다.

use rhwp::document_core::DocumentCore;
use serde_json::{json, Value};

type CellPath = Vec<(usize, usize, usize)>;

fn table(core: &mut DocumentCore, columns: u16) -> usize {
    let last = core.document().sections[0].paragraphs.len() - 1;
    core.insert_text_native(0, last, 0, "표 앞").unwrap();
    let result: Value =
        serde_json::from_str(&core.create_table_native(0, last, 3, 1, columns).unwrap()).unwrap();
    result["paraIdx"].as_u64().unwrap() as usize
}

fn nested_fields(columns: u16, guides: &[&str]) -> (DocumentCore, Vec<usize>, CellPath) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let source = table(&mut core, 1);
    let hosts: Vec<usize> = guides.iter().map(|_| table(&mut core, columns)).collect();
    core.copy_control_native(0, source, &[], 0).unwrap();
    let path = vec![(0, 0, 0), (0, 0, 0)];
    for (&host, &guide) in hosts.iter().zip(guides) {
        core.paste_internal_in_cell_native(0, host, 0, 0, 0, 0)
            .unwrap();
        core.insert_text_in_cell_by_path(0, host, &path, 0, "앞  뒤")
            .unwrap();
        core.insert_click_here_field_at_by_path(0, host, &path, 2, guide, "", guide, true)
            .unwrap();
    }
    (core, hosts, path)
}

fn guide_runs(core: &DocumentCore) -> Vec<Value> {
    (0..core.page_count())
        .flat_map(|page| {
            let layout: Value =
                serde_json::from_str(&core.get_page_text_layout_native(page).unwrap()).unwrap();
            layout["runs"].as_array().unwrap().clone()
        })
        .filter(|run| run["text"].as_str().unwrap().ends_with("GUIDE"))
        .collect()
}

fn assert_nested_paths(columns: u16) {
    let (core, hosts, path) = nested_fields(columns, &["FIRSTGUIDE", "SECONDGUIDE"]);
    let runs = guide_runs(&core);
    assert_eq!(runs.len(), 2);
    let expected = Value::Array(
        path.iter()
            .map(|&(control, cell, para)| {
                json!({"controlIndex": control, "cellIndex": cell, "cellParaIndex": para})
            })
            .collect(),
    );
    for (run, host) in runs.iter().zip(hosts) {
        assert_eq!(run["parentParaIdx"], host);
        assert_eq!(run["cellPath"], expected);
    }
}

#[test]
fn nontransparent_nested_table_preserves_full_cell_path() {
    // 두 열인 표는 1×1 펼침을 거치지 않는 대조군이다.
    assert_nested_paths(2);
}

#[test]
fn transparent_wrapper_preserves_full_cell_path() {
    assert_nested_paths(1);
}

#[test]
fn transparent_wrapper_activation_hides_its_guide() {
    // 부모 문단 identity 결함과 분리하려고 누름틀이 있는 외곽 표를 하나만 둔다.
    let (mut core, hosts, path) = nested_fields(1, &["FIRSTGUIDE"]);
    assert_eq!(guide_runs(&core).len(), 1);
    let original = core.render_page_svg_native(0).unwrap();
    assert!(core.set_active_field_by_path(0, hosts[0], &path, 2));
    assert!(guide_runs(&core).is_empty());
    core.clear_active_field();
    assert_eq!(core.render_page_svg_native(0).unwrap(), original);
}
