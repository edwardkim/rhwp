//! Task #653 보강 — `--summary` / `--max-lines` 출력 가드 검증.
//!
//! ir-diff 의 출력을 카테고리별 카운트 (요약) 또는 N 라인으로 제한 (truncation)
//! 가능한지 점검. 회귀 시 광범위 회귀 검증의 효율 떨어짐.

use std::process::Command;

#[test]
fn summary_mode_categorizes_diffs() {
    let exe = env!("CARGO_BIN_EXE_rhwp");
    let output = Command::new(exe)
        .args([
            "ir-diff",
            "samples/hwpx/aift.hwpx",
            "samples/aift.hwp",
            "--summary",
        ])
        .output()
        .expect("rhwp ir-diff --summary 실행 실패");

    assert!(output.status.success(), "비정상 종료: {:?}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 요약 헤더 출력
    assert!(
        stdout.contains("=== 카테고리별 차이 요약 ==="),
        "요약 모드 헤더 누락. 출력 (앞 500자):\n{}",
        &stdout.chars().take(500).collect::<String>()
    );

    // 최종 합계 라인 보존
    assert!(stdout.contains("=== 비교 완료: 차이 "), "비교 완료 합계 누락");

    // 본 모드에서는 paragraph 헤더(`--- 문단 ... ---`) 출력 안 됨
    assert!(
        !stdout.contains("--- 문단 "),
        "summary 모드에서는 paragraph 헤더가 출력되지 않아야 함"
    );

    // "=== IR 비교: ..." 헤더도 summary 모드에서는 출력 안 됨
    assert!(
        !stdout.contains("=== IR 비교:"),
        "summary 모드에서는 IR 비교 헤더가 출력되지 않아야 함"
    );

    // 카테고리별 카운트 형식: "  {N}건  {category}" 한 줄 이상 존재
    let has_count_line = stdout.lines().any(|l| {
        let t = l.trim_start();
        t.split_whitespace().next()
            .and_then(|s| s.strip_suffix("건"))
            .and_then(|s| s.parse::<u32>().ok())
            .is_some()
    });
    assert!(has_count_line, "카테고리별 카운트 라인 (`{{N}}건  {{category}}`) 누락");
}

#[test]
fn max_lines_truncates() {
    let exe = env!("CARGO_BIN_EXE_rhwp");
    let output = Command::new(exe)
        .args([
            "ir-diff",
            "samples/hwpx/aift.hwpx",
            "samples/aift.hwp",
            "--max-lines",
            "20",
        ])
        .output()
        .expect("rhwp ir-diff --max-lines 실행 실패");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("이하 생략 (--max-lines 20 도달)"),
        "max-lines 도달 마커 누락"
    );

    // 최종 합계 라인은 항상 출력 (가드 외부)
    assert!(stdout.contains("=== 비교 완료: 차이 "), "비교 완료 합계 누락");
}

#[test]
fn no_flags_preserves_full_output() {
    // 회귀 가드: --summary / --max-lines 없으면 기존 출력 형식 보존
    let exe = env!("CARGO_BIN_EXE_rhwp");
    let output = Command::new(exe)
        .args([
            "ir-diff",
            "samples/hwpx/aift.hwpx",
            "samples/aift.hwp",
        ])
        .output()
        .expect("rhwp ir-diff 실행 실패");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 기존 형식 보존 — IR 비교 헤더 + paragraph 헤더 + 차이 라인 + 합계
    assert!(stdout.contains("=== IR 비교:"), "IR 비교 헤더 누락");
    assert!(stdout.contains("--- 문단 "), "paragraph 헤더 누락");
    assert!(stdout.contains("[차이]"), "차이 라인 prefix 누락");
    assert!(stdout.contains("=== 비교 완료: 차이 "), "비교 완료 합계 누락");
    assert!(
        !stdout.contains("=== 카테고리별 차이 요약 ==="),
        "기본 모드에서는 요약 출력이 없어야 함"
    );
    assert!(
        !stdout.contains("이하 생략"),
        "기본 모드에서는 truncation 출력이 없어야 함"
    );
}
