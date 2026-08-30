//! [Issue #6409] 저장 vpos=0 쪽-상단 TAC 표가 앞쪽 leftover 에 끼어
//! 한글 42쪽 몫 ~200자를 41쪽으로 끌어온다
//! (`samples/issue6031/3249937_asset_management_rules.hwpx`).
//!
//! 붙임4 표(25×34, 한 줄 vertsize=57204)는 leftover 에 통째로 들어가 41쪽에
//! 남고, 바로 다음 부동산거래계약 신고서(39×22, treatAsChar, 한 줄
//! vertsize=69344)는 한글이 42쪽 상단에 둔다. leftover ~210px 에 신고서 앞
//! 11행을 끼우면 41쪽 본문 하단이 809pt 까지 내려간다.
//!
//! 수정: HWPX TAC 표가 쪽높이급 단일 LINE_SEG 로 저장됐고 그 높이가 잔여에
//! 안 들어가면, CellBreak 행 분할로 leftover 에 끼우지 않고 다음 쪽으로 이월한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6031/3249937_asset_management_rules.hwpx";

#[test]
fn issue_6409_form_table_starts_on_next_page_not_leftover() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(core.page_count(), 60, "한글 2020 정본은 60쪽이다");

    let p41 = core.render_page_svg_native(40).expect("page 41 svg");
    let p42 = core.render_page_svg_native(41).expect("page 42 svg");

    let p41_text = svg_glyphs(&p41);
    let p42_text = svg_glyphs(&p42);

    assert!(
        !p41_text.contains("신고서"),
        "41쪽 leftover 에 부동산거래계약 신고서가 끼면 안 된다: {}",
        clip(&p41_text, 120)
    );
    assert!(
        p42_text.contains("신고서") || p42_text.contains("부동산"),
        "42쪽 상단에 부동산거래계약 신고서가 있어야 한다: {}",
        clip(&p42_text, 120)
    );
}

fn svg_glyphs(svg: &str) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        if let Some(close) = chunk[tag_end + 1..].find("</text>") {
            out.push_str(&chunk[tag_end + 1..tag_end + 1 + close]);
        }
    }
    out
}

fn clip(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
