//! `<p>` 밖 인라인 서식(Chrome 이 문단 중간부터 고른 글)을 붙이면 서식과 한 문단이 남는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// Chrome 이 `<p>앞 <b>굵게</b> 뒤</p>` 를 "앞" 뒤부터 골랐을 때 쓰는 모양(style 은 줄였다)
const CHROME_BOLD: &str = r#"<span style="font-weight: 400;"><span> </span></span><b style="font-style: normal;">굵게</b><span style="font-weight: 400;"> 뒤</span>"#;
const CHROME_ITALIC: &str = r#"<span style="font-weight: 400;"><span> </span></span><i style="font-weight: 400;">기울임</i><span style="font-weight: 400;"> 뒤</span>"#;

fn paste(html: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank document");
    core.paste_html_native(0, 0, 0, html).expect("HTML paste");
    core
}

/// 빈 문서에 붙인 문단마다 글과, 글자마다 굵게(B)·기울임(I)·보통(.)
fn pasted(html: &str) -> Vec<(String, String)> {
    let core = paste(html);
    let document = core.document();
    document.sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| {
            let marks = (0..paragraph.text.chars().count())
                .map(|index| {
                    let id = paragraph.char_shape_id_at(index).unwrap_or(0);
                    let shape = &document.doc_info.char_shapes[id as usize];
                    match (shape.bold, shape.italic) {
                        (true, _) => 'B',
                        (_, true) => 'I',
                        _ => '.',
                    }
                })
                .collect();
            (paragraph.text.clone(), marks)
        })
        .collect()
}

fn one(text: &str, marks: &str) -> Vec<(String, String)> {
    lines(&[(text, marks)])
}

fn lines(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(text, marks)| (text.to_string(), marks.to_string()))
        .collect()
}

#[test]
fn loose_inline_formats_stay_in_one_paragraph() {
    for (html, expected) in [
        (
            r#"<span style="font-weight:bold">굵게</span> 뒤"#,
            one("굵게 뒤", "BB.."),
        ),
        ("<b>굵게</b> 뒤", one("굵게 뒤", "BB..")),
        ("<strong>굵게</strong> 뒤", one("굵게 뒤", "BB..")),
        ("<i>기울임</i> 뒤", one("기울임 뒤", "III..")),
        (CHROME_BOLD, one(" 굵게 뒤", ".BB..")),
        (CHROME_ITALIC, one(" 기울임 뒤", ".III..")),
        ("<span>굵</span><span>게</span>", one("굵게", "..")),
    ] {
        assert_eq!(pasted(html), expected, "{html}");
    }
}

#[test]
fn text_after_a_format_does_not_inherit_it() {
    assert_eq!(pasted("<p><b>굵게</b> 뒤</p>"), one("굵게 뒤", "BB.."));
}

#[test]
fn google_docs_normal_weight_wrapper_stays_normal() {
    let html = r#"<meta charset="utf-8"><b style="font-weight:normal;" id="docs-internal-guid-0"><span style="font-weight:400;">구글</span></b>"#;
    assert_eq!(pasted(html), one("구글", ".."));
}

#[test]
fn span_inside_a_paragraph_keeps_inner_formats_and_images() {
    assert_eq!(
        pasted(r#"<p><span style="color:#ff0000"><b>가</b>나</span>다</p>"#),
        one("가나다", "B.."),
    );

    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    let core = paste(&format!(
        r#"<p><span>앞<img src="data:image/png;base64,{png}">뒤</span></p>"#
    ));
    let paragraph = &core.document().sections[0].paragraphs[0];
    let pictures: Vec<usize> = paragraph
        .controls
        .iter()
        .zip(paragraph.control_text_positions())
        .filter(|(control, _)| matches!(control, Control::Picture(_)))
        .map(|(_, position)| position)
        .collect();
    assert_eq!((paragraph.text.as_str(), pictures), ("앞뒤", vec![1]));
}

#[test]
fn numeric_bold_weights_stay_bold() {
    for (html, expected) in [
        (
            r#"<p>앞<strong style="font-weight: 600;">굵게</strong></p>"#,
            one("앞굵게", ".BB"),
        ),
        (
            r#"<p>앞<strong style="font-weight: 800;">굵게</strong></p>"#,
            one("앞굵게", ".BB"),
        ),
        (
            r#"<p dir="auto">앞 <strong style="color: rgb(31, 35, 40); font-weight: 600;">굵게</strong> 뒤</p>"#,
            one("앞 굵게 뒤", "..BB.."),
        ),
        // 가장 안쪽 요소의 굵기를 따른다.
        (
            r#"<p><b>가<span style="font-weight:400">나</span></b></p>"#,
            one("가나", "B."),
        ),
    ] {
        assert_eq!(pasted(html), expected, "{html}");
    }
}

#[test]
fn line_break_inside_a_format_keeps_lines_and_format() {
    for html in [
        "<p>a</p><b><br></b><p>b</p>",
        "<div>a</div><div><b><br></b></div><div>b</div>",
    ] {
        assert_eq!(
            pasted(html),
            lines(&[("a", "."), ("", ""), ("b", ".")]),
            "{html}"
        );
    }
    // Google 문서는 끝 줄바꿈을 서식 태그 밖 <br> 로 쓴다.
    let docs = r#"<meta charset="utf-8"><b style="font-weight:normal;" id="docs-internal-guid-0"><p dir="ltr"><span style="font-weight:400;">a</span></p><br><p dir="ltr"><span style="font-weight:400;">b</span></p></b><br class="Apple-interchange-newline">"#;
    assert_eq!(
        pasted(docs),
        lines(&[("a", "."), ("", ""), ("b", "."), ("", "")])
    );
    assert_eq!(
        pasted("<b>가<br>나</b>"),
        lines(&[("가", "B"), ("나", "B")])
    );
}

// 기존 plain paste의 문자 수 제한을 서식·UTF-16·인라인 그림 경로에도 적용한다.
// 조판 좌표가 아닌 HTML import의 내용/서식/개체 보존 계약이다.
#[test]
fn long_loose_inline_preserves_content_and_styles_at_the_existing_limit() {
    let text = "😀".repeat(8001);
    let actual = pasted(&format!("<b>{text}</b>"));
    assert_eq!(
        actual
            .iter()
            .map(|(s, _)| s.chars().count())
            .collect::<Vec<_>>(),
        [4000, 4000, 1]
    );
    assert_eq!(
        actual.iter().map(|(s, _)| s.as_str()).collect::<String>(),
        text
    );
    assert!(actual
        .iter()
        .all(|(_, marks)| marks.chars().all(|m| m == 'B')));

    let actual = pasted(&format!(
        "<b>{}</b><i>{}</i>",
        "x".repeat(3999),
        "y".repeat(4002)
    ));
    assert_eq!(
        actual.iter().map(|(s, _)| s.len()).collect::<Vec<_>>(),
        [4000, 4000, 1]
    );
    assert_eq!(actual[0].1, format!("{}I", "B".repeat(3999)));
    assert_eq!(actual[1].1, "I".repeat(4000));
    assert_eq!(actual[2].1, "I");
}

#[test]
fn long_loose_inline_keeps_images_on_the_correct_side_of_the_cut() {
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    for position in [3999, 4000, 4001] {
        let core = paste(&format!(
            "<b><span>{}<img src=\"data:image/png;base64,{png}\">{}</span></b>",
            "x".repeat(position),
            "y".repeat(8001 - position)
        ));
        let paragraphs = &core.document().sections[0].paragraphs;
        assert_eq!(
            paragraphs
                .iter()
                .map(|p| p.text.chars().count())
                .collect::<Vec<_>>(),
            [4000, 4000, 1]
        );
        let images = paragraphs
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                p.controls
                    .iter()
                    .zip(p.control_text_positions())
                    .filter_map(move |(c, pos)| {
                        matches!(c, Control::Picture(_)).then_some((i, pos))
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            images,
            [(position / 4000, position % 4000)],
            "image at {position}"
        );
    }
}
