//! 위쪽 자동 쪽 번호(쪽 번호 매기기 가운데 위)의 세로 자리 — 한/글 인쇄 PDF 기준.
//!
//! 한/글(12.30 Mac) 인쇄는 쪽 번호 글자 칸(1em)의 위끝을 머리말 밴드의 위끝(위쪽 여백 선)에,
//! 머리말이 0이면 위쪽 여백의 가운데에 둔다. 아래쪽 번호가 아래쪽 여백 선(머리말·꼬리말 0이면
//! 아래쪽 여백 가운데)에 글자 칸 아래끝을 두는 것의 거울이다.
//!
//! 고정 문서는 한/글로 저장한 두 구역 A4이고, 두 구역 모두 가운데 위 `- n -`(10pt 함초롬바탕)다.
//! 한/글 PDF의 번호 기준선은 1구역(위쪽 10mm·머리말 30mm) 36.12pt, 2구역(위쪽 25mm·머리말 0mm)
//! 43.20pt다. 본문 첫 줄 기준선(121.92pt·79.44pt)은 rhwp와 이미 같다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const TOP: &str = "tests/fixtures/page_number_position/hancom-page-number-top.hwpx";
const PX_PER_PT: f64 = 96.0 / 72.0;

/// 쪽 SVG의 (글자, 기준선 y pt).
fn glyphs(core: &DocumentCore, page: u32) -> Vec<(String, f64)> {
    let svg = core.render_page_svg_native(page).expect("svg");
    svg.split("<text")
        .skip(1)
        .filter_map(|chunk| {
            let head = &chunk[..chunk.find('>')?];
            let y: f64 = head
                .split_once(" y=\"")?
                .1
                .split('"')
                .next()?
                .parse()
                .ok()?;
            let body = &chunk[chunk.find('>')? + 1..chunk.find("</text>")?];
            Some((body.to_string(), y / PX_PER_PT))
        })
        .collect()
}

#[test]
fn top_page_number_baseline_follows_hancom_print() {
    let core = DocumentCore::from_bytes(&std::fs::read(TOP).unwrap()).expect("open");
    for (page, number, body, expected) in [(0, "1", "본", 36.12), (1, "2", "본", 43.20)] {
        let glyphs = glyphs(&core, page);
        // 본문 `본문 n쪽`에도 숫자가 있어 가장 위의 글자를 고른다.
        let baseline = |text: &str| {
            glyphs
                .iter()
                .filter(|(glyph, _)| glyph == text)
                .map(|(_, y)| *y)
                .reduce(f64::min)
                .unwrap_or_else(|| panic!("{page}쪽에 {text} 없음: {glyphs:?}"))
        };
        let number = baseline(number);
        assert!(
            (number - expected).abs() < 0.3,
            "{page}쪽 번호 기준선 {number:.2}pt, 한/글 {expected}pt"
        );
        // 쪽 번호 줄표도 같은 기준선이고, 번호는 본문 첫 줄보다 위에 있다.
        assert!((baseline("-") - number).abs() < 0.01);
        assert!(number < baseline(body));
    }
}
