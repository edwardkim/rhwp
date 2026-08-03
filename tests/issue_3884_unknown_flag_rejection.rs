//! [#3884 G1·G2] 진단 명령이 미지 플래그를 조용히 무시하던 회귀.
//!
//! `capabilities.jsonContract.failure` 는 **"단건 실패 시 stdout 0바이트"** 를 선언하고,
//! #3349 인자 규약은 **미지 플래그 즉시 exit 2** 를 선언한다. 세 명령이 둘 다 어겼다.
//!
//! | 종전 실측 | exit | stdout |
//! |---|---:|---:|
//! | `dump <문서> --bogus-flag` | 0 | 18,643 B — 오타를 성공으로 |
//! | `dump <문서> --json` | 0 | 사람용 텍스트 — `--json` 침묵 무시 |
//! | `bench <문서> --json` | 1 | 518 B — 실패인데 stdout 이 비지 않음 |
//!
//! 조용히 무시하는 쪽이 왜 더 나쁜가: 오류라면 호출자가 고칠 수 있지만, 성공으로
//! 돌아오면 **자기가 요청한 것과 다른 것을 받고도 알 수 없다.** `--json` 을 준
//! 에이전트가 사람용 텍스트를 JSON 으로 파싱하다 깨지는 경로가 정확히 이것이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

/// 미지 플래그는 exit 2 로 거부하고 stdout 은 비운다.
#[test]
fn unknown_flags_are_rejected_with_usage_exit_and_empty_stdout() {
    let doc = sample();
    if !doc.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let p = doc.to_string_lossy().to_string();

    for (label, args) in [
        ("dump --bogus-flag", vec!["dump", &p, "--bogus-flag"]),
        ("dump --json", vec!["dump", &p, "--json"]),
        ("diag --json", vec!["diag", &p, "--json"]),
        ("bench --json", vec!["bench", &p, "--json"]),
    ] {
        let out = run(&args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{label}: 미지 플래그는 usage 오류(2)여야 합니다 — 조용히 성공하면 호출자가 \
             요청과 다른 결과를 받고도 알 수 없습니다. stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.stdout.is_empty(),
            "{label}: 실패 경로에서 stdout 이 {}바이트 나왔습니다 — 반쪽 출력은 소비자가 \
             파싱하다 죽거나 더 나쁘게는 잘린 값을 참으로 읽게 합니다",
            out.stdout.len()
        );
        assert!(
            !out.stderr.is_empty(),
            "{label}: 거부했으면 이유를 stderr 로 알려야 합니다"
        );
    }
}

/// 정상 경로는 그대로여야 한다 — 거부 규칙이 멀쩡한 호출을 깨면 안 된다.
#[test]
fn valid_invocations_still_succeed() {
    let doc = sample();
    if !doc.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let p = doc.to_string_lossy().to_string();

    for (label, args) in [
        ("dump", vec!["dump", &p]),
        ("dump --section 0", vec!["dump", &p, "--section", "0"]),
        ("diag", vec!["diag", &p]),
    ] {
        let out = run(&args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{label}: 정상 호출이 깨졌습니다. stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !out.stdout.is_empty(),
            "{label}: 정상 호출인데 출력이 없습니다"
        );
    }
}
