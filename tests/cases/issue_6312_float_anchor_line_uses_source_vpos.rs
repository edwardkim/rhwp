//! [#6312] 자리차지 표 앵커 문단의 자기 글줄이 흐름을 전진시킨다.
//!
//! `samples/issue6312/fiscal_trend_float_table_anchor.hwpx` 는 원본(기획재정부
//! 156721992)의 **구조 보존 슬라이스**다 — BinData 이미지만 8×8 PNG 로 바꿔
//! 874KB → 92KB 로 줄였고, 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가 원본과 같다.
//!
//! **형상.** 최상위 문단 0(`paraPr=59`)이 `TOP_AND_BOTTOM` 부동 표 3개를 달고, 글자는
//! 없지만 **자기 글줄**(`vertsize=1500` + `spacing=1200` = 27pt)을 따로 가진다. 한글은
//! 그 줄 자리를 지키는데 rhwp 는 버려, 다음 문단이 표 아래 괘선에 붙었다.
//!
//! **근인 — 사다리 등식을 재계산된 좌표로 재고 있었다.**
//!
//! `#6147` 이 세운 `stored_empty_anchor_band_host_line_advance_hu` 는
//! `next.vpos − host.vpos == lh + max(ls,0)` 를 증거로 쓴다. 그런데
//! `reflow_zero_height_paragraphs` 는 구역에 0-높이 lineseg 가 하나라도 있으면 그
//! 구역 **모든** 문단의 `vertical_pos` 를 자기 누적 좌표로 다시 쓴다(첫 문단은 0).
//! 그 좌표로 등식을 재면 rhwp 자신의 재계산을 rhwp 로 검증하는 순환이 된다.
//!
//! ```text
//! 원본 저장:   19829 − 17129 = 2700 = 1500 + 1200   → 등식 성립
//! 재계산 좌표: 11066 −     0 = 11066                 → 불성립 → 줄 27pt 소실
//! ```
//!
//! 그 재계산이 원본을 `source_line_seg_vertical_pos` 에 남겨 두므로, 등식을 그 값으로
//! 본다. 재계산이 걸리지 않은 문서는 필드가 비어 종전 동작 그대로다.
//!
//! **잔여는 별개 축이다.** 표 자체가 한글보다 10.03pt 위에 있고(괘선 230.25 vs
//! 240.28), 수정 후 문단 오차(−9.99pt)가 표 오차와 **같아진다** — 앵커 줄 축은 닫혔고
//! 남은 것은 표 위치 축이다(이슈 본문도 그렇게 분리해 적었다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6312/fiscal_trend_float_table_anchor.hwpx")
        .to_string_lossy()
        .into_owned()
}

/// `pi=1` 첫 줄의 y(px).
fn first_body_line_y() -> f64 {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    for raw in text.lines() {
        let line = raw.trim();
        if !line.starts_with("TextLine") || !line.contains(" pi=1 ") || !line.contains("line=0") {
            continue;
        }
        if let Some(y) = line
            .split("y=")
            .nth(1)
            .and_then(|r| r.split("..").next())
            .and_then(|v| v.trim().parse::<f64>().ok())
        {
            return y;
        }
    }
    panic!("pi=1 첫 줄을 찾지 못했다:\n{text}");
}

#[test]
fn anchor_paragraph_line_advances_the_flow() {
    // 표 아래 괘선 307.0px. 앵커 줄(27pt = 36px)을 버리면 문단이 거기 붙는다.
    let y = first_body_line_y();
    assert!(
        y > 307.0 + 30.0,
        "앵커 문단의 자기 글줄이 흐름에서 사라졌다 — 문단이 표 아래 괘선에 붙었다: y={y:.1}"
    );
    // 수정 실측 346.8px. 한글 2022 는 360.1px 인데, 남은 13.3px(=10pt)은 표 자체가
    // 한글보다 높은 별개 축이다(표 괘선 rhwp 307.0 vs 한글 320.4).
    assert!(
        (y - 346.8).abs() <= 6.0,
        "앵커 줄 전진량이 저장 사다리(27pt)와 맞지 않는다: y={y:.1} (기대 346.8)"
    );
}

#[test]
fn paragraph_error_matches_the_table_error() {
    // 앵커 줄 축이 닫혔다는 증거 — 문단 오차와 표 오차가 같아야 한다.
    // 표 아래 괘선 307.0px(한글 320.4px) → 오차 13.4px.
    // 문단 첫 줄 346.8px(한글 360.1px) → 오차 13.3px.
    let y = first_body_line_y();
    let para_error = 360.1 - y;
    let table_error = 320.4 - 307.0;
    assert!(
        (para_error - table_error).abs() <= 4.0,
        "문단 오차({para_error:.1}px)가 표 오차({table_error:.1}px)와 다르다 — \
         앵커 줄 몫이 여전히 어긋난다"
    );
}
