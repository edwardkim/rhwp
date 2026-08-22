//! `info --json`의 마지막 저장 한컴오피스 제품 메타데이터 계약.
//!
//! FileHeader의 HWP 형식 버전은 2022와 2024가 모두 `5.1.1.0`일 수 있으므로,
//! `HwpSummaryInformation.revisionNumber`에서 마지막 저장 제품을 별도로 판별한다.
//! 이 값은 원 작성 제품이 아니라 사용자가 지울 수 있는 저장 메타데이터다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

const HWP3: &str = "samples/hwp3-sample16.hwp";
const HWP2018: &str = "samples/hwp3-sample16-hwp5-2018.hwp";
const HWP2022: &str = "samples/hwp3-sample16-hwp5-2022.hwp";
const HWP2024: &str = "samples/hwp3-sample16-hwp5-2024.hwp";
const HWPX: &str = "samples/task2136/neartop_reset_sb2500.hwpx";

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
fn info_distinguishes_hancom_office_save_editions() {
    for (sample, product, version) in [
        (HWP2018, "hancom-office-2018", "10.0.0.14727"),
        (HWP2022, "hancom-office-2022", "12.0.0.535"),
        (HWP2024, "hancom-office-2024", "13.0.0.3457"),
    ] {
        let info = info_json(sample);
        assert_eq!(info["format"], "hwp5", "{sample}: {info}");
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
fn info_does_not_guess_a_non_hwp5_saving_product() {
    for (sample, format) in [(HWP3, "hwp3"), (HWPX, "hwpx")] {
        let info = info_json(sample);
        assert_eq!(info["format"], format, "{sample}: {info}");
        assert!(info["lastSavedWith"].is_null(), "{sample}: {info}");
    }
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
