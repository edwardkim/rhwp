//! 실제 글꼴 요청 이름과 명시 exact 소유자의 세션 프로토콜 계약.
//! 실제 페이지 좌표나 한컴 피델리티의 기대값을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

#[test]
fn pua_supply_clear_preserves_explicit_exact_font_owner() {
    let mut core = DocumentCore::new_empty();
    let bytes = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    let before: serde_json::Value = serde_json::from_str(
        &core
            .register_exact_font_source_native(0, 0, bytes, 0)
            .unwrap(),
    )
    .unwrap();
    core.clear_native_pua_fonts();
    let after: serde_json::Value = serde_json::from_str(
        &core
            .register_exact_font_source_native(0, 0, bytes, 0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        after["status"], "already-registered",
        "PUA만 해제해도 명시 exact 소유자는 동일한 채 유지한다"
    );
    assert_eq!(before["handle"], after["handle"]);
    assert_eq!(before["registry"], after["registry"]);
}

#[test]
fn host_font_request_names_are_not_css_quoted() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp"))
        .expect("구역과 첫 문단이 있는 실제 빈 문서");
    core.insert_text_native(0, 0, 0, "\u{f0854}").unwrap();
    let requests = core.native_pua_font_requests();
    assert!(!requests.is_empty(), "원 PUA의 실제 글꼴 요청");
    for request in requests {
        assert_eq!(request.paint_families.first(), Some(&request.family));
        assert!(request
            .paint_families
            .iter()
            .all(|family| !family.starts_with('\'')));
    }
}
