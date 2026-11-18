//! 관계형: 손상된 댓글 부모 순환은 편집을 거절하고 문서 내용·댓글 관계를 보존한다.
//! 한컴 12.30 댓글 원본을 파싱한 뒤 부모 관계만 변이해 실제 HWPX 저장·로드·삭제 경로를 호출한다.
//! 화면 px 검사가 아니므로 Visual Sweep 일치율의 적용 대상이 아니다.
//! 주석 갱신: 2026-11-10T15:14:25+09:00.
use rhwp::document_core::DocumentCore;

#[test]
fn cyclic_reply_parents_reject_removal_without_changing_the_document() {
    let mut document =
        rhwp::parser::parse_document(include_bytes!("../fixtures/memo_hancom/reply.hwpx")).unwrap();
    document
        .memo_parents
        .extend([(1238731703, 1238731705), (1238731705, 1238731703)]);
    let bytes = rhwp::serializer::serialize_hwpx(&document).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let before = core.get_memo_list_json();
    assert!(core.remove_memo(1238731703).is_err());
    assert_eq!(core.get_memo_list_json(), before);
    assert_eq!(core.document().memo_parents, document.memo_parents);
}

#[test]
fn cyclic_reply_parents_reject_insertion_without_changing_the_document() {
    let mut document =
        rhwp::parser::parse_document(include_bytes!("../fixtures/memo_hancom/reply.hwpx")).unwrap();
    document
        .memo_parents
        .extend([(1238731703, 1238731705), (1238731705, 1238731703)]);
    let bytes = rhwp::serializer::serialize_hwpx(&document).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let before = core.get_memo_list_json();
    assert!(core
        .insert_memo_reply_json_native(1238731703, r#"{"author":"검사","text":"순환"}"#)
        .is_err());
    assert_eq!(core.get_memo_list_json(), before);
    assert_eq!(core.document().memo_parents, document.memo_parents);
}

#[test]
fn nested_reply_finds_the_root_and_removal_preserves_the_other_thread() {
    let mut document =
        rhwp::parser::parse_document(include_bytes!("../fixtures/memo_hancom/reply.hwpx")).unwrap();
    document.memo_parents = vec![
        (1238731704, 1238731703),
        (1238731719, 1238731704),
        (1238731718, 1238731705),
    ];
    let bytes = rhwp::serializer::serialize_hwpx(&document).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let inserted: serde_json::Value = serde_json::from_str(
        &core
            .insert_memo_reply_json_native(1238731719, r#"{"author":"검사","text":"최상위 댓글"}"#)
            .unwrap(),
    )
    .unwrap();
    let new_id = inserted["id"].as_u64().unwrap() as u32;
    assert!(core.document().memo_parents.contains(&(new_id, 1238731703)));
    core.remove_memo(1238731703).unwrap();
    let list: serde_json::Value = serde_json::from_str(&core.get_memo_list_json()).unwrap();
    let ids: Vec<_> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|memo| memo["id"].as_u64().unwrap())
        .collect();
    assert_eq!(ids, [1238731705, 1238731718]);
    assert_eq!(core.document().memo_parents, [(1238731718, 1238731705)]);
}
