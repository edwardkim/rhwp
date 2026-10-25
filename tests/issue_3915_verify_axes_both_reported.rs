//! [#3915] `--verify` 와 `--verify-pages` 를 함께 주면 쪽수 실패가 IR 차이를 가린다.
//!
//! 쪽수 검증이 실패하면 그 자리에서 `process::exit(4)` 했다. `--verify` 를 함께 줬어도
//! IR 비교가 **아예 돌지 않아** 차이가 있어도 보고되지 않았다.
//!
//! 두 축은 서로 다른 결함을 잰다 — 쪽수는 조판 결과, IR 은 저장 손실이다. 한쪽이 실패했다고
//! 다른 쪽을 건너뛰면, 사람이 "쪽수만 문제고 내용은 온전하다" 로 잘못 읽는다. 이중 실패의
//! 종료 코드 우선순위는 바이너리 단위 테스트로, 실제 문서의 각 축 출력은 이 테스트로 지킨다.
//!
//! 종료 코드 계약은 바꾸지 않는다 — 쪽수 실패는 그대로 4 다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 두 축 모두 통과하는 표본 — 무회귀 기준선.
const CLEAN_SAMPLE: &str = "samples/table-001.hwp";

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// nextest archive가 런타임에 주입하는 binary 경로를 우선한다(#3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn export(sample: &str, out: &Path, flags: &[&str]) -> Output {
    let mut args: Vec<String> = vec![
        "export-hwpx".into(),
        repo(sample).to_string_lossy().into_owned(),
        out.to_string_lossy().into_owned(),
    ];
    args.extend(flags.iter().map(|f| (*f).to_string()));
    Command::new(rhwp_bin())
        .args(&args)
        .output()
        .expect("rhwp 실행 실패")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 정상 문서와 단독 축은 종전 그대로여야 한다.
#[test]
fn single_axis_and_clean_document_are_unchanged() {
    let dir = std::env::temp_dir().join(format!("rhwp-3915c-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 디렉터리");

    // --verify-pages 단독: 쪽수 축만 보고, IR 비교를 시작하지 않는다.
    let o = export(CLEAN_SAMPLE, &dir.join("p.hwpx"), &["--verify-pages"]);
    let combined = format!("{}{}", stderr(&o), String::from_utf8_lossy(&o.stdout));
    assert!(combined.contains("검증 통과(--verify-pages)"), "{combined}");
    assert!(
        !combined.contains("검증 실패(--verify)"),
        "--verify 를 주지 않았는데 IR 비교가 돌았습니다:\n{combined}"
    );
    assert_eq!(o.status.code(), Some(0), "{combined}");

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
