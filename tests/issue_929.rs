//! Issue #929: HWP3 ch=9 (탭) 처리 결함 — spec §10.5 표 39 미준수
//!
//! 본질: src/parser/hwp3/mod.rs 의 char loop `9 =>` 분기가 추가 6 bytes
//! (hunit 탭 폭 + word 점끌기 + hchar 닫기=9) 를 consume 하지 않아 cursor
//! 가 6 bytes 일찍 진행. tab 내부 hunit 값이 ch=17 (각주) 로 잘못 해석되며
//! 재귀 paragraph_list 가 garbage 위치에서 시작 → LineInfo::read EOF.
//!
//! 정정: spec §10.5 정합 처리 — 추가 6 bytes 읽고 i += 3 (총 4 hchar 차지).

use std::fs;
use std::path::Path;

fn read_sample(rel: &str) -> Vec<u8> {
    let root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(root).join(rel);
    fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e))
}

#[test]
fn issue_929_hwp3_sample19_parses_ok() {
    let data = read_sample("samples/hwp3-sample19.hwp");
    let doc = rhwp::parser::hwp3::parse_hwp3(&data)
        .expect("hwp3-sample19.hwp must parse without error (issue #929)");
    assert!(!doc.sections.is_empty(), "section count must be > 0");
    let para_count: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
    assert!(
        para_count > 20,
        "expected > 20 paragraphs, got {}",
        para_count,
    );
}

#[test]
fn issue_929_existing_hwp3_samples_no_regression() {
    for sample in &[
        "samples/hwp3-sample.hwp",
        "samples/hwp3-sample10.hwp",
        "samples/hwp3-sample11.hwp",
        "samples/hwp3-sample13.hwp",
        "samples/hwp3-sample14.hwp",
        "samples/hwp3-sample16.hwp",
    ] {
        let data = read_sample(sample);
        rhwp::parser::hwp3::parse_hwp3(&data)
            .unwrap_or_else(|e| panic!("regression in {}: {:?}", sample, e));
    }
}
