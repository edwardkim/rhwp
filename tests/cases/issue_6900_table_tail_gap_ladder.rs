//! [#6900] 표 뒤 host 앵커 줄의 후행 간격을 **사다리가 이미 다 준 자리에** 또 더한다.
//!
//! 표 아래 간격은 host 앵커 줄의 `LineSeg::line_spacing` 이다. 그런데 앵커 줄의 사다리가
//! **앞선 TAC 표 한 장만** 덮고 뒤따르는 비-TAC 자리차지 표는 덮지 않는 문단이 있다.
//! 그런 문단에서는 표 하단이 이미 사다리가 지목한 다음 문단 자리까지 내려와 있어서,
//! 간격을 또 더하면 후속 문단이 통째로 그만큼 밀린다.
//!
//! `156521182`(동아일보 보도설명자료) **4쪽** 실측:
//!
//! ```text
//!   pi=30 seg0   vpos 0      lh 45.3        ← 첫 TAC 표(41.5px)만 덮는다
//!   표 하단(그린 값)           977.4
//!   pi=31 저장 vpos 64803  →   977.4        ← 사다리가 표 하단을 지목
//!
//!   수정 전  977.4 + 14.9(seg.line_spacing) = 992.3 → 출처 줄 1017.9..1033.9
//!   수정 후                                   977.4 → 출처 줄 1003.0..1019.0
//!   본문 하단 1020.5 · 정본(engine 2020) 출처 줄 1000.6..1019.8
//! ```
//!
//! 판별은 문서가 준다 — 다음 문단의 저장 `vpos` 를 이 문단의 사다리 기준점으로 환산해
//! 현재 흐름 위치와 견준다. **한쪽 방향만** 본다: 사다리가 표 하단 **이하**를 지목할
//! 때만 간격을 거둔다(더 아래를 지목하면 종전대로 더한다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6900/156521182-covid-press-clarification.hwp";
/// 출처 줄이 있는 쪽(0 기준).
const PAGE: u32 = 3;

fn page_layout() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    serde_json::from_str(
        &core
            .get_page_text_layout_native(PAGE)
            .expect("공개 text-layout"),
    )
    .expect("text-layout JSON")
}

fn source_line_top(layout: &serde_json::Value) -> f64 {
    layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .find(|run| run["text"].as_str().unwrap_or_default().contains("출처"))
        .map(|run| run["y"].as_f64().expect("y"))
        .expect("`출처` 런이 있어야 한다")
}

#[test]
fn issue_6900_source_line_stays_inside_the_body() {
    let layout = page_layout();
    let top = source_line_top(&layout);
    let height = layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .find(|run| run["text"].as_str().unwrap_or_default().contains("출처"))
        .and_then(|run| run["h"].as_f64())
        .expect("출처 런 높이");

    // 본문 하단 — 문서 상수(용지 1121.3, 아래 여백 포함). 종전에는 1033.9 로 13.4px 넘었다.
    const BODY_BOTTOM: f64 = 1020.5;
    assert!(
        top + height <= BODY_BOTTOM + 0.5,
        "출처 줄이 본문 안에 있어야 한다: {top:.1}..{:.1} vs 본문 하단 {BODY_BOTTOM}",
        top + height
    );

    // 정본(engine 2020) 1000.6 — 사다리가 지목한 자리(977.4)에서 문단 간격만큼 내려온다.
    assert!(
        (top - 1003.0).abs() <= 1.0,
        "출처 줄이 사다리 자리에서 시작해야 한다: {top:.1} (기대 1003.0, 정본 1000.6)"
    );
}

#[test]
fn issue_6900_table_rows_do_not_move() {
    // ⚠ **음성 대조** — 이 수정은 표를 건드리지 않는다. 어긋난 축은 표 **뒤**의 간격
    // 하나이고, 표 하단은 종전에도 정본과 1.1px 안이었다. 표 마지막 두 행의 글자가
    // 제자리인지 함께 잰다(간격을 표 안으로 밀어 넣는 식의 수정을 배제한다).
    let layout = page_layout();
    let runs = layout["runs"].as_array().expect("runs").clone();
    let y_of = |needle: &str| -> f64 {
        runs.iter()
            .find(|run| run["text"].as_str().unwrap_or_default().trim() == needle)
            .and_then(|run| run["y"].as_f64())
            .unwrap_or_else(|| panic!("`{needle}` 런이 없다"))
    };

    // 마지막 두 행(37 뉴질랜드 / 38 일본)의 저장 자리 — 문서 상수다.
    for (needle, expected) in [("뉴질랜드", 939.4), ("일본", 960.3)] {
        let y = y_of(needle);
        assert!(
            (y - expected).abs() <= 0.5,
            "표 행 `{needle}` 이 제자리여야 한다: {y:.1} (기대 {expected})"
        );
    }
}
