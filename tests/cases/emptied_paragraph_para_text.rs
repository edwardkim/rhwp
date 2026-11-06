//! 관계형 회귀: 빈 문단의 PARA_TEXT 생략·줄 레코드·컨트롤 삭제와 저장 재열기 계약. 화면 좌표를 고정하지 않는다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! rhwp-after.hwp (8602-rhwp-after-hwp-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! rhwp-after.hwp (api-8602-empty-deleted-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! empty-deleted.hwpx (api-8602-empty-deleted-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형: 편집으로 글·컨트롤이 모두 빠진 문단은 HWP 저장에서 한/글이 쓰는 빈 문단과 같다 —
//! PARA_HEADER char_count 1, PARA_TEXT 없음. char_count 가 1 보다 큰 문단만 PARA_TEXT 를 갖는다.
//! 개체를 지운 문단은 0에서 시작하는 저장 줄을 하나 이상 남긴다.
//! 주석 갱신: 2026-11-04 23:40 KST(+09:00).
//! 측정: 2026-11-04 한/글 12.30(Mac)에서 저장본을 직접 열어 확인했다. "앞 문단" 뒤 빈 문단에 홀로 둔
//! 표·수식을 지운 rhwp 저장본은 그 문단에 끝 표시만 든 PARA_TEXT(`0d 00`, char_count 1)가 남아
//! "파일이 손상되었습니다"로 거절됐다. 다시 연 문서에서 한 문단의 글을 모두 지운 저장본도 같은 레코드로
//! 거절됐다. 그 레코드만 빠진 수정 뒤 저장본은 표·그림·그리기·글상자·수식·각주 모두 HWP와 HWPX로
//! 열렸다. PARA_TEXT와 줄 레코드가 함께 없는 빈 문단(그림을 지운 수정 전 저장본)도 열렸다.
//! 한/글이 저장한 빈 문단은 모두 PARA_TEXT 없이 줄 레코드 하나를 둔다. 쪽·픽셀 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::hyperlink::HyperlinkTarget;
use rhwp::document_core::inline_control::{InlineControlAddress, InlineControlCaret};
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags;
use rhwp::wasm_api::HwpDocument;

const PNG: &[u8] = include_bytes!("../../assets/logo/logo-16.png");
const TEXT: &str = "가나";

#[derive(Clone, Copy, Debug)]
enum Object {
    Table,
    Picture,
    Drawing,
    TextBox,
    Equation,
    Footnote,
}

/// 개체를 둘 자리: 빈 문단에 홀로, 또는 글 "가나"의 앞·가운데·끝.
#[derive(Clone, Copy, Debug)]
enum Place {
    Alone,
    Start,
    Middle,
    End,
}

fn ok(json: &str) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(value["ok"], true, "{value}");
    value
}

/// 앱처럼 공개 편집 API로 "앞 문단"과 그 뒤 문단 1을 만든다.
fn document(text: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "앞 문단").unwrap();
    doc.split_paragraph_native(0, 0, 4, None).unwrap();
    if !text.is_empty() {
        doc.insert_text_native(0, 1, 0, text).unwrap();
    }
    doc
}

fn reopen(doc: &HwpDocument) -> HwpDocument {
    HwpDocument::from_bytes(&doc.export_hwp_with_adapter_snapshot().unwrap()).unwrap()
}

fn create(doc: &mut HwpDocument, object: Object, at: usize) -> usize {
    let created = match object {
        Object::Table => ok(&doc.create_table_native(0, 1, at, 2, 2).unwrap()),
        Object::Picture => ok(&doc
            .insert_picture_native(
                0,
                1,
                at,
                &[],
                PNG,
                8000,
                5000,
                16,
                16,
                "png",
                "",
                None,
                None,
            )
            .unwrap()),
        Object::Drawing | Object::TextBox => ok(&doc
            .create_shape_control_native(
                0,
                1,
                at,
                8000,
                5000,
                0,
                0,
                false,
                "TopAndBottom",
                if matches!(object, Object::Drawing) {
                    "rectangle"
                } else {
                    "textbox"
                },
                false,
                false,
                &[],
            )
            .unwrap()),
        Object::Equation => ok(&doc
            .insert_equation_native(0, 1, at, "x+y", 1000, 0)
            .unwrap()),
        Object::Footnote => ok(&doc.insert_footnote_native(0, 1, at).unwrap()),
    };
    assert_eq!(created["paraIdx"], 1, "{object:?}: {created}");
    created["controlIdx"].as_u64().unwrap() as usize
}

/// 개체 갈래의 삭제 API, 또는 갈래를 가리지 않는 컨트롤 삭제(`DeleteCtrl`)로 지운다.
fn delete(doc: &mut HwpDocument, object: Object, control: usize, generic: bool) {
    let result = if generic {
        doc.delete_control_native(0, 1, control)
    } else {
        match object {
            Object::Table => doc.delete_table_control_native(0, 1, control),
            Object::Picture => doc.delete_picture_control_native(0, 1, control),
            Object::Drawing | Object::TextBox => doc.delete_shape_control_native(0, 1, control),
            Object::Equation => doc.delete_equation_control_native(0, 1, control),
            Object::Footnote => doc.delete_footnote_native(0, 1, control),
        }
    };
    ok(&result.unwrap());
}

/// 저장 HWP 본문 문단마다 (char_count, PARA_TEXT code unit, 줄 시작 목록).
fn saved_paragraphs(bytes: &[u8]) -> Vec<(u32, Option<Vec<u16>>, Vec<u32>)> {
    let mut cfb = CfbReader::open(bytes).expect("CFB");
    let header = cfb.read_stream_raw("/FileHeader").expect("FileHeader");
    let body = cfb
        .read_body_text_section(0, header[36] & 1 != 0, false)
        .expect("Section0");
    let mut paragraphs: Vec<(u32, Option<Vec<u16>>, Vec<u32>)> = Vec::new();
    for record in Record::read_all(&body).expect("records") {
        if record.level == 0 && record.tag_id == tags::HWPTAG_PARA_HEADER {
            let raw = u32::from_le_bytes(record.data[0..4].try_into().unwrap());
            paragraphs.push((raw & 0x7fff_ffff, None, Vec::new()));
        }
        let Some(paragraph) = paragraphs.last_mut().filter(|_| record.level == 1) else {
            continue;
        };
        if record.tag_id == tags::HWPTAG_PARA_TEXT {
            paragraph.1 = Some(
                record
                    .data
                    .chunks_exact(2)
                    .map(|unit| u16::from_le_bytes([unit[0], unit[1]]))
                    .collect(),
            );
        } else if record.tag_id == tags::HWPTAG_PARA_LINE_SEG {
            paragraph.2 = record
                .data
                .chunks_exact(36)
                .map(|seg| u32::from_le_bytes([seg[0], seg[1], seg[2], seg[3]]))
                .collect();
        }
    }
    paragraphs
}

/// 문단 1이 글 `text` 만 남은 문단으로 저장되고, HWP·HWPX로 다시 열어도 그대로인지 본다.
/// `line_row` 면 문단 1이 0에서 시작하는 저장 줄을 둬야 한다.
fn assert_emptied(label: &str, doc: &HwpDocument, text: &str, line_row: bool) {
    let paragraph = &doc.document().sections[0].paragraphs[1];
    assert_eq!(paragraph.text, text, "{label}: 편집 결과 글");
    assert!(paragraph.controls.is_empty(), "{label}: 남은 컨트롤");

    let bytes = doc.export_hwp_with_adapter_snapshot().unwrap();
    let paragraphs = saved_paragraphs(&bytes);
    for (index, (char_count, units, _)) in paragraphs.iter().enumerate() {
        // 한/글은 char_count 1 문단의 PARA_TEXT(끝 표시만 든 레코드)를 손상으로 거절한다.
        assert_eq!(
            units.is_none(),
            *char_count == 1,
            "{label}: 문단 {index} char_count {char_count}, PARA_TEXT {units:?}"
        );
    }
    let (char_count, units, starts) = &paragraphs[1];
    let expected: Vec<u16> = text.encode_utf16().chain([13]).collect();
    assert_eq!(*char_count as usize, expected.len(), "{label}: char_count");
    if text.is_empty() {
        assert_eq!(*units, None, "{label}: 빈 문단의 PARA_TEXT");
    } else {
        assert_eq!(units.as_deref(), Some(&expected[..]), "{label}: PARA_TEXT");
    }
    if line_row {
        assert_eq!(starts.first(), Some(&0), "{label}: 저장 줄 {starts:?}");
    }

    let hwpx = doc.export_hwpx_native().unwrap();
    for (format, reopened) in [
        ("HWP", HwpDocument::from_bytes(&bytes)),
        ("HWPX", HwpDocument::from_bytes(&hwpx)),
    ] {
        let reopened = reopened.unwrap_or_else(|error| panic!("{label} {format}: {error}"));
        let paragraph = &reopened.document().sections[0].paragraphs[1];
        assert_eq!(paragraph.text, text, "{label} {format}: 다시 연 글");
        assert!(
            paragraph.controls.is_empty(),
            "{label} {format}: 다시 연 컨트롤"
        );
        assert_eq!(reopened.page_count(), doc.page_count(), "{label} {format}");
    }
}

#[test]
fn deleting_an_object_keeps_the_paragraph_like_a_hancom_paragraph() {
    for object in [
        Object::Table,
        Object::Picture,
        Object::Drawing,
        Object::TextBox,
        Object::Equation,
        Object::Footnote,
    ] {
        // 표는 글 사이에 만들지 않고 제 문단에 홀로 둔다.
        let places: &[Place] = if matches!(object, Object::Table) {
            &[Place::Alone]
        } else {
            &[Place::Alone, Place::Start, Place::Middle, Place::End]
        };
        for &place in places {
            for generic in [false, true] {
                if generic && matches!(object, Object::Footnote) {
                    continue;
                }
                let (text, at) = match place {
                    Place::Alone => ("", 0),
                    Place::Start => (TEXT, 0),
                    Place::Middle => (TEXT, 1),
                    Place::End => (TEXT, TEXT.chars().count()),
                };
                for source in ["새 문서", "다시 연 문서"] {
                    let mut doc = document(text);
                    let control = create(&mut doc, object, at);
                    if source == "다시 연 문서" {
                        doc = reopen(&doc);
                    }
                    delete(&mut doc, object, control, generic);
                    let label = format!("{object:?} {place:?} generic={generic} {source}");
                    assert_emptied(&label, &doc, text, true);
                }
            }
        }
    }
}

#[test]
fn removing_the_last_text_or_control_of_a_reopened_paragraph_drops_its_para_text() {
    // 저장본에서 읽은 문단은 PARA_TEXT 가 있었다. 편집이 그 내용을 모두 걷어낸 경우들이다.
    let mut doc = reopen(&document(TEXT));
    ok(&doc
        .delete_text_native(0, 1, 0, TEXT.chars().count())
        .unwrap());
    assert_emptied("글 모두 지우기", &doc, "", true);

    let mut doc = document("");
    ok(&doc.add_bookmark_native(0, 1, 0, "책갈피").unwrap());
    let mut doc = reopen(&doc);
    ok(&doc.delete_bookmark_native(0, 1, 0).unwrap());
    assert_emptied("책갈피 지우기", &doc, "", false);

    let mut doc = document("");
    ok(&doc
        .set_page_hide_native(0, 1, true, false, false, false, false, false)
        .unwrap());
    let mut doc = reopen(&doc);
    ok(&doc
        .set_page_hide_native(0, 1, false, false, false, false, false, false)
        .unwrap());
    assert_emptied("감추기 끄기", &doc, "", true);

    let mut doc = document("");
    ok(&doc
        .insert_click_here_field_at(0, 1, 0, "안내", "", "이름", true)
        .unwrap());
    let mut doc = reopen(&doc);
    ok(&doc.remove_field_at(0, 1, 0).unwrap());
    assert_emptied("누름틀 지우기", &doc, "", true);

    // 글을 모두 지워 빈 하이퍼링크만 남은 문단에서 링크를 지운다.
    let mut doc = document(TEXT);
    let link = doc
        .insert_hyperlink_native(&HyperlinkTarget::body(0, 1), 0, 2, "https://example.com")
        .unwrap();
    let mut doc = reopen(&doc);
    ok(&doc
        .delete_text_native(0, 1, 0, TEXT.chars().count())
        .unwrap());
    doc.remove_hyperlink_native(&HyperlinkTarget::body(0, 1), link)
        .unwrap();
    assert_emptied("빈 하이퍼링크 지우기", &doc, "", true);

    let mut doc = document("");
    create(&mut doc, Object::Picture, 0);
    ok(&doc
        .set_picture_properties_native(0, 1, 0, r#"{"treatAsChar":true}"#)
        .unwrap());
    let mut doc = reopen(&doc);
    let source: InlineControlAddress = serde_json::from_str(
        r#"{"owner":{"kind":"body","sectionIndex":0,"paragraphIndex":1},"controlIndex":0}"#,
    )
    .unwrap();
    let destination: InlineControlCaret = serde_json::from_str(
        r#"{"owner":{"kind":"body","sectionIndex":0,"paragraphIndex":0},"charOffset":1}"#,
    )
    .unwrap();
    doc.move_inline_control_native(&source, &destination)
        .unwrap();
    assert_emptied("글자처럼 취급 그림을 앞 문단으로 옮기기", &doc, "", true);
}
