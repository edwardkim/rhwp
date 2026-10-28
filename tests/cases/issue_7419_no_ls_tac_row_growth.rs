//! #7419: 저장 줄 없는 TAC 표의 셀·인라인 줄·이월 소유를 보호한다.
//!
//! 독립 한컴 Print와 Native/fresh WASM 13문서·19쪽 모두100%, 직접 확대 판독:
//! mydocs/working/task_m100_7419_stage3.md 및 stage4.md. 원본2개 검사의 고정 영역 선택을
//! 실제 셀 소유·내용 보존·포함·순서 관계로 교체한다. 완료 쪽은 별도로±2px도 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

struct Outputs(PathBuf);
impl Drop for Outputs {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(if name.starts_with("tac_") {
            "samples/issue7419"
        } else {
            "tests/fixtures/issue7419-boundaries"
        })
        .join(format!("{name}.hwpx"))
}

fn export(name: &str, command: &str, extension: &str) -> Vec<String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let out =
        Outputs(std::env::temp_dir().join(format!("rhwp-7419-{}-{stamp}", std::process::id())));
    fs::create_dir(&out.0).unwrap();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(bin)
        .arg(command)
        .arg(source_path(name))
        .args(["--compat", "2024", "-o"])
        .arg(&out.0)
        .output()
        .expect("최종 출력 생산자");
    assert!(
        result.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut files: Vec<_> = fs::read_dir(&out.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == extension))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "최종 출력 누락: {name}");
    files
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect()
}

fn nodes(node: &Value) -> Vec<&Value> {
    let mut result = vec![node];
    for child in node["children"].as_array().into_iter().flatten() {
        result.extend(nodes(child));
    }
    result
}

fn own_nodes(node: &Value) -> Vec<&Value> {
    if node["type"] == "Table" {
        return Vec::new();
    }
    let mut result = vec![node];
    for child in node["children"].as_array().into_iter().flatten() {
        result.extend(own_nodes(child));
    }
    result
}

fn value(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().unwrap()
}
fn bottom(node: &Value) -> f64 {
    value(node, "y") + value(node, "h")
}
fn right(node: &Value) -> f64 {
    value(node, "x") + value(node, "w")
}
fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{fffc}')
        .collect()
}
fn text(nodes: &[&Value]) -> String {
    compact(
        &nodes
            .iter()
            .filter_map(|n| n["text"].as_str())
            .collect::<String>(),
    )
}
fn cell(table: &Value, row: u16) -> &Value {
    table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Cell" && node["row"] == row && node["col"] == 0)
        .expect("원본 행의 실제 셀")
}
fn source_tables<'a>(paragraphs: &'a [Paragraph], tables: &mut Vec<&'a Table>) {
    for paragraph in paragraphs {
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                tables.push(table);
                for cell in &table.cells {
                    source_tables(&cell.paragraphs, tables);
                }
            }
        }
    }
}

fn verified_pages(name: &str, count: usize) -> Vec<Value> {
    let pages: Vec<Value> = export(name, "export-render-tree", "json")
        .iter()
        .map(|json| serde_json::from_str(json).unwrap())
        .collect();
    assert_eq!(pages.len(), count, "독립 Print의 전체 쪽수: {name}");
    let document = rhwp::parser::parse_document(&fs::read(source_path(name)).unwrap()).unwrap();
    let mut expected = Vec::new();
    for section in &document.sections {
        source_tables(&section.paragraphs, &mut expected);
    }
    let actual: Vec<_> = pages
        .iter()
        .flat_map(nodes)
        .filter(|n| n["type"] == "Table")
        .collect();
    assert_eq!(actual.len(), expected.len(), "표 누락·중복: {name}");
    for (table, source) in actual.into_iter().zip(expected) {
        assert_eq!(table["rows"], source.row_count);
        assert_eq!(table["cols"], source.col_count);
        let cells: Vec<_> = table["children"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["type"] == "Cell")
            .collect();
        assert_eq!(cells.len(), source.cells.len(), "행 소유·누락·중복");
        for original in &source.cells {
            let actual = cells
                .iter()
                .find(|n| n["row"] == original.row && n["col"] == original.col)
                .expect("원본 셀 주소의 보존");
            let own = own_nodes(actual);
            let expected_text = compact(
                &original
                    .paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<String>(),
            );
            assert_eq!(
                text(&own),
                expected_text,
                "{name}: 셀 ({},{})의 내용·순서·누락·중복",
                original.row,
                original.col
            );
            for line in own.iter().filter(|n| n["type"] == "TextLine") {
                assert!(
                    value(line, "y") >= value(actual, "y") - 0.2
                        && bottom(line) <= bottom(actual) + 0.2,
                    "{name}: 글줄이 소유 셀 안에 있어야 한다: 줄 {:?}, 셀 {:?}",
                    line["bbox"],
                    actual["bbox"]
                );
            }
            for child in nodes(actual).into_iter().filter(|n| n["type"] == "Table") {
                assert!(
                    value(child, "y") >= value(actual, "y") - 0.2
                        && bottom(child) <= bottom(actual) + 0.2,
                    "{name}: 중첩 표의 실제 점유가 부모 셀 안에 있어야 한다"
                );
            }
        }
        assert!(
            bottom(cell(table, 0)) <= value(cell(table, 1), "y") + 0.2,
            "앞뒤 행 겹침"
        );
    }
    for page in &pages {
        let all = nodes(page);
        let body = all.iter().find(|n| n["type"] == "Body").unwrap();
        let top_level: Vec<_> = all
            .iter()
            .filter(|n| n["type"] == "Table" && n["pi"].as_u64().is_some_and(|pi| pi >= 3))
            .collect();
        for pair in top_level.windows(2) {
            assert!(
                bottom(pair[0]) <= value(pair[1], "y") + 0.2,
                "앞뒤 표 흐름의 비겹침"
            );
        }
        for table in all.iter().filter(|n| n["type"] == "Table") {
            assert!(
                value(table, "y") >= value(body, "y") - 0.2 && bottom(table) <= bottom(body) + 0.2,
                "{name}: 마지막 행까지 본문 예산 안에 포함"
            );
        }
    }
    pages
}

fn nested(name: &str) -> Vec<Value> {
    let pages = verified_pages(name, 1);
    let tables: Vec<_> = nodes(&pages[0])
        .into_iter()
        .filter(|n| n["type"] == "Table")
        .collect();
    let host = cell(tables[1], 1);
    let children: Vec<Value> = nodes(host)
        .into_iter()
        .filter(|n| n["type"] == "Table")
        .cloned()
        .collect();
    // 독립 Print는 마지막 개체 아래에 원본 셀의 하단 여백만 남긴다.
    // 같은 줄의 두 높이를 합쳐 빈 밴드를 예약하는 결함도 검출한다.
    let document = rhwp::parser::parse_document(&fs::read(source_path(name)).unwrap()).unwrap();
    let mut source = Vec::new();
    for section in &document.sections {
        source_tables(&section.paragraphs, &mut source);
    }
    let owner = source[1].cells.iter().find(|cell| cell.row == 1).unwrap();
    let padding = f64::from(owner.padding.bottom) * 96.0 / 7200.0;
    let occupied_bottom = children
        .iter()
        .map(bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (bottom(host) - occupied_bottom - padding).abs() <= 0.2,
        "{name}: 부모가 소비한 공간은 실제 마지막 개체와 원본 하단 여백까지여야 한다"
    );
    children
}

#[test]
fn issue_7419_two_line_cell_grows_row_and_next_table_follows() {
    let pages = verified_pages("tac_two_line_cell_no_lineseg", 1);
    let table = nodes(&pages[0])
        .into_iter()
        .find(|n| n["type"] == "Table")
        .unwrap();
    assert_eq!(
        own_nodes(cell(table, 1))
            .iter()
            .filter(|n| n["type"] == "TextLine")
            .count(),
        2,
        "Print와 같은 두 줄을 실제 셀 내부에 보존"
    );
}

#[test]
fn issue_7419_nested_tac_table_growth_expands_host_cell() {
    assert_eq!(nested("tac_nested_two_line_cell_no_lineseg").len(), 1);
}

#[test]
fn declaration_difference_does_not_change_reflowed_cell_height() {
    let original = verified_pages("tac_two_line_cell_no_lineseg", 1);
    let changed = verified_pages("common-height-4838", 1);
    let first = |pages: &[Value]| {
        nodes(&pages[0])
            .into_iter()
            .find(|n| n["type"] == "Table")
            .unwrap()
            .clone()
    };
    assert!(
        (value(&first(&original), "h") - value(&first(&changed), "h")).abs() <= 0.2,
        "독립 Print에서 같은 내용은 선언4838HU 차이로 줄어들지 않는다"
    );
}

#[test]
fn same_line_tables_share_one_occupied_band() {
    let children = nested("nested-same-line");
    assert_eq!(children.len(), 2);
    assert!((value(&children[0], "y") - value(&children[1], "y")).abs() <= 0.2);
    assert!(right(&children[0]) <= value(&children[1], "x") + 0.2);
}

#[test]
fn explicit_break_keeps_second_table_on_next_line() {
    let children = nested("nested-explicit-break");
    assert_eq!(children.len(), 2);
    assert!(
        bottom(&children[0]) <= value(&children[1], "y") + 0.2,
        "명시적 개행 뒤 표는 앞 표의 점유 다음 줄을 소유한다"
    );
}

#[test]
fn insufficient_width_wraps_second_table_instead_of_expanding_host() {
    let children = nested("nested-width-wrap");
    assert_eq!(children.len(), 2);
    assert!(
        bottom(&children[0]) <= value(&children[1], "y") + 0.2,
        "두 번째 슬롯도 폭 계산에 참여해야 한다"
    );
}

#[test]
fn surrounding_text_stays_before_and_after_the_inline_tables() {
    let pages = verified_pages("nested-surrounding-text", 1);
    let tables: Vec<_> = nodes(&pages[0])
        .into_iter()
        .filter(|n| n["type"] == "Table")
        .collect();
    let host = cell(tables[1], 1);
    let runs = own_nodes(host);
    let before = runs
        .iter()
        .find(|n| n["text"].as_str().is_some_and(|s| s.contains("앞 본문")))
        .unwrap();
    let after = runs
        .iter()
        .find(|n| n["text"].as_str().is_some_and(|s| s.contains("뒤 본문")))
        .unwrap();
    assert!(right(before) <= value(tables[2], "x") + 0.2);
    assert!(right(tables[3]) <= value(after, "x") + 0.2);
}

#[test]
fn outer_margins_are_consumed_once_around_the_actual_nested_body() {
    let pages = verified_pages("nested-outer-margins", 1);
    let tables: Vec<_> = nodes(&pages[0])
        .into_iter()
        .filter(|n| n["type"] == "Table")
        .collect();
    let host = cell(tables[1], 1);
    // 입력의 상300/하600HU를96dpi로 환산한 원본 계약. 위치 핀의 대용이 아니다.
    assert!(value(tables[2], "y") - value(host, "y") >= 300.0 * 96.0 / 7200.0);
    assert!(bottom(host) - bottom(tables[2]) >= 600.0 * 96.0 / 7200.0);
}

#[test]
fn small_page_moves_complete_tables_and_preserves_following_content() {
    let pages = verified_pages("page-height-28000", 3);
    let labels: Vec<_> = pages.iter().map(|page| text(&nodes(page))).collect();
    assert_eq!(labels[0], "셀높이재현");
    assert!(labels[1].starts_with("첫째표") && !labels[1].contains("둘째표"));
    assert!(labels[2].starts_with("둘째표이표의제목줄이"));
    for (page, count) in pages.iter().zip([0, 1, 1]) {
        assert_eq!(
            nodes(page).iter().filter(|n| n["type"] == "Table").count(),
            count
        );
    }
}

// 이 입력은 문단 안 텍스트가 없는 개체 호스트다. 원본의 문단·개체 순서를 따라
// 전체 내용을 읽어, 조각 경계에서 생기는 누락·중복도 검사한다.
fn source_text(paragraphs: &[Paragraph]) -> String {
    let mut result = String::new();
    for paragraph in paragraphs {
        result.push_str(&paragraph.text);
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    result.push_str(&source_text(&cell.paragraphs));
                }
            }
        }
    }
    compact(&result)
}

fn verify_split_nested(name: &str, child_pages: [usize; 2]) {
    let pages: Vec<Value> = export(name, "export-render-tree", "json")
        .iter()
        .map(|json| serde_json::from_str(json).unwrap())
        .collect();
    assert_eq!(pages.len(), 2, "독립 Print의 전체 쪽수");
    let document = rhwp::parser::parse_document(&fs::read(source_path(name)).unwrap()).unwrap();
    let paragraphs = &document.sections[0].paragraphs;
    assert_eq!(
        text(&pages.iter().flat_map(nodes).collect::<Vec<_>>()),
        source_text(paragraphs),
        "전체 문단·표 내용의 순서·누락·중복"
    );
    let mut expected = Vec::new();
    source_tables(paragraphs, &mut expected);
    let expected = &expected[expected.len() - 2..];
    let mut actual = Vec::new();
    for (page_index, page) in pages.iter().enumerate() {
        let all = nodes(page);
        let body = all.iter().find(|n| n["type"] == "Body").unwrap();
        for table in all.iter().filter(|n| n["type"] == "Table") {
            assert!(
                value(table, "y") >= value(body, "y") - 0.2 && bottom(table) <= bottom(body) + 0.2,
                "완료 조각의 외곽까지 본문 안에 포함"
            );
            let cells: Vec<_> = table["children"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["type"] == "Cell")
                .collect();
            for owner in &cells {
                for child in nodes(owner)
                    .iter()
                    .filter(|n| n["type"] == "Table" || n["type"] == "TextLine")
                {
                    assert!(
                        value(child, "y") >= value(owner, "y") - 0.2
                            && bottom(child) <= bottom(owner) + 0.2,
                        "조각의 줄·중첩 표는 소유 셀 안에 포함"
                    );
                }
            }
            if cells.len() == 2
                && nodes(table).iter().filter(|n| n["type"] == "Table").count() == 1
                && table["pi"] == 0
            {
                actual.push((page_index, *table));
            }
        }
    }
    assert_eq!(actual.len(), 2, "두 자식 표의 누락·중복");
    for (index, ((page, table), original)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*page, child_pages[index], "독립 Print의 자식 표 쪽 소유");
        assert_eq!(table["ci"], index, "원시 제어문 슬롯의 소유 보존");
        for source in &original.cells {
            let target = cell(table, source.row);
            assert_eq!(
                text(&own_nodes(target)),
                compact(
                    &source
                        .paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<String>()
                ),
                "각 자식 셀의 내용 소유"
            );
        }
        assert!(bottom(cell(table, 0)) <= value(cell(table, 1), "y") + 0.2);
    }
    let last_nodes = nodes(&pages[1]);
    let last_frame = last_nodes
        .iter()
        .find(|n| n["type"] == "Table" && n["pi"] == 4)
        .unwrap();
    let following = last_nodes
        .iter()
        .find(|n| n["type"] == "TextRun" && n["text"] == "후속 본문 보존")
        .unwrap();
    assert!(
        value(following, "y") >= bottom(last_frame) - 0.2,
        "표 종료 이후 본문은 마지막 조각과 겹치지 않는다"
    );
    assert!(!text(&nodes(&pages[0])).contains("후속본문보존"));
}

#[test]
fn partial_cell_moves_each_reflowed_inline_line_without_loss() {
    verify_split_nested("partial-nested-width-wrap", [0, 1]);
}

#[test]
fn one_inline_wrapper_keeps_both_children_together_on_the_next_page() {
    verify_split_nested("partial-nested-wrapper", [1, 1]);
}

#[test]
fn mixed_saved_and_reflowed_inline_children_keep_their_page_owners() {
    // 실제 한컴 저장본에서 부모와 첫 자식의 LineSeg만 제거했다.
    // 저장 줄이 있는 둘째 자식도 같은 호스트 프레임의 소유자여야 한다.
    verify_split_nested("mixed-child-storage", [0, 1]);
}

#[test]
fn reflowing_both_inline_children_preserves_the_mixed_storage_control() {
    // 혼재 입력과 기하·내용이 같고 두 자식의 저장 줄만 모두 제거한 대조군이다.
    verify_split_nested("no-child-storage", [0, 1]);
}

#[test]
fn completed_pages_keep_independent_print_title_origins_with_two_px_tolerance() {
    // 독립 Print PDF의 span.origin을72pt→96dpi로 환산했다. 구현 상자는 재인용하지 않는다.
    // Stage1의 입력·PDF SHA, 양쪽100% TSV와 직접 판독이 선행 조건이다.
    // 표 제목은 줄 시작·이어지는 표·중첩 소유·개행/폭 부족의 중요 경계를 보호한다.
    type Pins<'a> = (&'a str, &'a [(&'a str, f64, f64)]);
    let cases: &[Pins<'_>] = &[
        (
            "common-height-4838",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
            ],
        ),
        (
            "nested-explicit-break",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
                ("첫", 120.894480, 326.842814),
                ("첫", 134.327199, 490.503947),
            ],
        ),
        (
            "nested-outer-margins",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
                ("첫", 123.613007, 330.838440),
            ],
        ),
        (
            "nested-same-line",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
                ("첫", 120.894480, 326.842814),
                ("첫", 327.502502, 326.842814),
            ],
        ),
        (
            "nested-surrounding-text",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
                ("첫", 173.186137, 326.842814),
                ("첫", 379.634237, 326.842814),
            ],
        ),
        (
            "nested-width-wrap",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
                ("첫", 120.894480, 326.842814),
                ("첫", 134.327199, 469.087362),
            ],
        ),
        (
            "tac_nested_two_line_cell_no_lineseg",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
                ("첫", 120.894480, 326.842814),
            ],
        ),
        (
            "tac_two_line_cell_no_lineseg",
            &[
                ("첫", 117.216471, 214.645426),
                ("둘", 117.216471, 292.959839),
            ],
        ),
    ];
    for (name, expected) in cases {
        let outputs = export(name, "export-svg", "svg");
        assert_eq!(outputs.len(), 1);
        assert_title_origins(&outputs[0], expected, name);
    }
    let small = export("page-height-28000", "export-svg", "svg");
    assert_eq!(small.len(), 3);
    assert_title_origins(&small[1], &[("첫", 117.216471, 152.993144)], "작은 용지2쪽");
    assert_title_origins(&small[2], &[("둘", 117.216471, 152.993144)], "작은 용지3쪽");
    for (name, second_page) in [
        (
            "partial-nested-width-wrap",
            vec![
                ("첫", 134.327199, 154.882578),
                ("후", 113.378550, 281.924703),
            ],
        ),
        (
            "partial-nested-wrapper",
            vec![
                ("첫", 124.732401, 156.802612),
                ("첫", 138.005208, 299.204997),
                ("후", 113.378550, 428.167155),
            ],
        ),
        (
            "mixed-child-storage",
            vec![
                ("첫", 134.327199, 154.882578),
                ("후", 113.378550, 281.924703),
            ],
        ),
        (
            "no-child-storage",
            vec![
                ("첫", 134.327199, 154.882578),
                ("후", 113.378550, 281.924703),
            ],
        ),
    ] {
        let outputs = export(name, "export-svg", "svg");
        assert_eq!(outputs.len(), 2);
        let mut first_page = vec![
            ("첫", 117.216471, 214.883586),
            ("둘", 117.216471, 293.284892),
        ];
        if matches!(
            name,
            "partial-nested-width-wrap" | "mixed-child-storage" | "no-child-storage"
        ) {
            first_page.push(("첫", 120.894480, 327.205465));
        }
        assert_title_origins(&outputs[0], &first_page, name);
        assert_title_origins(&outputs[1], &second_page, name);
    }
}

fn assert_title_origins(svg: &str, expected: &[(&str, f64, f64)], name: &str) {
    let xml = roxmltree::Document::parse(svg).unwrap();
    let mut ordinals = std::collections::BTreeMap::new();
    for &(marker, expected_x, expected_y) in expected {
        let next = ordinals.entry(marker).or_insert(0);
        let ordinal = *next;
        *next += 1;
        let glyph = xml
            .descendants()
            .filter(|n| n.has_tag_name("text") && n.text().is_some_and(|s| s.starts_with(marker)))
            .nth(ordinal)
            .expect("독립 Print의 표 제목 글리프");
        let x: f64 = glyph.attribute("x").unwrap().parse().unwrap();
        let y: f64 = glyph.attribute("y").unwrap().parse().unwrap();
        assert!((x - expected_x).abs() <= 2.0 && (y - expected_y).abs() <= 2.0,
            "{name}: 제목 {marker}#{ordinal} ({x},{y}), 독립 Print ({expected_x},{expected_y}) ±2px");
    }
}
