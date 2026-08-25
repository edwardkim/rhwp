//! [Issue #5886] 2단 미주 본문이 용지 밖까지 그려져 풀이 줄이 소실된다.
//!
//! `samples/3-09월_교육_통합_2022.hwpx` 12쪽: 한글은 `[알짜 풀이]`·`ㄴ. [참]`·
//! `ㄷ. [참]` 을 단 안에 두는데, rhwp 는 문단-사이 vpos 되감김을 겹침 높이로
//! 계상한 채 순차 적층해 y=1162~1290 (용지 1122.5) 에 그린다. 같은 글자는
//! 다른 쪽에도 없어 인쇄·PDF 에서 사라진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/3-09월_교육_통합_2022.hwpx";
const PAGE: u32 = 11; // 12쪽 (0-based)

fn svg_text_concat(svg: &str) -> String {
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out
}

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

fn svg_page_height(svg: &str) -> f64 {
    let i = svg.find("height=\"").expect("svg height") + 8;
    let rest = &svg[i..];
    let e = rest.find('"').expect("svg height end");
    rest[..e].parse::<f64>().expect("svg height parse")
}

#[test]
fn issue_5886_endnote_column_stays_on_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert!(
        core.page_count() >= 12,
        "12쪽이 있어야 한다, got {}",
        core.page_count()
    );

    let page12 = core.render_page_svg_native(PAGE).expect("page 12 svg");
    let page13 = core
        .render_page_svg_native(PAGE + 1)
        .unwrap_or_else(|_| String::new());
    let page_h = svg_page_height(&page12);
    let max_y = text_baselines(&page12).into_iter().fold(0.0_f64, f64::max);

    assert!(
        max_y <= page_h + 24.0,
        "12쪽 글자 baseline({max_y:.1})이 용지({page_h:.1}) 근처여야 한다 \
         — 수정 전 알짜 풀이 줄이 1290px 까지 나가 소실됐다"
    );

    let visible = format!("{}{}", svg_text_concat(&page12), svg_text_concat(&page13));
    assert!(
        visible.contains("알짜") && visible.contains("ㄴ") && visible.contains("ㄷ"),
        "알짜 풀이·ㄴ·ㄷ 항목이 12·13쪽에 보여야 한다 — 용지 밖 전용 방출이면 소실"
    );
}
