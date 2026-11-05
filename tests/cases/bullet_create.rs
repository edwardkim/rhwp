//! 관계형 회귀: 입력 글자 모양·그림 자원·직렬화 보존. 크기는 입력 단위 계약이며 절대 배치 핀이 아니다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! character-bullet.hwp (api-8558-character-bullet-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! character-bullet.hwpx (api-8558-character-bullet-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! picture-bullet.hwp (api-8558-picture-bullet-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! picture-bullet.hwpx (api-8558-picture-bullet-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! actual-api-character-bullet.hwp (api-8558-actual-r3-character-bullet-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! actual-api-character-bullet.hwpx (api-8558-actual-r3-character-bullet-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! actual-api-picture-bullet.hwp (api-8558-actual-r3-picture-bullet-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! actual-api-picture-bullet.hwpx (api-8558-actual-r3-picture-bullet-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 글머리표·그림 글머리표 사용자 정의 모양(한/글 12.30 Mac).
//!
//! 한/글 '글머리표 사용자 정의 모양'은 문자·글자 모양·너비 조정·정렬·본문과의 간격·자동 내어쓰기를,
//! '그림 글머리표 사용자 정의 모양'은 그림 파일과 같은 머리 값을 고른다. 한/글 저장본(❏·가운데·
//! 10pt·70%·자동 내어쓰기 끔)은 HWP 글머리표 속성 1, 너비 1000, 간격 70이고, 그림 글머리표는
//! 여부 1과 이미지 글머리 데이터 넷째 바이트에 그림 번호를, HWPX는 `useImage="1"`과 core 이름공간
//! `<hc:img>`를 쓴다. 한/글은 head 이름공간 `<hh:img>`를 무시하고 글머리표 문자(●)를 그린다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const PT10_PX: f64 = 1000.0 / 75.0;
/// 1×1 PNG.
const PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// 20pt "가 문단" 한 문단.
fn paragraph() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "가 문단").unwrap();
    doc.apply_char_format_native(0, 0, 0, 4, r#"{"fontSize":2000}"#)
        .unwrap();
    doc
}

fn apply(doc: &mut HwpDocument, id: u16) {
    doc.apply_para_format_native(
        0,
        0,
        &format!(r#"{{"headType":"Bullet","paraLevel":0,"numberingId":{id}}}"#),
    )
    .unwrap();
}

fn reopen(doc: &HwpDocument, hwpx: bool) -> HwpDocument {
    let bytes = if hwpx {
        doc.export_hwpx_native().unwrap()
    } else {
        doc.export_hwp_native().unwrap()
    };
    HwpDocument::from_bytes(&bytes).unwrap()
}

fn header_xml(hwpx: &[u8]) -> String {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).unwrap();
    let mut out = String::new();
    zip.by_name("Contents/header.xml")
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    out
}

/// 글머리표 마커 run(원문 자리 없는 ❏)의 (크기, 기울임).
fn marker(doc: &HwpDocument) -> (f64, bool) {
    let layout: Value = serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    let run = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|run| run.get("charStart").is_none() && run["text"] == "❏")
        .expect("글머리표 마커");
    (
        run["fontSize"].as_f64().unwrap(),
        run["italic"].as_bool().unwrap_or(false),
    )
}

#[test]
fn custom_bullet_keeps_its_head_values_and_char_shape_in_hwp_and_hwpx() {
    let mut doc = paragraph();
    let id = doc
        .create_bullet_native(
            r#"{"char":"❏","align":1,"autoIndent":false,"widthAdjust":1000,"textOffset":70,
                "charShape":{"fontSize":1000,"italic":true}}"#,
            None,
        )
        .unwrap();
    assert_eq!(usize::from(id), doc.document().doc_info.bullets.len());
    apply(&mut doc, id);
    let (size, italic) = marker(&doc);
    assert!((size - PT10_PX).abs() < 0.06 && italic, "{size} {italic}");
    for (saved, hwpx) in [(reopen(&doc, false), false), (reopen(&doc, true), true)] {
        let bullet = &saved.document().doc_info.bullets[usize::from(id) - 1];
        // 한/글 저장본과 같은 머리 값: 가운데(1), 자동 내어쓰기 끔, 너비 10pt, 간격 70%.
        assert_eq!(
            (
                bullet.bullet_char,
                bullet.attr,
                bullet.width_adjust,
                bullet.text_distance
            ),
            ('❏', 1, 1000, 70),
            "hwpx={hwpx}"
        );
        assert_eq!(bullet.image_bullet, 0, "hwpx={hwpx}");
        let shape = &saved.document().doc_info.char_shapes[bullet.char_shape_id as usize];
        assert_eq!((shape.base_size, shape.italic), (1000, true), "hwpx={hwpx}");
        let (size, italic) = marker(&saved);
        assert!((size - PT10_PX).abs() < 0.06 && italic, "hwpx={hwpx}");
    }
}

/// 값을 주지 않으면 한/글 기본 머리 모양이고 글자 모양을 지정하지 않는다(참조 없음).
#[test]
fn bullet_defaults_follow_hancom_head_and_body_char_shape() {
    let mut doc = paragraph();
    let id = doc.create_bullet_native(r#"{"char":"■"}"#, None).unwrap();
    let bullet = &doc.document().doc_info.bullets[usize::from(id) - 1];
    assert_eq!(
        (
            bullet.attr,
            bullet.width_adjust,
            bullet.text_distance,
            bullet.char_shape_id
        ),
        (1 << 3, 0, 50, u32::MAX)
    );
}

#[test]
fn picture_bullet_embeds_its_image_draws_it_and_survives_save() {
    let mut doc = paragraph();
    let images = doc.document().bin_data_content.len();
    let id = doc
        .create_bullet_native(
            r#"{"char":"●","align":2,"autoIndent":false,"widthAdjust":800,"textOffset":30}"#,
            Some((PNG, "PNG")),
        )
        .unwrap();
    let bullet = &doc.document().doc_info.bullets[usize::from(id) - 1];
    assert_eq!(bullet.image_reference(), Some(images as u16 + 1));
    apply(&mut doc, id);
    let svg = doc.render_page_svg_native(0).unwrap();
    assert!(svg.contains("<image"), "그림 글머리표를 그림으로 그린다");
    let header = header_xml(&doc.export_hwpx_native().unwrap());
    assert!(
        header.contains(r#"<hc:img binaryItemIDRef="#) && !header.contains("<hh:img"),
        "한/글이 읽는 core 이름공간의 그림 참조"
    );
    for (saved, hwpx) in [(reopen(&doc, false), false), (reopen(&doc, true), true)] {
        let info = &saved.document().doc_info;
        let bullet = &info.bullets[usize::from(id) - 1];
        assert_eq!(
            (bullet.attr, bullet.width_adjust, bullet.text_distance),
            (2, 800, 30),
            "hwpx={hwpx}"
        );
        let image = bullet.image_reference().unwrap();
        let content = &saved.document().bin_data_content[usize::from(image) - 1];
        assert_eq!(&*content.data.load(), PNG, "hwpx={hwpx}");
        assert_eq!(content.extension, "png", "hwpx={hwpx}");
        assert!(saved.render_page_svg_native(0).unwrap().contains("<image"));
    }
}

#[test]
fn bullet_rejects_bad_values() {
    let mut doc = paragraph();
    let bullets = doc.document().doc_info.bullets.len();
    for json in [
        r#"{}"#,
        r#"{"char":""}"#,
        r#"{"char":"가나"}"#,
        r#"{"char":"\u000b"}"#,
        r#"{"char":"😀"}"#,
        r#"{"char":"●","align":3}"#,
        r#"{"char":"●","textOffset":101}"#,
        r#"{"char":"●","autoIndent":1}"#,
        r#"{"char":"●","charShape":12}"#,
    ] {
        assert!(doc.create_bullet_native(json, None).is_err(), "{json}");
    }
    assert!(doc
        .create_bullet_native(r#"{"char":"●"}"#, Some((&[], "png")))
        .is_err());
    assert!(doc
        .create_bullet_native(r#"{"char":"●"}"#, Some((PNG, "p/ng")))
        .is_err());
    assert_eq!(doc.document().doc_info.bullets.len(), bullets);
}
