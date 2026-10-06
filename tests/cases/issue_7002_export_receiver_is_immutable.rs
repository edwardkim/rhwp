#![cfg(not(target_arch = "wasm32"))]

//! [#7002 후속] HWP 저장 경로가 문서를 바꾸지 않는다 — 수신자로 강제한다.
//!
//! 저장 어댑터는 `prepare_hwp_export_snapshot`(`&self`)이 뜬 **사본**에 적용되고 live
//! 문서는 그대로다. 그런데 진입점 셋이 `&mut self` 로 선언돼 있어 그 사실이 서명에
//! 드러나지 않았고, undo 라우팅 레지스트리는 "저장은 문서를 바꾸지 않는다" 를 주석으로만
//! 붙들고 있었다(#7002 에서 `exportHwp*` 3종을 EXCLUDED 로 등재하며 적은 사유).
//!
//! 수신자를 `&self` 로 좁히면 그 주석이 컴파일러 계약이 된다. 이 시험은 세 진입점이
//! 불변 참조로 호출 가능한지를 컴파일 시점에 고정한다 — 누군가 다시 `&mut self` 로
//! 넓히면 여기서 깨진다.

use rhwp::document_core::DocumentCore;

fn sample() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/ta-pic-001-r.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("표본 로드")).expect("파싱")
}

/// 세 저장 진입점을 **불변 참조로** 부른다. 서명이 넓어지면 컴파일되지 않는다.
fn export_all(core: &DocumentCore) -> (usize, usize, usize) {
    let plain = core.export_hwp_with_adapter().expect("HWP 저장");
    let secret = core
        .export_hwp_with_adapter_with_password(b"pw")
        .expect("비밀번호 저장");
    let verified = core.serialize_hwp_with_verify().expect("검증 저장");
    (plain.len(), secret.len(), verified.bytes_len)
}

#[test]
fn issue_7002_export_entry_points_take_shared_receiver() {
    let core = sample();
    let (plain, secret, verified) = export_all(&core);
    assert!(
        plain > 0 && secret > 0 && verified > 0,
        "저장 산출이 비었다"
    );
}

/// 같은 문서를 두 번 저장하면 같은 바이트가 나온다 — 저장이 문서를 바꾸지 않는 것의 관측면.
/// (`&self` 는 내부 가변성까지 막지는 않으므로 서명만으로는 부족하다.)
#[test]
fn issue_7002_export_is_repeatable_without_mutating_document() {
    let core = sample();
    let first = core.export_hwp_with_adapter().expect("1회차");
    let second = core.export_hwp_with_adapter().expect("2회차");
    assert_eq!(
        first, second,
        "같은 문서를 두 번 저장했는데 바이트가 다르다 — 저장이 문서를 바꾼다"
    );
}
