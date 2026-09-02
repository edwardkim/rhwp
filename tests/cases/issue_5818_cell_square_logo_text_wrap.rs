//! [Issue #5818] 머리 표 로고 옆 글자가 로고를 32.8px 파고들어 겹친다 —
//! 그림 가로 폭만큼 글자를 밀지 않는다 (156599239 1쪽).
//!
//! 근인: 셀 문단 줄의 저장 LINE_SEG cs/sw 존중 게이트가 전부 `cell_ctx.is_none()`
//! 이라, 셀 안 어울림(Square) 로고의 wrap 배제(한컴 저장 cs=4037HU=53.8px)가
//! 무시되고 글자가 셀 왼끝에서 시작했다(rhwp 102.2 ↔ 한글 151.7).
//!
//! 수정: 같은 셀에 Square 계열 float 그림/도형이 실재할 때만(#547 문단 테두리
//! inset 오인 차단) 셀 줄도 저장 cs/sw 를 존중한다.
//!
//! 픽스처는 원본 HWPX 구역0 문단 0..2 절단 + BinData 1×1 스텁 축소본(69KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5818/cell_square_logo_text_wrap.hwpx";

#[test]
fn issue_5818_cell_text_starts_after_square_logo() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 로고(x=86.9, w=48.2 → 오른쪽 끝 135.1) 옆 `경 찰 대 학` 첫 글자 x.
    // 결함 시 102.2(로고 안쪽 32.8px), 정상 156.1(한글 151.7 — 로고 오른쪽 끝
    // 뒤에서 시작).
    let logo_right = 86.9 + 48.2;
    let mut first_gyeong_x = f64::MAX;
    for cap in svg.split("<text ").skip(1) {
        let Some(end) = cap.find("</text>") else {
            continue;
        };
        let node = &cap[..end];
        if !node.ends_with(">경") {
            continue;
        }
        if let Some(x) = node
            .split_once("x=\"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .and_then(|(v, _)| v.parse::<f64>().ok())
        {
            first_gyeong_x = first_gyeong_x.min(x);
        }
    }
    assert!(first_gyeong_x < f64::MAX, "경 글자가 있어야 한다");
    assert!(
        first_gyeong_x > logo_right + 5.0,
        "글자가 로고 오른쪽 끝({logo_right:.1}) 뒤에서 시작해야 한다 \
         (한글 151.7, 결함 시 102.2): {first_gyeong_x:.1}"
    );
    assert!(
        (150.0..162.0).contains(&first_gyeong_x),
        "저장 cs(53.8px) 자리여야 한다: {first_gyeong_x:.1}"
    );
}
