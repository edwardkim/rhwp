//! #1868 `export-hwpx` CLI 의 코어 경로 검증 — HWP→HWPX 변환 후 산출물이
//! 유효한 HWPX(ZIP) 이고 재파싱 시 페이지 수가 보존되는지.
//!
//! CLI 함수(main.rs)는 인자 처리 + 이 코어 경로(`from_bytes` → `export_hwpx_native`)의
//! 배선일 뿐이므로, 통합 테스트는 코어 경로를 직접 검증한다(기존 export 계열 관례).

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn convert_and_verify(sample: &str) -> (u32, u32) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let src = HwpDocument::from_bytes(&data).expect("parse source");
    let src_pages = src.page_count();

    let bytes = src.export_hwpx_native().expect("export hwpx");
    // HWPX = ZIP 로컬 헤더 매직.
    assert!(
        bytes.len() > 4 && &bytes[0..4] == b"PK\x03\x04",
        "{sample}: 산출물이 ZIP(HWPX) 매직으로 시작해야 한다"
    );

    let round = HwpDocument::from_bytes(&bytes).expect("reparse exported hwpx");
    (src_pages, round.page_count())
}

#[test]
fn hwp5_to_hwpx_preserves_pages() {
    // HWP5 (CFB) 원본 — pr-1674 (문체부 공고, 한글 2020 정답지 35쪽).
    let (src, round) = convert_and_verify("samples/pr-1674.hwp");
    assert_eq!(src, round, "HWP5→HWPX 변환 후 페이지 수가 보존돼야 한다");
}

#[test]
fn hwp3_to_hwpx_preserves_pages() {
    // HWP3 (고전 바이너리) 원본 — 파서 자동 감지 경로 커버.
    let (src, round) = convert_and_verify("samples/hwp3-sample.hwp");
    assert_eq!(src, round, "HWP3→HWPX 변환 후 페이지 수가 보존돼야 한다");
}

#[test]
fn hwpx_reserialize_preserves_pages() {
    // HWPX 입력 → HWPX 재직렬화(re-serialize)도 허용 — roundtrip 디버깅 경로.
    let (src, round) = convert_and_verify("samples/hwpx/pr-1674.hwpx");
    assert_eq!(src, round, "HWPX 재직렬화 후 페이지 수가 보존돼야 한다");
}
