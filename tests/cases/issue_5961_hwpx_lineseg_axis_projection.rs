//! #5961: HWPX 저장 축과 명시적으로 기록한 소비 시점 투영의 계약.
//!
//! 저장 `text_start`는 파일 값 그대로 보존한다. 명시적 rhwp-origin 생산자가 생략한
//! 슬롯은 `hwpx_axis_shift`로 투영하지만 일반 한컴 원본의 `secPr` 존재만으로
//! 구역 슬롯 생략을 추정하지 않는다. raw 값을 바꾸면 x2x 왕복에서 축이 누적 이동한다.
//!
//! 원 PR #5970의 한컴 2024 캐럿 관측과 원본 저장 버전에 맞는 한컴 2020 Print를
//! 구분한다. 같은 대조 원본의 2020 Print 둘째 줄은 본문 인덱스 46에서 시작하며
//! 현재 Native/fresh WASM 전체 1쪽 raw 일치율이 각각 100%다. 독립 출처·SHA와
//! 직접 판독은 `mydocs/working/task_m100_7642_stage3885.md`에 연결했다.
//!
//! `aift` raw 24(컨트롤 슬롯)와 본문 시작 32는 같은 첫 표시 글자로 대응하므로
//! 이 사실로 보정폭 8을 강제하지 않는다. 파일 값·표시 글자 소유 관계와 별도로,
//! 명시적 투영폭·문단 끝 경계의 일반 계약도 아래에서 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

/// `aift.hwpx` 문단 0 의 슬롯 사다리는 `secd@0 · cold@8 · tbl@16 · pgnp@24 · text@32` 이고
/// `char_count` 는 56(= 32 + 텍스트 23 + 문단부호)이다. 둘째 줄 raw `textpos` 24는
/// 쪽번호 제어의 경계이며, 별도 명시적 축 마커가 없는 원본 값을 보존한다.
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

/// 일반 한컴 원본의 명시적 구역 정의를 생략된 슬롯으로 세지 않는다.
#[test]
fn unmarked_hwpx_section_keeps_its_explicit_control_axis() {
    let para = aift_first_paragraph();
    assert_eq!(
        para.hwpx_axis_shift, 0,
        "명시적 생산자 축 마커가 없는 원본에 구역 슬롯 생략을 추정하면 안 된다"
    );
}

/// 저장된 컨트롤 경계와 첫 표시 글자는 같은 줄에 속한다.
#[test]
fn stored_control_boundary_keeps_the_first_visible_character_on_its_line() {
    let para = aift_first_paragraph();
    let start = para.line_seg_text_start(1);
    assert_eq!(
        start, para.line_segs[1].text_start,
        "일반 원본의 저장 컨트롤 경계를 임의의 다른 축으로 옮기면 안 된다"
    );
    assert!(
        start <= *para.char_offsets.first().expect("원본의 본문 시작"),
        "줄 머리 컨트롤 경계가 첫 표시 글자 뒤로 이동했다"
    );
    let first_visible = para.char_offsets.partition_point(|&offset| offset < start);
    assert_eq!(
        first_visible, 0,
        "저장 줄에 속한 첫 표시 글자를 이전 줄로 넘기면 안 된다"
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

/// 올린 값이 문단 끝을 넘으면 올리지 않는다 — 이미 HWP5 축인 줄의 증거다.
///
/// `issue5595_rotated_picture_topbottom.hwpx` 문단 0 의 실측 형상: 컨트롤 3개에
/// `char_count=25`, 줄은 `[0, 24]`. 둘째 줄은 이미 컨트롤 3개(24유닛)를 세고 있어
/// 보정폭 16 을 얹으면 40 — 문단 끝을 15 넘는다. 그대로 올리면 저장기가 그 줄을
/// 축 밖으로 보고 떨궈 `convert --verify` 가 줄 수 손실(2→1)로 잡는다.
#[test]
fn the_projection_stops_at_the_paragraph_end() {
    use rhwp::model::paragraph::{LineSeg, Paragraph};

    let para = Paragraph {
        hwpx_axis_shift: 16,
        char_count: 25,
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
    assert_eq!(para.line_seg_text_start(0), 0);
    assert_eq!(
        para.line_seg_text_start(1),
        24,
        "문단 끝(25)을 넘는 투영은 그 줄이 이미 HWP5 축이라는 뜻이다"
    );
}

/// 끝 마커를 가리키는 `text_start == char_count` 는 정상이므로 그대로 올린다.
#[test]
fn the_projection_allows_landing_exactly_on_the_end_marker() {
    use rhwp::model::paragraph::{LineSeg, Paragraph};

    let para = Paragraph {
        hwpx_axis_shift: 16,
        char_count: 25,
        line_segs: vec![
            LineSeg {
                text_start: 0,
                ..Default::default()
            },
            LineSeg {
                text_start: 9,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert_eq!(para.line_seg_text_start(1), 25);
}

// 줄을 다시 계산하면 보정폭이 0 으로 지워진다는 계약(`replace_line_segs`)은 그 메서드가
// crate 내부라 여기서 직접 부를 수 없다. 계약은 해당 메서드의 주석이 지키고, 어겼을 때의
// 증상(두 번 더해진 축)은 위 투영 시험들이 잡는다.
