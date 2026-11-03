//! 관계형·고정형: 글꼴 선택 계획은 Native 모델과 비교하고 저장 제품 메타데이터는 고정값과 비교한다.
//! 주석 갱신: 2026-11-03 19:10:30 KST; 직전 변경 3e02fe2749866a8453af3eb3901b9c1a674cccaf (2026-10-25 22:58:32 KST).
//! `info --json`의 HWP5/HWPX 마지막 저장 한컴오피스 제품 메타데이터 계약.
//!
//! FileHeader의 HWP 형식 버전은 2022와 2024가 모두 `5.1.1.0`일 수 있으므로,
//! HWP5는 `HwpSummaryInformation.revisionNumber`, HWPX는
//! `version.xml/appVersion`에서 마지막 저장 제품을 별도로 판별한다.
//! 이 값은 원 작성 제품이 아니라 사용자가 지울 수 있는 저장 메타데이터다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn info_font_plan_matches_the_document_selection_plan() {
    for relative in [
        "samples/task-001.hwp",
        "samples/task2136/neartop_reset_sb2500.hwpx",
    ] {
        let sample = sample_path(relative);
        let bytes = std::fs::read(&sample).expect("문서 읽기");
        let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("문서 열기");
        let expected: serde_json::Value =
            serde_json::from_str(&document.get_document_font_selection_plan())
                .expect("모델 글꼴 선택 계획");
        let scoped = expected["scopedRequests"].as_array().expect("슬롯별 요청");
        assert_eq!(scoped.len(), expected["requests"].as_array().unwrap().len());
        assert!(scoped.iter().all(|request| request["languageSlot"].is_u64()
            && request["fontId"].is_u64()
            && request["source"].is_string()));
        let output = Command::new(rhwp_bin())
            .args(["info", "--font-plan", sample.to_str().expect("UTF-8 경로")])
            .output()
            .expect("rhwp 실행");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let actual: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("CLI 글꼴 선택 계획");
        assert_eq!(actual, expected, "{relative}");
    }
}

const HWP2010: &str = "samples/basic/Hyper(hwp2010).hwp";
const NEW_SAVE_HWP2010: &str = "samples/pr5935/test-2010.hwp";
const NEW_SAVE_HWP2018: &str = "samples/pr5935/test-2018.hwp";
const NEW_SAVE_HWP2022: &str = "samples/pr5935/test-2022.hwp";
const NEW_SAVE_HWP2024: &str = "samples/pr5935/test-2024.hwp";
const HWPX2020: &str = "samples/task2136/neartop_reset_sb2500.hwpx";
const NEW_SAVE_HWPX2018: &str = "samples/pr5935/test-2018.hwpx";
const NEW_SAVE_HWPX2022: &str = "samples/pr5935/test-2022.hwpx";
const NEW_SAVE_HWPX2024: &str = "samples/pr5935/test-2024.hwpx";

fn sample_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn info_json(relative: &str) -> serde_json::Value {
    let sample = sample_path(relative);
    let output = Command::new(rhwp_bin())
        .args(["info", "--json", sample.to_str().expect("UTF-8 경로")])
        .output()
        .expect("rhwp 실행 실패");
    assert!(
        output.status.success(),
        "info 실패: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    serde_json::from_slice(&output.stdout).expect("info JSON")
}

#[test]
fn info_distinguishes_hancom_office_save_editions_for_hwp5_and_hwpx() {
    for (sample, format, product, version) in [
        (HWP2010, "hwp5", "hancom-office-2010", "8.0.0.460"),
        (NEW_SAVE_HWP2010, "hwp5", "hancom-office-2010", "8.0.0.466"),
        (
            NEW_SAVE_HWP2018,
            "hwp5",
            "hancom-office-2018",
            "10.0.0.5060",
        ),
        (
            NEW_SAVE_HWP2022,
            "hwp5",
            "hancom-office-2022",
            "12.0.0.4605",
        ),
        (
            NEW_SAVE_HWP2024,
            "hwp5",
            "hancom-office-2024",
            "13.0.0.3379",
        ),
        (
            NEW_SAVE_HWPX2018,
            "hwpx",
            "hancom-office-2018",
            "10.0.0.5060",
        ),
        (HWPX2020, "hwpx", "hancom-office-2020", "11.0.0.7257"),
        (
            NEW_SAVE_HWPX2022,
            "hwpx",
            "hancom-office-2022",
            "12.0.0.4605",
        ),
        (
            NEW_SAVE_HWPX2024,
            "hwpx",
            "hancom-office-2024",
            "13.0.0.3379",
        ),
    ] {
        let info = info_json(sample);
        assert_eq!(info["format"], format, "{sample}: {info}");
        assert_eq!(
            info["lastSavedWith"]["product"], product,
            "{sample}: {info}"
        );
        assert_eq!(
            info["lastSavedWith"]["version"], version,
            "{sample}: {info}"
        );
        assert_eq!(
            info["lastSavedWith"]["confidence"], "metadata",
            "{sample}: {info}"
        );
    }
}

#[test]
fn hwpx_version_xml_requires_a_parseable_app_version() {
    use rhwp::parser::hwp_summary::hwpx_last_saved_with;

    let missing = vec![(
        "version.xml".to_string(),
        br#"<hv:HCFVersion xmlns:hv="http://www.hancom.co.kr/hwpml/2011/version"/>"#.to_vec(),
    )];
    assert!(hwpx_last_saved_with(&missing).is_none());

    let malformed = vec![(
        "version.xml".to_string(),
        br#"<hv:HCFVersion xmlns:hv="http://www.hancom.co.kr/hwpml/2011/version" appVersion="not-a-version"/>"#
            .to_vec(),
    )];
    assert!(hwpx_last_saved_with(&malformed).is_none());

    let unknown = vec![(
        "version.xml".to_string(),
        br#"<version appVersion="99, 1, 2, 3 custom"/>"#.to_vec(),
    )];
    let parsed = hwpx_last_saved_with(&unknown).expect("알 수 없는 주버전도 원문 버전은 보존");
    assert_eq!(parsed.version, "99.1.2.3");
    assert_eq!(parsed.product, None);
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
