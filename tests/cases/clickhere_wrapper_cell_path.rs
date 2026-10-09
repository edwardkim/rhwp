//! 화면에서 펼치는 1×1 표도 내부 누름틀의 논리 셀 주소를 보존해야 한다.

use rhwp::document_core::DocumentCore;
use serde_json::{json, Value};

type CellPath = Vec<(usize, usize, usize)>;

/// 래퍼가 한 쪽을 넘어 부분 표로 나뉘게 하는 안쪽 표의 행 수.
const SPLIT_ROWS: u16 = 80;

fn table(core: &mut DocumentCore, rows: u16, columns: u16) -> usize {
    let last = core.document().sections[0].paragraphs.len() - 1;
    core.insert_text_native(0, last, 0, "표 앞").unwrap();
    let result: Value =
        serde_json::from_str(&core.create_table_native(0, last, 3, rows, columns).unwrap())
            .unwrap();
    result["paraIdx"].as_u64().unwrap() as usize
}

/// 바깥 표마다 `rows`행 안쪽 표를 붙이고 그 마지막 칸에 누름틀을 넣는다.
fn nested_fields(rows: u16, columns: u16, guides: &[&str]) -> (DocumentCore, Vec<usize>, CellPath) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let source = table(&mut core, rows, 1);
    let hosts: Vec<usize> = guides
        .iter()
        .map(|_| table(&mut core, 1, columns))
        .collect();
    core.copy_control_native(0, source, &[], 0).unwrap();
    let path = vec![(0, 0, 0), (0, usize::from(rows) - 1, 0)];
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

/// 모든 쪽의 글자 run을 쪽 번호와 함께 모은다.
fn runs(core: &DocumentCore) -> Vec<(u32, Value)> {
    (0..core.page_count())
        .flat_map(|page| {
            let layout: Value =
                serde_json::from_str(&core.get_page_text_layout_native(page).unwrap()).unwrap();
            let runs = layout["runs"].as_array().unwrap().clone();
            runs.into_iter().map(move |run| (page, run))
        })
        .collect()
}

fn guide_runs(core: &DocumentCore) -> Vec<(u32, Value)> {
    runs(core)
        .into_iter()
        .filter(|(_, run)| run["text"].as_str().unwrap().ends_with("GUIDE"))
        .collect()
}

fn path_json(path: &[(usize, usize, usize)]) -> Value {
    path.iter()
        .map(|&(control, cell, para)| {
            json!({"controlIndex": control, "cellIndex": cell, "cellParaIndex": para})
        })
        .collect()
}

fn assert_nested_paths(columns: u16) {
    let (core, hosts, path) = nested_fields(1, columns, &["FIRSTGUIDE", "SECONDGUIDE"]);
    let runs = guide_runs(&core);
    assert_eq!(runs.len(), 2);
    for ((_, run), host) in runs.iter().zip(hosts) {
        assert_eq!(run["parentParaIdx"], host);
        assert_eq!(run["cellPath"], path_json(&path));
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
    let (mut core, hosts, path) = nested_fields(1, 1, &["FIRSTGUIDE", "SECONDGUIDE"]);
    let original = core.render_page_svg_native(0).unwrap();
    assert!(core.set_active_field_by_path(0, hosts[0], &path, 2));
    // 같은 셀 주소를 쓰는 다른 바깥 표의 안내문은 남는다.
    let remaining = guide_runs(&core);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].1["text"], "SECONDGUIDE");
    core.clear_active_field();
    assert_eq!(core.render_page_svg_native(0).unwrap(), original);
}

#[test]
fn split_transparent_wrapper_preserves_full_cell_path() {
    // 한 쪽을 넘는 래퍼는 부분 표로 나눠 그린다. 첫 칸과 마지막 칸이 다른 쪽에 놓여
    // 첫 조각과 이어지는 조각을 모두 지난다.
    let (mut core, hosts, last) = nested_fields(SPLIT_ROWS, 1, &["SPLITGUIDE"]);
    let first = vec![(0, 0, 0), (0, 0, 0)];
    core.insert_text_in_cell_by_path(0, hosts[0], &first, 0, "첫 칸")
        .unwrap();
    let all = runs(&core);
    let page_of = |text: &str, path: &CellPath| {
        let (page, run) = all.iter().find(|(_, run)| run["text"] == text).unwrap();
        assert_eq!(run["parentParaIdx"], hosts[0]);
        assert_eq!(run["cellPath"], path_json(path));
        *page
    };
    assert!(page_of("첫 칸", &first) < page_of("SPLITGUIDE", &last));
    assert!(core.set_active_field_by_path(0, hosts[0], &last, 2));
    assert!(guide_runs(&core).is_empty());
}
