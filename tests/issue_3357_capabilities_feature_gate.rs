//! [#3357] feature 게이트 명령의 자기서술 계약 회귀 테스트.
//!
//! 계약: `capabilities` 의 export-png 항목은 `requiresFeature`·`available` 을 항상
//! 방출하고(스키마 안정), `available` 값은 **실제 호출 가능 여부와 일치**한다 —
//! 기능 부재 오류는 available=false 인 빌드에서만 난다. native-skia 유무 어느 빌드에서
//! 돌려도 통과하도록 실측 대조로 작성한다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::{Command, Output};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn capabilities() -> serde_json::Value {
    let output = run(&["capabilities"]);
    assert_eq!(output.status.code(), Some(0));
    serde_json::from_slice(&output.stdout).expect("capabilities 는 순수 JSON")
}

fn export_png_entry() -> serde_json::Value {
    let caps = capabilities();
    caps["commands"]
        .as_array()
        .expect("commands 배열")
        .iter()
        .find(|c| c["name"] == "export-png")
        .expect("export-png 항목")
        .clone()
}

/// 두 필드는 빌드와 무관하게 항상 있어야 한다 (스키마 안정성).
#[test]
fn export_png_declares_feature_gate() {
    let entry = export_png_entry();
    assert_eq!(entry["requiresFeature"], "native-skia", "{entry}");
    assert!(entry["available"].is_boolean(), "{entry}");
}

/// 자기서술은 거짓말하지 않는다 — available 값과 실제 호출 결과가 일치한다.
#[test]
fn available_matches_actual_invocability() {
    let entry = export_png_entry();
    let available = entry["available"].as_bool().expect("available bool");

    // 인자 없이 호출: 기능 부재 빌드는 feature 안내, 기능 빌드는 일반 사용법 오류.
    let output = run(&["export-png"]);
    assert_eq!(output.status.code(), Some(2));
    let feature_error =
        String::from_utf8_lossy(&output.stderr).contains("native-skia feature 가 활성화");
    assert_eq!(
        feature_error, !available,
        "available={available} 인데 기능 부재 오류={feature_error} — 자기서술이 실제와 어긋납니다"
    );
}

/// 게이트 없는 명령에는 두 필드를 붙이지 않는다 (의미 오염 방지).
#[test]
fn ungated_commands_have_no_gate_fields() {
    let caps = capabilities();
    for name in ["info", "export-text", "export-svg"] {
        let entry = caps["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("{name} 항목"));
        assert!(
            entry.get("requiresFeature").is_none() && entry.get("available").is_none(),
            "{name} 은 게이트 명령이 아닙니다: {entry}"
        );
    }
}
