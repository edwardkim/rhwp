//! Issue #937: 복학원서 서명란 `(인)` PUA 기호 렌더링 불일치.
//!
//! `samples/복학원서.hwp` 1페이지 서명란은 한컴/PDF 기준 `(인)` 으로 표시된다.
//! 원본 HWP5 IR 에서는 이 기호가 한컴 PUA `U+F012B` 1글자로 저장되어 있으므로,
//! 원문 문자는 보존하되 렌더링/측정 경로에서 표시 문자열 `(인)` 으로 치환해야 한다.

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::composer::pua_to_display_text;
use std::fs;
use std::path::Path;

fn collect_shape_texts<'a>(shape: &'a rhwp::model::shape::ShapeObject, out: &mut Vec<&'a str>) {
    if let Some(text_box) = shape.drawing().and_then(|drawing| drawing.text_box.as_ref()) {
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

#[test]
fn issue_937_bokhakwonseo_signature_cell_contains_f012b() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
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
