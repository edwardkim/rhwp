//! [#6280] 텍스트 없는 셀 문단의 저장 줄 높이가 이미 품은 그림을 다시 더하지 않는다.
//!
//! `samples/issue6280/prosecutor_transfer_title_cells.hwp` 21쪽에는 **같은 서식**의
//! 제목 표가 둘 있다. 둘 다 `1행×2열` 글자처럼 취급 표이고, 왼쪽 칸[0]은 텍스트 없이
//! 비-TAC 자리차지 그림 하나만, 오른쪽 칸[1]은 제목 글자를 담는다. 칸은 `valign=Center`
//! 라 표가 부풀면 제목이 아래로 내려가 장식 막대를 덮는다.
//!
//! | | 저장 `line_height` | 그림 흐름(높이+양수 오프셋) | 판정 |
//! |---|---|---|---|
//! | `타기관 파견 등 (1명)` | 18.7px | 30.2px | `lh < flow` — 흡수 아님 |
//! | `의원면직 (11명)` | **40.0px** | **36.7px** | `lh ≥ flow` — **흡수** |
//!
//! 종전에는 후자에서 저장 줄(40.0px)에 그림(36.7px)을 **또 더해** content 76.7px 로
//! 재고, 그 초과가 클램프를 넘겨 행이 선언 높이(43.8px)의 1.84배로 부풀었다. 그
//! 결과 제목 `의원면직 (11명)`(630.4..657.1)이 장식 막대(645.4..652.9)를 11.7px 덮었다.
//!
//! 이 테스트는 **두 표를 함께** 잠근다 — 통제군(타기관)이 함께 있어야 새 증인이
//! `lh < flow` 쪽으로 번지지 않았음을 확인할 수 있다(#1282 반례가 그 경계다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6280/prosecutor_transfer_title_cells.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6280-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 21쪽 render tree JSON 전문.
fn page21_render_tree() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "20",
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

/// `needle` 을 담은 노드의 가장 가까운 앞선 `bbox` 에서 (y, y+h) 를 읽는다.
fn text_node_y_range(json: &str, needle: &str) -> (f64, f64) {
    let at = json
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} 을 render tree 에서 찾지 못했다"));
    let head = &json[..at];
    let bbox_at = head
        .rfind("\"bbox\"")
        .unwrap_or_else(|| panic!("{needle} 앞에 bbox 가 없다"));
    let seg = &json[bbox_at..(bbox_at + 200).min(json.len())];
    let num = |key: &str| -> f64 {
        seg.split(&format!("\"{key}\""))
            .nth(1)
            .and_then(|r| r.trim_start().strip_prefix(':'))
            .and_then(|r| {
                r.trim_start()
                    .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                    .find(|s| !s.is_empty())
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or_else(|| panic!("bbox.{key} 를 읽지 못했다: {seg}"))
    };
    let y = num("y");
    (y, y + num("h"))
}

#[test]
fn title_cell_does_not_inflate_when_stored_line_already_covers_its_picture() {
    let json = page21_render_tree();
    let (top, bottom) = text_node_y_range(&json, "의원면직");

    // 흡수를 인정하면 표가 선언 높이로 그려지고 제목이 612.1..638.8 에 온다.
    // 이중 계상하면 630.4..657.1 로 18.3px 내려가 장식 막대(645.4..652.9)를 덮는다.
    assert!(
        (top - 612.1).abs() <= 3.0,
        "제목이 부푼 행 때문에 내려갔다: y={top:.1}..{bottom:.1} (기대 612.1..638.8)"
    );
    // 장식 막대 상단 645.4 위에 글자 바닥이 있어야 한다.
    assert!(
        bottom <= 645.4 + 0.5,
        "제목이 장식 막대를 덮는다: 글자 바닥 {bottom:.1} > 막대 상단 645.4"
    );
}

#[test]
fn sibling_title_cell_with_shorter_stored_line_is_unchanged() {
    // 통제군 — 같은 쪽 같은 서식이지만 저장 줄(18.7px)이 그림(30.2px)보다 **작다**.
    // 한글은 이때 행을 그림만큼 키우므로(#1282 반례) 종전 회계를 유지해야 한다.
    let json = page21_render_tree();
    let (top, bottom) = text_node_y_range(&json, "타기관");
    assert!(
        (top - 422.4).abs() <= 3.0,
        "통제군 제목이 움직였다 — 새 증인이 `lh < flow` 쪽으로 번졌다: y={top:.1}..{bottom:.1}"
    );
    assert!(
        bottom <= 453.0 + 0.5,
        "통제군 제목이 장식 막대를 덮는다: {bottom:.1}"
    );
}
