//! [Issue #6025] CELL 분할 조각의 쪽 채움 회계가 페인트보다 +24.6px 커서 1쪽에
//! 들어갈 "8. 라." 줄이 2쪽으로 밀리고 총 5쪽(한글 2020 4쪽)이 된다는 보고의
//! 회귀 핀 (`samples/issue6025/3232693_employment_support_criteria.hwpx`).
//!
//! 검증 결과 현행 devel 에서는 재현되지 않는다 — 보고 base(385e93b2c) 이후의
//! 랜딩들이 닫았다. 현행 실측: 총 4쪽, 1쪽 마지막 글줄 = '라. 국민행복기금…'
//! baseline 1059.8px(794.8pt) — **한글 실측 y0=794.8pt 와 정확 일치**. 이 핀은
//! 그 계약(마지막 줄의 1쪽 소유 + 총 쪽수)을 고정한다. #5782/#6013 과 같은
//! "조각 회계 vs 페인트" 축이라 재발 시 즉시 잡힌다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6025/3232693_employment_support_criteria.hwpx";

#[test]
fn issue_6025_la_line_stays_on_first_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(
        core.page_count(),
        4,
        "한글 2020 정본은 4쪽이다 (보고 결함 시 5쪽)"
    );

    // 1쪽 마지막 글줄 = '라. 국민행복기금…' (한글 y0=794.8pt=1059.8px).
    // 보고 결함 시 '있는 자로서…'(다.의 둘째 줄, 770.8pt)로 끝난다.
    let p1 = core.render_page_svg_native(0).expect("page 1 svg");
    let tail = text_glyphs_in_band(&p1, 1050.0, 1070.0);
    for needle in ['라', '국', '민', '행', '복'] {
        assert!(
            tail.contains(needle),
            "1쪽 말미(1050~1070px)에 '라. 국민행복기금…' 글리프({needle})가 있어야 한다: {tail:?}"
        );
    }
}

fn text_glyphs_in_band(svg: &str, y_min: f64, y_max: f64) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let Some(y) = attr(&chunk[..tag_end], "y") else {
            continue;
        };
        if y < y_min || y > y_max {
            continue;
        }
        if let Some(close) = chunk[tag_end + 1..].find("</text>") {
            out.push_str(&chunk[tag_end + 1..tag_end + 1 + close]);
        }
    }
    out
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
