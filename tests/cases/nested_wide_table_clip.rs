//! 부모 칸보다 넓은 중첩 표는 한컴처럼 선언 폭을 지키고, 보이는 범위는 부모 칸에서
//! 잘리며, 잘린 칸은 클릭으로 고를 수 없다.
//!
//! 재현 문서 `tests/fixtures/nested_wide_clip/nested-wide-hancom.hwpx`는 한컴에서
//! 만들었다. 148mm 2×2 표(폭 41954, 칸 20977)를 1×2 표의 오른쪽 칸에 붙이고, 붙인 표의
//! 왼쪽 아래 칸에 한 번 더 붙였다. 한컴은 어느 표도 줄이지 않고 바깥 표 오른쪽
//! 테두리에서 그림을 자른다. 맨 안쪽 오른쪽 위 칸("셋")은 다 가려져 클릭되지 않고
//! Tab으로만 들어간다.

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "tests/fixtures/nested_wide_clip/nested-wide-hancom.hwpx";

fn load(bytes: &[u8]) -> HwpDocument {
    HwpDocument::from_bytes(bytes).expect("문서를 연다")
}

fn collect_tables<'a>(table: &'a Table, out: &mut Vec<&'a Table>) {
    out.push(table);
    for cell in &table.cells {
        for para in &cell.paragraphs {
            for control in &para.controls {
                if let Control::Table(nested) = control {
                    collect_tables(nested, out);
                }
            }
        }
    }
}

/// 문서의 모든 표(중첩 포함)의 (표 폭, 칸 폭들).
fn table_widths(doc: &HwpDocument) -> Vec<(u32, Vec<u32>)> {
    let mut tables = Vec::new();
    for para in &doc.document().sections[0].paragraphs {
        for control in &para.controls {
            if let Control::Table(table) = control {
                collect_tables(table, &mut tables);
            }
        }
    }
    tables
        .iter()
        .map(|t| (t.common.width, t.cells.iter().map(|c| c.width).collect()))
        .collect()
}

fn assert_declared_widths(doc: &HwpDocument, label: &str) {
    let widths = table_widths(doc);
    assert_eq!(widths.len(), 4, "{label}: 표 네 개(본문 둘, 중첩 둘)");
    for (table, cells) in widths {
        assert_eq!(table, 41954, "{label}: 표 폭을 줄이지 않는다");
        assert!(
            cells.iter().all(|&w| w == 20977),
            "{label}: 칸 폭을 줄이지 않는다: {cells:?}"
        );
    }
}

fn bbox(node: &Value) -> (f64, f64, f64, f64) {
    let b = &node["bbox"];
    let f = |k: &str| b[k].as_f64().unwrap();
    (f("x"), f("y"), f("w"), f("h"))
}

/// "셋" 글 런과 그 런을 품은 조상 칸들의 bbox(=셀 clip).
fn find_set_run(
    node: &Value,
    cells: &mut Vec<(f64, f64, f64, f64)>,
) -> Option<(Value, Vec<(f64, f64, f64, f64)>)> {
    if node["type"] == "TextRun" && node["text"] == "셋" {
        return Some((node.clone(), cells.clone()));
    }
    let is_cell = node["type"] == "Cell";
    if is_cell {
        cells.push(bbox(node));
    }
    let found = node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| find_set_run(child, cells));
    if is_cell {
        cells.pop();
    }
    found
}

fn innermost_tables(node: &Value, depth: usize, out: &mut Vec<(usize, (f64, f64, f64, f64))>) {
    let depth = if node["type"] == "Table" {
        out.push((depth, bbox(node)));
        depth + 1
    } else {
        depth
    };
    for child in node["children"].as_array().into_iter().flatten() {
        innermost_tables(child, depth, out);
    }
}

fn hit(doc: &HwpDocument, x: f64, y: f64) -> Value {
    serde_json::from_str(&doc.hit_test_native(0, x, y).expect("hit test")).unwrap()
}

/// 맨 안쪽 표의 칸 경로: 바깥 표 오른쪽 칸 → 가운데 표 왼쪽 아래 칸 → 맨 안쪽 표의 칸.
fn innermost_path(cell_index: u64) -> Value {
    serde_json::json!([
        {"controlIndex": 0, "cellIndex": 1, "cellParaIndex": 0},
        {"controlIndex": 1, "cellIndex": 2, "cellParaIndex": 0},
        {"controlIndex": 1, "cellIndex": cell_index, "cellParaIndex": 0},
    ])
}

#[test]
fn nested_wide_tables_keep_declared_widths_through_save() {
    let bytes = std::fs::read(SAMPLE).unwrap();
    let doc = load(&bytes);
    assert_declared_widths(&doc, "열기");
    let hwpx = doc.export_hwpx().expect("HWPX 저장");
    assert_declared_widths(&load(&hwpx), "HWPX 다시 열기");
    let hwp = doc.export_hwp().expect("HWP 저장");
    assert_declared_widths(&load(&hwp), "HWP 다시 열기");
}

#[test]
fn nested_wide_table_is_clipped_at_its_host_and_clipped_cells_are_not_clickable() {
    let doc = load(&std::fs::read(SAMPLE).unwrap());
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let table_px = hwpunit_to_px(41954, DEFAULT_DPI);
    let column_px = hwpunit_to_px(20977, DEFAULT_DPI);

    // 중첩 표도 선언 폭 그대로 놓인다.
    let mut tables = Vec::new();
    innermost_tables(&tree, 0, &mut tables);
    assert_eq!(tables.iter().map(|t| t.0).max(), Some(2), "세 겹 표");
    for (_, (_, _, w, _)) in &tables {
        assert!((w - table_px).abs() < 0.5, "표 폭 {w} != {table_px}");
    }
    let (inner_x, _, _, _) = tables.iter().find(|t| t.0 == 2).unwrap().1;
    let (host_x, _, host_w, _) = tables
        .iter()
        .find(|t| t.0 == 0 && t.1 .1 > 170.0)
        .unwrap()
        .1;
    let visible_right = host_x + host_w;

    // "셋" 칸은 선언 폭대로 열 경계 뒤에 있고, 조상 칸 clip에 다 가려진다.
    let (run, cells) = find_set_run(&tree, &mut Vec::new()).expect("셋 런");
    let (run_x, run_y, _, run_h) = bbox(&run);
    assert!(run_x > inner_x + column_px, "열 경계가 선언 폭에 있다");
    assert!(
        run_x > visible_right,
        "셋은 바깥 표 오른쪽 테두리 밖에 있다"
    );
    let visible = cells
        .iter()
        .fold((f64::MIN, f64::MAX), |(l, r), (x, _, w, _)| {
            (l.max(*x), r.min(x + w))
        });
    assert!(
        visible.1 - visible.0 < 0.5,
        "셋 칸은 다 가려진다: {visible:?}"
    );
    assert!(visible.1 <= visible_right + 0.5);

    // 잘린 칸은 클릭으로 고를 수 없다. 그 줄의 오른쪽을 눌러도 보이는 왼쪽 칸에 간다.
    let y = run_y + run_h / 2.0;
    for x in [run_x + 2.0, visible_right + 8.0] {
        let got = hit(&doc, x, y);
        assert_eq!(
            got["cellPath"],
            innermost_path(0),
            "({x},{y})는 보이는 왼쪽 칸: {got}"
        );
    }
    // 보이는 칸은 그대로 누른다.
    assert_eq!(hit(&doc, inner_x + 20.0, y)["cellPath"], innermost_path(0));

    // Tab으로 들어간 잘린 칸도 캐럿 자리는 준다. 그 자리를 눌러도 잘린 칸은 잡히지 않는다.
    let rect: Value = serde_json::from_str(
        &doc.get_cursor_rect_by_path(0, 1, &innermost_path(1).to_string(), 0)
            .expect("잘린 칸 캐럿"),
    )
    .unwrap();
    let caret_x = rect["x"].as_f64().unwrap();
    assert!(caret_x > visible_right, "캐럿은 가려진 자리에 있다: {rect}");
    let got = hit(&doc, caret_x, rect["y"].as_f64().unwrap() + 2.0);
    assert_ne!(
        got["cellPath"],
        innermost_path(1),
        "잘린 칸은 클릭되지 않는다: {got}"
    );
}
