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
        "bffcbf7de3bab9ff3b05dda97815afcbfe3d953e8a85098d2ba78ef9d37284ea",
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
        "45f876bb12042d4a6539780c4eaf043975fca51e21e479c80fd79e975ea8641d",
    );
}
