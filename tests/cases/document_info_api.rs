//! 문서 정보 창 API — 문서 요약 읽기·쓰기, 그림 정보, 저작권, 개체 수.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::queries::document_info::DocumentSummary;
use rhwp::wasm_api::HwpDocument;

fn open(name: &str) -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(name);
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn patch() -> DocumentSummary {
    DocumentSummary {
        title: Some("새 제목 & <부제>".into()),
        subject: Some("주제".into()),
        author: Some("지은이".into()),
        date: Some("2026년 11월 15일 일요일 오후 9:36:59".into()),
        keywords: Some("가, 나".into()),
        comments: Some("첫 줄\n둘째 줄".into()),
        last_saved_by: Some("kkom".into()),
        created_at: Some("2004-09-15T12:15:12Z".into()),
        last_saved_at: Some("2026-11-15T12:46:54Z".into()),
    }
}

/// HWP 로 연 문서와 HWPX 로 연 문서 모두, 고친 요약이 HWP·HWPX 어느 쪽으로 저장해도 남는다.
#[test]
fn summary_survives_hwp_and_hwpx_saves_from_either_source() {
    for name in ["field-01.hwp", "pic2.hwpx"] {
        let mut doc = open(name);
        let before = doc.document_summary();
        assert!(before.title.is_some(), "{name}: 읽으면 모든 글 칸을 채운다");
        doc.update_document_summary(patch());
        assert_eq!(doc.document_summary(), patch(), "{name}");
        for bytes in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
            let reopened = HwpDocument::from_bytes(&bytes).unwrap();
            assert_eq!(reopened.document_summary(), patch(), "{name}");
        }
    }
}

/// 일부 칸만 고치면 나머지 칸은 그대로다(저장할 때 마지막 저장한 사람·날짜만 바꾸는 경우).
#[test]
fn partial_summary_patch_keeps_other_fields() {
    let mut doc = open("field-01.hwp");
    let before = doc.document_summary();
    doc.set_document_summary_json(
        r#"{"lastSavedBy":"새 사람","lastSavedAt":"2026-11-15T00:00:00Z"}"#,
    )
    .unwrap();
    let after = doc.document_summary();
    assert_eq!(after.last_saved_by.as_deref(), Some("새 사람"));
    assert_eq!(after.last_saved_at.as_deref(), Some("2026-11-15T00:00:00Z"));
    assert_eq!(after.title, before.title);
    assert_eq!(after.author, before.author);
    assert_eq!(after.created_at, before.created_at);
    assert!(doc
        .set_document_summary_json(r#"{"createdAt":"어제"}"#)
        .is_err());
    // 빈 날짜는 지운다(새 문서가 서식 파일의 저장 정보를 물려받지 않게).
    doc.set_document_summary_json(r#"{"lastSavedAt":"","lastSavedBy":""}"#)
        .unwrap();
    for bytes in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap().document_summary();
        assert_eq!(reopened.last_saved_at, None);
        assert_eq!(reopened.last_saved_by.as_deref(), Some(""));
        assert_eq!(reopened.created_at, before.created_at);
    }
}

/// 머리말 안 그림도 표·그림·글상자 수와 그림 목록에 든다.
#[test]
fn header_pictures_are_counted_and_listed_with_their_page() {
    let doc = open("pic-in-head-01.hwp");
    let count = doc.object_count();
    assert!(count.pictures >= 1, "{count:?}");
    let controls: Vec<serde_json::Value> = serde_json::from_str(&doc.get_controls()).unwrap();
    let body_pictures = controls.iter().filter(|c| c["userDesc"] == "그림").count() as u32;
    assert!(
        count.pictures > body_pictures,
        "{count:?} 본문 {body_pictures}"
    );
    let list = doc.bin_data_infos();
    assert!(!list.is_empty());
    assert!(list.iter().all(|b| b.kind == "embed" && b.path.is_empty()));
    assert_eq!(list[0].id, 1);
    assert_eq!(
        list.iter()
            .find(|b| b.first_page.is_some())
            .and_then(|b| b.first_page),
        Some(1)
    );
}

/// 저작권 — HWPX `hh:licensemark`, HWP FileHeader 의 CCL 비트·조건·언어(한컴 12.30 저장본에서 잰 자리).
#[test]
fn license_mark_reads_hwpx_and_hwp_storage() {
    let mut doc = open("pic2.hwpx");
    assert_eq!(doc.license_mark(), None);
    doc.document_mut().doc_info.hwpx_head_tail = Some(
        r#"<hh:docOption><hh:linkinfo path="" pageInherit="0" footnoteInherit="0"/><hh:licensemark type="CCL" flag="5" lang="6"/></hh:docOption>"#
            .into(),
    );
    assert_eq!(doc.license_mark(), Some(("CCL".into(), 5, 6)));
    let json: serde_json::Value = serde_json::from_str(&doc.license_mark_json()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "type": "CCL", "flag": 5, "lang": 6 })
    );

    let mut doc = open("field-01.hwp");
    assert_eq!(doc.license_mark(), None);
    let raw = doc.document_mut().header.raw_data.as_mut().unwrap();
    raw[36..40].copy_from_slice(&(0x801u32).to_le_bytes());
    raw[40..44].copy_from_slice(&2u32.to_le_bytes());
    raw[48..52].copy_from_slice(&6u32.to_le_bytes());
    assert_eq!(doc.license_mark(), Some(("CCL".into(), 2, 6)));
}
