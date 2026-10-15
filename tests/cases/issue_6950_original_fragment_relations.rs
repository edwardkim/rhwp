//! #6950의 절대 좌표 검사를 동일 #6025 원본과 독립 Print의 관계 검사로 옮긴다.
//! Print 전4쪽은 앞3쪽의 표 조각을 본문 아래까지 열고 마지막은 내용 끝에서 닫는다.
//! 원본 일반 행의 최소 높이와 모든 셀 내용은 조각 사이에서 보존되어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{model::control::Control, model::table::Table, parse_document};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

const INPUT: &str = "samples/issue6025/3232693_employment_support_criteria.hwpx";
const PRINT_PAGES: usize = 4;
// 렌더 트리 JSON의 bbox 네 값은 각각 소수 첫 자리에서 반올림된다.
// 두 끝점의 차이는 네 반올림 오차를 포함한다. 문서의 절대 위치 공차가 아니다.
const JSON_QUANTUM: f64 = 0.1;
const ENDPOINT_ROUNDOFF: f64 = 2.0 * JSON_QUANTUM;
const SOURCE_UNIT: f64 = 96.0 / 7200.0;

struct OutputDir(PathBuf);
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source_table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let source = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT))
            .expect("수정하지 않은 원본 입력");
        let document = parse_document(&source).expect("원본 저장 정보 파싱");
        let Control::Table(table) = &document.sections[0].paragraphs[1].controls[0] else {
            panic!("원본 호스트의 표")
        };
        table.as_ref().clone()
    })
}

fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("시각")
            .as_nanos();
        let out = OutputDir(
            std::env::temp_dir().join(format!("rhwp-6950-original-{}-{nonce}", std::process::id())),
        );
        fs::create_dir(&out.0).expect("검사 전용 출력 경로");
        let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("export-render-tree")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT))
            .arg("-o")
            .arg(&out.0)
            .args(["--compat", "2022"])
            .output()
            .expect("실제 Native 출력");
        assert!(
            result.status.success(),
            "원본 출력 실패: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let count = fs::read_dir(&out.0)
            .expect("출력 목록")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with("render_tree_") && name.ends_with(".json"))
            })
            .count();
        assert_eq!(count, PRINT_PAGES, "같은 원본의 독립 Print 전체 쪽수");
        (1..=PRINT_PAGES)
            .map(|page| {
                serde_json::from_slice(
                    &fs::read(out.0.join(format!("render_tree_{page:03}.json")))
                        .expect("모든 독립 Print 쪽의 실제 출력"),
                )
                .expect("실제 트리 JSON")
            })
            .collect()
    })
}

fn find<'a>(node: &'a Value, kind: &str) -> Option<&'a Value> {
    if node["type"] == kind {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|c| find(c, kind))
}
fn collect<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        out.push(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect(child, kind, out);
        }
    }
}
fn body_items<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    if node["type"] == "Table" || node["type"] == "TextLine" {
        out.push(node);
        return;
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            body_items(child, out);
        }
    }
}
fn geom(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().expect("실제 출력 상자")
}
fn bottom(node: &Value) -> f64 {
    geom(node, "y") + geom(node, "h")
}
fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

#[test]
fn original_partial_boxes_close_the_body_and_the_final_box_stops_at_content() {
    let margin = f64::from(source_table().outer_margin_bottom) * SOURCE_UNIT;
    for (index, page) in pages().iter().enumerate() {
        let body = find(page, "Body").expect("실제 본문 상자");
        let table = find(body, "Table").expect("각 쪽의 동일 원본 표 조각");
        let mut cells = Vec::new();
        collect(table, "Cell", &mut cells);
        let last = cells.last().expect("현재 조각의 마지막 실제 셀");
        for cell in &cells {
            assert!(
                geom(cell, "y") + ENDPOINT_ROUNDOFF >= geom(body, "y")
                    && bottom(cell) <= bottom(body) + ENDPOINT_ROUNDOFF,
                "{}쪽의 셀은 본문 안에서 실제 물리 상자를 소유한다",
                index + 1
            );
            let mut lines = Vec::new();
            collect(cell, "TextLine", &mut lines);
            assert!(
                lines.iter().all(|line| {
                    geom(line, "y") + ENDPOINT_ROUNDOFF >= geom(cell, "y")
                        && bottom(line) <= bottom(cell) + ENDPOINT_ROUNDOFF
                }),
                "현재 컷의 모든 글줄은 자기 셀 안에 표시되어야 한다"
            );
        }
        let gap = bottom(body) - bottom(last);
        if index + 1 < PRINT_PAGES {
            assert!(
                (gap - margin).abs() <= ENDPOINT_ROUNDOFF,
                "{}쪽의 미완료 조각은 Print처럼 원본 바깥 아래 여백을 남기고 본문을 닫는다: gap={gap}, margin={margin}",
                index + 1
            );
        } else {
            assert!(
                gap > margin + ENDPOINT_ROUNDOFF,
                "내용을 끝낸 마지막 조각을 본문 아래까지 빈 상자로 늘리지 않는다"
            );
        }
        if index == 0 {
            let mut items = Vec::new();
            body_items(body, &mut items);
            let title = items
                .iter()
                .find(|node| node["type"] == "TextLine" && node["pi"].as_u64() == Some(1))
                .expect("원본 호스트의 표 앞 제목");
            assert!(
                bottom(title) <= geom(table, "y") + ENDPOINT_ROUNDOFF,
                "Print처럼 제목과 표를 앞뒤 순서로 배치하며 겹치지 않는다"
            );
        }
    }
}

#[test]
fn split_ordinary_row_boxes_preserve_the_original_cell_minimum() {
    let source = source_table()
        .cells
        .iter()
        .find(|cell| cell.row == 7 && cell.col == 0)
        .expect("원본 분할 일반 행");
    let minimum = f64::from(source.height) * SOURCE_UNIT;
    let mut parts = Vec::new();
    for page in pages() {
        let mut cells = Vec::new();
        collect(page, "Cell", &mut cells);
        parts.extend(
            cells
                .into_iter()
                .filter(|cell| cell["row"].as_u64() == Some(7) && cell["col"].as_u64() == Some(0)),
        );
    }
    assert_eq!(
        parts.len(),
        2,
        "독립 Print는 같은 행을 두 물리 조각으로 나눈다"
    );
    let physical: f64 = parts.iter().map(|part| geom(part, "h")).sum();
    let rounding = parts.len() as f64 * JSON_QUANTUM / 2.0;
    assert!(
        physical + rounding >= minimum,
        "앞·뒤 실제 상자는 원본 행 최소 높이를 보존한다: physical={physical}, minimum={minimum}"
    );
}

#[test]
fn every_original_cell_keeps_its_content_once_in_page_order() {
    let expected: BTreeMap<_, _> = source_table()
        .cells
        .iter()
        .map(|cell| {
            let text: String = cell
                .paragraphs
                .iter()
                .map(|paragraph| paragraph.text.as_str())
                .collect();
            (
                (u64::from(cell.row), u64::from(cell.col)),
                normalized(&text),
            )
        })
        .collect();
    let mut actual: BTreeMap<(u64, u64), String> = BTreeMap::new();
    for page in pages() {
        let mut cells = Vec::new();
        collect(page, "Cell", &mut cells);
        for cell in cells {
            let key = (
                cell["row"].as_u64().expect("원본 행"),
                cell["col"].as_u64().expect("원본 열"),
            );
            let mut runs = Vec::new();
            collect(cell, "TextRun", &mut runs);
            let text: String = runs.iter().filter_map(|run| run["text"].as_str()).collect();
            actual.entry(key).or_default().push_str(&normalized(&text));
        }
    }
    assert_eq!(
        actual, expected,
        "원본 모든 셀의 글자는 앞·뒤 조각에서 순서대로 한 번만 소유한다"
    );
}
