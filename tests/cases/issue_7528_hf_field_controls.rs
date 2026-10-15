//! [#7528] 머리말/꼬리말 필드(쪽 번호·전체 쪽수·파일 이름)가 HWP·HWPX 저장 후 다시 열면
//! 사라지던 결함의 회귀 가드.
//!
//! 편집 중 필드는 머리말 문단에 마커 한 글자(U+0015~U+0017)로 있고 렌더러만 그 값을
//! 바꿔 그린다. 저장은 마커를 컨트롤로 바꾸지 않아 HWP 에는 맨 코드 유닛이, HWPX 에는
//! 아무것도 남지 않았다. 저장본에는 한컴이 만든 머리말과 같은 컨트롤 — 자동 번호와
//! 파일 이름 필드(`%pat`, `$F`) — 이 들어가야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::{AutoNumberType, Control, FieldType};
use rhwp::model::paragraph::Paragraph;
use rhwp::wasm_api::HwpDocument;

const FILE_NAME: &str = "보고서.hwp";

#[derive(Clone, Copy, Debug)]
enum Format {
    Hwp,
    Hwpx,
}

fn reopen(doc: &HwpDocument, format: Format) -> HwpDocument {
    let bytes = match format {
        Format::Hwp => doc.export_hwp().expect("HWP 저장"),
        Format::Hwpx => doc.export_hwpx().expect("HWPX 저장"),
    };
    HwpDocument::from_bytes(&bytes).expect("저장본 다시 열기")
}

fn header_paragraphs(doc: &HwpDocument) -> Vec<Paragraph> {
    doc.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|ctrl| match ctrl {
            Control::Header(header) => Some(header.paragraphs.clone()),
            _ => None,
        })
        .expect("머리말")
}

fn header_controls(doc: &HwpDocument) -> Vec<Control> {
    header_paragraphs(doc)
        .into_iter()
        .flat_map(|para| para.controls)
        .collect()
}

fn page_text(doc: &HwpDocument, page: u32) -> String {
    doc.extract_page_text_native(page).expect("쪽 텍스트")
}

/// 빈 문서에 마당을 적용한다 — 본문이 비어 쪽 텍스트가 곧 머리말이다.
fn doc_with_template(template: u8) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("빈 문서");
    doc.set_file_name(FILE_NAME);
    doc.apply_hf_template_native(0, true, 0, template)
        .expect("머리말 마당");
    doc
}

#[test]
fn centered_page_number_template_survives_save_and_reopen() {
    let doc = doc_with_template(2);
    assert_eq!(page_text(&doc, 0).trim(), "1", "저장 전 머리말");

    for format in [Format::Hwp, Format::Hwpx] {
        let reopened = reopen(&doc, format);
        assert_eq!(
            page_text(&reopened, 0),
            page_text(&doc, 0),
            "{format:?} 저장본도 같은 쪽 번호를 그려야 한다"
        );
        assert!(
            header_controls(&reopened).iter().any(|ctrl| matches!(
                ctrl,
                Control::AutoNumber(number) if number.number_type == AutoNumberType::Page
            )),
            "{format:?} 저장본 머리말에 쪽 번호 자동 번호가 있어야 한다"
        );
    }
}

#[test]
fn page_number_and_file_name_template_survives_save_and_reopen() {
    let doc = doc_with_template(4);
    let before = page_text(&doc, 0);
    assert!(
        before.starts_with('1') && before.contains(FILE_NAME),
        "저장 전 머리말: {before:?}"
    );

    for format in [Format::Hwp, Format::Hwpx] {
        let reopened = reopen(&doc, format);
        assert_eq!(
            page_text(&reopened, 0),
            before,
            "{format:?} 저장본도 쪽 번호와 파일 이름을 그려야 한다"
        );
        let controls = header_controls(&reopened);
        assert!(
            controls.iter().any(|ctrl| matches!(
                ctrl,
                Control::AutoNumber(number) if number.number_type == AutoNumberType::Page
            )),
            "{format:?} 저장본 머리말에 쪽 번호 자동 번호가 있어야 한다: {controls:?}"
        );
        assert!(
            controls.iter().any(|ctrl| matches!(
                ctrl,
                Control::Field(field)
                    if field.field_type == FieldType::Path && field.command == "$F"
            )),
            "{format:?} 저장본 머리말에 파일 이름 필드가 있어야 한다: {controls:?}"
        );
    }
}

/// 파일 이름·탭·쪽 번호 마당(5·10)도 저장본에서 탭 뒤에 쪽 번호를 그린다. 저장본의 쪽 번호는
/// 문단 마지막 글자인 자동 번호 자리표라, 그 자리를 놓치면 앞의 탭이 번호로 그려진다.
#[test]
fn file_name_then_page_number_template_keeps_tab_after_save_and_reopen() {
    for template in [5, 10] {
        let doc = doc_with_template(template);
        let before = page_text(&doc, 0);
        assert!(
            before.starts_with(&format!("{FILE_NAME}\t1")),
            "마당 {template} 저장 전 머리말: {before:?}"
        );

        for format in [Format::Hwp, Format::Hwpx] {
            assert_eq!(
                page_text(&reopen(&doc, format), 0),
                before,
                "마당 {template} {format:?} 저장본도 파일 이름·탭·쪽 번호를 그려야 한다"
            );
        }
    }
}

/// 필드 넣기로 넣은 전체 쪽수도 저장본에서 같은 수로 그려진다.
#[test]
fn inserted_total_page_field_survives_save_and_reopen() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("빈 문서");
    doc.insert_text_native(0, 0, 0, &"가나다라마바사 ".repeat(1_500))
        .expect("여러 쪽 본문");
    assert!(doc.page_count() >= 2, "여러 쪽 전제");
    doc.create_header_footer_native(0, true, 0)
        .expect("머리말 생성");
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, "/")
        .expect("머리말 입력");
    doc.insert_field_in_hf_native(0, true, 0, 0, 0, 1)
        .expect("쪽 번호 필드");
    doc.insert_field_in_hf_native(0, true, 0, 0, 2, 2)
        .expect("전체 쪽수 필드");
    let total = doc.page_count();
    let header_line = |text: String| text.lines().next().unwrap_or_default().to_string();
    assert_eq!(
        header_line(page_text(&doc, 1)),
        format!("2/{total}"),
        "저장 전 둘째 쪽 머리말"
    );

    for format in [Format::Hwp, Format::Hwpx] {
        let reopened = reopen(&doc, format);
        assert_eq!(reopened.page_count(), total, "{format:?} 쪽 수");
        assert_eq!(
            header_line(page_text(&reopened, 1)),
            format!("2/{total}"),
            "{format:?} 저장본도 쪽 번호와 전체 쪽수를 그려야 한다"
        );
    }
}

/// 붙여 넣은 필드도 저장본에서 제자리를 지킨다. 파일 이름 바로 앞의 쪽 번호·전체 쪽수가
/// 파일 이름 필드 안으로 들어가면 HWPX 에는 필드 끝이 시작보다 앞서 범위를 잃고, HWP 에는
/// 자리표 공백이 필드 글자에 섞인다.
#[test]
fn adjacent_fields_survive_save_and_reopen() {
    const NAME: &str = "a.hwp";
    // 필드 종류(1 쪽 번호 · 2 전체 쪽수 · 3 파일 이름)를 머리말 앞에서부터 차례로 넣는다.
    for kinds in [&[1, 3][..], &[2, 3], &[1, 2, 3], &[3, 1], &[3, 2]] {
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().expect("빈 문서");
        doc.set_file_name(NAME);
        doc.create_header_footer_native(0, true, 0)
            .expect("머리말 생성");
        for (offset, &kind) in kinds.iter().enumerate() {
            doc.insert_field_in_hf_native(0, true, 0, 0, offset, kind)
                .expect("필드 넣기");
        }
        let before = page_text(&doc, 0);

        for format in [Format::Hwp, Format::Hwpx] {
            let reopened = reopen(&doc, format);
            assert_eq!(
                page_text(&reopened, 0),
                before,
                "{kinds:?} {format:?} 저장본도 같은 머리말을 그려야 한다"
            );
            let para = &header_paragraphs(&reopened)[0];
            let range = para
                .field_ranges
                .iter()
                .find(|range| {
                    matches!(
                        para.controls.get(range.control_idx),
                        Some(Control::Field(field)) if field.field_type == FieldType::Path
                    )
                })
                .unwrap_or_else(|| panic!("{kinds:?} {format:?} 파일 이름 필드 범위"));
            let field_text: String = para
                .text
                .chars()
                .skip(range.start_char_idx)
                .take(range.end_char_idx - range.start_char_idx)
                .collect();
            assert_eq!(
                field_text, NAME,
                "{kinds:?} {format:?} 파일 이름 필드는 파일 이름만 감싸야 한다"
            );
        }
    }
}

/// 저장은 편집 중인 문서를 바꾸지 않는다 — 마커는 그대로 남아 이후 편집·되돌리기가
/// 종전처럼 한 글자 단위로 동작한다.
#[test]
fn saving_leaves_the_live_header_unchanged() {
    let doc = doc_with_template(4);
    let before: Vec<String> = header_paragraphs(&doc)
        .into_iter()
        .map(|p| p.text)
        .collect();

    reopen(&doc, Format::Hwp);
    reopen(&doc, Format::Hwpx);

    let after: Vec<String> = header_paragraphs(&doc)
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(after, before);
    assert!(
        header_controls(&doc).is_empty(),
        "편집 문서에는 컨트롤을 넣지 않는다"
    );
}
