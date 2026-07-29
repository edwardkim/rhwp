use std::path::Path;

use rhwp::wasm_api::HwpDocument;

/// #2428: 각주가 없는 페이지는 render tree 기반 hit-test 전에 빠르게 제외한다.
///
/// `footnote-01.hwp` 첫 페이지에는 본문 각주 marker가 있고 마지막 빈 페이지에는
/// 각주가 없다. 페이지네이션 메타데이터를 읽는 native query와 WASM 공개 wrapper가
/// 같은 판정을 내려야 한다.
#[test]
fn issue_2428_footnote_fast_reject_matches_page_metadata() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/footnote-01.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse footnote-01.hwp");
    let page_count = doc.page_count();
    assert!(
        page_count >= 2,
        "fixture requires a later page without footnotes"
    );

    assert!(
        doc.page_has_footnote_footholds_native(0),
        "첫 페이지의 각주 marker는 fast-reject 대상이 아니어야 함"
    );
    assert!(
        doc.page_has_footnote_footholds(0),
        "WASM wrapper도 첫 페이지의 각주를 보고해야 함"
    );

    let last_page = page_count - 1;
    assert!(
        !doc.page_has_footnote_footholds_native(last_page),
        "마지막 빈 페이지는 native fast-reject 대상이어야 함"
    );
    assert!(
        !doc.page_has_footnote_footholds(last_page),
        "WASM wrapper도 각주 없는 마지막 페이지를 제외해야 함"
    );
    assert!(
        !doc.page_has_footnote_footholds_native(page_count),
        "범위 밖 페이지는 native query에서 false여야 함"
    );
    assert!(
        !doc.page_has_footnote_footholds(page_count),
        "범위 밖 페이지는 WASM wrapper에서도 false여야 함"
    );
}
