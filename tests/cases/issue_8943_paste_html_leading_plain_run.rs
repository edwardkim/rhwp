//! 굵은 글자 앞의 보통 글자는 문단이 몇째든 보통으로 남는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

/// 문단마다 글과, 글자마다 굵게(B)·기울임(I)·밑줄(U)·보통(.)
fn marks(core: &DocumentCore) -> Vec<(String, String)> {
    let document = core.document();
    document.sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| {
            let marks = (0..paragraph.text.chars().count())
                .map(|index| {
                    let id = paragraph.char_shape_id_at(index).unwrap_or(0);
                    let shape = &document.doc_info.char_shapes[id as usize];
                    if shape.bold {
                        'B'
                    } else if shape.italic {
                        'I'
                    } else if shape.underline_type != rhwp::model::style::UnderlineType::None {
                        'U'
                    } else {
                        '.'
                    }
                })
                .collect();
            (paragraph.text.clone(), marks)
        })
        .collect()
}

fn paste(html: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank document");
    core.paste_html_native(0, 0, 0, html).expect("HTML paste");
    core
}

fn lines(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(text, marks)| (text.to_string(), marks.to_string()))
        .collect()
}

#[test]
fn plain_text_before_a_format_stays_plain_in_later_paragraphs() {
    for (html, expected) in [
        (
            "<p>첫 문단</p><p>앞 <b>굵게</b> 뒤</p>",
            lines(&[("첫 문단", "...."), ("앞 굵게 뒤", "..BB..")]),
        ),
        (
            "<p>첫 문단</p>\n<p>앞 <i>기울임</i> 뒤</p>",
            lines(&[("첫 문단", "...."), ("앞 기울임 뒤", "..III..")]),
        ),
        (
            "<p>첫</p><p>둘</p><p>앞 <u>밑줄</u></p>",
            lines(&[("첫", "."), ("둘", "."), ("앞 밑줄", "..UU")]),
        ),
        (
            r#"<p>첫 문단</p><p>앞 <span style="font-weight:bold">굵게</span></p>"#,
            lines(&[("첫 문단", "...."), ("앞 굵게", "..BB")]),
        ),
        (
            "<ul><li>하나</li><li>앞 <b>굵게</b></li></ul>",
            lines(&[("• 하나", "...."), ("• 앞 굵게", "....BB")]),
        ),
    ] {
        assert_eq!(marks(&paste(html)), expected, "{html}");
    }
}

#[test]
fn first_paragraph_keeps_the_same_result() {
    assert_eq!(
        marks(&paste("<p>앞 <b>굵게</b> 뒤</p>")),
        lines(&[("앞 굵게 뒤", "..BB..")])
    );
}

#[test]
fn leading_plain_run_survives_hwpx_save_and_reopen() {
    let core = paste("<p>첫 문단</p><p>앞 <b>굵게</b> 뒤</p>");
    let hwpx = core.export_hwpx_native().expect("HWPX export");
    let reopened = DocumentCore::from_bytes(&hwpx).expect("reopen HWPX");
    assert_eq!(
        marks(&reopened),
        lines(&[("첫 문단", "...."), ("앞 굵게 뒤", "..BB..")])
    );
}
