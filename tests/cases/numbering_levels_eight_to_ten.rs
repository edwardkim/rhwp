//! 관계형·고정형: 검사 대상 — 한/글 12.30이 저장한 8~10수준 개요·번호 문단.
//! 입력·기준: `tests/fixtures/numbering_levels/levels.hwp`·`levels.hwpx`(한/글 12.30 Mac 저장본, 저장자 이름만
//! 바꿈). 개요 1·7·8·9·10수준과 번호(1. 1.1. 1.1.1. 견본) 1·8·9·10수준 문단이 있다.
//! 모델·API·직렬화 계약을 검사하며 화면 시각 일치율은 측정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
//! 한/글은 이 문서의 번호 문단을 `1.`·`1.1.1.1.1.1.1.1.`·`1.1.1.1.1.1.1.1.1.`·
//! `1.1.1.1.1.1.1.1.1.1.`로, 개요 9·10수준은 번호 서식이 비어 번호 없이 그린다.
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;
use std::io::{Cursor, Read};

fn fixture(name: &str) -> HwpDocument {
    let path = format!(
        "{}/tests/fixtures/numbering_levels/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

/// 문단 수준과 위에서부터의 번호 마커 글(원문 자리 없는 글).
fn markers(doc: &HwpDocument) -> (Vec<u8>, Vec<String>) {
    let document = doc.document();
    let levels = document.sections[0]
        .paragraphs
        .iter()
        .map(|p| document.doc_info.para_shapes[p.para_shape_id as usize].para_level)
        .collect();
    let layout: Value = serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    let texts = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run.get("charStart").is_none())
        .filter_map(|run| run["text"].as_str().map(str::to_string))
        .collect();
    (levels, texts)
}

#[test]
fn hancom_levels_eight_to_ten_keep_their_level_and_number_after_saving() {
    let number = |n: usize| "1.".repeat(n);
    for name in ["levels.hwp", "levels.hwpx"] {
        let doc = fixture(name);
        let check = |doc: &HwpDocument, label: &str| {
            let (levels, texts) = markers(doc);
            assert_eq!(levels, [0, 6, 7, 8, 9, 0, 7, 8, 9], "{label}");
            // 개요 1·7·8수준과 번호 1·8·9·10수준만 번호를 그린다. 개요 9·10수준은 서식이 비었다.
            assert_eq!(texts.len(), 7, "{label}: {texts:?}");
            assert_eq!(
                texts[3..],
                [number(1), number(8), number(9), number(10)],
                "{label}"
            );
        };
        check(&doc, name);
        for hwpx in [false, true] {
            let saved = if hwpx {
                doc.export_hwpx_native().unwrap()
            } else {
                doc.export_hwp_native().unwrap()
            };
            if hwpx {
                // 한/글처럼 8~10수준 머리는 2016 문단 네임스페이스 case 에만 두고 default 는 머리 없음이다.
                let mut header = String::new();
                zip::ZipArchive::new(Cursor::new(&saved))
                    .unwrap()
                    .by_name("Contents/header.xml")
                    .unwrap()
                    .read_to_string(&mut header)
                    .unwrap();
                assert!(header.contains(concat!(
                    r#"<hp:switch><hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/paragraph">"#,
                    r#"<hh:heading type="NUMBER" idRef="2" level="9"/></hp:case><hp:default>"#,
                    r#"<hh:heading type="NONE" idRef="0" level="0"/></hp:default></hp:switch>"#,
                )));
            }
            check(
                &HwpDocument::from_bytes(&saved).unwrap(),
                &format!("{name} → hwpx={hwpx}"),
            );
        }
    }
}

/// 한/글 5.1은 번호 레코드의 7수준 시작 번호 뒤에 8~10수준 머리 정보·형식 문자열 3개와 시작 번호 3개를
/// 덧붙인다. 모델에서 다시 쓴 레코드가 한/글 저장본과 바이트 단위로 같다.
#[test]
fn hancom_numbering_records_are_rewritten_byte_for_byte() {
    let mut doc = fixture("levels.hwp");
    let hancom: Vec<Vec<u8>> = doc
        .document()
        .doc_info
        .numberings
        .iter()
        .map(|n| n.raw_data.clone().unwrap())
        .collect();
    let formats = |doc: &HwpDocument| -> Vec<Vec<String>> {
        doc.document()
            .doc_info
            .numberings
            .iter()
            .map(|n| n.level_formats.to_vec())
            .collect()
    };
    assert_eq!(formats(&doc)[0][7..], ["^8", "", ""]);
    assert_eq!(formats(&doc)[1][9], "^1.^2.^3.^4.^5.^6.^7.^8.^9.^:.");
    let info = &mut doc.document_mut().doc_info;
    for numbering in &mut info.numberings {
        numbering.raw_data = None;
    }
    info.raw_stream_dirty = true;
    let saved = HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    let rewritten: Vec<Vec<u8>> = saved
        .document()
        .doc_info
        .numberings
        .iter()
        .map(|n| n.raw_data.clone().unwrap())
        .collect();
    assert_eq!(rewritten, hancom);
}

/// 개요 찾아가기도 8~10수준을 제 수준으로 보인다. 9·10수준은 번호 서식이 비어 목록에 넣지 않는다.
#[test]
fn outline_navigation_lists_level_eight_on_its_own_level() {
    for name in ["levels.hwp", "levels.hwpx"] {
        let navigation: Value =
            serde_json::from_str(&fixture(name).get_outline_navigation_native().unwrap()).unwrap();
        let levels: Vec<u64> = navigation["outline"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["level"].as_u64().unwrap())
            .collect();
        assert_eq!(levels, [1, 7, 8], "{name}");
    }
}
