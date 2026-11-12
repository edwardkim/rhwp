//! 관계형: HTML 원 속성·문자 서식과 편집·저장 후 내용/소유 보존을 검사한다.
//! 주석 갱신: 2026-11-12T14:12:12+09:00; 직전 파일 커밋: 2026-11-12T09:30:16+09:00 (9d1f6248c4ffd9365c9a0aab79d949fabde5c567).
//! 합성 공개 API 계약 검사이며 절대 px·고정 영역·쪽수 golden을 검사하지 않는다.
//! 별도 사전 시각 검증(2026-11-12, PDFium, 96dpi, 색 경계 보정 전 2px raw):
//! samples/issue8942의 border-hwp/hwpx와 속성 경계 4종, samples/issue8943의 leading-hwp/hwpx와
//! styled-caret/styled-cell, samples/issue8942-8943의 combined-hwp/hwpx 각각 전 1쪽에서
//! Native/fresh WASM 모두 100.00%, 전체 12문서·12쪽씩 최저도 양쪽 100.00%.
//! 수치는 위 저장본의 실물 측정이며 각 합성 API 입력의 픽셀 측정값으로 대신하지 않는다.
//! 독립 한컴 2020 Print와 확대 판독 및 입력/출력 SHA 근거: mydocs/pr/assets/bongseop-kim-20261112/README.md.
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

fn first_cell_marks(core: &DocumentCore) -> Vec<(String, String)> {
    let doc = core.document();
    let table = doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|c| {
            if let rhwp::model::control::Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .unwrap();
    table.cells[0]
        .paragraphs
        .iter()
        .filter(|p| !p.text.is_empty())
        .map(|p| {
            let marks = p
                .text
                .chars()
                .enumerate()
                .map(|(i, _)| {
                    let shape =
                        &doc.doc_info.char_shapes[p.char_shape_id_at(i).unwrap_or(0) as usize];
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
            (p.text.clone(), marks)
        })
        .collect()
}

#[test]
fn html_plain_run_keeps_surrounding_styled_caret_text() {
    let mut core = paste("<p><i>LR</i></p>");
    core.paste_html_native(0, 0, 1, "<p>앞 <b>굵게</b> 뒤</p>")
        .unwrap();
    assert_eq!(marks(&core), lines(&[("L앞 굵게 뒤R", "I..BB..I")]));
}

#[test]
fn cell_html_plain_run_and_surrounding_styles_survive_both_formats_twice() {
    for format in ["hwp", "hwpx"] {
        let mut core = paste(r#"<table border="1"><tr><td><i>LR</i></td></tr></table>"#);
        let (parent, control) = core.document().sections[0]
            .paragraphs
            .iter()
            .enumerate()
            .find_map(|(pi, p)| {
                p.controls
                    .iter()
                    .position(|c| matches!(c, rhwp::model::control::Control::Table(_)))
                    .map(|ci| (pi, ci))
            })
            .unwrap();
        core.paste_html_in_cell_native(0, parent, control, 0, 0, 1, "<p>앞 <b>굵게</b> 뒤</p>")
            .unwrap();
        let expected = lines(&[("L앞 굵게 뒤R", "I..BB..I")]);
        assert_eq!(first_cell_marks(&core), expected);
        for _ in 0..2 {
            let bytes = if format == "hwp" {
                core.export_hwp_native().unwrap()
            } else {
                core.export_hwpx_native().unwrap()
            };
            core = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(first_cell_marks(&core), expected);
        }
    }
}

#[test]
fn mixed_body_and_table_styles_survive_both_formats_twice() {
    for format in ["hwp", "hwpx"] {
        let mut core = paste(
            r#"<p>첫 문단</p><p>앞 <b>굵게</b> 뒤</p><table border="1"><tr><td>셀 앞 <b>굵게</b> 뒤</td><td>일반 셀</td></tr></table><p>AFTER</p>"#,
        );
        for _ in 0..2 {
            let bytes = if format == "hwp" {
                core.export_hwp_native().unwrap()
            } else {
                core.export_hwpx_native().unwrap()
            };
            core = DocumentCore::from_bytes(&bytes).unwrap();
            let body: Vec<_> = marks(&core)
                .into_iter()
                .filter(|(s, _)| !s.is_empty())
                .collect();
            assert_eq!(
                body,
                lines(&[
                    ("첫 문단", "...."),
                    ("앞 굵게 뒤", "..BB.."),
                    ("AFTER", ".....")
                ])
            );
            assert_eq!(
                first_cell_marks(&core),
                lines(&[("셀 앞 굵게 뒤", "....BB..")])
            );
        }
    }
}
