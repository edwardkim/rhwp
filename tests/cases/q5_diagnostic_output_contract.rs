//! [#5511 Q5] 사람용 진단 조회 출력의 move-only 계약.
//!
//! `info`, `dump-pages`, `dump`는 사람이 읽는 stdout을 오래된 진단 계약으로 제공한다.
//! Q5가 큰 handler를 책임별 모듈로 나눌 때 공백·순서·숫자 표기까지 바뀌지 않도록,
//! 대표 HWP3 fixture의 경로만 정규화한 뒤 stdout 전체 바이트를 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행")
}

fn assert_stdout_digest(args: &[&str], expected: &str) {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "명령 실패: rhwp {}\nstderr={}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "성공 진단은 stderr를 오염시키지 않아야 한다: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let sample = sample_path();
    let sample = sample.to_string_lossy();
    let stdout = String::from_utf8(output.stdout).expect("진단 stdout UTF-8");
    let normalized = stdout.replace(sample.as_ref(), "<SAMPLE>");
    let digest = Sha256::digest(normalized.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        digest,
        expected,
        "stdout byte 계약 변화: rhwp {}\n{}",
        args.join(" "),
        normalized
    );
}

#[test]
fn info_human_stdout_is_byte_stable() {
    let sample = sample_path();
    assert_stdout_digest(
        &["info", sample.to_str().expect("UTF-8 sample path")],
        // [#4680] HWP3 스타일 풀 중복 제거로 두 줄이 바뀌었다 — 나머지 출력은 동일하다.
        //   ParaShape: 119 → 16 · CharShape: 747 → 23
        "9ce249e42a04c9d4461833568930ffbbe89f2e8262cb3982e9fa453712b884f4",
    );
}

#[test]
fn dump_pages_human_stdout_is_byte_stable() {
    let sample = sample_path();
    assert_stdout_digest(
        &[
            "dump-pages",
            sample.to_str().expect("UTF-8 sample path"),
            "-p",
            "0",
        ],
        "e542bef7cea773d38d6108588b8255005032567ce3fc964472ae84255cfbb5db",
    );
}

#[test]
fn dump_filtered_human_stdout_is_byte_stable() {
    let sample = sample_path();
    assert_stdout_digest(
        &[
            "dump",
            sample.to_str().expect("UTF-8 sample path"),
            "--section",
            "0",
            "--para",
            "0",
        ],
        // [#4680] 스타일 풀 중복 제거로 `[CS] id=` · `[PS] ps_id=` 숫자만 바뀌었다.
        // 속성이 한 글자도 다르지 않던 런 넷(id 14·16·17·18)이 이제 한 id 를 공유하고,
        // 실제로 다른 런(bold=false)은 그대로 별개 id 를 받는다. 나머지 필드는 동일하다.
        "9d12a0c03b8a545ad1979a4af552b28da6aa07d9a3e465dc3c0d0ce5786b7cbe",
    );
}
