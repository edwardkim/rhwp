//! [#8079] `고딕` HFT 프로그램의 ASCII 는 반각(0.5em) 고정폭으로 잰다.
//!
//! ## 무엇이 문제였나
//!
//! HWP5/HWPX 가 영문 슬롯에 `고딕`(HFT)을 선언하면 rhwp 는 표시 대체 글꼴 `돋움`의 비례 폭으로
//! 영문을 쟀다. 한/글은 원본 HFT 프로그램(Type3)으로 그리며, 그 ASCII 95자는 모두 500/1000 이다.
//! 줄 끝 공백 압축을 없애자(#8079 규칙 2) `hwp3-sample11-hwpx` 66쪽 `A.V.Aho, P.J. Weinberger …`
//! 줄이 넓은 영문 폭 때문에 PDF 보다 길어졌다.
//!
//! ## 정본 실측
//!
//! ```text
//!   pdf/hwp3-sample11-hwpx-hwpx-2020.pdf · pdf/hwp3-sample11-hwp-2020.pdf 66쪽
//!     영문 run 의 Type3 글꼴: 문자 코드 0x20–0x7E glyph 폭 95개 모두 500/1000
//!     글자 원점 간격 0.494–0.506em (600dpi 격자)
//!   pdf/synth_cell_enter_table_growth-2020.pdf 1쪽 (이 시험의 fixture, 한컴 2020 Print)
//!     `*` 원점 228.32px → `렵` 234.40px = 6.08px (9pt 의 0.5067em)
//!     수정 전 rhwp 6.99px (돋움 비례 폭) / 수정 후 6.03px
//! ```
//!
//! ## 적용하지 않는 경우
//!
//! HFT 로 선언하지 않은 `고딕`(HWP3 원본 경로, alt_type 0)은 종전대로 반각 휴리스틱을 쓴다.
//! 한글은 이 표에 없어 표시 face 폭을 그대로 쓴다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6882/synth_cell_enter_table_growth.hwp";

struct Glyph {
    ch: char,
    x: f64,
    font_size: f64,
    ratio: f64,
    letter_spacing: f64,
}

fn page_glyphs(page: u32) -> Vec<Glyph> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let raw = core.get_page_text_layout_native(page).expect("text-layout");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
    let mut glyphs = Vec::new();
    for run in layout["runs"].as_array().into_iter().flatten() {
        let x = run["x"].as_f64().unwrap_or(0.0);
        let char_x: Vec<f64> = run["charX"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_f64)
            .collect();
        for (i, ch) in run["text"].as_str().unwrap_or_default().chars().enumerate() {
            glyphs.push(Glyph {
                ch,
                x: x + char_x.get(i).copied().unwrap_or(0.0),
                font_size: run["fontSize"].as_f64().unwrap_or(0.0),
                ratio: run["ratio"].as_f64().unwrap_or(1.0),
                letter_spacing: run["letterSpacing"].as_f64().unwrap_or(0.0),
            });
        }
    }
    glyphs
}

#[test]
fn hft_gothic_ascii_advances_by_half_em() {
    let glyphs = page_glyphs(0);
    let at = glyphs
        .windows(2)
        .position(|pair| pair[0].ch == '*' && pair[1].ch == '렵')
        .expect("`*렵` 이 1쪽에 있어야 한다");
    let star = &glyphs[at];
    let advance = glyphs[at + 1].x - star.x;
    // 자간은 글자 전진 비례(#2279)라 반각 글자에는 자간px 의 절반이 붙는다.
    let half_em = star.ratio * 0.5 * (star.font_size + star.letter_spacing);
    assert!(
        (advance - half_em).abs() < 0.15,
        "`*` 전진 {advance:.3}px — 고딕 HFT 반각 {half_em:.3}px 이어야 한다"
    );
}
