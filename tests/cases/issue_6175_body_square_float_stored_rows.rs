//! [#6175] 어울림 개체 옆 문단이 저장된 좁은 폭을 지킨다.
//!
//! `samples/issue6175/seed_expo_square_float_body.hwpx` 는 원본(농촌진흥청 156655489,
//! 940KB)의 **구조 보존 슬라이스**다 — BinData 이미지만 8×8 PNG 로 바꿔 101KB 로
//! 줄였고, 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가 원본과 같다.
//!
//! **형상.** 1쪽 오른쪽에 용지 기준 `SQUARE` 그림이 있고, 문단 5~7 은 **문단 전체가**
//! 그 옆에 들어간다. 그래서 저장 사다리가 `cs=0 · sw=26692` 로 **균일하게** 좁다.
//!
//! **종전 결함.** `stored_rows_require_external_geometry` 는 한 문단 안에서 폭이
//! **변할** 때만 외부 기하의 증거로 인정했다. 문단 전체가 개체 옆이면 그 변화가
//! 사라져 증거가 소멸하고, 프레임(전폭)과 대조 → 불일치 → 전폭 재래핑 → 본문이
//! 그림 아래로 들어가 가려졌다.
//!
//! **판별자 — 개체 폭 대조.** 결손 폭이 문서에 실재하는 어울림 개체의 흐름 폭과
//! 맞으면 좁음의 출처는 문단 자신이 아니라 외부 기하다.
//!
//! ```text
//! 본문 폭 48188 − 저장 (cs 0 + sw 26692) = 결손 21496
//! 용지기준 SQUARE 그림 폭 21212 (offset 32361 → 프레임 좌표 26692)
//!   → 저장 사다리의 끝이 개체 왼쪽 변과 단위까지 일치
//! ```
//!
//! ⚠ "균일하게 좁다"만으로 판정하면 **문단 테두리 박스의 inset** 을 어울림으로
//! 오인해 #547·#1440 핀이 깨진다(#6129 에서 국소 판별자 2종이 그렇게 반증됐다).
//! 셀에서는 #5818 이 같은 혼동을 "같은 셀에 Square float 실재"로 갈랐고, 이것은 그
//! 계약의 본문 판이다.
//!
//! 한글 2022 실측(문서 편집 버전 = 한글 2020, major 11 → 가장 가까운 설치본):
//! 1쪽 본문 줄의 오른쪽 끝이 323~329pt.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6175/seed_expo_square_float_body.hwpx")
        .to_string_lossy()
        .into_owned()
}

/// `pi=<index>` 본문 줄의 (y, width) 목록.
fn body_line_widths(para_index: usize) -> Vec<(f64, f64)> {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let needle = format!(" pi={para_index} ");
    let mut rows = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if !line.starts_with("TextLine") || !line.contains(&needle) {
            continue;
        }
        let y = line
            .split("y=")
            .nth(1)
            .and_then(|r| r.split("..").next())
            .and_then(|v| v.trim().parse::<f64>().ok());
        let w = line
            .split(" w=")
            .nth(1)
            .and_then(|r| r.split_whitespace().next())
            .and_then(|v| v.parse::<f64>().ok());
        if let (Some(y), Some(w)) = (y, w) {
            rows.push((y, w));
        }
    }
    rows
}

#[test]
fn paragraph_beside_square_float_keeps_its_stored_narrow_width() {
    // 저장 sw=26692HU = 355.9px. 전폭 재래핑하면 642.5px 로 벌어져 그림을 덮는다.
    for para_index in [5usize, 7] {
        let rows = body_line_widths(para_index);
        assert!(!rows.is_empty(), "pi={para_index} 본문 줄을 찾지 못했다");
        for (y, w) in &rows {
            assert!(
                (*w - 355.9).abs() <= 6.0,
                "pi={para_index} y={y:.1} 줄이 전폭으로 재래핑됐다: w={w:.1} (기대 355.9)"
            );
        }
    }
}

#[test]
fn stored_rows_are_not_dropped_by_reflow() {
    // 전폭 재래핑은 줄 수도 줄인다 — pi=5 는 저장 3줄이다.
    let rows = body_line_widths(5);
    assert_eq!(
        rows.len(),
        3,
        "pi=5 의 저장 줄 수가 유지되지 않았다: {rows:?}"
    );
}
