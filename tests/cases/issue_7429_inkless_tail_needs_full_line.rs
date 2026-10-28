//! [#7429] 쪽 나누기 앞 빈 문단은 **줄 높이가 온전히 들어갈 때만** 그 쪽에 놓인다.
//!
//! #6854 는 다음 문단이 쪽 나누기를 선언하면 잉크 없는 빈 문단의 넘침을 "한 줄 미만"까지
//! 이 쪽에 흡수했다. 한/글의 규칙이 아니었다 — `80168_regulatory_analysis` 152쪽에서 한/글은
//! 흡수하지 않고 쪽번호만 있는 153쪽을 만든다(넘침 +7.9px).
//!
//! # 기대값의 출처 — 한/글 2024 합성 실험
//!
//! `samples/issue7429/inkless_tail_synthetic.hwpx` 는 블록 14개다. 블록마다 [첫 문단(글자 크기
//! z), 한 줄 문단 39개, 빈 문단]이고, 다음 블록의 첫 문단이 쪽 나누기를 선언한다. z 만 바꿔 빈
//! 문단 줄의 넘침을 100 HWPUNIT 씩 조절했다. 생성기는
//! `mydocs/tech/investigations/issue-7429/make_inkless_tail_fixture.py`, 한/글 2024 PDF 는
//! `pdf/issue7429/inkless_tail_synthetic-2024.pdf`(27쪽)다.
//!
//! ```text
//!   블록  z     빈 문단 줄 넘침      한/글
//!   B1   1414   −100 HWPUNIT        같은 쪽
//!   B2   1476     −0.4              같은 쪽
//!   B3   1539   +100                빈 쪽
//!   B4~  …      +200 …              빈 쪽
//! ```
//!
//! B0(z=1000)은 첫 쪽이라 구역 정의 문단 한 줄이 더 있어 역시 넘친다(빈 쪽 0 기준 1).
//!
//! 넘침 0 은 들어가고 +100 HWPUNIT(1.3px)만 넘쳐도 빈 쪽이다 — 일반 적합 판정과 같다.
//! 문서가 다음 문단의 저장 vpos 로 쪽 끝을 적어 둔 경우(#7288)의 흡수는 그대로다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue7429/inkless_tail_synthetic.hwpx";

/// 한/글 2024 PDF 에서 본문 글자가 없는 쪽(0 기준).
const HANCOM_BODYLESS_PAGES: [u32; 12] = [1, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25];
const HANCOM_PAGE_COUNT: usize = 27;

fn svg_text(svg: &str) -> String {
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out
}

#[test]
fn empty_paragraph_before_a_page_break_needs_its_whole_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read fixture"))
        .expect("open fixture");
    let page_count = core.page_count() as u32;
    let bodyless: Vec<u32> = (0..page_count)
        .filter(|page| {
            let svg = core.render_page_svg_native(*page).expect("svg");
            let text: String = svg_text(&svg)
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '-')
                .collect();
            text.chars().all(|c| c.is_ascii_digit())
        })
        .collect();
    assert_eq!(
        (core.page_count() as usize, bodyless.as_slice()),
        (HANCOM_PAGE_COUNT, &HANCOM_BODYLESS_PAGES[..]),
        "한/글 2024 와 쪽 수·빈 쪽 위치가 달라야 할 이유가 없다 — 빈 문단 줄이 1.3px 만 넘쳐도 \
         한/글은 다음 쪽으로 넘긴다"
    );
}
