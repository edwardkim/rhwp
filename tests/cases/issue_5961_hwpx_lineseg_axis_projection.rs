//! [Issue #5961] HWPX 저장 lineseg 의 축이 IR 안에서 통일돼 있지 않다.
//!
//! `LineSeg::text_start` 는 파서가 **파일 값을 그대로** 담으므로 출처마다 축이 다르다.
//! HWP5·HWP3·HML 은 확장 제어 하나가 8 UTF-16 유닛을 차지하는 HWP5 축이고, HWPX 는
//! `hp:secPr`(구역 머리 run 소속)이 자리를 차지하지 않는 더 짧은 축이다. 반면 같은 문단의
//! `char_count`·`char_offsets`·`char_shapes` 는 **출처와 무관하게 언제나 HWP5 축**이다.
//! 그래서 HWPX 출처의 구역 첫 문단은 IR 안에서 두 축이 섞인다.
//!
//! 그 상태로 `text_start` 를 `char_offsets` 에 투영하면 줄이 보정폭만큼 **일찍** 끊긴다.
//!
//! ## 정답지 — 한글 2024 에게 직접 물었다
//!
//! `MoveDocBegin` + `MoveLineDown` 으로 캐럿이 앉는 `(Para, Pos)` 를 읽으면 한글 자신의 줄
//! 분할이 나온다. 코퍼스 `36497307_결재문서본문.hwpx` 문단 0:
//!
//! ```text
//! LINE0  para=0  pos=24   <- 본문 시작 (컨트롤 4개 중 3개만 자리를 차지)
//! LINE1  para=0  pos=78   <- 둘째 줄
//! ```
//!
//! 본문이 24 에서 시작하므로 둘째 줄의 글자 인덱스는 `78 - 24 = 54` 다. 보정 없이
//! 투영하면 `78 - 32 = 46` 이 나온다 — 정확히 8유닛 어긋난다. 실제로 그 문서는
//! `… 위하여 지방` / `세징수법 제106조 …` 로 끊기고, 한글은 `… 제10` / `6조 …` 로 끊는다.
//! HWPX 500건 표본 중 49건(9.9%)이 해당한다.
//!
//! ## 계약
//!
//! 축은 **읽는 쪽에서만** 맞춘다. `text_start` 자체는 파일 값 그대로 두고
//! `Paragraph::hwpx_axis_shift` 를 함께 실어, 소비자가 `line_seg_text_start()` 로 올려 본다.
//!
//! 파일 축을 옮기면 안 되는 이유는 실측돼 있다 — 파서가 값을 고치면 x2x 재수출이 왕복마다
//! 8씩 흘러내려 3회 만에 `textpos` 가 0 으로 무너지고(24 → 16 → 8 → 0), h2x 의 #5943
//! 재기준화(02502 는 40 이면 한글이 열지 못하고 32 여야 한다)와도 충돌한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

/// `aift.hwpx` 문단 0 의 슬롯 사다리는 `secd@0 · cold@8 · tbl@16 · pgnp@24 · text@32` 이고
/// `char_count` 는 56(= 32 + 텍스트 23 + 문단부호)이다. 파일이 실은 둘째 줄 `textpos` 는
/// 24 — `hp:secPr` 한 자리만큼 짧은 HWPX 축이다.
fn aift_first_paragraph() -> rhwp::model::paragraph::Paragraph {
    let path = Path::new("samples/hwpx/aift.hwpx");
    assert!(path.exists(), "샘플이 없다: {}", path.display());
    let data = std::fs::read(path).expect("aift.hwpx 읽기");
    let doc = rhwp::parser::parse_document(&data).expect("aift.hwpx 파싱");
    doc.sections[0].paragraphs[0].clone()
}

/// 파일 값은 건드리지 않는다 — 이게 재수출 고정점의 근거다.
#[test]
fn the_stored_text_start_keeps_the_file_value() {
    let para = aift_first_paragraph();
    assert_eq!(
        para.line_segs[1].text_start, 24,
        "저장 lineseg 의 textpos 가 파일 값(24)에서 움직였다 — 파일 축을 옮기면 \
         x2x 재수출이 왕복마다 흘러내린다(24 → 16 → 8 → 0 실측)."
    );
}

/// 보정폭은 파서가 만들어 넣은 비점유 슬롯 수 × 8 이다.
#[test]
fn the_axis_shift_counts_the_slots_the_parser_synthesized() {
    let para = aift_first_paragraph();
    assert_eq!(
        para.hwpx_axis_shift, 8,
        "`hp:secPr` 한 자리(8유닛)가 보정폭이어야 한다. 이 문서의 `hp:colPr` 은 secPr 안이 \
         아니라 별도 `hp:ctrl` 이라 HWPX 축을 차지한다(한글 실측: 본문 시작 pos 24)."
    );
}

/// 투영하면 `char_offsets` 와 같은 자가 된다 — 둘째 줄은 본문 시작(32)에서 시작한다.
#[test]
fn the_projection_puts_the_line_on_the_hwp5_axis() {
    let para = aift_first_paragraph();

    assert_eq!(
        para.line_seg_text_start(1),
        32,
        "투영값이 HWP5 축이 아니다. 이 문단은 컨트롤 4개(=32유닛) 뒤에서 본문이 시작하고 \
         둘째 줄이 곧 본문 시작이다."
    );
    // 투영값을 char_offsets 에 넣으면 텍스트 0번 글자가 나와야 한다.
    let idx = para
        .char_offsets
        .partition_point(|&o| o < para.line_seg_text_start(1));
    assert_eq!(
        idx, 0,
        "투영값을 char_offsets 로 옮기니 본문 첫 글자가 아니다 — 축이 여전히 섞여 있다."
    );
}

/// 문단 시작(0)은 두 축에서 같은 자리다.
#[test]
fn the_first_line_is_not_shifted() {
    let para = aift_first_paragraph();
    assert_eq!(
        para.line_seg_text_start(0),
        0,
        "첫 줄을 올렸다 — 0 은 두 축에서 같은 자리라 건드리면 안 된다."
    );
}

/// 보정폭이 0 인 문단(HWP5·HWP3·HML 출처)은 투영이 항등이다 — 과잉 적용 방지.
#[test]
fn a_zero_shift_projection_is_the_identity() {
    use rhwp::model::paragraph::{LineSeg, Paragraph};

    let para = Paragraph {
        line_segs: vec![
            LineSeg {
                text_start: 0,
                ..Default::default()
            },
            LineSeg {
                text_start: 48,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert_eq!(para.line_seg_text_start(0), 0);
    assert_eq!(
        para.line_seg_text_start(1),
        48,
        "보정폭이 0 인데 투영이 값을 바꿨다 — HWP5 출처 문단까지 축을 옮기면 안 된다."
    );
}

/// 보정폭이 실린 문단은 그만큼만 올린다 — 첫 줄(0)은 그대로.
#[test]
fn the_projection_adds_exactly_the_recorded_shift() {
    use rhwp::model::paragraph::{LineSeg, Paragraph};

    let para = Paragraph {
        hwpx_axis_shift: 8,
        line_segs: vec![
            LineSeg {
                text_start: 0,
                ..Default::default()
            },
            LineSeg {
                text_start: 24,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert_eq!(
        para.line_seg_text_start(0),
        0,
        "문단 시작은 두 축에서 같은 자리다"
    );
    assert_eq!(para.line_seg_text_start(1), 32);
}

// 줄을 다시 계산하면 보정폭이 0 으로 지워진다는 계약(`replace_line_segs`)은 그 메서드가
// crate 내부라 여기서 직접 부를 수 없다. 계약은 해당 메서드의 주석이 지키고, 어겼을 때의
// 증상(두 번 더해진 축)은 위 투영 시험들이 잡는다.
