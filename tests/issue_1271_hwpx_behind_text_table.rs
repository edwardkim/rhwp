//! Issue #1271: HWPX 글뒤로 paper-anchored RowBreak 표가 본문 흐름을 밀어
//! 앞쪽 페이지와 바탕쪽 홀짝 적용을 함께 어긋나게 하는 회귀 가드.
//!
//! 재현 문서: `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`.
//! 한컴 PDF 기준 앞쪽 페이지 배치는 page 2 = MEMO, page 3 = 1주차 표지,
//! page 4 = 본문 시작이다. 현재 TypesetEngine 은 section 0 paragraph 3 의
//! 글뒤로 표를 `PartialTable` 로 분할해 page 2 를 차지시키므로, 이후 페이지가
//! 한 쪽씩 밀리고 Odd/Even 바탕쪽 적용도 반대로 보인다.

use std::fs;
use std::path::Path;

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let rel = "samples/hwpx/[2027] 온새미로 1 본교재.hwpx";
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"))
}

#[derive(Debug)]
struct SvgGlyph {
    text: String,
    x: f64,
    y: f64,
    font_size: f64,
}

fn svg_text_glyphs(svg: &str) -> Vec<SvgGlyph> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = svg[from..].find("<text ") {
        let tag = from + rel;
        from = tag + 6;

        let after = &svg[tag..];
        let Some(gt) = after.find('>') else { break };
        let attrs = &after[..gt];
        let Some(end_rel) = after[gt + 1..].find("</text>") else {
            break;
        };
        let content = &after[gt + 1..gt + 1 + end_rel];

        let Some(p) = attrs.find("font-size=\"") else {
            continue;
        };
        let start = p + 11;
        let Some(end) = attrs[start..].find('"') else {
            continue;
        };
        let Ok(font_size) = attrs[start..start + end].parse::<f64>() else {
            continue;
        };

        let x = {
            let Some(p) = attrs.find("x=\"") else {
                continue;
            };
            let start = p + 3;
            let Some(end) = attrs[start..].find('"') else {
                continue;
            };
            let Ok(x) = attrs[start..start + end].parse::<f64>() else {
                continue;
            };
            x
        };
        let y = {
            let Some(p) = attrs.find("y=\"") else {
                continue;
            };
            let start = p + 3;
            let Some(end) = attrs[start..].find('"') else {
                continue;
            };
            let Ok(y) = attrs[start..start + end].parse::<f64>() else {
                continue;
            };
            y
        };

        out.push(SvgGlyph {
            text: content.to_string(),
            x,
            y,
            font_size,
        });
    }
    out
}

#[test]
fn onsaemiro_front_matter_is_not_shifted_by_behind_text_table_fragment() {
    let doc = load_doc();

    let page2 = doc.dump_page_items(Some(1));
    assert!(
        page2.contains("\"MEMO\""),
        "PDF 기준 page 2 는 MEMO 쪽이어야 한다. 글뒤로 표 조각이 끼어 있으면 \
         이후 page_num 과 바탕쪽 홀짝이 밀린다.\n{page2}"
    );
    assert!(
        !page2.contains("PartialTable   pi=3 ci=0"),
        "section 0 paragraph 3 의 글뒤로 표는 본문 흐름을 차지하는 \
         PartialTable 로 분할되면 안 된다.\n{page2}"
    );

    let page3 = doc.dump_page_items(Some(2));
    assert!(
        page3.contains("Shape          pi=5 ci=0"),
        "PDF 기준 page 3 은 1주차 표지 도형들이 있는 쪽이어야 한다.\n{page3}"
    );
    assert!(
        !page3.contains("\"MEMO\""),
        "MEMO 쪽이 page 3 으로 밀리면 바탕쪽 홀짝 적용도 함께 어긋난다.\n{page3}"
    );

    let page4 = doc.dump_page_items(Some(3));
    assert!(
        page4.contains("section=1, page_num=4"),
        "PDF 기준 page 4 에서 section 1 본문이 page_num=4 로 시작해야 한다.\n{page4}"
    );
    assert!(
        page4.contains("\"강의 01.\""),
        "PDF 기준 page 4 는 강의 01 본문 시작 쪽이어야 한다.\n{page4}"
    );
}

#[test]
fn onsaemiro_master_page_bottom_textbox_is_not_microscopic() {
    let doc = load_doc();
    let svg = doc.render_page_svg_native(3).expect("render page 4");
    let glyphs = svg_text_glyphs(&svg);
    assert!(!glyphs.is_empty(), "page 4 SVG should contain text glyphs");

    let microscopic: Vec<&SvgGlyph> = glyphs
        .iter()
        .filter(|glyph| glyph.font_size < 1.0 && !glyph.text.trim().is_empty())
        .collect();

    assert!(
        microscopic.is_empty(),
        "바탕쪽 하단 글상자 글꼴이 1px 미만으로 붕괴함. \
         HWPX curSz 음수 래핑값을 확대율로 오해하면 재현된다: {:?}",
        microscopic
    );
}

#[test]
fn onsaemiro_odd_master_page_number_stays_after_title_text() {
    let doc = load_doc();
    let svg = doc.render_page_svg_native(4).expect("render page 5");
    let glyphs = svg_text_glyphs(&svg);

    let bottom_glyphs: Vec<&SvgGlyph> = glyphs
        .iter()
        .filter(|glyph| (1040.0..=1065.0).contains(&glyph.y))
        .collect();
    let page_number_x = bottom_glyphs
        .iter()
        .find(|glyph| glyph.text == "5")
        .map(|glyph| glyph.x)
        .expect("odd master page bottom textbox should render page number 5");
    let literature_last_x = bottom_glyphs
        .iter()
        .filter(|glyph| glyph.text == "학")
        .map(|glyph| glyph.x)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        page_number_x > literature_last_x,
        "홀수 바탕쪽 쪽번호는 '독서·문학' 뒤에 출력되어야 한다. \
         page_number_x={page_number_x}, literature_last_x={literature_last_x}, \
         bottom_glyphs={bottom_glyphs:?}"
    );
}
