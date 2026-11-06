//! 관계형 회귀: 그림 이동 뒤 저장 줄 시작이 원문 제어 문자 경계에 속하고 왕복 내용을 보존한다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! rhwp-after.hwp (8549-rhwp-after-hwp-2022): 전 1쪽, Native 최저 99.91686%, fresh WASM 최저 99.91794%.
//! rhwp-after.hwp (api-8549-moved-inline-picture-hwp): 전 1쪽, Native 최저 99.91686%, fresh WASM 최저 99.91794%.
//! moved-inline-picture.hwpx (api-8549-moved-inline-picture-hwpx): 전 1쪽, Native 최저 99.91678%, fresh WASM 최저 99.91787%.
//! moved-terminal-tab.hwp (api-8549-moved-terminal-tab-hwp): 전 1쪽, Native 최저 99.92003%, fresh WASM 최저 99.92149%.
//! moved-terminal-tab.hwpx (api-8549-moved-terminal-tab-hwpx): 전 1쪽, Native 최저 99.91994%, fresh WASM 최저 99.92141%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형: 저장 HWP의 모든 줄 시작(PARA_LINE_SEG text_start)이 같은 문단 PARA_TEXT의 글자 경계에 있다.
//! 주석 갱신: 2026-11-04 00:20 KST(+09:00).
//! 측정: 2026-11-03 한/글 12.30(Mac)에서 저장본을 직접 열어 확인했다. 떠 있는 그림(어울림)을 옮겨
//! "글자처럼 취급한 그림: [그림]" 문단이 그림 둘레로 다시 나뉜 rhwp 저장본은 오른쪽 빈 줄 조각이 그림
//! 확장 제어 문자(13..21) 가운데인 14에서 시작했고, 한/글은 "문서가 손상되었거나 변조되었을 가능성이
//! 있습니다"로 열기를 거부했다(앱 저장본 셋, 이 검사가 만드는 수정 전 저장본). 같은 파일에서 그 값만
//! 21(그림 뒤)이나 13(그림 앞)으로 고치면 열리고, 21이면 그림이 글 옆 왼쪽 조각에 놓여 rhwp 배치와
//! 같다. 문단 끝 탭(8유닛)도 수정 전 14로 거부, 수정 뒤 21로 열린다. 쪽·픽셀 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags;
use rhwp::wasm_api::HwpDocument;

const PNG: &[u8] = include_bytes!("../../assets/logo/logo-16.png");
const BODY: &str = "그림 회전 확인 문서입니다. 떠 있는 그림 둘레로 글이 흐르는지 봅니다. ";
const LABEL: &str = "글자처럼 취급한 그림: ";
const TAIL: &str = "끝 문단입니다. ";

/// 둘째 문단 끝에 둘 8유닛 제어 문자.
#[derive(Clone, Copy, Debug)]
enum LabelEnd {
    InlinePicture,
    Tab,
}

fn ok(json: &str) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(value["ok"], true, "{value}");
    value
}

/// 앱처럼 공개 편집 API로 만든다: 떠 있는 그림이 있는 본문, 끝에 글자처럼 취급 그림이나 탭을 둔 문단, 뒤 문단.
fn document(end: LabelEnd) -> (HwpDocument, usize) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let body = BODY.repeat(5);
    doc.insert_text_native(0, 0, 0, &body).unwrap();
    doc.split_paragraph_native(0, 0, body.chars().count(), None)
        .unwrap();
    let label = match end {
        LabelEnd::InlinePicture => {
            doc.insert_text_native(0, 1, 0, LABEL).unwrap();
            let at = LABEL.chars().count();
            let picture = ok(&doc
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
                    "글자처럼",
                    None,
                    None,
                )
                .unwrap());
            let control = picture["controlIdx"].as_u64().unwrap() as usize;
            ok(&doc
                .set_picture_properties_native(0, 1, control, r#"{"treatAsChar":true}"#)
                .unwrap());
            at + 1
        }
        LabelEnd::Tab => {
            let text = format!("{LABEL}\t");
            doc.insert_text_native(0, 1, 0, &text).unwrap();
            text.chars().count()
        }
    };
    doc.split_paragraph_native(0, 1, label, None).unwrap();
    doc.insert_text_native(0, 2, 0, &TAIL.repeat(6)).unwrap();
    let floating = ok(&doc
        .insert_picture_native(
            0,
            0,
            0,
            &[],
            PNG,
            15000,
            9375,
            16,
            16,
            "png",
            "떠 있는",
            None,
            None,
        )
        .unwrap());
    let control = floating["controlIdx"].as_u64().unwrap() as usize;
    ok(&doc
        .set_picture_properties_native(
            0,
            0,
            control,
            r#"{"treatAsChar":false,"textWrap":"Square","vertRelTo":"Para","horzRelTo":"Para","horzOffset":20750,"vertOffset":2000}"#,
        )
        .unwrap());
    (doc, control)
}

/// 입력한 탭은 저장 확장 데이터가 없어 그 문단 줄을 저장하지 않는다(`can_serialize_line_partition`).
/// 한/글이 저장한 탭처럼 폭·종류를 담아 다시 연다.
fn with_hancom_tab(doc: &HwpDocument) -> HwpDocument {
    const SAVED_TAB: [u16; 7] = [2000, 0, 0x0100, 0, 0, 0, 0x0009];
    let mut model = rhwp::parser::parse_document(&doc.export_hwp_with_adapter().unwrap()).unwrap();
    model.sections[0].paragraphs[1].tab_extended = vec![SAVED_TAB];
    HwpDocument::from_bytes(&rhwp::serializer::cfb_writer::serialize_hwp(&model).unwrap()).unwrap()
}

/// 본문 문단마다 (PARA_TEXT code unit, PARA_LINE_SEG text_start 목록).
fn saved_paragraphs(bytes: &[u8]) -> Vec<(Vec<u16>, Vec<u32>)> {
    let mut cfb = CfbReader::open(bytes).expect("CFB");
    let header = cfb.read_stream_raw("/FileHeader").expect("FileHeader");
    let body = cfb
        .read_body_text_section(0, header[36] & 1 != 0, false)
        .expect("Section0");
    let mut paragraphs: Vec<(Vec<u16>, Vec<u32>)> = Vec::new();
    for record in Record::read_all(&body).expect("records") {
        if record.level == 0 && record.tag_id == tags::HWPTAG_PARA_HEADER {
            paragraphs.push((Vec::new(), Vec::new()));
        }
        let Some(paragraph) = paragraphs.last_mut().filter(|_| record.level == 1) else {
            continue;
        };
        if record.tag_id == tags::HWPTAG_PARA_TEXT {
            paragraph.0 = record
                .data
                .chunks_exact(2)
                .map(|unit| u16::from_le_bytes([unit[0], unit[1]]))
                .collect();
        } else if record.tag_id == tags::HWPTAG_PARA_LINE_SEG {
            paragraph.1 = record
                .data
                .chunks_exact(36)
                .map(|seg| u32::from_le_bytes([seg[0], seg[1], seg[2], seg[3]]))
                .collect();
        }
    }
    paragraphs
}

/// HWP5 PARA_TEXT에서 8유닛을 차지하는 인라인·확장 제어 문자의 시작 위치.
/// 0·10(줄 나눔)·13(문단 끝)·24~31만 한 유닛인 문자 제어다.
fn eight_unit_controls(units: &[u16]) -> Vec<u32> {
    let mut starts = Vec::new();
    let mut at = 0;
    while at < units.len() {
        if units[at] < 32 && !matches!(units[at], 0 | 10 | 13 | 24..=31) {
            starts.push(at as u32);
            at += 8;
        } else {
            at += 1;
        }
    }
    starts
}

#[test]
fn moved_floating_picture_keeps_line_starts_off_eight_unit_controls() {
    for end in [LabelEnd::InlinePicture, LabelEnd::Tab] {
        let (mut doc, control) = document(end);
        if matches!(end, LabelEnd::Tab) {
            doc = with_hancom_tab(&doc);
        }
        let mut wrapped = 0;
        // 앱의 끌어 옮기기처럼 세로 위치만 바꿔 둘째 문단이 그림 둘레로 다시 나뉘게 한다.
        for vertical in [8000, 5000, 2000] {
            ok(&doc
                .set_picture_properties_native(
                    0,
                    0,
                    control,
                    &format!(r#"{{"vertOffset":{vertical}}}"#),
                )
                .unwrap());
            let bytes = doc.export_hwp_with_adapter().unwrap();
            let paragraphs = saved_paragraphs(&bytes);
            let (units, starts) = &paragraphs[1];
            assert_eq!(
                eight_unit_controls(units),
                [13],
                "{end:?}: 둘째 문단 끝 제어 문자"
            );
            if starts.len() > 1 {
                wrapped += 1;
            }
            for (index, (units, starts)) in paragraphs.iter().enumerate() {
                let controls = eight_unit_controls(units);
                for &start in starts {
                    assert!(
                        start as usize <= units.len()
                            && !controls.iter().any(|&at| at < start && start < at + 8),
                        "{end:?} {vertical}: 문단 {index} 줄 시작 {start}이 글자 경계가 아니다 \
                         (줄 시작 {starts:?}, 8유닛 제어 {controls:?}, 길이 {})",
                        units.len()
                    );
                }
            }

            let reopened = HwpDocument::from_bytes(&bytes).expect("저장본을 다시 연다");
            assert_eq!(reopened.page_count(), doc.page_count());
            for paragraph in 0..3 {
                assert_eq!(
                    reopened
                        .get_text_range_native(0, paragraph, 0, usize::MAX)
                        .unwrap(),
                    doc.get_text_range_native(0, paragraph, 0, usize::MAX)
                        .unwrap(),
                    "{end:?} {vertical}: 문단 {paragraph}"
                );
            }
        }
        assert!(
            wrapped > 0,
            "{end:?}: 둘째 문단이 그림 둘레로 나뉜 저장본이 없다"
        );
    }
}
