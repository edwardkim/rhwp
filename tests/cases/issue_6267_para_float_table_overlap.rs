//! [#6267] 자리차지(TopAndBottom) 표가 호스트 문단 본문과 겹치지 않는다.
//!
//! `samples/issue6267/kdt_result_para_float_table.hwpx` 1쪽 문단 8 은 본문 4줄을 가진
//! 문단이 세로 offset 9507HU 짜리 자리차지 표를 안고 있다. 결함은 두 오차가
//! 마주 달려 생겼다.
//!
//! ① **호스트 문단 sb 이중 계상** — `layout.rs` 의 "표 위 간격" 블록이 표 호스트
//!    문단의 `y_offset` 에 `spacing_before` 를 더하는데, 자리차지 표는 흐름을
//!    소비하지 않고 `compute_table_y_position` 이 sb 이전 앵커로 따로 앉는다.
//!    그래서 그 가산은 표에 닿지 않고 호스트 텍스트만 밀어냈고, 그 텍스트를 그리는
//!    `layout_composed_paragraph` 가 sb 를 또 더해 이중이 됐다. 저장 사다리 대비
//!    문단 5~7 은 +75.6px 인데 문단 8 만 +91.6px(= sb 16.0px 초과)로 튀었다.
//!
//! ② **body_bottom 클램프** — 표의 raw 위치 954.0px 를 본문 하단 937.0px 로
//!    끌어올려 직전 줄(926.4..945.1)을 침범했다. #5699 J3 해제가 바로 이 겹침을
//!    막으려고 있는데 `vertical_offset == 0` 관문에 걸려 발동하지 못했다.
//!
//! 한글 2024 실측: 표 상단 952.9px, 마지막 줄 바닥과의 간격 +6.8pt.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6267/kdt_result_para_float_table.hwpx")
        .to_string_lossy()
        .into_owned()
}

/// `dump-extents` 에서 `pi=8` 아이템의 (종류, top, bottom) 을 모은다.
fn page8_para8_items() -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();

    let mut tables = Vec::new();
    let mut lines = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if !line.contains("pi=8") {
            continue;
        }
        let kind = if line.starts_with("Table") {
            0
        } else if line.starts_with("TextLine") {
            1
        } else {
            continue;
        };
        // `y=   954.0..  1064.0` 형태에서 두 수를 뽑는다.
        let Some(rest) = line.split("y=").nth(1) else {
            continue;
        };
        let Some((top, tail)) = rest.split_once("..") else {
            continue;
        };
        let Ok(top) = top.trim().parse::<f64>() else {
            continue;
        };
        let Some(bottom) = tail.split_whitespace().next() else {
            continue;
        };
        let Ok(bottom) = bottom.parse::<f64>() else {
            continue;
        };
        if kind == 0 {
            tables.push((top, bottom));
        } else {
            lines.push((top, bottom));
        }
    }
    (tables, lines)
}

#[test]
fn para_float_table_does_not_overlap_host_text() {
    let (tables, lines) = page8_para8_items();
    assert_eq!(
        tables.len(),
        1,
        "문단 8 의 자리차지 표 1개를 기대했다: {tables:?}"
    );
    assert!(!lines.is_empty(), "호스트 문단 본문 줄이 없다");

    let (table_top, _) = tables[0];
    let text_bottom = lines
        .iter()
        .map(|(_, b)| *b)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        table_top >= text_bottom - 0.5,
        "자리차지 표가 호스트 본문과 겹친다: table_top={table_top:.1} < text_bottom={text_bottom:.1}"
    );

    // 한글 2024 실측 952.9px. 클램프가 다시 끼어들면 937.0 으로 내려앉는다.
    assert!(
        (table_top - 952.9).abs() <= 4.0,
        "표 상단이 한글(952.9px)에서 벗어났다: {table_top:.1}"
    );
}

#[test]
fn host_paragraph_spacing_before_is_not_double_charged() {
    let (_, lines) = page8_para8_items();
    let first_top = lines.iter().map(|(t, _)| *t).fold(f64::INFINITY, f64::min);

    // 저장 사다리(vpos 57295HU = 763.9px) + 본문 상단 75.6px = 839.5px.
    // sb(16.0px)를 이중 계상하면 855.5px 로 튄다.
    assert!(
        (first_top - 839.5).abs() <= 1.0,
        "호스트 문단 첫 줄이 저장 사다리와 어긋난다(sb 이중 계상): {first_top:.1}"
    );
}
