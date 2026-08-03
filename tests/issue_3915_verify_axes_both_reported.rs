//! [#3915] `--verify` 와 `--verify-pages` 를 함께 주면 쪽수 실패가 IR 차이를 가린다.
//!
//! 쪽수 검증이 실패하면 그 자리에서 `process::exit(4)` 했다. `--verify` 를 함께 줬어도
//! IR 비교가 **아예 돌지 않아** 차이가 있어도 보고되지 않았다.
//!
//! 두 축은 서로 다른 결함을 잰다 — 쪽수는 조판 결과, IR 은 저장 손실이다. 한쪽이 실패했다고
//! 다른 쪽을 건너뛰면, 사람이 "쪽수만 문제고 내용은 온전하다" 로 잘못 읽는다. 실제로
//! `synam-001.hwp` 는 두 축이 **함께** 실패하는데 쪽수 실패만 보였다.
//!
//! 종료 코드 계약은 바꾸지 않는다 — 쪽수 실패는 그대로 4 다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 두 축이 함께 실패하는 표본 — 쪽수 35→36, IR 차이 3건.
const BOTH_FAIL_SAMPLE: &str = "samples/synam-001.hwp";
/// 두 축 모두 통과하는 표본 — 무회귀 기준선.
const CLEAN_SAMPLE: &str = "samples/table-001.hwp";

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn export(sample: &str, out: &Path, flags: &[&str]) -> Output {
    let mut args: Vec<String> = vec![
        "export-hwpx".into(),
        repo(sample).to_string_lossy().into_owned(),
        out.to_string_lossy().into_owned(),
    ];
    args.extend(flags.iter().map(|f| (*f).to_string()));
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(&args)
        .output()
        .expect("rhwp 실행 실패")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// #3906 본체 — 쪽수가 실패해도 IR 비교를 마저 돌려 함께 보고한다.
#[test]
fn page_failure_no_longer_hides_ir_differences() {
    let dir = std::env::temp_dir().join(format!("rhwp-3915-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 디렉터리");
    let out = dir.join("both.hwpx");

    let o = export(BOTH_FAIL_SAMPLE, &out, &["--verify", "--verify-pages"]);
    let err = stderr(&o);

    assert!(
        err.contains("검증 실패(--verify-pages)"),
        "쪽수 실패가 보고되지 않았습니다:\n{err}"
    );
    assert!(
        err.contains("검증 실패(--verify)"),
        "쪽수 실패가 IR 차이를 가렸습니다 — 두 축은 서로 다른 결함을 재므로 함께 보고해야 \
         합니다:\n{err}"
    );

    // 종료 코드 계약 무변경 — 쪽수 실패가 우선한다.
    assert_eq!(
        o.status.code(),
        Some(4),
        "두 축이 함께 실패해도 종료 코드는 종전대로 4 여야 합니다:\n{err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// 실패인데 "통과" 도 함께 찍히면 안 된다 — 조기 종료를 걷어낼 때 흔한 사고다.
#[test]
fn failing_page_axis_does_not_also_report_pass() {
    let dir = std::env::temp_dir().join(format!("rhwp-3915b-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 디렉터리");
    let out = dir.join("nopass.hwpx");

    let o = export(BOTH_FAIL_SAMPLE, &out, &["--verify", "--verify-pages"]);
    let combined = format!("{}{}", stderr(&o), String::from_utf8_lossy(&o.stdout));

    assert!(
        !combined.contains("검증 통과(--verify-pages)"),
        "쪽수 축이 실패했는데 통과 메시지도 찍혔습니다:\n{combined}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// 단독 사용과 정상 문서는 종전 그대로여야 한다.
#[test]
fn single_axis_and_clean_document_are_unchanged() {
    let dir = std::env::temp_dir().join(format!("rhwp-3915c-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 디렉터리");

    // --verify-pages 단독: 쪽수만 보고, exit 4.
    let o = export(BOTH_FAIL_SAMPLE, &dir.join("p.hwpx"), &["--verify-pages"]);
    let err = stderr(&o);
    assert!(err.contains("검증 실패(--verify-pages)"), "{err}");
    assert!(
        !err.contains("검증 실패(--verify)"),
        "--verify 를 주지 않았는데 IR 비교가 돌았습니다:\n{err}"
    );
    assert_eq!(o.status.code(), Some(4), "{err}");

    // 두 축 모두 통과하는 문서: exit 0, 양쪽 통과 메시지.
    let o = export(
        CLEAN_SAMPLE,
        &dir.join("c.hwpx"),
        &["--verify", "--verify-pages"],
    );
    let combined = format!("{}{}", stderr(&o), String::from_utf8_lossy(&o.stdout));
    assert_eq!(o.status.code(), Some(0), "{combined}");
    assert!(combined.contains("검증 통과(--verify-pages)"), "{combined}");
    assert!(combined.contains("검증 통과(--verify)"), "{combined}");

    let _ = std::fs::remove_dir_all(&dir);
}
