//! [Issue #5872] leader 없는 RIGHT 인라인 탭이 저장 width 를 버리고 본문 우측 끝에
//! 정렬해 목차 개요번호가 쪽번호 위에 겹친다 (113424 6쪽 7줄).
//!
//! 원본 줄은 `\t I. \t 총 칙 \t 1` 처럼 탭이 셋이다. 첫 탭(RIGHT, leader 없음)은
//! **줄 앞머리 정렬 탭**인데, 종전 코드가 leader 있는 RIGHT 탭(목차 쪽번호)과
//! 같은 갈래로 보내 본문 우측 끝으로 끌어갔다 — 로마숫자가 x≈709 로 가서 쪽번호와
//! 포개졌다(한글 x≈101).
//!
//! 수정: leader 없는 RIGHT 탭 **뒤에 탭이 더 있으면** 줄 중간 정렬 탭이므로 한컴이
//! 저장한 `width`(정렬을 이미 마친 전진 거리)를 그대로 쓴다. 줄 끝의 RIGHT 탭은
//! 종전대로 우측 끝 정렬. 실측: I@101.1 / II 의 I@96.3 (한글 101.0 / 96.2).
//!
//! 픽스처는 원본 HWPX 구역1 문단 20..32(목차 대항목) 절단 + 스텁(36KB) — 목차는
//! 2쪽에 온다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5872/toc_midline_right_tab.hwpx";

#[test]
fn issue_5872_midline_right_tab_keeps_outline_number_left() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(1).expect("page 2 svg");

    // 로마숫자 글리프(I/V)의 x 를 모은다.
    let mut roman_x: Vec<f64> = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(head_end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..head_end];
        let Some(body_end) = chunk.find("</text>") else {
            continue;
        };
        let body = &chunk[head_end + 1..body_end];
        if body != "I" && body != "V" {
            continue;
        }
        let Some(x) = head
            .split_once("translate(")
            .and_then(|(_, r)| r.split_once(','))
            .and_then(|(v, _)| v.trim().parse::<f64>().ok())
        else {
            continue;
        };
        roman_x.push(x);
    }
    assert!(
        !roman_x.is_empty(),
        "목차 로마숫자 글리프를 찾아야 검증이 유효하다"
    );

    // 결함 시 전부 본문 우측 끝(≈690~710)으로 끌려간다.
    let rightmost = roman_x.iter().copied().fold(f64::MIN, f64::max);
    assert!(
        rightmost < 200.0,
        "개요번호가 쪽번호 자리로 끌려갔다 (결함 시 ≈709): {roman_x:?}"
    );
    // 한글 실측 101.0 / 96.2 근방.
    assert!(
        roman_x.iter().any(|x| (98.0..=104.0).contains(x)),
        "첫 대항목 로마숫자가 x≈101 이어야 한다: {roman_x:?}"
    );
}
