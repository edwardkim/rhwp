//! 관계형·고정형: 원본 HWPX의 dash 글리프·전진과 셀 내부 포함을 보호한다.
//! 주석 갱신: 2026-11-22T03:03:17+09:00; 직전 파일 변경 커밋: 2026-10-04T19:03:36+09:00 731de9e1b4bb946d76f35108ed7e186ebe4ebecb.
//! 측정: 2026-11-22 KST, Print profile, compat2024, 96DPI, PDFium, 보정 전2px raw.
//! 전64쪽 최저 Native98.69% / fresh WASM98.69%; 쪽수도 독립 Print와 같다.
//! 대상33쪽 Native99.99/WASM99.98%,34쪽 양쪽99.97%. 같은 영역의 직접 판독 근거는 source32/README.md.
//! PDF33쪽 dash 전진0.49857~0.56977em,34쪽0.56960~0.57260em을 독립 재측정했다.
//! 근거: mydocs/pr/assets/issues7754_7765/source32/README.md와 issue7765-hwpx의 전쪽 TSV.
//! 실제 선택 프로그램 metadata만 공급하고 시스템 글꼴·전체 SVG hash·고정 px 영역을 요구하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
mod regulatory_svg {
    include!("../common/regulatory_dash_svg.rs");
}
use regulatory_svg::{hyphen_glyphs, Run};
fn render_page_svg() -> String {
    regulatory_svg::render_page_svg(33)
}
fn hyphen_runs(svg: &str) -> Vec<Run> {
    regulatory_svg::hyphen_runs(svg, 4)
}
fn widest_run(runs: &[Run]) -> &Run {
    runs.iter()
        .max_by(|a, b| {
            let width = |r: &Run| r.pts.last().unwrap().0 - r.pts[0].0;
            width(a).total_cmp(&width(b))
        })
        .expect("최대 dash 런")
}

#[test]
fn dash_leader_is_drawn_as_glyphs_not_a_line() {
    let svg = render_page_svg();
    let glyphs = hyphen_glyphs(&svg);
    assert!(
        glyphs.len() >= 60,
        "개정문 하이픈은 글자로 그려야 한다 — 이 쪽의 하이픈 글자 수 실측 {} (수정 전 2)",
        glyphs.len()
    );
}

#[test]
fn dash_leader_run_keeps_the_elastic_advance() {
    // 새 정본 p34의 28자 런: x 311.88 → 528.94pt, advance 약 8.04pt,
    // font-size 14.04pt → 0.573em. SVG는 96dpi, em 비율로 비교한다.
    let svg = render_page_svg();
    let runs = hyphen_runs(&svg);
    assert!(!runs.is_empty(), "하이픈 런을 하나도 못 찾았다");

    let widest = widest_run(&runs);
    let font_size = widest.font_size();
    let advance = widest.advance();
    let em = advance / font_size;
    assert!(
        (0.50..=0.65).contains(&em),
        "탄력 leader 의 글자당 advance 가 정본 범위(0.58 em 안팎)를 벗어났다: {em:.3} em \
         — 레이아웃의 슬랙 분배(extra_dash_advance)가 깨졌을 수 있다"
    );
}

#[test]
fn dash_leader_no_longer_emits_a_substitute_rule() {
    // 치환 라인은 글리프와 같은 줄, baseline 바로 위(`-font_size * 0.32`)에 그려졌었다.
    // 글자가 살아난 뒤에도 라인이 같이 남으면 이중선이 된다.
    //
    // 표 괘선도 같은 x 구간을 가로지르므로 x 만으로 판정하면 오탐이 난다 — 이 쪽의
    // 괘선은 y=75.9 · 108.0 · 466.4 이고 런 baseline 은 192.0 이다. 따라서 **그 줄 높이에
    // 놓인 가로선만** 본다.
    let svg = render_page_svg();
    let runs = hyphen_runs(&svg);
    let widest = widest_run(&runs);
    let (x1, x2) = (widest.pts[0].0, widest.pts[widest.pts.len() - 1].0);
    let font_size = widest.font_size();
    let band = font_size * 0.6;

    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let attrs = &chunk[..end];
        let pick = |key: &str| -> Option<f64> {
            let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
            let tail = &attrs[at..];
            let stop = tail.find('"')?;
            tail[..stop].parse::<f64>().ok()
        };
        let (Some(lx1), Some(ly1), Some(lx2), Some(ly2)) =
            (pick("x1"), pick("y1"), pick("x2"), pick("y2"))
        else {
            continue;
        };
        let horizontal = (ly1 - ly2).abs() < 0.5;
        let on_this_line = (ly1 - widest.baseline).abs() <= band;
        let covers = lx1 <= x1 + 1.0 && lx2 >= x2 - 1.0;
        assert!(
            !(horizontal && on_this_line && covers),
            "하이픈 런 자리에 치환 라인이 그대로 남았다: \
             line ({lx1:.1},{ly1:.1})→({lx2:.1},{ly2:.1}), 런 {x1:.1}→{x2:.1} baseline {:.1}",
            widest.baseline
        );
    }
}
