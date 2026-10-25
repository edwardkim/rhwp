//! [#7765] 원본 글꼴의 현재 줄 구성과 중첩 표의 실제 물리 프레임을 보호한다.
//!
//! 같은 원본/공급 글꼴 한컴 2024 Print 전64쪽을 먼저 비교했다.
//! Native/fresh WASM 최저94.31413/94.07347%, 미달0쪽이며,
//! 중요한25~29쪽은 양쪽98% 이상이다. px 기대값은 PDF 괘선 좌표를
//! 96CSSpx로 환산한 값이며, 글자 모양은 별도 review/overlay로 판독한다.
//! 기준 PDF: pdf/issue7382-regression-review/86712_regulatory_analysis-font-supplied-2024.pdf
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{fs, path::PathBuf, process::Command, sync::OnceLock};

fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let output = root.join("output").join(format!(
            "issue7765-current-frames-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&output).unwrap();
        let binary = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
        let mut result = Vec::new();
        for page in 25..=29 {
            let folder = output.join(format!("p{page:03}"));
            let exported = Command::new(&binary)
                .arg("export-render-tree")
                .arg(root.join("samples/86712_regulatory_analysis.hwp"))
                .args(["--compat", "2024", "--page", &(page - 1).to_string(), "-o"])
                .arg(&folder)
                .output()
                .unwrap();
            assert!(
                exported.status.success(),
                "{page}쪽 실제 출력: {}",
                String::from_utf8_lossy(&exported.stderr)
            );
            let files: Vec<_> = fs::read_dir(&folder)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
                .collect();
            assert_eq!(files.len(), 1, "요청한 한 쪽의 렌더 트리만 사용한다");
            result.push(serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap());
        }
        fs::remove_dir_all(output).unwrap();
        result
    })
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

fn table(node: &Value, paragraph: u64, rows: u64, columns: u64) -> Option<&Value> {
    if node["type"] == "Table"
        && node["pi"] == paragraph
        && node["ci"] == 0
        && node["rows"] == rows
        && node["cols"] == columns
    {
        return Some(node);
    }
    children(node).find_map(|child| table(child, paragraph, rows, columns))
}

fn bounds(node: &Value) -> [f64; 4] {
    let b = &node["bbox"];
    let number = |name: &str| b[name].as_f64().unwrap();
    let (x, y) = (number("x"), number("y"));
    [x, y, x + number("w"), y + number("h")]
}

fn near(actual: f64, expected: f64, description: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{description}: 독립 Print {expected:.5}, 실제 {actual:.5}, 허용 ±2CSSpx"
    );
}

#[test]
fn continuation_keeps_the_terminal_text_on_its_own_page_once() {
    let phrase = "각주민대표단별로약1년간운영하는것으로가정";
    let owner_counts: Vec<_> = pages()
        .iter()
        .map(|page| text(page).matches(phrase).count())
        .collect();
    // 독립 Print p26의 끝 문장이다. 앞쪽으로 당기거나 이어받기에서 복제하지 않는다.
    assert_eq!(owner_counts, [0, 1, 0, 0, 0]);
}

#[test]
fn completed_current_object_frame_keeps_the_next_table_print_origin() {
    let page = &pages()[3];
    let prior = bounds(table(page, 172, 28, 4).expect("p28의 첫 표 끝 조각"));
    let next = bounds(table(page, 182, 5, 2).expect("p28의 다음 표 첫 조각"));
    // 공급 글꼴 한컴2024 Print p28의 두 표 괘선(PDF point ×96/72).
    near(prior[0], 77.5146, "이전 표 왼쪽");
    near(prior[2], 712.5333, "이전 표 오른쪽");
    near(
        prior[3],
        384.6986,
        "현재 본체와 빈 종료 줄을 담은 아래 괘선",
    );
    near(
        next[1],
        400.3613,
        "원본 끝 줄간격을 재가산하지 않은 다음 표 위 괘선",
    );
    assert!(prior[3] <= next[1], "두 소유 표가 겹치면 안 된다");
}

#[test]
fn final_fragment_keeps_the_empty_line_box_inside_its_print_border() {
    let page = &pages()[4];
    let frame = bounds(table(page, 182, 5, 2).expect("p29의 마지막 표 조각"));
    // 독립 Print p29의 마지막 괘선이다. 글자가 없는 종료 문단도 줄 상자를 소유한다.
    near(frame[1], 77.5146, "끝 조각 위 괘선");
    near(frame[3], 288.4840, "빈 종료 줄을 보존한 끝 조각 아래 괘선");
    assert!(text(page).contains("안정적인주택공급"));
}
