//! [#2525] 비마스킹 단일-lineseg 과밀 문서(기계생성 HWPX)의 본문 줄을
//! rhwp 가 한 줄에 욱여넣어 음수 자간으로 압축 → SVG 글리프 겹침(숫자
//! advance 0.5× 클램프)이 발생하던 회귀 가드.
//!
//! 정본(한컴 COM PDF, 2026-07-21): `samples/hwpx/hwpx-02.hwpx` p0 숫자
//! "2024" advance = 0.590em, 첫 본문 줄 ≈ 30자로 재래핑. 수정 전 rhwp 는
//! 110자를 한 줄에 넣고 숫자 advance 0.297em(char_px*ratio*0.5 최소 클램프)
//! 으로 압축 → 겹침. 근인: `stored_lines_overflow`(composer.rs)의 오버플로우
//! 재래핑이 마스킹(`*`) 문서에만 발동 → 비마스킹 과밀 제외. 수정: 장평 반영
//! 실폭이 내폭 ≥1.8× 인 저장 lineseg 도 fresh 재래핑.

use std::path::Path;

fn render_page_svg(rel: &str, page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).expect("read sample");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    doc.render_page_svg(page).expect("render svg")
}

/// 같은 baseline(y) 을 공유하는 <text> 글리프 최대 개수.
/// 과밀(한 줄에 전 문단) 이면 매우 큼(수정 전 110), 재래핑되면 작음(≈30).
fn max_glyphs_on_one_baseline(svg: &str) -> usize {
    use std::collections::HashMap;
    let mut by_y: HashMap<String, usize> = HashMap::new();
    let mut rest = svg;
    while let Some(p) = rest.find("<text x=\"") {
        rest = &rest[p + 8..];
        if let Some(yp) = rest.find(" y=\"") {
            let after = &rest[yp + 4..];
            if let Some(end) = after.find('"') {
                let y = after[..end].to_string();
                *by_y.entry(y).or_insert(0) += 1;
            }
        }
    }
    by_y.values().copied().max().unwrap_or(0)
}

#[test]
fn hwpx02_single_lineseg_paragraph_is_rewrapped_not_crammed() {
    let svg = render_page_svg("samples/hwpx/hwpx-02.hwpx", 0);
    let max_line = max_glyphs_on_one_baseline(&svg);
    // 한컴 정본 첫 줄 ≈ 30자. 수정 전 crammed = 110. 60 미만이면 재래핑됨.
    assert!(
        max_line < 60,
        "본문 단일-lineseg 문단이 재래핑되지 않고 한 줄에 과밀 배치됨: \
         한 baseline 최대 {max_line}자 (한컴 정본 ≈30, 수정 전 110). #2525"
    );
}
