//! #6963 후속: 문서 내부 책갈피 링크의 Command 인코딩·삽입·수정·조회·저장 왕복.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::hyperlink::HyperlinkTarget;
use rhwp::document_core::DocumentCore;
use rhwp::model::hyperlink::{bookmark_command, command_bookmark};

const NAME: &str = r"대상:가;나\다";

/// 0번 문단은 링크를 걸 글, 1번 문단은 책갈피 두 개를 담는다.
fn doc_with_bookmarks() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "링크 문단책갈피 문단")
        .unwrap();
    core.split_paragraph_native(0, 0, 5, None).unwrap();
    for (offset, name) in [(0, NAME), (3, "둘째")] {
        let result = core.add_bookmark_native(0, 1, offset, name).unwrap();
        assert!(result.contains("\"ok\":true"), "{result}");
    }
    core
}

fn roundtrips(core: &DocumentCore) -> Vec<DocumentCore> {
    [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .iter()
    .map(|bytes| DocumentCore::from_bytes(bytes).unwrap())
    .collect()
}

fn only_link(core: &DocumentCore) -> (Option<String>, String, usize, usize) {
    let links = core
        .hyperlinks_native(&HyperlinkTarget::body(0, 0))
        .unwrap();
    assert_eq!(links.len(), 1, "{links:?}");
    let l = &links[0];
    (l.bookmark.clone(), l.uri.clone(), l.start, l.end)
}

#[test]
fn bookmark_command_matches_hancom_encoding() {
    // 한컴 샘플의 실제 Command와 같은 문자열이어야 한다.
    assert_eq!(
        bookmark_command("예제 : 배포용 문서 만들기").unwrap(),
        r"?예제 \: 배포용 문서 만들기;0;0;0;"
    );
    let command = bookmark_command(NAME).unwrap();
    assert_eq!(command, r"?대상\:가\;나\\다;0;0;0;");
    assert_eq!(command_bookmark(&command).as_deref(), Some(NAME));
    assert_eq!(command_bookmark(r"https\://a.example;1;0;0;"), None);
    for invalid in ["", "  ", "줄\n바꿈"] {
        assert!(bookmark_command(invalid).is_err(), "{invalid:?}");
    }
}

#[test]
fn hancom_sample_bookmark_link_reports_its_bookmark() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwpctl_Action_Table__v1.1.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let cell = HyperlinkTarget {
        section: 0,
        para: 8,
        cell_path: vec![(0, 200, 2)],
    };
    let links = core.hyperlinks_native(&cell).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].text, "배포용 문서 만들기");
    assert_eq!(
        links[0].bookmark.as_deref(),
        Some("예제 : 배포용 문서 만들기")
    );
    // 대상은 블록 책갈피 필드다. 같은 대상으로 링크를 수정하면 변경 없음이다.
    let id = links[0].field_id;
    assert!(!core
        .update_bookmark_hyperlink_native(&cell, id, "예제 : 배포용 문서 만들기")
        .unwrap());
}

#[test]
fn bookmark_link_inserts_updates_and_survives_hwp_and_hwpx() {
    let mut core = doc_with_bookmarks();
    let target = HyperlinkTarget::body(0, 0);
    assert!(core
        .insert_bookmark_hyperlink_native(&target, 0, 2, "없는 책갈피")
        .is_err());
    let id = core
        .insert_bookmark_hyperlink_native(&target, 0, 2, NAME)
        .unwrap();
    for reopened in roundtrips(&core) {
        assert_eq!(
            only_link(&reopened),
            (Some(NAME.to_string()), format!("?{NAME}"), 0, 2)
        );
    }
    // 웹 주소로 바꾸거나 없는 책갈피로 바꾸지 않는다.
    assert!(core
        .update_hyperlink_native(&target, id, "https://example.com")
        .is_err());
    assert!(core
        .update_bookmark_hyperlink_native(&target, id, "없는 책갈피")
        .is_err());
    assert!(!core
        .update_bookmark_hyperlink_native(&target, id, NAME)
        .unwrap());
    assert!(core
        .update_bookmark_hyperlink_native(&target, id, "둘째")
        .unwrap());
    for reopened in roundtrips(&core) {
        assert_eq!(
            only_link(&reopened),
            (Some("둘째".to_string()), "?둘째".to_string(), 0, 2)
        );
    }
}

#[test]
fn web_link_is_not_turned_into_bookmark_link() {
    let mut core = doc_with_bookmarks();
    let target = HyperlinkTarget::body(0, 0);
    let id = core
        .insert_hyperlink_native(&target, 0, 2, "https://example.com")
        .unwrap();
    assert!(core
        .update_bookmark_hyperlink_native(&target, id, "둘째")
        .is_err());
    assert_eq!(only_link(&core).0, None);
}
