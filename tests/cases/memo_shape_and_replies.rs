//! 메모 모양과 메모 댓글 (#8017 후속). 기준 파일은 한컴 12.30 Mac이 저장한 문서다.
//!
//! - `shape-new-section.hwp(x)`: 메모 모양(삼중선 3.0mm)을 문서 전체에 준 뒤 0.12mm를 새 구역으로 적용
//! - `reply.hwp(x)`: 메모 둘과 댓글 셋

use rhwp::document_core::DocumentCore;
use rhwp::model::memo_extended;
use rhwp::model::memo_shape::{self, MemoShape};
use rhwp::parser::tags::HWPTAG_MEMO_SHAPE;

const SHAPE_HWP: &[u8] = include_bytes!("../fixtures/memo_hancom/shape-new-section.hwp");
const SHAPE_HWPX: &[u8] = include_bytes!("../fixtures/memo_hancom/shape-new-section.hwpx");
const REPLY_HWP: &[u8] = include_bytes!("../fixtures/memo_hancom/reply.hwp");
const REPLY_HWPX: &[u8] = include_bytes!("../fixtures/memo_hancom/reply.hwpx");

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}

const SHAPE: &str = r##"{"width":19842,"lineType":2,"lineWidth":7,"lineColor":"#C75252","fillColor":"#DFEAF5","activeColor":"#FAEDD2"}"##;

#[test]
fn hancom_memo_shape_record_reads_and_writes_back() {
    // 한컴 12.30 Mac: 너비 70mm, 파선(DOT), 0.6mm, #C75252 / #DFEAF5 / #FAEDD2
    let data = [
        0x82, 0x4d, 0, 0, 2, 7, 0xc7, 0x52, 0x52, 0, 0xdf, 0xea, 0xf5, 0, 0xfa, 0xed, 0xd2, 0, 0,
        0, 0, 0,
    ];
    let shape = MemoShape::from_record(&data).unwrap();
    assert_eq!(shape.width, 19842);
    assert_eq!(memo_shape::color_hex(shape.line_color), "#C75252");
    assert_eq!(shape.to_record(), data);
    assert!(memo_shape::memo_properties_xml(&[shape]).contains(
        r##"<hh:memoPr id="1" width="19842" lineWidth="7" lineType="DOT" lineColor="#C75252" fillColor="#DFEAF5" activeColor="#FAEDD2" memoType="NOMAL"/>"##
    ));
    assert_eq!(
        memo_shape::color_from_hex("#C75252"),
        Some(shape.line_color)
    );
    // 한컴 메모 모양 창의 둘째 그림(파선)은 DOT·2, 셋째 그림(점선)은 DASH·3, 삼중선은 SLIM_THICK_SLIM·11.
    assert_eq!(memo_shape::line_type_from_name("DOT"), 2);
    assert_eq!(memo_shape::line_type_from_name("DASH"), 3);
    assert_eq!(memo_shape::line_type_from_name("SLIM_THICK_SLIM"), 11);
}

#[test]
fn hancom_hwpx_and_hwp_memo_shapes_agree() {
    let records = |core: &DocumentCore| {
        core.document()
            .doc_info
            .extra_records
            .iter()
            .filter(|record| record.tag_id == HWPTAG_MEMO_SHAPE)
            .map(|record| record.data.clone())
            .collect::<Vec<_>>()
    };
    let hwp = DocumentCore::from_bytes(SHAPE_HWP).unwrap();
    let hwpx = DocumentCore::from_bytes(SHAPE_HWPX).unwrap();
    assert_eq!(records(&hwpx), records(&hwp));
}

#[test]
fn hancom_new_section_shapes_follow_each_section_in_both_formats() {
    let check = |core: &DocumentCore| {
        assert!(core
            .get_memo_shape_json(0)
            .unwrap()
            .contains(r#""lineType":11,"lineWidth":12"#));
        assert!(core
            .get_memo_shape_json(1)
            .unwrap()
            .contains(r#""lineType":11,"lineWidth":1,"#));
    };
    for bytes in [SHAPE_HWP, SHAPE_HWPX] {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        check(&core);
        for out in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            check(&DocumentCore::from_bytes(&out).unwrap());
        }
    }
}

#[test]
fn set_memo_shape_round_trips_and_rewrites_the_shared_shape() {
    let mut core = blank();
    assert!(core
        .get_memo_shape_json(0)
        .unwrap()
        .contains(r##""lineColor":"#B6D7AE""##));
    core.set_memo_shape_json(None, SHAPE).unwrap();
    assert_eq!(core.get_memo_shape_json(0).unwrap(), SHAPE);
    // 문서 전체에 다시 주면 같은 모양을 고쳐 쓴다.
    let wider = SHAPE.replace("19842", "20000");
    core.set_memo_shape_json(None, &wider).unwrap();
    assert_eq!(memo_shape::memo_shapes(&core.document().doc_info).len(), 1);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.get_memo_shape_json(0).unwrap(), wider);
    }
}

#[test]
fn set_memo_shape_rejects_a_shape_it_cannot_store() {
    let mut core = blank();
    assert!(core
        .set_memo_shape_json(None, &SHAPE.replace("#C75252", "red"))
        .is_err());
    assert!(core
        .set_memo_shape_json(None, &SHAPE.replace(r#""lineType":2"#, r#""lineType":20"#))
        .is_err());
    assert!(core.set_memo_shape_json(Some(9), SHAPE).is_err());
}

#[test]
fn hancom_memo_extended_stream_and_part_round_trip() {
    // 한컴 12.30 Mac reply.hwp의 MemoExtended(풀어 쓴 것): 댓글 셋.
    let hwp = [
        0x65, 0x00, 0xc0, 0x01, 3, 0, 0, 0, 0xb8, 0x8b, 0xd5, 0x49, 0xb7, 0x8b, 0xd5, 0x49, 0xc6,
        0x8b, 0xd5, 0x49, 0xb9, 0x8b, 0xd5, 0x49, 0xc7, 0x8b, 0xd5, 0x49, 0xb7, 0x8b, 0xd5, 0x49,
    ];
    let links = memo_extended::decode_hwp(&hwp).unwrap();
    assert_eq!(
        links,
        [
            (1238731704, 1238731703),
            (1238731718, 1238731705),
            (1238731719, 1238731703)
        ]
    );
    assert_eq!(memo_extended::encode_hwp(&links), hwp);
    let xml = memo_extended::encode_hwpx(&links);
    assert!(xml.contains(r#"<he:memo id="1238731718" parentId="1238731705"/>"#));
    assert_eq!(memo_extended::decode_hwpx(&xml), links);
    assert_eq!(memo_extended::decode_hwp(&hwp[..20]), None);
}

/// (번호, 부모 번호, 글)을 문서 차례로.
fn memos(core: &DocumentCore) -> Vec<(u64, Option<u64>, String)> {
    let list: serde_json::Value = serde_json::from_str(&core.get_memo_list_json()).unwrap();
    list.as_array()
        .unwrap()
        .iter()
        .map(|memo| {
            (
                memo["id"].as_u64().unwrap(),
                memo["parentId"].as_u64(),
                memo["text"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn hancom_replies() -> Vec<(u64, Option<u64>, String)> {
    vec![
        (1238731703, None, "첫 메모 내용입니다.".into()),
        (1238731704, Some(1238731703), "댓글입니다.".into()),
        (1238731719, Some(1238731703), "첫 메모의 둘째 댓글".into()),
        (1238731705, None, "캐럿 자리 메모입니다.".into()),
        (1238731718, Some(1238731705), "셋째 메모의 댓글".into()),
    ]
}

#[test]
fn hancom_replies_survive_both_formats() {
    for bytes in [REPLY_HWP, REPLY_HWPX] {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        assert_eq!(memos(&core), hancom_replies());
        for out in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            assert_eq!(
                memos(&DocumentCore::from_bytes(&out).unwrap()),
                hancom_replies()
            );
        }
    }
}

#[test]
fn reply_shares_the_anchor_and_goes_with_its_parent() {
    let mut core = DocumentCore::from_bytes(REPLY_HWPX).unwrap();
    // 댓글의 댓글은 맨 위 메모의 댓글이 된다.
    let result = core
        .insert_memo_reply_json_native(1238731718, r#"{"author":"t","text":"새 댓글"}"#)
        .unwrap();
    let id: serde_json::Value = serde_json::from_str(&result).unwrap();
    let id = id["id"].as_u64().unwrap();
    let list: serde_json::Value = serde_json::from_str(&core.get_memo_list_json()).unwrap();
    let find = |id: u64| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|memo| memo["id"].as_u64() == Some(id))
            .unwrap()
            .clone()
    };
    let (reply, parent) = (find(id), find(1238731705));
    assert_eq!(reply["parentId"].as_u64(), Some(1238731705));
    for key in ["paraIndex", "startCharIdx", "endCharIdx"] {
        assert_eq!(reply[key], parent[key], "{key}");
    }
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    assert!(memos(&reopened).contains(&(id, Some(1238731705), "새 댓글".into())));

    // 댓글을 지우면 관계만 사라지고, 부모를 지우면 댓글도 지운다.
    core.remove_memo(1238731704).unwrap();
    assert!(!core
        .document()
        .memo_parents
        .iter()
        .any(|(reply, _)| *reply == 1238731704));
    core.remove_memo(1238731705).unwrap();
    let left: Vec<u64> = memos(&core).into_iter().map(|(id, _, _)| id).collect();
    assert_eq!(left, [1238731703, 1238731719]);
    let out = core.export_hwpx_native().unwrap();
    assert_eq!(
        memos(&DocumentCore::from_bytes(&out).unwrap()),
        [
            (1238731703, None, "첫 메모 내용입니다.".into()),
            (1238731719, Some(1238731703), "첫 메모의 둘째 댓글".into()),
        ]
    );
}
