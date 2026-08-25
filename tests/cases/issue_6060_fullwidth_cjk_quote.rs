//! [#6060] 휴먼명조·HY헤드라인M 전각 낫표 「」 를 반각으로 누르지 않는다.
//!
//! `is_halfwidth_cjk_quote` 가 U+300C/U+300D 를 글꼴 무관 반각 오버레이로 처리해
//! 돋움체(#2020 여권신청서) 는 맞지만, 휴먼명조·HY헤드라인M 에서는 한글이 전폭을
//! 쓴다. 공개 샘플 `samples/hwp3-sample16-hwp5.hwpx` 5쪽
//! `가)「국가를당사자로하는계약에관한법률시행령」` (휴먼명조 13pt) 에서 낫표→다음
//! 한글 간격이 전각(~13px) 이어야 한다. 반각 강제면 ~6.5px 로 붙는다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwp3-sample16-hwp5.hwpx";
const PAGE: u32 = 4; // 5쪽
const NEEDLE: &str = "「국가를당사자";

fn load_doc() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"))
}

fn parse_svg_attr(attrs: &str, key: &str) -> Option<f64> {
    let p = attrs.find(&format!("{key}=\""))?;
    let s = p + key.len() + 2;
    let e = attrs[s..].find('"')? + s;
    attrs[s..e].parse::<f64>().ok()
}

fn svg_line_with_text(svg: &str, needle: &str) -> Option<(String, Vec<(f64, String)>)> {
    let mut by_y: BTreeMap<i32, Vec<(f64, String)>> = BTreeMap::new();
    let mut i = 0;
    while i < svg.len() {
        let Some(rel) = svg[i..].find("<text ") else {
            break;
        };
        let abs = i + rel;
        let after = &svg[abs + 6..];
        let Some(close) = after.find('>') else {
            i = abs + 6;
            continue;
        };
        let attrs = &after[..close];
        let content_start = abs + 6 + close + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else {
            i = abs + 6;
            continue;
        };
        let content = &svg[content_start..content_start + end_rel];
        if let (Some(x), Some(y)) = (parse_svg_attr(attrs, "x"), parse_svg_attr(attrs, "y")) {
            let y_key = (y * 10.0).round() as i32;
            by_y.entry(y_key)
                .or_default()
                .push((x, content.to_string()));
        }
        i = content_start + end_rel + 7;
    }

    for (_y, mut chars) in by_y {
        chars.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let full: String = chars.iter().map(|(_, s)| s.as_str()).collect();
        if full.contains(needle) {
            return Some((full, chars));
        }
    }
    None
}

#[test]
fn issue_6060_human_myeongjo_corner_quote_keeps_fullwidth_gap() {
    let doc = load_doc();
    let svg = doc
        .render_page_svg_native(PAGE)
        .expect("render hwp3-sample16-hwp5 page 5 SVG");
    let (line_text, chars) =
        svg_line_with_text(&svg, NEEDLE).expect("5쪽 법령명 낫표 줄을 찾아야 함");

    let open_idx = chars
        .iter()
        .position(|(_, text)| text == "「")
        .expect("opening corner quote");
    let guk_idx = chars[open_idx + 1..]
        .iter()
        .position(|(_, text)| text == "국")
        .map(|idx| idx + open_idx + 1)
        .expect("Hangul after opening corner quote");
    let close_idx = chars
        .iter()
        .position(|(_, text)| text == "」")
        .expect("closing corner quote");
    let je_idx = chars[close_idx + 1..]
        .iter()
        .position(|(_, text)| text == "제")
        .map(|idx| idx + close_idx + 1)
        .expect("Hangul after closing corner quote");

    let open_gap = chars[guk_idx].0 - chars[open_idx].0;
    let close_gap = chars[je_idx].0 - chars[close_idx].0;
    // 휴먼명조 13pt 전각 ≈ 13px. 반각 오버레이면 ≈ 6.5px.
    assert!(
        open_gap >= 11.0 && close_gap >= 11.0,
        "휴먼명조 낫표는 전각 전진이어야 함: open_gap={open_gap:.2}, \
         close_gap={close_gap:.2}, line={line_text}"
    );
}
