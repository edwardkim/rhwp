//! 관계형 회귀: 번호 글자 모양과 본문 글자 크기의 추종·서식·저장 참조 관계. 입력 단위의 크기 환산은 절대 배치 핀이 아니다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! follow-two-sizes.hwp (8528-follow-two-sizes-hwp-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! follow-end-20pt.hwp (8528-follow-end-20pt-hwp-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! follow-two-sizes.hwpx (8528-follow-two-sizes-hwpx-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 문단 번호 사용자 정의 모양의 글자 모양 지정·본문 글자 크기 따름(한/글 12.30 Mac).
//!
//! 한/글은 '본문 글자 크기 따름' 번호 글자 모양의 기준 크기를 0으로 저장하고, 번호를 문단 끝(문단
//! 나눔 표시) 글자 모양의 크기로 그린다. 첫 글자 20pt·끝 10pt 문단의 번호는 10pt 다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const PT20_PX: f64 = 2000.0 / 75.0;
const PT10_PX: f64 = 1000.0 / 75.0;

/// `sizes`의 문단마다 "가 문단"을 그 크기(HWPUNIT)로 넣고, 첫 글자만 `first`로 바꾼다.
fn paragraphs(sizes: &[i32], first: Option<i32>) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    for (para, size) in sizes.iter().enumerate() {
        if para > 0 {
            doc.split_paragraph_native(0, para - 1, 4, None).unwrap();
        }
        doc.insert_text_native(0, para, 0, "가 문단").unwrap();
        doc.apply_char_format_native(0, para, 0, 4, &format!(r#"{{"fontSize":{size}}}"#))
            .unwrap();
        if let Some(first) = first {
            doc.apply_char_format_native(0, para, 0, 1, &format!(r#"{{"fontSize":{first}}}"#))
                .unwrap();
        }
    }
    doc
}

/// 1수준 번호 정의를 만들어 글자 모양 `shape`를 달고 모든 문단에 건다.
fn number_all(doc: &mut HwpDocument, count: usize, shape: Option<&str>) -> (u16, u32) {
    let id =
        doc.create_numbering(r#"{"levelFormats":["^1."],"numberFormats":[0],"startNumber":1}"#);
    let char_shape = doc.set_numbering_char_shape_native(id, 0, shape).unwrap();
    for para in 0..count {
        doc.apply_para_format_native(
            0,
            para,
            &format!(r#"{{"headType":"Number","paraLevel":0,"numberingId":{id}}}"#),
        )
        .unwrap();
    }
    (id, char_shape)
}

/// 번호 마커 run(원문 자리 없는 "1."·"2.")의 (크기, 기울임)을 위에서부터.
fn markers(doc: &HwpDocument) -> Vec<(f64, bool)> {
    let layout: Value = serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| {
            run.get("charStart").is_none()
                && run["text"]
                    .as_str()
                    .is_some_and(|text| text.ends_with('.') && text.len() > 1)
        })
        .map(|run| {
            (
                run["fontSize"].as_f64().unwrap(),
                run["italic"].as_bool().unwrap_or(false),
            )
        })
        .collect()
}

fn assert_sizes(actual: &[(f64, bool)], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len(), "{actual:?}");
    for ((size, italic), want) in actual.iter().zip(expected) {
        // 글 배치 JSON 은 크기를 0.1px 로 반올림한다.
        assert!((size - want).abs() < 0.06, "{actual:?} != {expected:?}");
        assert!(italic, "번호 글자 모양의 기울임: {actual:?}");
    }
}

fn reopen(doc: &HwpDocument, hwpx: bool) -> HwpDocument {
    let bytes = if hwpx {
        doc.export_hwpx_native().unwrap()
    } else {
        doc.export_hwp_native().unwrap()
    };
    HwpDocument::from_bytes(&bytes).unwrap()
}

#[test]
fn follow_body_size_number_draws_at_each_paragraph_end_size_and_survives_save() {
    let mut doc = paragraphs(&[2000, 1000], None);
    let (id, char_shape) = number_all(&mut doc, 2, Some(r#"{"fontSize":0,"italic":true}"#));
    let shape = &doc.document().doc_info.char_shapes[char_shape as usize];
    assert_eq!((shape.base_size, shape.italic), (0, true));
    assert_sizes(&markers(&doc), &[PT20_PX, PT10_PX]);
    for hwpx in [false, true] {
        let saved = reopen(&doc, hwpx);
        let head = saved.document().doc_info.numberings[usize::from(id) - 1].heads[0];
        assert_eq!(head.char_shape_id, char_shape, "hwpx={hwpx}");
        assert_eq!(
            saved.document().doc_info.char_shapes[char_shape as usize].base_size,
            0,
            "hwpx={hwpx}"
        );
        assert_sizes(&markers(&saved), &[PT20_PX, PT10_PX]);
    }
}

/// 한/글 12.30: 첫 글자 20pt·나머지와 문단 끝 10pt 문단의 '본문 글자 크기 따름' 번호는 10pt 다.
#[test]
fn follow_body_size_number_uses_paragraph_end_not_first_character() {
    let mut doc = paragraphs(&[1000], Some(2000));
    number_all(&mut doc, 1, Some(r#"{"fontSize":0,"italic":true}"#));
    assert_sizes(&markers(&doc), &[PT10_PX]);
}

#[test]
fn fixed_number_char_shape_keeps_its_size_and_detaching_restores_body_shape() {
    let mut doc = paragraphs(&[2000], None);
    let (id, char_shape) = number_all(&mut doc, 1, Some(r#"{"fontSize":1000,"italic":true}"#));
    assert_ne!(char_shape, 0, "글자 모양 지정 API의 기존 양수 ID 반환");
    assert_sizes(&markers(&doc), &[PT10_PX]);
    assert_eq!(
        doc.set_numbering_char_shape_native(id, 0, None).unwrap(),
        u32::MAX
    );
    let (size, italic) = markers(&doc)[0];
    assert!((size - PT20_PX).abs() < 0.06 && !italic, "{size} {italic}");
}

#[test]
fn number_char_shape_rejects_missing_numbering_and_level() {
    let mut doc = paragraphs(&[1000], None);
    let (id, _) = number_all(&mut doc, 1, None);
    assert!(doc
        .set_numbering_char_shape_native(id + 1, 0, None)
        .is_err());
    assert!(doc.set_numbering_char_shape_native(0, 0, None).is_err());
    assert!(doc.set_numbering_char_shape_native(id, 10, None).is_err());
}

/// HWPX에서 연 번호 정의는 원본 `hh:paraHead` 구간을 그대로 다시 쓰므로 그 수준의 글자 모양
/// 참조(`charPrIDRef`)도 함께 바뀌어야 저장본에 남는다.
#[test]
fn number_char_shape_reaches_hwpx_numbering_kept_from_source() {
    let mut doc = paragraphs(&[2000], None);
    let (id, _) = number_all(&mut doc, 1, None);
    let mut opened = reopen(&doc, true);
    assert!(opened.document().doc_info.numberings[usize::from(id) - 1]
        .raw_para_heads
        .is_some());
    let char_shape = opened
        .set_numbering_char_shape_native(id, 0, Some(r#"{"fontSize":0,"italic":true}"#))
        .unwrap();
    let saved = reopen(&opened, true);
    assert_eq!(
        saved.document().doc_info.numberings[usize::from(id) - 1].heads[0].char_shape_id,
        char_shape
    );
    assert_sizes(&markers(&saved), &[PT20_PX]);
}

fn fixture(name: &str) -> HwpDocument {
    let path = format!(
        "{}/tests/fixtures/numbering_char_shape/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

/// 한/글 12.30 Mac 저장본(저장자 이름만 바꿈). 번호 글자 모양은 기준 크기 0(본문 글자 크기 따름)이다.
///
/// - `follow-two-sizes`: 20pt 문단과 10pt 문단에 기울임 번호. 한/글은 20pt·10pt 번호로 그린다.
/// - `follow-end-20pt.hwp`: 글자는 10pt, 문단 끝만 20pt인 문단에 진하게 번호. 한/글은 20pt 번호로 그린다.
#[test]
fn hancom_follow_body_size_numbers_draw_at_the_paragraph_end_size() {
    for (name, sizes) in [
        ("follow-two-sizes.hwp", vec![PT20_PX, PT10_PX]),
        ("follow-two-sizes.hwpx", vec![PT20_PX, PT10_PX]),
        ("follow-end-20pt.hwp", vec![PT20_PX]),
    ] {
        let doc = fixture(name);
        let got = markers(&doc);
        assert_eq!(got.len(), sizes.len(), "{name}: {got:?}");
        for ((size, _), want) in got.iter().zip(&sizes) {
            assert!((size - want).abs() < 0.06, "{name}: {got:?}");
        }
    }
}
