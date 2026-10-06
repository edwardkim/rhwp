//! 거의 빈 쪽의 저장 near-top 리셋 완화에 대한 작은 대조 계약.
//!
//! 큰205쪽 `1480000-201900698-native-neartop-reset.hwp`의 잠정 쪽수 검사는
//! 전쪽 Native 비교에서102쪽이90% 미달하여 사용자 요청으로 #7445에 분리했다.
//! 원본·독립 PDF는 보존하고 아래 작은 기존 검사는 유지한다.
//! 증적: mydocs/pr/assets/issue7445/neartop5941_green_batch_deferral_validation.json

#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

/// `#5921` 의 원 계약은 그대로 — 거의 빈 쪽에서는 완화가 걸려 1쪽이어야 한다.
#[test]
fn issue_5921_empty_page_relaxation_still_applies() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task2136/neartop_reset_sb2500.hwpx");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse fixture");
    assert_eq!(
        doc.page_count(),
        1,
        "거의 빈 쪽(채움 2%)에서는 #5921 완화가 그대로 걸려 1쪽이어야 한다"
    );
}
