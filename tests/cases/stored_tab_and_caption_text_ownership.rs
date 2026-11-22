//! 관계형 검사. 저장 탭 전진, 캡션 끝 공백의 소유와 원문에 있는 아래아·현대 음절의 표시를 확인한다.
//! 주석 갱신 2026-11-22T00:08+09:00. 직전 파일 변경 d395b9861727af9efeeac83fe049cbb0efbc6f70, 2026-11-21T23:03:59+09:00.
//! 2026-11-21 형식 사양서 1.3 원 HWP와 독립 한컴 PDF를 96 DPI에서 비교한 2px raw 실루엣이다.
//! 전 71쪽 최저는 Native 99.92193%, fresh WASM 99.92196%다.
//! Native/fresh WASM 2쪽 100.00000%/100.00000%, 4쪽 100.00000%/100.00000%, 62쪽 100.00000%/100.00000%.
//! Native/fresh WASM 10쪽 100.00000%/100.00000%, 11쪽 100.00000%/100.00000%, 71쪽 100.00000%/100.00000%.
//! TSV·확대 판독은 mydocs/pr/assets/semanticist21-20261121/validation/final/README.md를 참조한다.
//! 2026-11-21 편람 HWP·HWPX 각 383쪽 중 43쪽을 독립 한컴 Print와 96 DPI로 대조했다.
//! 편람 전쪽 최저는 HWP에서 Native 99.96504%, fresh WASM 99.96448%이고,
//! HWPX에서 Native 99.96504%, fresh WASM 99.96448%다.
//! 두 형식의 43쪽은 Native·fresh WASM 모두 raw 100.00000%다. 옛한글 공급자의
//! 한 음절 전진과 현대 음절의 기본 전진을 저장소의 Latin-only 글꼴로 구분한다.
//! 글리프 윤곽과 전쪽 TSV 경로는 `mydocs/pr/assets/semanticist21-20261121/validation/final/handbook-old-hangul.json`에 있다.

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

#[test]
fn source_araea_and_modern_han_remain_distinct_in_body_and_table_text() {
    fn matching_line(node: &RenderNode, needle: &str) -> Option<(String, String)> {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut source = String::new();
            let mut drawn = String::new();
            for child in &node.children {
                if let RenderNodeType::TextRun(run) = &child.node_type {
                    source.push_str(&run.text);
                    drawn.push_str(run.display_or_text());
                }
            }
            if source.contains(needle) {
                return Some((source, drawn));
            }
        }
        node.children
            .iter()
            .find_map(|child| matching_line(child, needle))
    }

    let core = original();
    for (page, needle) in [
        (9, "이후에 출시된 ᄒᆞᆫ글 제품군"),
        (10, "내부 코드는 한글,"),
        (70, "ᄒᆞᆫ글 문서 파일 구조"),
    ] {
        let tree = core.build_page_render_tree(page).expect("형식 사양서 쪽");
        let (source, drawn) = matching_line(&tree.root, needle).expect("원문이 있는 줄");
        // 한양 PUA와 세 옛자모는 같은 아래아 글리프를 표현한다.
        assert_eq!(
            drawn.replace('\u{f53a}', "ᄒᆞᆫ"),
            source.replace('\u{f53a}', "ᄒᆞᆫ"),
            "{}쪽 본문 또는 표의 인쇄 문자열",
            page + 1
        );
    }
}

#[test]
fn old_hangul_fallback_uses_the_declared_suppliers_cluster_advance() {
    use rhwp::renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        native_pua_fonts::NativePuaFonts,
        TextStyle,
    };
    let mut fonts = NativePuaFonts::default();
    let primary = "Latin-only test program";
    fonts
        .register_otf_for_source(
            primary,
            false,
            false,
            true,
            include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf"),
            0,
        )
        .unwrap();
    let mut style = TextStyle {
        font_family: "KoPub돋움체 Light,Haansoft Batang".into(),
        font_size: 1000.0,
        ..Default::default()
    };
    let positions =
        |s: &TextStyle, text: &str| EmbeddedTextMeasurer.compute_char_positions(text, s);
    let ordinary = positions(&style, "한");
    let unconfirmed = positions(&style, "ᄒᆞᆫ");
    style.native_otf_metrics = fonts.get_otf_for_source(primary, false, false, true);
    // 독립 Print의 HBATANG 실제 글리프 hmtx는 1024/1024 em. 세 자모가 한 번 전진한다.
    assert_eq!(
        positions(&style, "ᄒᆞᆫ"),
        [0.0, style.font_size, style.font_size, style.font_size]
    );
    assert!(unconfirmed[1] < style.font_size);
    assert_eq!(positions(&style, "한"), ordinary);
    // 앞선 미확인 공급자를 뛰어넘어 뒤 한컴바탕의 전진을 빌리지 않는다.
    style.font_family = "KoPub돋움체 Light,Unknown Test Face,Haansoft Batang".into();
    assert_eq!(positions(&style, "ᄒᆞᆫ"), unconfirmed);
}
