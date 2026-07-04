//! Issue #1768: 배포용(distribution) HWPX 를 HWP5 로 저장하면 재로드 가능해야 한다.
//!
//! Regression shape (samples/task1768/distribution_doc.hwpx, 배포용=예):
//! - 수정 전: FileHeader 배포용 플래그(0x04)는 복사되지만 DISTRIBUTE_DOC_DATA 레코드를
//!   기록하지 않아, 산출물 재로드가 InvalidFile("암호 오류: DISTRIBUTE_DOC_DATA 레코드
//!   없음") 으로 실패 (hwpdocs 5,000건 render-diff LOAD_FAIL 4건 전수).
//! - 수정 후: 직렬화 시 일반 문서로 강하(배포용·암호화 플래그 클리어 — IR 은 이미
//!   복호화된 평문) → 재로드 성공.

use std::fs;
use std::path::Path;

use rhwp::parse_document;
use rhwp::serializer::serialize_hwp;

const SAMPLE: &str = "samples/task1768/distribution_doc.hwpx";

#[test]
fn issue_1768_distribution_hwpx_saved_hwp_reloads() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = parse_document(&bytes).expect("parse distribution hwpx");
    assert!(
        doc.header.distribution,
        "샘플은 배포용 플래그를 가져야 한다 (전제)"
    );

    let hwp = serialize_hwp(&doc).expect("serialize hwp");
    let reloaded = parse_document(&hwp)
        .expect("배포용 강하 후 산출물은 재로드 가능해야 한다 (DISTRIBUTE_DOC_DATA 불필요)");

    assert!(
        !reloaded.header.distribution && !reloaded.header.encrypted,
        "산출물 FileHeader 는 일반 문서로 강하되어야 한다"
    );
    assert_eq!(reloaded.sections.len(), doc.sections.len(), "구역 수 보존");
}
