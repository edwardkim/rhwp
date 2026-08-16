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

/// 쪽수 축만 실패하는 표본 — 35→36쪽.
///
/// [#4677] `synam-001.hwp`의 이전 IR 차이는 책갈피 위치 보정으로 해소되었지만, 저장 HWPX의
/// 쪽수 차이는 여전히 남아 있다.
const PAGE_FAIL_SAMPLE: &str = "samples/synam-001.hwp";
/// IR 축만 실패하고 쪽수는 안정적인 표본 — 1쪽, IR 차이 2건(pic 드롭, #3893).
///
/// [#4916 계열] `issue1937` 의 이전 IR 차이(각주 subList lineseg vertpos)는
/// HWP5-origin 노트 vpos 보정 스킵으로 해소되었다. 정상화된 문서를 표본으로
/// 계속 쓰지 않는다(#3820 때 교체와 같은 관례).
const IR_FAIL_SAMPLE: &str = "samples/pic-crop-01.hwp";
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

/// 두 검증 축은 실제 실패 상태를 각각 보고한다.
#[test]
fn page_and_ir_axes_report_their_actual_results() {
    let dir = std::env::temp_dir().join(format!("rhwp-3915-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 디렉터리");
    let page_out = dir.join("page-fail.hwpx");

    let page = export(PAGE_FAIL_SAMPLE, &page_out, &["--verify", "--verify-pages"]);
    let page_combined = format!("{}{}", stderr(&page), String::from_utf8_lossy(&page.stdout));

    assert!(
        page_combined.contains("검증 실패(--verify-pages)"),
        "쪽수 실패가 보고되지 않았습니다:\n{page_combined}"
    );
    assert!(
        page_combined.contains("검증 통과(--verify): IR 차이 없음"),
        "IR 축의 실제 통과 상태가 보고되지 않았습니다:\n{page_combined}"
    );
    assert_eq!(
        page.status.code(),
        Some(4),
        "쪽수 실패의 종료 코드는 4 여야 합니다:\n{page_combined}"
    );

    let ir = export(
        IR_FAIL_SAMPLE,
        &dir.join("ir-fail.hwpx"),
        &["--verify", "--verify-pages"],
    );
    let ir_combined = format!("{}{}", stderr(&ir), String::from_utf8_lossy(&ir.stdout));
    assert!(
        ir_combined.contains("검증 통과(--verify-pages)"),
        "쪽수 축의 실제 통과 상태가 보고되지 않았습니다:\n{ir_combined}"
    );
    assert!(
        ir_combined.contains("검증 실패(--verify)"),
        "IR 실패가 보고되지 않았습니다:\n{ir_combined}"
    );
    assert_eq!(
        ir.status.code(),
        Some(3),
        "IR 실패만 있을 때 종료 코드는 3 이어야 합니다:\n{ir_combined}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// 실패인데 "통과" 도 함께 찍히면 안 된다 — 조기 종료를 걷어낼 때 흔한 사고다.
#[test]
fn failing_page_axis_does_not_also_report_pass() {
    let dir = std::env::temp_dir().join(format!("rhwp-3915b-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 디렉터리");
    let out = dir.join("nopass.hwpx");

    let o = export(PAGE_FAIL_SAMPLE, &out, &["--verify", "--verify-pages"]);
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
    let o = export(PAGE_FAIL_SAMPLE, &dir.join("p.hwpx"), &["--verify-pages"]);
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
