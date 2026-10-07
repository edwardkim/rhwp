//! [#6145] `lineWrap="SQUEEZE"`("한 줄로 입력") 칸은 넘쳐도 안 여백을 깎지 않는다.
//!
//! `samples/issue6145/worklife_balance_index_156607916.hwpx` 는 고용노동부
//! 「2022년 기준 지역별 일･생활 균형 지수 발표」 보도자료 원본(HWPX, 272KB)이다.
//!
//! **증상.** 6쪽 마지막 열의 글자가 오른쪽 괘선을 넘어간다. 사용자 신고는
//! "`담당조직 형태 만점(1점)+`, `기업지원 명시 만점(1점)` 이 가려진다"였다.
//!
//! **근인 — 넘침 방어가 SQUEEZE 와 정반대로 동작한다.**
//!
//! `composer::shrunk_cell_horizontal_padding` 은 줄이 안쪽 폭을 `1.15` 배 넘으면
//! 좌우 안 여백을 **1px 까지 깎아** 자리를 만든다. 그런데 한/글이 SQUEEZE 칸에서
//! 하는 일은 반대다 — 여백은 그대로 두고 **자간을 줄여** 글자를 밀어 넣는다.
//!
//! 이 칸이 그 차이를 그대로 드러낸다. 선언은 `cellSz width=9906`,
//! `hasMargin="0"` 이므로 표의 `inMargin left/right=283` 이 적용돼야 하고, 저장
//! lineseg 3개가 전부 `horzsize=9340`(= 9906 − 283 − 283)으로 **안쪽 폭을 못박아**
//! 두었다. 그런데 여백을 1px 로 깎으면 안쪽 폭이 `9906 − 150 = 9756`(97.56pt)이
//! 되어 자간이 덜 줄고, 줄이 괘선 밖으로 나간다.
//!
//! **오라클 — 한글 2020 기준 PDF**
//! (`pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf`,
//! `Creator=Hwp 2020 0.34.0.0`, `Producer=Hancom PDF 1.3.0.550`, 6쪽).
//!
//! | 6쪽 마지막 열 | 종전 | **수정 후** | 한글 2020 PDF |
//! |---|---:|---:|---:|
//! | 글자 시작 x | 438.98 | **441.06** | 441.00 |
//! | `일･생활 균형 조례 제정,` 우단 | 534.67 | 533.14 | 534.37 |
//! | `기업지원 명시 만점(1점)` 우단 | 537.50 | **536.28** | 534.44 |
//! | `담당조직 형태 만점(1점)+` 우단 | **538.48 (괘선 밖)** | **537.28** | 534.41 |
//!
//! 괘선은 `537.79`(rhwp) / `537.43`(한글)이다. 종전에는 마지막 줄이 괘선을
//! `+0.69pt` 넘었고, 수정 후에는 세 줄 모두 안쪽에서 끝난다.
//!
//! 남는 우단 차(한글 대비 최대 2.9pt)는 조판 폭과 페인트 폭이 갈리는 **별개 축**
//! 이다(`#6303` 에서 같은 발산을 확인했다). 이 시험은 이 이슈가 지목한 축 —
//! **안 여백 보존** — 만 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 표 `inMargin left/right = 283 HWPUNIT` = 3.773px. 여백이 깎이면 1px 로 떨어진다.
const DECLARED_INNER_MARGIN_PX: f64 = 283.0 / 7200.0 * 96.0;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6145/worklife_balance_index_156607916.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6145-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 6쪽 render tree JSON.
fn page6_render_tree() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "5",
            "-o",
            &dir.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let path = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .expect("render tree JSON 이 없다");
    std::fs::read_to_string(path).unwrap()
}

fn target_table(node: &Value) -> Option<&Value> {
    if node["type"] == "Table" && node["pi"] == 56 {
        return Some(node);
    }
    node["children"].as_array()?.iter().find_map(target_table)
}

fn target_cell(table: &Value, row: u64) -> &Value {
    table["children"]
        .as_array()
        .expect("표 칸")
        .iter()
        .find(|cell| cell["type"] == "Cell" && cell["row"] == row && cell["col"] == 5)
        .unwrap_or_else(|| panic!("원문 {row}행 마지막 칸이 없다"))
}

fn text_line_containing<'a>(cell: &'a Value, needle: &str) -> &'a Value {
    cell["children"]
        .as_array()
        .expect("칸 글줄")
        .iter()
        .find(|line| {
            line["type"] == "TextLine"
                && line["children"].as_array().is_some_and(|runs| {
                    runs.iter()
                        .filter_map(|run| run["text"].as_str())
                        .collect::<String>()
                        .contains(needle)
                })
        })
        .unwrap_or_else(|| panic!("`{needle}` 글줄이 없다"))
}

/// SQUEEZE 칸의 글자는 선언된 안 여백만큼 안쪽에서 시작한다.
///
/// 종전에는 넘침 방어가 여백을 `1px` 로 깎았다. 한/글은 선언된 여백을 지킨다.
#[test]
fn squeeze_cell_text_starts_inside_the_declared_inner_margin() {
    let json: Value = serde_json::from_str(&page6_render_tree()).expect("렌더 트리 JSON");
    let table = target_table(&json).expect("원문 마지막 표 문단");
    let cell = target_cell(table, 21);
    let line = text_line_containing(cell, "담당조직 형태 만점");
    let left = line["children"]
        .as_array()
        .expect("글줄 run")
        .iter()
        .filter_map(|run| run["bbox"]["x"].as_f64())
        .fold(f64::INFINITY, f64::min);
    assert!(left.is_finite(), "검사할 글자의 좌단이 없다");
    let cell_left = cell["bbox"]["x"].as_f64().expect("칸 좌단");
    let inset = left - cell_left;
    assert!(
        inset >= DECLARED_INNER_MARGIN_PX - 0.5,
        "SQUEEZE 칸은 선언된 안 여백({DECLARED_INNER_MARGIN_PX:.2}px)을 지켜야 한다: \
         좌단 {left:.2}px, 들여쓴 폭 {inset:.2}px"
    );
}

/// SQUEEZE 칸의 줄 폭이 저장 lineseg 가 못박은 안쪽 폭 가까이로 조여진다.
///
/// 저장 lineseg 는 `horzsize=9340`(= 124.53px)이다. 여백을 1px 로 깎으면 목표가
/// `130.10px` 로 헐거워져 자간이 덜 줄고, 그 결과 `담당조직 형태 만점(1점)+` 이
/// PDF 우단 `538.48pt` 로 괘선(`537.79pt`)을 `+0.69pt` 넘었다.
///
/// 이 검사는 원문 셀 폭에서 좌우 선언 여백을 뺀 폭과 현재 글줄 폭을 대조한다.
/// 칸의 절대 x 좌표가 달라져도 같은 원문 폭 계약을 확인할 수 있다.
#[test]
fn squeeze_cell_line_width_tracks_the_stored_inner_width() {
    let pages = Command::new(rhwp_bin())
        .args(["dump-pages", &sample(), "--json"])
        .output()
        .expect("쪽 구성을 읽는다");
    assert_eq!(pages.status.code(), Some(0), "{pages:?}");
    let pages: Value = serde_json::from_slice(&pages.stdout).expect("쪽 구성 JSON");
    assert_eq!(pages["pageCount"], 6, "정상 PDF와 같은 6쪽이어야 한다");

    let json: Value = serde_json::from_str(&page6_render_tree()).expect("렌더 트리 JSON");
    let table = target_table(&json).expect("원문 마지막 표 문단");
    // 원본의 25×6 표와 저장 안쪽 폭은 좌표가 변해도 같은 칸 계약이다.
    let stored_inner_width = (9906.0 - 283.0 - 283.0) / 7200.0 * 96.0;
    for (row, needle) in [(20, "기업지원 명시 만점"), (21, "담당조직 형태 만점")] {
        let cell = target_cell(table, row);
        let line = text_line_containing(cell, needle);
        let line_width = line["bbox"]["w"].as_f64().expect("글줄 폭");
        let cell_width = cell["bbox"]["w"].as_f64().expect("칸 폭");
        assert!(
            (line_width - stored_inner_width).abs() <= 0.6,
            "`{needle}` 글줄은 원문 안쪽 폭을 써야 한다: {line_width:.2}px"
        );
        assert!(
            (cell_width - line_width - 2.0 * DECLARED_INNER_MARGIN_PX).abs() <= 0.7,
            "`{needle}` 칸은 좌우 원문 여백을 유지해야 한다"
        );
    }
}
