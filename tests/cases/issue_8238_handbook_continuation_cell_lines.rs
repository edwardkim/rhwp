//! [#8238] 원본 글꼴이 한 자 이내 폭 오차로 재현하는 저장 줄을 다시 채우지 않는다.
//!
//! 편람 HWP 154쪽은 구역 4 문단 47의 11×2 표가 행 4..8로 이어진다. 같은 표의
//! 셀 문단 몇 개는 대체 글꼴(HCR Batang)이 저장 줄을 재현한다는 이유로 원본
//! KoPub 글꼴로 다시 채워져 줄 끝 글자 하나가 앞줄로 올라갔다. 그 결과 표 전체가
//! 재조판 표로 바뀌어 이어받기 셀의 첫 줄 다음 전진이 5.4px 줄고 뒤 행 괘선이
//! 위로 당겨졌다. 한컴 Print는 원본 KoPub로도 저장 줄과 같은 줄 구성이다.
//!
//! 같은 원본/KoPub 공급 한컴 2024 Print 전383쪽을 먼저 비교했다. 154쪽은
//! Native/fresh WASM 2px 실루엣 raw 98% 이상이며, px 기대값은 PDF 글줄 상단과
//! 괘선(384DPI 래스터의 가로선 중심)을 96CSSpx로 환산한 값이다.
//! 기준 PDF: pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{fs, path::PathBuf, process::Command, sync::OnceLock};

fn export(page_index: usize) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("output").join(format!(
        "issue8238-handbook-p{}-{}-{}",
        page_index + 1,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&output).unwrap();
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    let exported = Command::new(&binary)
        .arg("export-render-tree")
        .arg(root.join("samples/2025 행정업무운영 편람(최종).hwp"))
        .args(["--compat", "2024", "--page", &page_index.to_string(), "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}쪽 실제 출력: {}",
        page_index + 1,
        String::from_utf8_lossy(&exported.stderr)
    );
    let files: Vec<_> = fs::read_dir(&output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    assert_eq!(files.len(), 1, "요청한 한 쪽의 렌더 트리만 사용한다");
    let page = serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap();
    fs::remove_dir_all(output).unwrap();
    page
}

fn page() -> &'static Value {
    static PAGE: OnceLock<Value> = OnceLock::new();
    PAGE.get_or_init(|| export(153))
}

fn children(node: &Value) -> impl Iterator<Item = &Value> {
    node.get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn text(node: &Value) -> String {
    let own = if node["type"] == "TextRun" {
        node["text"].as_str().unwrap_or_default()
    } else {
        ""
    };
    let mut result: String = own.chars().filter(|ch| !ch.is_whitespace()).collect();
    for child in children(node) {
        result.push_str(&text(child));
    }
    result
}

fn find<'a>(node: &'a Value, matches: &dyn Fn(&Value) -> bool) -> Option<&'a Value> {
    if matches(node) {
        return Some(node);
    }
    children(node).find_map(|child| find(child, matches))
}

fn collect<'a>(node: &'a Value, kind: &str, result: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        result.push(node);
    }
    for child in children(node) {
        collect(child, kind, result);
    }
}

fn bounds(node: &Value) -> [f64; 4] {
    let b = &node["bbox"];
    let number = |name: &str| b[name].as_f64().unwrap();
    let (x, y) = (number("x"), number("y"));
    [x, y, x + number("w"), y + number("h")]
}

fn table() -> &'static Value {
    find(page(), &|node| {
        node["type"] == "Table" && node["pi"] == 47 && node["rows"] == 11 && node["cols"] == 2
    })
    .expect("154쪽의 11×2 표 이어받기 조각")
}

fn cell(row: u64, col: u64) -> &'static Value {
    find(table(), &|node| {
        node["type"] == "Cell" && node["row"] == row && node["col"] == col
    })
    .unwrap_or_else(|| panic!("셀 ({row},{col})"))
}

fn lines(node: &Value) -> Vec<&Value> {
    let mut result = Vec::new();
    collect(node, "TextLine", &mut result);
    result
}

fn first_line_starting(node: &Value, prefix: &str) -> [f64; 4] {
    bounds(
        lines(node)
            .into_iter()
            .find(|line| text(line).starts_with(prefix))
            .unwrap_or_else(|| panic!("{prefix} 글줄")),
    )
}

fn near(actual: f64, expected: f64, description: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{description}: 독립 Print {expected:.2}, 실제 {actual:.2}, 허용 ±2CSSpx"
    );
}

#[test]
fn continuation_cell_lines_keep_their_order_inside_each_row() {
    // 반복 머리행 뒤에 원본 행 4..7의 오른쪽 셀이 순서대로 이어진다.
    let rows = [0, 4, 5, 6, 7];
    let mut previous_bottom = f64::NEG_INFINITY;
    for row in rows {
        let frame = bounds(cell(row, 1));
        assert!(
            frame[1] >= previous_bottom - 0.5,
            "행 {row}이 앞 행과 겹친다: 위 {:.2}, 앞 행 아래 {previous_bottom:.2}",
            frame[1]
        );
        let mut previous_line = f64::NEG_INFINITY;
        for line in lines(cell(row, 1)) {
            let box_ = bounds(line);
            assert!(box_[1] > previous_line, "행 {row}의 글줄 순서가 뒤집혔다");
            assert!(
                box_[1] >= frame[1] - 0.5 && box_[3] <= frame[3] + 0.5,
                "행 {row}의 글줄이 셀 밖에 있다: {box_:?} / {frame:?}"
            );
            previous_line = box_[1];
        }
        previous_bottom = frame[3];
    }
    assert!(text(cell(4, 1)).starts_with("하.부수부분의보관의"));
    assert!(text(cell(7, 1)).starts_with("가.한칸의높이는"));
}

#[test]
fn continuation_cell_first_line_keeps_the_print_advance() {
    // 머리행 첫 줄 기준의 상대 전진만 비교해 글줄 상자와 PDF 글자 상단의
    // 일정한 차이를 상쇄한다. PDF 154쪽 글줄 상단 160.93 / 192.40 / 307.33 / 782.80.
    let header = first_line_starting(cell(0, 1), "설계기준")[1];
    near(
        first_line_starting(cell(4, 1), "하.부수부분")[1] - header,
        192.40 - 160.93,
        "이어받기 셀 첫 줄 전진",
    );
    near(
        first_line_starting(cell(5, 1), "가.쉽게인지할")[1] - header,
        307.33 - 160.93,
        "다음 행(5. 표) 첫 줄",
    );
    near(
        first_line_starting(cell(7, 1), "가.한칸의높이는")[1] - header,
        782.80 - 160.93,
        "넷째 행(7. 칸) 첫 줄",
    );
}

#[test]
fn continuation_rows_keep_the_print_rules() {
    // PDF 154쪽 행 경계 이중선의 아래 선(96CSSpx).
    near(bounds(cell(5, 1))[1], 295.62, "행 5 위 괘선");
    near(bounds(cell(6, 1))[1], 642.38, "행 6 위 괘선");
    near(bounds(cell(7, 1))[1], 775.88, "행 7 위 괘선");
    near(bounds(cell(7, 1))[3], 881.38, "조각 아래 괘선");
}

#[test]
fn body_rows_keep_the_stored_source_partition() {
    // 독립 Print 18쪽은 원본 KoPub 글꼴로도 저장 줄과 같은 글자 소속이다.
    // rhwp 측정에서 한 자가 더 들어간다는 이유로 낱말 중간을 앞줄로 올리지 않는다.
    static PAGE: OnceLock<Value> = OnceLock::new();
    let page = PAGE.get_or_init(|| export(17));
    let rows: Vec<String> = lines(page).into_iter().map(text).collect();
    let following = |prefix: &str| {
        let index = rows
            .iter()
            .position(|row| row.starts_with(prefix))
            .unwrap_or_else(|| panic!("{prefix} 글줄"));
        (
            rows[index].as_str(),
            rows.get(index + 1).map(String::as_str),
        )
    };
    let (row, next) = following("기관간원활한보고");
    assert!(row.ends_with("원칙,처리"), "18쪽 첫 줄 끝: {row}");
    assert!(next.is_some_and(|next| next.starts_with("기간,보고")));
    let (row, next) = following("행정기관내에표준화된");
    assert!(
        row.ends_with("정착됨에따라"),
        "18쪽 셋째 항목 첫 줄 끝: {row}"
    );
    assert!(next.is_some_and(|next| next.starts_with("시스템에대한")));
    let (row, next) = following("시스템에대한인증제가");
    assert!(
        row.ends_with("삭제하고"),
        "18쪽 셋째 항목 둘째 줄 끝: {row}"
    );
    assert!(next.is_some_and(|next| next.starts_with("표준에적합한")));
}
