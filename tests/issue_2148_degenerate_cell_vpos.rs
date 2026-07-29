//! Issue #2148/#2279: 셀 저장 lineSeg vpos 퇴화(다문단 전부 vpos=0) 신뢰 가드.
//!
//! 36399374 결재문서 pi=79 병합 셀(15문단, 전부 stored vpos=0·1줄)은 저장
//! extent(≈35px)가 줄높이 합(≈260px)을 물리적으로 담지 못한다. 종전에는
//! `trust_stored_cell_flow` 가 이 퇴화 흐름을 신뢰해 전 문단을 셀 상단 한
//! y 에 겹쳐 그렸다(한글 PDF 실측: 14.4pt 피치 순차 적층). extent 가 줄높이
//! 합의 절반에도 못 미치면 저장 흐름을 불신하고 누적 적층으로 폴백한다.

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue_2148_degenerate_cell_vpos.hwpx";

/// SVG 의 글자 단위 <text> 요소들을 (y, 문자) 로 수집한다.
fn collect_text_chars(svg: &str) -> Vec<(f64, String)> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(i) = rest.find("<text") {
        rest = &rest[i..];
        let Some(open_end) = rest.find('>') else {
            break;
        };
        let attrs = &rest[..open_end];
        let Some(close) = rest.find("</text>") else {
            break;
        };
        let content = &rest[open_end + 1..close];
        // y 는 `transform="translate(x,y)"` 또는 ` y="..."` 로 표현된다.
        let y_val = attrs
            .find("translate(")
            .and_then(|ts| {
                let tv = &attrs[ts + 10..];
                let comma = tv.find(',')?;
                let end = tv[comma + 1..]
                    .find(|c: char| c == ')' || c == ' ')
                    .map(|i| comma + 1 + i)?;
                tv[comma + 1..end].parse::<f64>().ok()
            })
            .or_else(|| {
                let ys = attrs.find(" y=\"")?;
                let yv = &attrs[ys + 4..];
                let ye = yv.find('"')?;
                yv[..ye].parse::<f64>().ok()
            });
        if let Some(y) = y_val {
            out.push((y, content.to_string()));
        }
        rest = &rest[close + 7..];
    }
    out
}

/// `needle` 문자열이 등장하는 줄(y 반올림 그룹)의 y 를 찾는다.
fn line_y_containing(chars: &[(f64, String)], needle: &str) -> f64 {
    use std::collections::BTreeMap;
    let mut lines: BTreeMap<i64, Vec<(f64, &str)>> = BTreeMap::new();
    for (y, c) in chars {
        lines
            .entry((*y * 2.0).round() as i64)
            .or_default()
            .push((*y, c));
    }
    for group in lines.values() {
        let joined: String = group.iter().map(|(_, c)| *c).collect();
        let compact: String = joined.chars().filter(|c| !c.is_whitespace()).collect();
        let needle_compact: String = needle.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.contains(&needle_compact) {
            return group[0].0;
        }
    }
    panic!("no rendered line contains: {needle}");
}

#[test]
fn issue_2148_degenerate_cell_paragraphs_stack_sequentially() {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse sample");
    let svg = doc.render_page_svg_native(0).expect("render page 1");
    let chars = collect_text_chars(&svg);
    assert!(
        chars.len() > 100,
        "too few text chars: {} (svg len={}, head={})",
        chars.len(),
        svg.len(),
        &svg[..svg.len().min(400)]
    );

    // 문서 순서상 뒤 문단일수록 아래에 그려져야 한다. 퇴화 vpos 신뢰 시
    // 전부 같은 y(셀 상단)로 붕괴한다.
    let y_15 = line_y_containing(&chars, "1.5 (안전우선)");
    let y_23 = line_y_containing(&chars, "2.3 (차선침범)");
    let y_242 = line_y_containing(&chars, "2.4.2 교차로 진입");
    let y_last = line_y_containing(&chars, "비추어 차가 접근");

    assert!(
        y_23 > y_15 + 10.0,
        "2.3 은 1.5 보다 아래여야 함: y_23={y_23:.1} y_15={y_15:.1}"
    );
    assert!(
        y_242 > y_23 + 10.0,
        "2.4.2 는 2.3 보다 아래여야 함: y_242={y_242:.1} y_23={y_23:.1}"
    );
    assert!(
        y_last > y_242 + 10.0,
        "마지막 줄은 2.4.2 보다 아래여야 함: y_last={y_last:.1} y_242={y_242:.1}"
    );

    // 16줄 총 스팬은 200px 이상 (한글 실측 ≈ 288px 스팬).
    assert!(
        y_last - y_15 > 200.0,
        "셀 줄 스팬 과소: {:.1}px",
        y_last - y_15
    );
}
