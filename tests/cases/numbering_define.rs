//! 문단 번호 정의 만들기·제자리 고치기(`set_numbering_native`).
//!
//! 한/글 12.30의 개요 번호 모양은 구역이 가리키는 번호 정의를 고쳐 쓴다. 고친 정의는 번호가 그대로라
//! 그 번호를 가리키는 개요 문단·구역 정의가 함께 바뀌고, 번호 정의 수는 늘지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::style::NUMBERING_LEVELS;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const USE_INST_WIDTH: u32 = 1 << 2;
const AUTO_INDENT: u32 = 1 << 3;

fn level(format: &str, code: u8) -> String {
    format!(
        r#"{{"format":"{format}","numberFormat":{code},"start":3,"align":1,"useInstWidth":false,"autoIndent":true,"widthAdjust":1000,"textOffset":70}}"#
    )
}

fn levels(count: usize, level: impl Fn(usize) -> String) -> String {
    let levels = (1..=count).map(level).collect::<Vec<_>>();
    format!(r#"{{"levels":[{}]}}"#, levels.join(","))
}

fn reopen(doc: &HwpDocument, hwpx: bool) -> HwpDocument {
    let bytes = if hwpx {
        doc.export_hwpx_native().unwrap()
    } else {
        doc.export_hwp_native().unwrap()
    };
    HwpDocument::from_bytes(&bytes).unwrap()
}

/// 1쪽의 문단 머리 run(원문 자리 없는 글).
fn heads(doc: &HwpDocument) -> Vec<String> {
    let layout: Value = serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run.get("charStart").is_none())
        .filter_map(|run| run["text"].as_str().map(|text| text.trim().to_string()))
        .filter(|text| !text.is_empty())
        .collect()
}

/// 개요 1·2 문단 둘을 둔 새 문서와 구역이 가리키는 개요 번호 정의.
fn outline_document() -> (HwpDocument, u16) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "첫 장").unwrap();
    doc.split_paragraph_native(0, 0, 3, None).unwrap();
    doc.insert_text_native(0, 1, 0, "첫 절").unwrap();
    for para in 0..2 {
        doc.apply_para_format_native(
            0,
            para,
            &format!(r#"{{"headType":"Outline","paraLevel":{para}}}"#),
        )
        .unwrap();
    }
    let id = doc.document().sections[0].section_def.outline_numbering_id;
    assert!(id > 0, "새 문서 구역은 개요 번호 정의를 가리킨다");
    (doc, id)
}

#[test]
fn creates_numbering_with_every_head_value_on_ten_levels() {
    let mut doc = HwpDocument::create_empty();
    let before = doc.document().doc_info.numberings.len();
    let id = doc
        .set_numbering_native(0, &levels(10, |n| level(&format!("^{n})"), 10)))
        .unwrap();
    assert_eq!(usize::from(id), before + 1);
    let numbering = &doc.document().doc_info.numberings[before];
    assert_eq!(numbering.level_formats[2], "^3)");
    assert_eq!(numbering.level_formats[9], "^10)");
    assert_eq!(numbering.start_number, 3);
    assert_eq!(numbering.level_start_numbers, [3; NUMBERING_LEVELS]);
    for head in numbering.heads {
        assert_eq!(head.attr, 1 | AUTO_INDENT | (10 << 5));
        assert_eq!((head.width_adjust, head.text_distance), (1000, 70));
        assert_eq!(head.char_shape_id, u32::MAX);
    }
    assert!(numbering.raw_para_heads.is_none());
    assert!(doc.document().doc_info.raw_stream_dirty);
}

/// 예전 앱이 남긴 7수준 모양은 한/글의 빈 8~10수준(서식 없음·본문과의 간격 50%)을 더한다.
#[test]
fn fills_levels_eight_to_ten_of_a_seven_level_shape_like_hancom() {
    let mut doc = HwpDocument::create_empty();
    let id = doc
        .set_numbering_native(0, &levels(7, |n| level(&format!("^{n}"), 0)))
        .unwrap();
    let numbering = &doc.document().doc_info.numberings[usize::from(id) - 1];
    assert_eq!(numbering.level_formats[6], "^7");
    assert!(numbering.level_formats[7..].iter().all(String::is_empty));
    assert_eq!(numbering.heads[9].attr, USE_INST_WIDTH | AUTO_INDENT);
    assert_eq!(numbering.heads[9].text_distance, 50);
}

/// 한/글 '본문 글자 크기 따름'은 번호 글자 모양의 기준 크기를 0으로 저장한다.
#[test]
fn attaches_number_char_shapes_and_keeps_them_in_hwp_and_hwpx() {
    let mut doc = HwpDocument::create_empty();
    let json = levels(10, |n| {
        let base = level(&format!("^{n}."), 0);
        match n {
            1 => base.replace('}', r#","charShape":{"fontSize":0,"italic":true}}"#),
            2 => base.replace('}', r#","charShape":{"fontSize":1200,"bold":true}}"#),
            _ => base.replace('}', r#","charShape":null}"#),
        }
    });
    let id = doc.set_numbering_native(0, &json).unwrap();
    let check = |doc: &HwpDocument| {
        let info = &doc.document().doc_info;
        let heads = info.numberings[usize::from(id) - 1].heads;
        let follow = &info.char_shapes[heads[0].char_shape_id as usize];
        assert_eq!((follow.base_size, follow.italic), (0, true));
        let fixed = &info.char_shapes[heads[1].char_shape_id as usize];
        assert_eq!((fixed.base_size, fixed.bold), (1200, true));
        assert_eq!(heads[2].char_shape_id, u32::MAX);
    };
    check(&doc);
    for hwpx in [false, true] {
        check(&reopen(&doc, hwpx));
    }
}

#[test]
fn rejects_out_of_range_values_without_changing_the_document() {
    let (mut doc, id) = outline_document();
    let before = format!("{:?}", doc.document().doc_info.numberings);
    let bad = level("^1", 0).replace(r#""textOffset":70"#, r#""textOffset":101"#);
    let shape = level("^1", 0).replace('}', r#","charShape":12}"#);
    for json in [
        levels(10, |_| bad.clone()),
        r#"{"levels":[]}"#.to_string(),
        levels(10, |_| level("^1", 17)),
        // XML에 쓸 수 없는 제어 문자는 막는다.
        levels(10, |_| level("^1\\u000b", 0)),
        levels(10, |_| shape.clone()),
    ] {
        assert!(doc.set_numbering_native(id, &json).is_err());
        assert!(doc.set_numbering_native(0, &json).is_err());
    }
    let missing = doc.document().doc_info.numberings.len() as u16 + 1;
    let good = levels(10, |n| level(&format!("^{n}."), 0));
    assert!(doc.set_numbering_native(missing, &good).is_err());
    assert_eq!(format!("{:?}", doc.document().doc_info.numberings), before);
}

/// 한/글 12.30처럼 개요 번호 모양을 거듭 바꿔도 번호 정의가 쌓이지 않고, 개요 문단·구역이 같은 정의를
/// 가리킨 채 새 모양으로 그려진다. 원본 레코드(HWP)·원본 paraHead(HWPX)가 있는 문서도 새 값으로 저장한다.
#[test]
fn edits_the_outline_numbering_in_place_and_keeps_references_in_hwp_and_hwpx() {
    let (blank, id) = outline_document();
    for source_hwpx in [false, true] {
        // 저장했다 연 문서는 번호 정의의 원본(HWP 레코드·HWPX paraHead)을 지닌다.
        let mut doc = reopen(&blank, source_hwpx);
        let count = doc.document().doc_info.numberings.len();
        let para_ids: Vec<u16> = doc
            .document()
            .doc_info
            .para_shapes
            .iter()
            .map(|ps| ps.numbering_id)
            .collect();
        for (round, chapter) in ["제^1장", "^1편", "제^1장"].iter().enumerate() {
            let json = levels(10, |n| match n {
                1 => level(chapter, 0),
                2 => level("^2절", 0),
                _ => level(&format!("^{n}."), 0),
            })
            .replace(r#""start":3"#, r#""start":1"#);
            assert_eq!(doc.set_numbering_native(id, &json).unwrap(), id);
            let label = chapter.replace("^1", "1");
            for saved in [None, Some(false), Some(true)] {
                let reopened = saved.map(|hwpx| reopen(&doc, hwpx));
                let check = reopened.as_ref().unwrap_or(&doc);
                let info = &check.document().doc_info;
                let at = format!("source_hwpx={source_hwpx} round={round} saved={saved:?}");
                assert_eq!(info.numberings.len(), count, "{at}");
                assert_eq!(
                    info.numberings[usize::from(id) - 1].level_formats[0],
                    *chapter,
                    "{at}"
                );
                assert_eq!(
                    check.document().sections[0]
                        .section_def
                        .outline_numbering_id,
                    id,
                    "{at}"
                );
                let shown = heads(check);
                assert!(shown.contains(&label), "{at}: {shown:?}");
                assert!(shown.contains(&"1절".to_string()), "{at}: {shown:?}");
            }
            assert_eq!(
                doc.document()
                    .doc_info
                    .para_shapes
                    .iter()
                    .map(|ps| ps.numbering_id)
                    .collect::<Vec<_>>(),
                para_ids
            );
        }
    }
}
