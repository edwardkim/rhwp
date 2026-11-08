//! #8105: 독립 최소 높이를 가진 완료 셀의 마지막 슬롯과 다음 쪽 소속.
//! 같은 원본의 한컴 2024 Print는 검사관리의 두 번째 줄을 10쪽으로 넘긴다.
//! 원본 슬롯은 완결 수용량에만 포함하며, 이어받은 잉크 뒤에 다시 더하지 않는다.
//! 원본: samples/issue1880_takeplace_host_before.hwpx
//! Print: pdf/issue1880/issue1880_takeplace_host_before-2024.pdf
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

const INPUT: &str = "samples/issue1880_takeplace_host_before.hwpx";
const CONTINUATION: &str = "시료분석의뢰절차및조치방법을규정하여야한다.";
// JSON bbox의 소수 첫 자리 반올림 네 항의 끝점 차이 공차다. 문서 위치 핀이 아니다.
const ENDPOINT_ROUNDOFF: f64 = 2.0 * 0.1;

struct OutputDir(PathBuf);
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("시각")
            .as_nanos();
        let out = OutputDir(root.join("output").join(format!(
            "issue8105-closing-slot-{}-{nonce}",
            std::process::id()
        )));
        fs::create_dir_all(&out.0).expect("검사 전용 output");
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let result = Command::new(bin)
            .arg("export-render-tree")
            .arg(root.join(INPUT))
            .args(["--compat", "2024", "-o"])
            .arg(&out.0)
            .output()
            .expect("실제 Native CLI");
        assert!(
            result.status.success(),
            "원본 출력 실패: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut files: Vec<_> = fs::read_dir(&out.0)
            .expect("출력 목록")
            .filter_map(Result::ok)
            .map(|p| p.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|s| s.to_string_lossy().starts_with("render_tree_"))
                    && p.extension().is_some_and(|s| s == "json")
            })
            .collect();
        files.sort();
        assert_eq!(files.len(), 10, "같은 원본 독립 Print의 전체 쪽수");
        files
            .iter()
            .map(|p| serde_json::from_slice(&fs::read(p).expect("실제 트리")).expect("트리 JSON"))
            .collect()
    })
}
fn children(node: &Value) -> impl Iterator<Item = &Value> {
    node.get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}
fn cell(node: &Value, in_table: bool, row: u64, col: u64) -> Option<&Value> {
    let inside = if node["type"] == "Table" {
        node["pi"] == 9 && node["ci"] == 0
    } else {
        in_table
    };
    if inside && node["type"] == "Cell" && node["row"] == row && node["col"] == col {
        return Some(node);
    }
    children(node).find_map(|c| cell(c, inside, row, col))
}
fn text(node: &Value) -> String {
    let mut out = String::new();
    if node["type"] == "TextRun" {
        out.push_str(node["text"].as_str().expect("본문"));
    }
    for c in children(node) {
        out.push_str(&text(c));
    }
    out.chars().filter(|c| !c.is_whitespace()).collect()
}
fn runs<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    if node["type"] == "TextRun" {
        out.push(node);
    }
    for c in children(node) {
        runs(c, out);
    }
}
fn bottom(node: &Value) -> f64 {
    node["bbox"]["y"].as_f64().expect("위") + node["bbox"]["h"].as_f64().expect("높이")
}
#[test]
fn completed_cell_keeps_the_last_line_on_the_next_print_page() {
    let pages = pages();
    assert!(
        !text(&pages[8]).contains(CONTINUATION),
        "9쪽이 다음 원본 슬롯을 흡수하면 안 된다"
    );
    let next = cell(&pages[9], false, 6, 1).expect("10쪽 검사관리 이어받기 칸");
    assert!(
        text(next).contains(CONTINUATION),
        "독립 Print10쪽의 같은 source 셀이 마지막 줄을 소유한다"
    );
    assert_eq!(
        pages
            .iter()
            .map(text)
            .map(|s| s.matches(CONTINUATION).count())
            .sum::<usize>(),
        1,
        "마지막 줄은 전쪽에 정확히 한 번 보존한다"
    );
}
#[test]
fn continued_cell_closes_after_ink_and_its_original_bottom_padding() {
    let next = cell(&pages()[9], false, 6, 1).expect("10쪽 검사관리 이어받기 칸");
    let mut lines = Vec::new();
    runs(next, &mut lines);
    let ink = lines
        .iter()
        .filter(|n| n["text"].as_str().is_some_and(|s| !s.trim().is_empty()))
        .map(|n| bottom(n))
        .reduce(f64::max)
        .expect("이어받은 글줄");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let document =
        rhwp::parse_document(&fs::read(root.join(INPUT)).expect("원본")).expect("원본 파싱");
    let Control::Table(table) = &document.sections[0].paragraphs[9].controls[0] else {
        panic!("원본 표")
    };
    let source = table
        .cells
        .iter()
        .find(|c| c.row == 6 && c.col == 1)
        .expect("원본 칸");
    let padding = f64::from(source.effective_padding(&table.padding).bottom) * 96.0 / 7200.0;
    let gap = bottom(next) - ink;
    assert!((gap-padding).abs()<=ENDPOINT_ROUNDOFF,"종료 전진을 이어받기 paint에 다시 더하지 않는다: 잉크 뒤 {gap}, 원본 아래 안여백 {padding}");
}
