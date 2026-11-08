//! 같은 원본의 독립 한컴 Print는 p9의 표를 본문 안에 담고, p11의 새 행 최소 높이를
//! 앞 병합 잔여보다 크게 보존하며, p14의 남은 한 줄을 조각 가운데에 정렬한다.
//! p17/p18은 병합 셀 유닛을 나누어 보존하고 다음 완전 행의 물리 높이를 유지한다.
//! 절대 픽셀 위치 대신 출력 소유·포함·순서·정렬 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

struct OutputDir(PathBuf);
impl OutputDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("시각")
            .as_nanos();
        let p =
            std::env::temp_dir().join(format!("rhwp-5877-frames-{}-{nonce}", std::process::id()));
        fs::create_dir(&p).expect("원본 출력 임시 경로");
        Self(p)
    }
}
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue5877/fragment_ghost_vrules.hwp");
        let out = OutputDir::new();
        let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg("export-render-tree")
            .arg(input)
            .arg("-o")
            .arg(&out.0)
            .args(["--compat", "2022"])
            .output()
            .expect("Native 실제 쪽 출력");
        assert!(
            result.status.success(),
            "실제 출력 실패: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        (1..=31)
            .map(|p| {
                serde_json::from_slice(
                    &fs::read(out.0.join(format!("render_tree_{p:03}.json")))
                        .expect("독립 Print와 같은 원본 쪽"),
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
fn outer_tables<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    if node["type"] == "Table" {
        out.push(node);
        return;
    }
    if let Some(cs) = node["children"].as_array() {
        for c in cs {
            outer_tables(c, out);
        }
    }
}
fn cell<'a>(node: &'a Value, row: u64, col: u64) -> Option<&'a Value> {
    if node["type"] == "Cell"
        && node["row"].as_u64() == Some(row)
        && node["col"].as_u64() == Some(col)
    {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|c| cell(c, row, col))
}
fn geom(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().expect("출력 상자")
}
fn bottom(node: &Value) -> f64 {
    geom(node, "y") + geom(node, "h")
}
fn text_nodes<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    if node["text"].as_str().is_some_and(|s| !s.trim().is_empty()) {
        out.push(node)
    }
    if let Some(cs) = node["children"].as_array() {
        for c in cs {
            text_nodes(c, out)
        }
    }
}
fn words(node: &Value) -> String {
    let mut out = Vec::new();
    text_nodes(node, &mut out);
    out.iter()
        .filter_map(|n| n["text"].as_str())
        .collect::<String>()
        .split_whitespace()
        .collect()
}
#[test]
fn coanchored_float_and_inline_sibling_stay_inside_the_same_body() {
    let page = &pages()[8];
    let body = find(page, "Body").expect("본문 소유 상자");
    let mut tables = Vec::new();
    outer_tables(body, &mut tables);
    assert_eq!(
        tables.len(),
        2,
        "같은 호스트의 인라인 표와 흐름 표를 보존한다"
    );
    assert!(
        geom(tables[1], "y") >= bottom(tables[0]),
        "흐름 표는 앞 인라인 표 아래에서 시작한다"
    );
    for table in tables {
        assert!(
            geom(table, "y") >= geom(body, "y") && bottom(table) <= bottom(body),
            "독립 Print처럼 표 조각 전체를 본문 안에서 수용해야 한다"
        );
    }
}
#[test]
fn an_uncut_new_row_keeps_its_own_box_after_a_spanning_remainder() {
    let before = cell(&pages()[9], 23, 2).expect("앞 쪽 마지막 부분 행");
    let own = cell(&pages()[10], 24, 2).expect("다음 쪽 새 완전 행");
    assert!(
        geom(own, "h") > geom(before, "h"),
        "독립 Print에서 새 완전 행은 앞 쪽 잔여보다 크다"
    );
    let next = cell(&pages()[10], 25, 2).expect("다음 원본 행");
    assert!(
        geom(next, "y") >= bottom(own),
        "새 행의 상자 아래에서 다음 내용이 시작해야 한다"
    );
}
#[test]
fn restart_frames_align_selected_content_and_preserve_the_spanning_tail() {
    let remaining = cell(&pages()[13], 23, 1).expect("다음 쪽의 한 줄 소유자");
    let mut text = Vec::new();
    text_nodes(remaining, &mut text);
    assert_eq!(
        text.len(),
        1,
        "첫 줄은 앞 쪽이 소유하고 남은 한 줄만 보존한다"
    );
    let middle = geom(remaining, "y") + geom(remaining, "h") / 2.0;
    assert!(
        geom(text[0], "y") < middle && bottom(text[0]) > middle,
        "독립 Print의 Center 정렬처럼 한 줄의 상자가 셀 중앙을 덮어야 한다"
    );
    assert!(
        geom(text[0], "y") >= geom(remaining, "y") && bottom(text[0]) <= bottom(remaining),
        "글줄은 셀 안에 있어야 한다"
    );
    let front = cell(&pages()[16], 76, 0).expect("병합 셀 앞 조각");
    let tail = cell(&pages()[17], 76, 0).expect("병합 셀 마지막 조각");
    assert_eq!(
        words(front),
        "11.소화용수",
        "앞 조각의 세 유닛을 누락/중복하지 않는다"
    );
    assert_eq!(
        words(tail),
        "설비",
        "마지막 유닛은 다음 쪽에 한 번만 남는다"
    );
    let mut title = Vec::new();
    text_nodes(front, &mut title);
    let top = title
        .iter()
        .map(|n| geom(n, "y"))
        .fold(f64::INFINITY, f64::min);
    let end = title
        .iter()
        .map(|n| bottom(n))
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        top > geom(front, "y") && end < bottom(front),
        "앞 병합 조각도 선택한 글줄 양쪽 정렬 공간을 보존한다"
    );
    assert!(
        cell(&pages()[17], 76, 1).is_none(),
        "완료한 일반 행의 빈 꼬리로 새 행을 대체하지 않는다"
    );
    let first = cell(&pages()[17], 77, 1).expect("새 일반 행");
    let closing = cell(&pages()[17], 80, 1).expect("병합 종료 행");
    assert!(
        geom(closing, "h") > geom(first, "h"),
        "독립 Print처럼 실제 앞 상자를 차감한 병합 종료 행이 자체 최소 높이보다 커야 한다"
    );
}

#[test]
fn a_spanning_minimum_subtracts_both_boxes_of_a_split_ordinary_row() {
    use rhwp::{document_core::DocumentCore, model::control::Control};
    let original = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue5877/fragment_ghost_vrules.hwp"),
    )
    .expect("원본 저장 높이");
    let core = DocumentCore::from_bytes(&original).expect("원본 모델");
    let Control::Table(source) = &core.document().sections[0].paragraphs[20].controls[0] else {
        panic!("원본 표")
    };
    let spanning = source
        .cells
        .iter()
        .find(|c| c.row == 34 && c.col == 1)
        .expect("원본 병합 소유자");
    let front = cell(&pages()[13], 34, 1).expect("병합 셀 앞 물리 상자");
    let tail = cell(&pages()[14], 34, 1).expect("병합 셀 뒤 물리 상자");
    let physical = geom(front, "h") + geom(tail, "h");
    let declared = f64::from(spanning.height) / 75.0;
    // Print의 병합 최소 높이는 앞/뒤 조각을 합친 공간이다. 이 경우 내용이
    // 더 크지 않아 원본 최소 높이에서 닫힌다. 두 JSON bbox는 소수 첫째
    // 자리로 반올림하므로 각 0.05의 직렬화 오차만 합산한다.
    assert!((physical - declared).abs() <= 0.1,
        "분할된 일반 행의 앞/뒤 상자를 함께 차감해야 병합 최소가 보존된다: {physical} vs {declared}");
}
