//! [#2707] CLI 종료 코드 계약 회귀 테스트.
//!
//! 계약: 0 성공 / 1 런타임 실패(읽기·파싱·렌더·쓰기) / 2 사용법 오류
//! (인자 없음, 알 수 없는 옵션, 알 수 없는 명령, 페이지 범위 초과).
//! 3(`--verify` IR 차이)·4(`--verify-pages` 페이지 수 불일치)는 기존 문서화 계약이라
//! 본 테스트가 다루지 않는다 — `tests/issue_1638_convert_verify_gate.rs` 참조.
#![cfg(not(target_arch = "wasm32"))]

#[path = "support/cli_exit_code_support.rs"]
mod cli_exit_code_support;

use cli_exit_code_support::{assert_code, describe, unique_temp_path};
use std::path::{Path, PathBuf};

/// 파싱까지 성공하는 실제 샘플 (페이지 범위 초과·쓰기 실패 경로 검증용).
const SAMPLE: &str = "samples/hwp3-sample.hwp";

/// 인자 없이 호출했을 때 사용법 오류(2)가 나와야 하는 명령들.
const COMMANDS_WITHOUT_ARGS: &[&str] = &[
    "export-svg",
    "export-render-tree",
    "export-structure",
    "export-text",
    "export-markdown",
    "convert",
    "export-hwpx",
];

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 샘플을 결정적으로 손상시켜(바이트 플립) 임시 파일로 쓴다 — 퍼징 재현자용.
fn write_flipped(sample: &str, flip_pct: usize, label: &str) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples")
        .join(sample);
    let mut data = std::fs::read(&src).expect("샘플 읽기");
    let pos = data.len() * flip_pct / 100;
    data[pos] ^= 0xFF;
    let path = unique_temp_path(label);
    std::fs::write(&path, &data).expect("손상본 쓰기");
    path
}

/// [robustness] 초인적 규모 퍼징(6371 손상)이 잡은 렌더러 i32 덧셈 오버플로 패닉
/// 회귀. `s.vertical_pos + s.line_height`(typeset.rs·table_layout.rs)가 손상 입력의
/// 거대 layout 값으로 오버플로해 패닉(exit 101)하던 것을 saturating 으로 막았다.
/// 이제 손상 입력을 패닉 없이 우아하게 처리한다(101 이 아니어야 한다).
#[test]
fn corrupt_input_does_not_panic_in_renderer() {
    // 초인적 규모 퍼징이 잡은 렌더러 오버플로 사이트들의 재현자 — info(레이아웃) 와
    // export-text(전체 렌더) 두 경로 모두.
    for (sample, pct, cmd, label) in [
        ("hwp3-sample11.hwp", 45, "info", "typeset-vpos"),
        (
            "issue1949_giant_cell_nested_tables_perf.hwp",
            55,
            "info",
            "tablelayout-vpos",
        ),
        (
            "HWP5-nopassword-123456.hwp",
            90,
            "export-text",
            "typeset-lhls",
        ),
        (
            "issue1937_rowbreak_footnote_overpagination.hwp",
            90,
            "export-text",
            "heightmeasurer-vpos",
        ),
    ] {
        let path = write_flipped(sample, pct, label);
        let arg = path.to_str().expect("경로");
        let args: Vec<&str> = if cmd == "info" {
            vec![cmd, arg, "--json"]
        } else {
            vec![cmd, arg]
        };
        let output = assert_code(&args, 0);
        assert_ne!(
            output.status.code(),
            Some(101),
            "손상 입력이 렌더러에서 패닉했다: {sample} ({cmd})"
        );
        let _ = std::fs::remove_file(&path);
    }
}

// --- 2: 사용법 오류 -------------------------------------------------------

#[test]
fn missing_arguments_report_usage_error() {
    for command in COMMANDS_WITHOUT_ARGS {
        assert_code(&[command], 2);
    }
}

#[test]
fn unknown_command_writes_usage_to_stderr_and_fails() {
    let args = ["expport-svg", "foo.hwp"];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 명령"),
        "알 수 없는 명령을 stderr 로 알려야 한다\n{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "사용법 안내가 stdout 을 오염시키면 안 된다\n{}",
        describe(&args, &output)
    );
}

#[test]
fn missing_command_reports_usage_error() {
    let args: [&str; 0] = [];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("명령을 지정해주세요"),
        "명령 누락을 stderr 로 알려야 한다\n{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn unknown_option_is_fatal_instead_of_silently_ignored() {
    // `--font-path` 오타. 경고만 찍고 렌더를 계속하면 잘못된 산출물이 성공으로 보고된다.
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");
    let output_dir = unique_temp_path("unknown-option");
    let output_dir = output_dir.to_str().expect("utf-8 경로").to_string();

    let args = [
        "export-svg",
        sample,
        "--fontpath",
        "./ttfs",
        "-o",
        &output_dir,
    ];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 옵션: --fontpath"),
        "어떤 옵션이 문제인지 알려야 한다\n{}",
        describe(&args, &output)
    );
    assert!(
        !Path::new(&output_dir).exists(),
        "옵션 파싱 실패 뒤에는 산출물을 만들면 안 된다"
    );
}

#[test]
fn page_out_of_range_reports_usage_error() {
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");
    let output_dir = unique_temp_path("page-range");
    let output_dir = output_dir.to_str().expect("utf-8 경로").to_string();

    assert_code(&["export-text", sample, "-p", "9999", "-o", &output_dir], 2);

    let _ = std::fs::remove_dir_all(&output_dir);
}

// --- 1: 런타임 실패 -------------------------------------------------------

#[test]
fn unreadable_input_reports_runtime_failure() {
    let missing = unique_temp_path("missing.hwp");
    let missing = missing.to_str().expect("utf-8 경로").to_string();
    let out_dir = unique_temp_path("runtime-out");
    let out_dir = out_dir.to_str().expect("utf-8 경로").to_string();
    let mut hwp_out_file = unique_temp_path("runtime-out");
    hwp_out_file.set_extension("hwp");
    let hwp_out_file = hwp_out_file.to_str().expect("utf-8 경로").to_string();
    let mut hwpx_out_file = unique_temp_path("runtime-out");
    hwpx_out_file.set_extension("hwpx");
    let hwpx_out_file = hwpx_out_file.to_str().expect("utf-8 경로").to_string();

    for args in [
        vec!["export-svg", &missing, "-o", &out_dir],
        vec!["export-render-tree", &missing, "-o", &out_dir],
        vec!["export-text", &missing, "-o", &out_dir],
        vec!["export-markdown", &missing, "-o", &out_dir],
        vec!["export-structure", &missing],
        vec!["convert", &missing, &hwp_out_file],
        vec!["export-hwpx", &missing, &hwpx_out_file],
    ] {
        assert_code(&args, 1);
    }
}

#[test]
fn page_write_failure_is_counted_and_reported() {
    // 출력 폴더 자리에 일반 파일을 두면 모든 페이지 저장이 실패한다.
    // 이때 성공 메시지는 0개를 보고하고 종료 코드는 1이어야 한다.
    let blocker = unique_temp_path("blocker-not-a-dir");
    std::fs::write(&blocker, b"not a directory").expect("차단용 파일 생성");
    let blocker_arg = blocker.to_str().expect("utf-8 경로").to_string();
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");

    let args = ["export-text", sample, "-o", &blocker_arg];
    let output = assert_code(&args, 1);

    assert!(
        String::from_utf8_lossy(&output.stdout).contains("텍스트 내보내기 완료: 0개 TXT 파일"),
        "실제로 쓴 페이지 수(0)를 보고해야 한다\n{}",
        describe(&args, &output)
    );

    let _ = std::fs::remove_file(&blocker);
}

// --- 손상 입력 DoS 패닉 방어 (writer/convert 경로) -----------------------

/// CARGO_BIN_EXE_rhwp(런타임 우선, #3289) 로 rhwp 를 실행해 Output 을 돌려준다.
fn run_cli(args: &[&str]) -> std::process::Output {
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    std::process::Command::new(bin)
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

/// 손상된 HWP3 의 footer_length 를 과대(0xFFFF)로 만들면 margin_footer 가 용지
/// 높이를 넘어 `PageAreas::from_page_def_for_page` 의 본문 영역 계산에서 u32
/// 뺄셈이 언더플로해 convert/export-hwpx/export-markdown 이 패닉(종료 101)하던
/// DoS 를 막는다. HWP3 DocInfo 는 파일 오프셋 30 에서 시작하고 footer_length(u16)
/// 는 그 안 오프셋 20 → 파일 바이트 50..52 다. saturating 화라 정상 데이터 동작은
/// 불변이고, 손상 입력은 패닉 대신 우아하게(101 이 아닌 코드) 끝나야 한다.
#[test]
fn corrupt_page_margin_does_not_panic_in_writer() {
    let mut data = std::fs::read(sample_path()).expect("hwp3 샘플 읽기");
    assert!(
        data.len() > 52,
        "샘플이 DocInfo(footer_length) 를 포함할 만큼 커야 한다"
    );
    // footer_length = 0xFFFF → margin_footer 과대 → page_height - margin_footer 언더플로.
    data[50] = 0xFF;
    data[51] = 0xFF;

    let corrupt = unique_temp_path("corrupt-footer.hwp");
    std::fs::write(&corrupt, &data).expect("손상 샘플 쓰기");
    let corrupt = corrupt.to_str().expect("utf-8 경로").to_string();

    let mut out_hwp = unique_temp_path("corrupt-footer-out");
    out_hwp.set_extension("hwp");
    let out_hwp = out_hwp.to_str().expect("utf-8 경로").to_string();
    let mut out_hwpx = unique_temp_path("corrupt-footer-out");
    out_hwpx.set_extension("hwpx");
    let out_hwpx = out_hwpx.to_str().expect("utf-8 경로").to_string();
    let md_dir = unique_temp_path("corrupt-footer-md");
    let md_dir = md_dir.to_str().expect("utf-8 경로").to_string();

    for args in [
        vec!["convert", &corrupt, &out_hwp],
        vec!["export-hwpx", &corrupt, &out_hwpx],
        vec!["export-markdown", &corrupt, "-o", &md_dir],
    ] {
        let output = run_cli(&args);
        assert_ne!(
            output.status.code(),
            Some(101),
            "손상 입력이 패닉(101)하면 안 된다 — 우아하게 처리해야 한다\n{}",
            describe(&args, &output)
        );
    }

    let _ = std::fs::remove_file(&corrupt);
    let _ = std::fs::remove_file(&out_hwp);
    let _ = std::fs::remove_file(&out_hwpx);
    let _ = std::fs::remove_dir_all(&md_dir);
}

// --- 0: 성공 경로 회귀 방지 ----------------------------------------------

#[test]
fn help_and_version_still_succeed() {
    for args in [["--help"], ["--version"], ["-h"], ["-V"]] {
        assert_code(&args, 0);
    }
}

#[test]
fn successful_export_returns_zero() {
    let sample = sample_path();
    let sample = sample.to_str().expect("utf-8 경로");
    let output_dir = unique_temp_path("success");
    let output_dir = output_dir.to_str().expect("utf-8 경로").to_string();

    assert_code(&["export-text", sample, "-p", "0", "-o", &output_dir], 0);

    let _ = std::fs::remove_dir_all(&output_dir);
}

#[cfg(not(feature = "native-skia"))]
#[test]
fn export_png_without_native_skia_reports_usage_error() {
    // feature 가 빠진 바이너리에서 기능이 아예 없는데 0으로 끝나면 스크립트가 성공으로 읽는다.
    let args = ["export-png", "foo.hwp"];
    let output = assert_code(&args, 2);

    assert!(
        String::from_utf8_lossy(&output.stderr).contains("native-skia"),
        "왜 못 쓰는지 알려야 한다\n{}",
        describe(&args, &output)
    );
}
