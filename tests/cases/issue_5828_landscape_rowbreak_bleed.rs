//! [Issue #5828] 가로 용지 RowBreak 표가 쪽마다 26행을 그려 용지를 121px 넘는다
//! (156505020 구역3, #1672 landscape bleed 누적 — rhwp 33쪽 vs 한글 43쪽의 주범).
//!
//! 근인: #1672 의 short-row 흡수(tolerance 260px)가 **누적 consumed** 에 계속
//! 적용돼, 경계를 넘어선 뒤에도 행을 받았다(가용 604.8px 에 26행=843px). 커밋
//! 의도는 "경계에 걸친 짧은 잔여 행 흡수" — 한글은 쪽마다 17행이다.
//!
//! 수정: **같은 높이 행의 연속 흡수 금지**(whole-row·short-row 두 흡수 분기
//! 공유). 한글 정합 문서(편람 383쪽 핀, #4763)의 연속 흡수는 전부 이질
//! 높이(38.7~280.2px)라 그대로 통과하고, 균일 pitch 기계 표만 쪽당 1행에서
//! 닫힌다. 원본 33→41쪽(한글 43; 잔여 −2는 #5751 행 성장 축, PR #5827 대기),
//! 표 조각 하한 914.7→687.7px(본문 699.3 안).
//!
//! 픽스처는 원본 HWP5 구역3 문단 0..6(가로 용지 대형 표) 절단 + 스텁(26.6KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5828/landscape_rowbreak_bleed.hwp";

#[test]
fn issue_5828_landscape_rowbreak_absorbs_only_one_boundary_row() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 결함 시 쪽마다 26행을 얹어 쪽수가 줄고(≈10쪽) 표가 용지 밖까지 내려간다.
    let pages = core.page_count();
    assert!(
        pages >= 13,
        "쪽당 행 수가 한글(17행)에 준해 표가 13쪽 이상으로 나뉘어야 한다: {pages}"
    );

    // 각 쪽 표 조각이 본문 하한(699.3px, 가로 용지) + 경계 행 1개(#1672 가
    // 허용하는 bleed, 행 pitch 32.4px) 안에서 끝나야 한다. 결함 시 연속 흡수로
    // 914.7px 까지 내려갔다.
    for p in 0..pages.min(6) {
        let svg = core.render_page_svg_native(p).expect("page svg");
        let mut max_line_y: f64 = 0.0;
        for cap in svg.split("<line ").skip(1) {
            let head = &cap[..cap.find('>').unwrap_or(cap.len())];
            if let Some(y) = head
                .split_once("y1=\"")
                .and_then(|(_, rest)| rest.split_once('"'))
                .and_then(|(v, _)| v.parse::<f64>().ok())
            {
                max_line_y = max_line_y.max(y);
            }
        }
        assert!(
            max_line_y <= 732.0,
            "p{} 표 괘선이 본문 하한(699.3)+경계 행 1개(32.4) 안이어야 한다 \
             (결함 시 914.7): {max_line_y:.1}",
            p + 1
        );
    }
}
