//! Issue #937: 복학원서 서명란 `(인)` PUA 기호 렌더링 불일치.
//!
//! `samples/복학원서.hwp` 1페이지 서명란은 한컴/PDF 기준 `(인)` 으로 표시된다.
//! 원본 HWP5 IR 에서는 이 기호가 한컴 PUA `U+F012B` 1글자로 저장되어 있으므로,
//! 원문 문자는 보존하되 렌더링/측정 경로에서 표시 문자열 `(인)` 으로 치환해야 한다.

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::composer::{expand_pua_render_text, pua_to_display_text};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

fn svg_text_content(svg: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(svg);
    let mut buf = Vec::new();
    let mut in_text = false;
    let mut out = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) if e.name().as_ref() == b"text" => {
                in_text = true;
            }
            Ok(quick_xml::events::Event::End(e)) if e.name().as_ref() == b"text" => {
                in_text = false;
            }
            Ok(quick_xml::events::Event::Text(e)) if in_text => {
                out.push_str(&e.decode().expect("decode SVG text node"));
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("parse SVG text nodes: {}", e),
        }
        buf.clear();
    }

    out
}

fn collect_shape_texts<'a>(shape: &'a rhwp::model::shape::ShapeObject, out: &mut Vec<&'a str>) {
    if let Some(text_box) = shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
    {
        collect_paragraph_texts(&text_box.paragraphs, out);
    }
    if let rhwp::model::shape::ShapeObject::Group(group) = shape {
        for child in &group.children {
            collect_shape_texts(child, out);
        }
    }
}

fn collect_paragraph_texts<'a>(paragraphs: &'a [Paragraph], out: &mut Vec<&'a str>) {
    for para in paragraphs {
        out.push(&para.text);
        for control in &para.controls {
            match control {
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect_paragraph_texts(&cell.paragraphs, out);
                    }
                }
                Control::Header(header) => collect_paragraph_texts(&header.paragraphs, out),
                Control::Footer(footer) => collect_paragraph_texts(&footer.paragraphs, out),
                Control::Footnote(footnote) => collect_paragraph_texts(&footnote.paragraphs, out),
                Control::Endnote(endnote) => collect_paragraph_texts(&endnote.paragraphs, out),
                Control::HiddenComment(comment) => {
                    collect_paragraph_texts(&comment.paragraphs, out);
                }
                Control::Shape(shape) => collect_shape_texts(shape, out),
                _ => {}
            }
        }
    }
}

fn read_bokhakwonseo() -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/복학원서.hwp");
    fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e))
}

#[test]
fn issue_937_bokhakwonseo_signature_cell_contains_f012b() {
    let bytes = read_bokhakwonseo();
    let doc = rhwp::parser::parse_hwp(&bytes).expect("parse samples/복학원서.hwp");

    let mut texts = Vec::new();
    for section in &doc.sections {
        collect_paragraph_texts(&section.paragraphs, &mut texts);
    }

    let target = texts
        .iter()
        .find(|text| text.contains('\u{F012B}'))
        .copied()
        .expect("복학원서 서명란 PUA U+F012B 텍스트를 찾아야 함");

    assert!(
        target.contains("(Signature)"),
        "U+F012B 는 서명란 `(Signature)` 앞 셀 텍스트에서 발견되어야 함. got: {:?}",
        target,
    );
}

#[test]
fn issue_937_f012b_display_text_should_be_signature_seal() {
    let display = pua_to_display_text('\u{F012B}');
    assert_eq!(
        display.as_deref(),
        Some("(인)"),
        "U+F012B 한컴 PUA 서명/날인 기호는 렌더링 시 `(인)` 으로 표시되어야 함",
    );
}

#[test]
fn issue_937_f081c_filler_should_not_render_as_text() {
    assert_eq!(
        expand_pua_render_text("\u{F081C}"),
        "",
        "U+F081C HWP TAC filler 는 실제 렌더 출력에 글리프로 표시되면 안 됨",
    );
    assert_eq!(
        expand_pua_render_text("A\u{F081C}B"),
        "AB",
        "U+F081C filler 제거가 주변 텍스트 순서를 바꾸면 안 됨",
    );
}

#[test]
fn issue_937_f02fc_callout_bullet_should_render_as_pointer() {
    assert_eq!(
        expand_pua_render_text("\u{F02FC} 전자서명"),
        "► 전자서명",
        "U+F02FC 한컴 PUA callout bullet 는 missing glyph 대신 right pointer 로 표시되어야 함",
    );
    assert_eq!(
        pua_to_display_text('\u{F02FC}').as_deref(),
        Some("►"),
        "CharOverlap/display helper 도 같은 U+F02FC 표시 문자열을 반환해야 함",
    );
}

#[test]
fn issue_937_f031c_toc_bullet_should_render_as_square() {
    assert_eq!(
        expand_pua_render_text("\u{F031C} 행정업무"),
        "■ 행정업무",
        "U+F031C 한컴 PUA TOC bullet 는 missing glyph 대신 black square 로 표시되어야 함",
    );
    assert_eq!(
        pua_to_display_text('\u{F031C}').as_deref(),
        Some("■"),
        "CharOverlap/display helper 도 같은 U+F031C 표시 문자열을 반환해야 함",
    );
}

#[test]
fn issue_937_svg_renders_f012b_as_signature_seal() {
    let bytes = read_bokhakwonseo();
    let doc = HwpDocument::from_bytes(&bytes).expect("parse samples/복학원서.hwp");
    let svg = doc
        .render_page_svg_native(0)
        .expect("render samples/복학원서.hwp page 1");
    let text = svg_text_content(&svg);

    assert!(
        text.contains("(인)(Signature)"),
        "복학원서 1페이지 SVG는 서명란 기호를 `(인)` 으로 렌더링해야 함",
    );
    assert!(
        !svg.contains('\u{F012B}'),
        "복학원서 1페이지 SVG에 원본 PUA U+F012B가 그대로 출력되면 안 됨",
    );
    assert!(
        !svg.contains('\u{F081C}'),
        "복학원서 1페이지 SVG에 TAC filler U+F081C가 글리프로 출력되면 안 됨",
    );
}
