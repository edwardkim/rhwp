//! [Issue #5755] 1쪽 아래에서 쪽을 안 넘겨 문단 6줄이 본문 칸 밖·용지 밖에 그려진다.
//!
//! 156677324: pi=9(6줄, 199.2px)가 1쪽 잔여 137.2px 에 통째로 붙어 used=995.6px 로
//! 본문(933.6px)을 62px 넘기고, 마지막 두 줄은 용지 밖(+54.6px)까지 나갔다.
//! 원본 저장 lineseg 는 pi=9 앞에서 vertpos 가 65552→1500 으로 **되감겨** 한글이
//! 여기서 쪽을 끊었음을 선언하는데, split 경로의 저장-꼬리 적합(saved_tail_vpos_fit)이
//! 그 새 쪽 쪽-지역 좌표를 현재 쪽 꼬리 좌표로 오독해 전 줄을 통과시켰다.
//!
//! 수정: 저장 vpos 되감김 + 전체 fit 실패면 split 전에 쪽을 넘긴다
//! (`stored_vpos_rewind_overflow_break`). 한글 배치 = 2쪽 199.2+725.9=925.1≤933.6.
//!
//! 픽스처 `samples/issue5755/rewind_overflow_page_break.hwpx` 는 원본에서 대형 이미지
//! 3장을 1×1 스텁으로 바꾼 축소본(26KB) — 원본과 같은 쪽 기하를 재현한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5755/rewind_overflow_page_break.hwpx";

/// SVG 는 글자 단위 `<text>` 로 방출된다 — 순서대로 이어 붙여 문구를 찾는다.
fn svg_text_concat(svg: &str) -> String {
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out
}

/// SVG 의 모든 `<text … y="…">` baseline 을 모은다.
fn text_baselines(svg: &str) -> Vec<f64> {
    let mut out = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let Some(ys) = head.find("y=\"") else {
            continue;
        };
        let s = ys + 3;
        if let Some(e) = head[s..].find('"') {
            if let Ok(y) = head[s..s + e].parse::<f64>() {
                out.push(y);
            }
        }
    }
    out
}

#[test]
fn issue_5755_rewound_overflow_paragraph_moves_whole_to_next_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 3, "한글 2022 와 같은 3쪽이어야 한다");

    let page1 = core.render_page_svg_native(0).expect("page 1 svg");
    let page2 = core.render_page_svg_native(1).expect("page 2 svg");

    // 되감긴 문단("법인이 다른 사립대학…")은 통째로 2쪽에 있어야 한다.
    let page1_text = svg_text_concat(&page1);
    let page2_text = svg_text_concat(&page2);
    assert!(
        !page1_text.contains("법인이"),
        "되감긴 pi=9 문단이 1쪽에 남아 있다 — 저장 vpos 리셋(65552→1500)을 무시한 것"
    );
    assert!(
        page2_text.contains("법인이"),
        "되감긴 pi=9 문단이 2쪽에 통째로 있어야 한다(한글 2022 배치)"
    );

    // 1쪽 글자 baseline 이 본문 칸(y=94.5+933.6=1028.1) 안에 있어야 한다.
    // 결함 시 1027.8/1057.7/…/1177.1 로 용지(1122.5) 밖까지 나갔다.
    let max_y = text_baselines(&page1).into_iter().fold(f64::MIN, f64::max);
    assert!(
        max_y <= 1028.6,
        "1쪽 최대 글자 baseline({max_y:.1})이 본문 칸 바닥(1028.1) 안이어야 한다"
    );
}
