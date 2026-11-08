//! 본문 PAGE 필드는 현재 논리 쪽수이며, 저장 num이나 파싱 순번이 아니다.
//! 독립 한컴2020 Print: pdf/semanticist21-20261015/pr7671-body-page-fields*-2020.pdf.
//! 같은 쪽 두 필드는 각각1, 둘째 쪽은2 / 새 번호12를 표시한다.
//! 36px 본문 상자의 줄 나눔은 같은 PDF의 실제 글줄에서 확인한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;

fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).unwrap() + key.len();
    &tag[start..start + tag[start..].find('"').unwrap()]
}
fn lines(core: &DocumentCore, page: u32) -> Vec<String> {
    let svg = core.render_page_svg_native(page).expect("실제 본문 paint");
    let mut rest = svg.as_str();
    let mut lines: Vec<(f64, String)> = Vec::new();
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap();
        let close = rest.find("</text>").unwrap();
        let text = compact(&rest[end + 1..close]);
        if !text.is_empty() {
            let y: f64 = attr(&rest[..end], "y").parse().unwrap();
            if let Some(line) = lines.iter_mut().find(|l| (l.0 - y).abs() < 0.001) {
                line.1.push_str(&text)
            } else {
                lines.push((y, text));
            }
        }
        rest = &rest[close + 7..];
    }
    lines.sort_by(|a, b| a.0.total_cmp(&b.0));
    lines.into_iter().map(|l| l.1).collect()
}
fn check(name: &str, expected: &[&[&str]]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/semanticist21_20261015")
        .join(name);
    let bytes = std::fs::read(path).unwrap();
    let source = rhwp::parser::parse_document(&bytes).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(
        core.page_count(),
        expected.len() as u32,
        "독립 Print 전체 쪽수"
    );
    for (page, expected) in expected.iter().enumerate() {
        assert_eq!(
            lines(&core, page as u32),
            *expected,
            "{name} {}쪽의 내용·순서·줄 소속",
            page + 1
        );
    }
    for (before, after) in source.sections[0]
        .paragraphs
        .iter()
        .zip(&core.document().sections[0].paragraphs)
    {
        assert_eq!(
            before.text, after.text,
            "표시값으로 모델 문자를 확장하지 않는다"
        );
        assert_eq!(before.char_offsets, after.char_offsets, "UTF-16 축 보존");
        assert_eq!(
            before.control_text_positions(),
            after.control_text_positions(),
            "컨트롤 소속 보존"
        );
    }
    let model_json = serde_json::to_string(&core.document().sections[0].paragraphs).unwrap();
    assert!(
        !model_json.contains("layout_field_values"),
        "조판 전용 값은 IR 원문에 직렬화하지 않는다"
    );
}
#[test]
fn same_page_fields_and_next_page_use_actual_page_values() {
    check(
        "pr7671-body-page-fields.hwpx",
        &[&["1첫문단1경계"], &["2앞뒤"]],
    );
}
#[test]
fn restarted_two_digit_page_field_and_narrow_lines_follow_print() {
    check(
        "pr7671-body-page-fields-narrow-restart12.hwpx",
        &[&["1첫", "문단", "1경", "계"], &["12앞", "뒤"]],
    );
}
#[test]
fn two_digit_page_field_keeps_one_cell_fit_reservation() {
    check("pr7671-body-page-field12-one-cell.hwpx", &[&["12경계"]]);
}
#[test]
fn page_field_fit_reservation_scales_with_declared_font_size() {
    check("pr7671-body-page-field-size20.hwpx", &[&["1경", "계"]]);
}
#[test]
fn total_pages_and_page_number_keep_separate_values_on_the_same_line() {
    check(
        "pr7671-body-total-page-mixed.hwpx",
        &[&["2첫문단1경계"], &["2앞뒤"]],
    );
}
#[test]
fn partial_paragraph_fields_follow_their_actual_fragment_pages() {
    check(
        "pr7671-body-page-fields-partial-paragraph.hwpx",
        &[&["1가", "나", "1다"], &["라", "2마", "바"], &["3사", "아"]],
    );
}
#[test]
fn leading_middle_and_two_digit_fields_keep_print_justified_lines() {
    check(
        "pr7671-body-page-field-justification.hwpx",
        &[
            &["1가나", "다라마", "바사"],
            &["가2나", "다라마", "바사"],
            &["12가나", "다라마", "바사"],
        ],
    );
}
