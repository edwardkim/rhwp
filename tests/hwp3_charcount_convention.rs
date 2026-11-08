//! [#3510] HWP3 파서의 `char_count` 규약을 HWP5·HWPX 와 맞춘다.
//!
//! HWPX(`parser/hwpx/section.rs`)와 HWP5(`parser/body_text.rs`)는 문단 끝 마커를
//! `char_count` 에 포함하는데 HWP3(`parser/hwp3/mod.rs`)만 포함하지 않았다. 그래서
//! HWP3 → HWPX 변환이 **내용은 완전히 같은데** 모든 문단·셀에서 1씩 차이가 나고,
//! `--verify` 게이트가 정상 변환을 exit 3 으로 거부했다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 표를 가진 HWP3 문서 — 차이가 셀 안에서도 나타난다.
const SAMPLE_HWP3: &str = "samples/hwp3-sample.hwp";
/// 무회귀 확인용 HWP5 문서.
const SAMPLE_HWP5: &str = "samples/field-01.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-cc-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// 원본을 HWPX 로 내보내고 `ir-diff --json` 봉투를 돌려준다.
fn roundtrip_diff(src_rel: &str, tag: &str) -> Option<serde_json::Value> {
    let src = sample(src_rel);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀: {src_rel}");
        return None;
    }
    let out = temp_path(tag, "hwpx");
    let conv = ["export-hwpx", src.to_str().unwrap(), out.to_str().unwrap()];
    let c = run(&conv);
    assert_eq!(c.status.code(), Some(0), "{}", describe(&conv, &c));
    assert!(out.exists(), "HWPX 산출물이 생성되어야 합니다");

    let args = [
        "ir-diff",
        src.to_str().unwrap(),
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert!(
        matches!(output.status.code(), Some(0 | 3)),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ir-diff --json 이 순수 JSON 이어야 합니다");
    assert!(
        v["identical"].is_boolean(),
        "비교 판정 필드가 있어야 합니다"
    );
    assert!(v["diffCount"].is_u64(), "비교 건수 필드가 있어야 합니다");
    let _ = std::fs::remove_file(&out);
    Some(v)
}

/// 카테고리 맵에서 `cc`(char_count) 차이 건수를 센다.
fn cc_diff_count(v: &serde_json::Value) -> u64 {
    v["categories"]
        .as_object()
        .expect("구조화된 차이 카테고리가 있어야 합니다")
        .get("cc")
        .map_or(0, |count| {
            count.as_u64().expect("cc 건수는 정수여야 합니다")
        })
}

#[test]
fn hwp3_roundtrip_char_count_is_not_off_by_exactly_one() {
    // 끝 마커를 빼던 구현은 모든 문단·셀에서 cc 차이 298건을 만들었다.
    // 사람이 읽는 종료 문구 대신 구조화된 비교에서 cc 차이가 없음을 확인한다.
    let Some(diff) = roundtrip_diff(SAMPLE_HWP3, "offbyone") else {
        return;
    };
    assert_eq!(
        cc_diff_count(&diff),
        0,
        "문단·셀의 char_count를 왕복 저장에서 보존해야 합니다: {diff}"
    );
}

#[test]
fn hwp3_roundtrip_char_count_diff_count_dropped_sharply() {
    // 무회귀 가드: 수정 전 이 표본은 diffCount=298(전부 cc)였다. 수정 후에는
    // #3510 과 무관한 잔여 결함만 남아 diffCount 가 크게 줄어야 한다.
    let Some(v) = roundtrip_diff(SAMPLE_HWP3, "count") else {
        return;
    };
    let diff_count = v["diffCount"].as_u64().unwrap_or(u64::MAX);
    assert!(
        diff_count < 50,
        "끝 마커 규약 수정 후에도 차이가 298 근처로 많이 남아 있습니다: {v}"
    );
}

#[test]
fn hwp5_roundtrip_char_count_unaffected() {
    // 무회귀 가드: HWP5 는 원래 cc 차이가 0이었고 그대로여야 한다.
    let Some(v) = roundtrip_diff(SAMPLE_HWP5, "hwp5") else {
        return;
    };
    assert_eq!(
        cc_diff_count(&v),
        0,
        "HWP5 왕복에 char_count 차이가 새로 생겼습니다: {v}"
    );
}
