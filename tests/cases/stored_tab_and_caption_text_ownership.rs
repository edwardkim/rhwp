//! 관계형 검사. 목차 탭의 앞 글자 전진과 가운데 정렬 캡션의 끝 공백 소유를 원본 대조군과 비교한다.
//! 주석 갱신 2026-11-21 17:07 KST. 2018년 형식 사양서 1.3 독립 한컴 Print, 96 DPI, 2px raw.
//! 11월 21일 탭 보정본의 71쪽 최저는 Native·fresh WASM 모두 99.92%다. 2·4쪽은 모두
//! 100.00%, 62쪽은 모두 99.97%였다. 캡션 보정 후 측정은 같은 사양서의 전쪽 TSV에 기록한다.
//! 원본·기준 PDF와 확대 판독은 mydocs/pr/assets/semanticist21-20261121/validation/README.md 참조.

use rhwp::{
    model::{control::Control, style::UnderlineType},
    renderer::{
        render_tree::{RenderNode, RenderNodeType},
        HWPUNIT_PER_INCH,
    },
    DocumentCore,
};

const SOURCE: &[u8] = include_bytes!("../../samples/한글문서파일형식_5.0_revision1.3.hwp");

fn original() -> DocumentCore {
    let mut core = DocumentCore::from_bytes(SOURCE).expect("형식 사양서");
    core.set_hangul2024_compat(false);
    core
}

fn find_line<'a>(
    node: &'a RenderNode,
    matches: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if node.children.iter().any(matches) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_line(child, matches))
}

fn title_and_number(
    doc: &DocumentCore,
    page: u32,
    title: &str,
    number: &str,
) -> (f64, f64, Option<f64>) {
    let tree = doc.build_page_render_tree(page).expect("목차 쪽");
    let is_title = |node: &RenderNode| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.trim_end_matches('\t') == title);
    let line = find_line(&tree.root, &is_title).expect("옛한글 제목이 있는 줄");
    let prefix = line.children.iter().find(|node| is_title(node)).unwrap();
    let number_x = line.children.iter().find_map(|node| match &node.node_type {
        RenderNodeType::TextRun(run) if run.text == number => Some(node.bbox.x),
        _ => None,
    });
    (prefix.bbox.x, prefix.bbox.x + prefix.bbox.width, number_x)
}

#[test]
fn stored_right_tab_advances_from_the_end_of_the_displayed_title() {
    let core = original();
    for (page, title) in [
        (1, "\u{f53a}글 파일 구조"),
        (1, "\u{f53a}글 파일 구조 요약"),
        (3, "\u{f53a}글 수식 개체"),
    ] {
        let mut document = core.document().clone();
        let paragraph = document
            .sections
            .iter_mut()
            .flat_map(|section| &mut section.paragraphs)
            .find(|paragraph| {
                paragraph
                    .text
                    .split_once('\t')
                    .is_some_and(|(prefix, number)| {
                        prefix.ends_with(title)
                            && !number.is_empty()
                            && number.bytes().all(|c| c.is_ascii_digit())
                    })
            })
            .expect("원본 목차 문단");
        let (prefix, number) = paragraph.text.split_once('\t').unwrap();
        let prefix_len = prefix.chars().count();
        let number = number.to_owned();
        let tab = paragraph.tab_extended.last().expect("원본의 확장 탭");
        assert_eq!(tab[2] >> 8, 2, "원본 RIGHT 탭");
        let stored_hu = u32::from(tab[0]) | (u32::from(tab[1]) << 16);
        let stored_px = f64::from(stored_hu) * 96.0 / HWPUNIT_PER_INCH;
        let (start, _, number_x) = title_and_number(&core, page, title, &number);
        paragraph.delete_text_at(prefix_len, 1 + number.len());
        let mut control = original();
        control.set_document(document);
        let (control_start, control_end, removed_number) =
            title_and_number(&control, page, title, &number);
        assert!(
            removed_number.is_none(),
            "{title} 대조군에서 탭과 번호만 제거"
        );
        assert!((control_start - start).abs() < 1e-6, "{title} 앞 글자 원점");
        let actual = number_x.expect("원본의 쪽 번호") - control_end;
        assert!(
            (actual - stored_px).abs() < 1e-6,
            "{title} 전진 {actual} / 원본 {stored_px}"
        );
    }
}

fn numbered_caption(doc: &DocumentCore) -> (f64, f64, String) {
    let page = doc.build_page_render_tree(61).expect("62쪽");
    let line = find_line(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run) if run.display_text.as_deref() == Some("140"))
    }).expect("표 140의 번호와 설명이 있는 줄");
    let last = line.children.last().expect("캡션 마지막 런");
    let RenderNodeType::TextRun(run) = &last.node_type else {
        panic!("캡션 끝 본문")
    };
    assert_eq!(run.style.underline, UnderlineType::Bottom);
    (
        line.children[0].bbox.x,
        last.bbox.x + last.bbox.width,
        run.text.clone(),
    )
}

#[test]
fn underline_owns_the_trailing_space_after_a_centered_caption_number() {
    let mut core = original();
    let (original_start, original_end, text) = numbered_caption(&core);
    let mut document = core.document().clone();
    let mut removed = 0;
    for paragraph in document
        .sections
        .iter_mut()
        .flat_map(|section| &mut section.paragraphs)
    {
        for control in &mut paragraph.controls {
            let Control::Table(table) = control else {
                continue;
            };
            let Some(caption) = table.caption.as_mut() else {
                continue;
            };
            for line in &mut caption.paragraphs {
                if line.text.ends_with("머리말/꼬리말 ") {
                    line.delete_text_at(line.text.chars().count() - 1, 1);
                    removed += 1;
                }
            }
        }
    }
    assert_eq!(removed, 1, "원본 문단의 공백 한 칸만 삭제");
    core.set_document(document);
    let (control_start, control_end, control_text) = numbered_caption(&core);
    assert_eq!(text.strip_suffix(' '), Some(control_text.as_str()));
    assert!(
        original_end > control_end,
        "원본에는 밑줄을 그릴 끝 공백이 있다"
    );
    assert!(
        (original_start - control_start).abs() < 1e-6,
        "두 캡션의 보이는 제목 원점"
    );
}
