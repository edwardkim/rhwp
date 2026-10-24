//! #8152 sample2: 저장 프레임의 내용 소유·행 잔여·후속 개체 흐름.
//! samples/hwpx_sample2.hwp와 독립 한컴2024 Print 전29쪽 Native/fresh WASM90%,
//! p4/p13 각각98%·직접 판독 뒤 등록한다. 증적은
//! mydocs/pr/assets/issue8152/sample2-source-frame-20261025/validation.json에 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command, sync::OnceLock};
const INPUT: &str = "samples/hwpx_sample2.hwp";

fn source() -> rhwp::model::document::Document {
    rhwp::parse_document(&fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(INPUT)).unwrap())
        .unwrap()
}
fn source_table(doc: &rhwp::model::document::Document, pi: usize) -> &rhwp::model::table::Table {
    doc.sections[0].paragraphs[pi]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .unwrap()
}
fn pages() -> &'static [Value] {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out = root.join("output").join(format!(
            "sample2-source-frame-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&out).unwrap();
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
        // 저장 줄·프레임의 Native 조판 계약은 private 글꼴 설치를 요구하지 않는다.
        let run = Command::new(bin)
            .arg("export-render-tree")
            .arg(root.join(INPUT))
            .args(["--compat", "2024", "-o"])
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let mut files: Vec<_> = fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("render_tree_")
            })
            .collect();
        files.sort();
        let result: Vec<Value> = files
            .iter()
            .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap())
            .collect();
        fs::remove_dir_all(out).unwrap();
        assert_eq!(
            result.len(),
            29,
            "독립 Print의 쪽수와 내용 소속을 함께 보존한다"
        );
        result
    })
}
fn nodes(n: &Value) -> Vec<&Value> {
    let mut out = vec![n];
    if let Some(cs) = n["children"].as_array() {
        for c in cs {
            out.extend(nodes(c));
        }
    }
    out
}
fn table(page: usize, pi: u64) -> &'static Value {
    nodes(&pages()[page - 1])
        .into_iter()
        .find(|n| n["type"] == "Table" && n["pi"] == pi)
        .unwrap()
}
fn cell(table: &Value, row: u64, col: u64) -> &Value {
    table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Cell" && n["row"] == row && n["col"] == col)
        .unwrap()
}
fn text(n: &Value) -> String {
    nodes(n)
        .into_iter()
        .filter(|n| n["type"] == "TextRun")
        .map(|n| n["text"].as_str().unwrap())
        .collect()
}
fn normalized(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn y(n: &Value) -> f64 {
    n["bbox"]["y"].as_f64().unwrap()
}
fn h(n: &Value) -> f64 {
    n["bbox"]["h"].as_f64().unwrap()
}
fn bottom(n: &Value) -> f64 {
    y(n) + h(n)
}

#[test]
fn split_row_preserves_its_declared_space_and_the_following_objects() {
    let doc = source();
    let src = source_table(&doc, 127);
    let opening = table(13, 127);
    let continuation = table(14, 127);
    let right = src.cells.iter().find(|c| c.row == 9 && c.col == 1).unwrap();
    assert_eq!(right.row_span, 1);
    let total = h(cell(opening, 9, 1)) + h(cell(continuation, 9, 1));
    // CLI bbox는 소수 첫째 자리까지 표시하므로 두 조각 합의 양자화 오차만 허용한다.
    assert!(
        (total - f64::from(right.height) / 75.0).abs() <= 0.11,
        "소비한 내용으로 원본 물리 잔여를 대체하면 안 된다: {total}"
    );
    for col in [0_u64, 1] {
        let original = src
            .cells
            .iter()
            .find(|c| c.row == 9 && u64::from(c.col) == col)
            .unwrap();
        let expected: String = original
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        let actual = text(cell(opening, 9, col)) + &text(cell(continuation, 9, col));
        assert_eq!(
            normalized(&actual),
            normalized(&expected),
            "앞/뒤 조각은 같은 원문을 한 번씩 보존해야 한다: 열{col}"
        );
    }
    let family = table(14, 128);
    let child = table(14, 129);
    assert!(text(family).contains("신청자를 제외한"));
    assert!(text(child).contains("이혼 재혼의 경우"));
    assert!(
        y(family) >= bottom(continuation),
        "이전 표의 실제 상자를 흐름 예약에서도 소비해야 한다"
    );
    assert!(
        y(child) >= bottom(family),
        "뒤따르는 표 순서와 비겹침을 보존해야 한다"
    );
}

#[test]
fn accepted_opening_frame_keeps_its_print_alignment() {
    let c = cell(table(13, 127), 9, 1);
    let line = nodes(c)
        .into_iter()
        .find(|n| n["type"] == "TextLine" && n["pi"] == 0)
        .unwrap();
    assert!(text(line).contains("공공주택 특별법 시행규칙"));
    // 독립 pdf/hwpx_sample2-hwp-2024.pdf p13의 첫 줄 top, 96 CSS px 환산.
    let print_top = 878.66928;
    assert!(
        (y(line) - print_top).abs() <= 2.0,
        "인증한 첫 프레임의 Center 정렬이 사라졌다: {} vs {print_top}",
        y(line)
    );
}

#[test]
fn complete_object_tail_keeps_its_print_alignment() {
    let c = cell(table(4, 32), 1, 1);
    let line = c["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "TextLine" && text(n).contains("단 아래에 해당하는 사람은"))
        .unwrap();
    // 같은 독립 Print p4의 첫 비어 있지 않은 줄 top, 96 CSS px 환산.
    let print_top = 72.6576;
    assert!(
        (y(line) - print_top).abs() <= 2.0,
        "완결된 개체 꼬리를 Top으로 바꾸면 안 된다: {} vs {print_top}",
        y(line)
    );
    assert!(y(line) >= y(c) && bottom(line) <= bottom(c));
}
