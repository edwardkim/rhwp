//! `dump-pages` 옵션 오류가 문서 전체 덤프로 이어지지 않도록 하는 CLI 회귀 테스트.

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

#[test]
fn invalid_page_number_reports_the_value_and_stops() {
    let output = run(&["dump-pages", "samples/hwp3-sample.hwp", "-p", "not-a-page"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("페이지 번호가 올바르지 않습니다: not-a-page"),
        "잘못된 페이지 값이 오류에 보여야 한다: {stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("문서 로드:"),
        "잘못된 -p가 문서 전체 덤프로 이어지면 안 된다"
    );
}

#[test]
fn unknown_option_stops_before_file_io() {
    let output = run(&[
        "dump-pages",
        "does-not-need-to-exist.hwp",
        "--respect-vpos-resets",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("알 수 없는 옵션: --respect-vpos-resets"),
        "알 수 없는 옵션을 명확히 보고해야 한다: {stderr}"
    );
    assert!(
        !stderr.contains("파일을 읽을 수 없습니다"),
        "옵션 파싱 실패 뒤에는 파일 읽기를 시도하면 안 된다: {stderr}"
    );
}

#[test]
fn missing_page_value_names_the_option_used() {
    let output = run(&["dump-pages", "samples/hwp3-sample.hwp", "-p"]);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("-p 뒤에 페이지 번호가 필요합니다."),
        "실제 사용한 옵션 이름을 오류에 보여야 한다"
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
