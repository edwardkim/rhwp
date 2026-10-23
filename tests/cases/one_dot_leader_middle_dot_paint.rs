//! 가운뎃점으로 쓴 `․`(U+2024)이 기준선 마침표로 그려진다.
//!
//! 한국어 문서는 `주거․수도․광열`처럼 가운뎃점을 U+2024 로 적기도 한다(기호 슬롯).
//! 한컴 글꼴은 이 글자를 `·`(U+00B7)처럼 x-height 가운데에 그리는데, 대체 글꼴의
//! U+2024 는 ONE DOT LEADER 라 기준선에 붙어 `주거.수도`처럼 보였다. U+00B7 과 같은
//! 벡터 점(전진 상자 가운데, 기준선 위 0.35em)으로 그린다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn page_svg(text: &str) -> String {
    let mut core = DocumentCore::new_empty();
    {
        let mut section = rhwp::model::document::Section::default();
        section.section_def.page_def = rhwp::model::page::PageDef::a4_default();
        section
            .paragraphs
            .push(rhwp::model::paragraph::Paragraph::new_empty());
        let mut document = rhwp::model::document::Document::default();
        document.sections.push(section);
        core.set_document(document);
    }
    core.insert_text_native(0, 0, 0, text).expect("글 삽입");
    core.render_page_svg_native(0).expect("SVG")
}

fn attr(tag: &str, name: &str) -> f64 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).expect("속성") + key.len();
    let end = start + tag[start..].find('"').expect("속성 끝");
    tag[start..end].parse().expect("숫자")
}

/// `(cy, baseline y)` — 그린 점과 같은 자리에 겹친 투명 원문 글자의 기준선.
fn painted_dots(svg: &str, dot: char) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(i) = rest.find("<circle ") {
        let circle_end = i + rest[i..].find("/>").expect("circle 끝");
        let circle = &rest[i..circle_end];
        let after = &rest[circle_end..];
        let text_start = after.find("<text ").expect("겹친 원문");
        let text_end = text_start + after[text_start..].find("</text>").expect("text 끝");
        let text = &after[text_start..text_end];
        if text.ends_with(dot) && text.contains("fill-opacity=\"0\"") {
            out.push((attr(circle, "cy"), attr(text, "y")));
        }
        rest = &rest[circle_end..];
    }
    out
}

#[test]
fn one_dot_leader_is_painted_as_vector_middle_dot() {
    let svg = page_svg("주거\u{2024}수도\u{2024}광열");
    let dots = painted_dots(&svg, '\u{2024}');
    assert_eq!(
        dots.len(),
        2,
        "U+2024 두 개를 벡터 점으로 그려야 한다:\n{svg}"
    );
    for (cy, baseline) in dots {
        assert!(
            baseline - cy > 0.0,
            "점은 기준선 위 x-height 가운데여야 한다: cy={cy} baseline={baseline}"
        );
    }
    // 보이는 글리프로 U+2024 를 그리지 않는다 (투명 원문만 남는다).
    for text in svg.split("<text ").skip(1) {
        let body = &text[..text.find("</text>").expect("text 끝")];
        if body.ends_with('\u{2024}') {
            assert!(
                body.contains("fill-opacity=\"0\""),
                "U+2024 를 대체 글꼴 글리프로 그리면 기준선 마침표가 된다: {body}"
            );
        }
    }
}
