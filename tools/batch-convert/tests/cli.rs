//! batch-convert 통합 테스트.
//!
//! 실제 rhwp 대신 mock-rhwp 보조 바이너리(src/bin/mock_rhwp.rs)를 `--rhwp-bin`
//! 으로 주입해 병렬 상한·재시도·overwrite 같은 **배치 오케스트레이션 계약**을
//! 검증한다. 동시 실행 수는 벽시계 시간이 아니라 mock 이 남기는 원자적 표식
//! 파일(active/·samples/)로 판정한다.

use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn batch_convert_bin() -> &'static str {
    env!("CARGO_BIN_EXE_batch-convert")
}

fn mock_rhwp_bin() -> &'static str {
    env!("CARGO_BIN_EXE_mock-rhwp")
}

/// 테스트별 격리 작업 폴더 (입력/출력/mock 상태/설정).
struct TestBed {
    root: PathBuf,
    input: PathBuf,
    output: PathBuf,
    state: PathBuf,
}

impl TestBed {
    fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "batch-convert-test-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        let input = root.join("input");
        let output = root.join("output");
        let state = root.join("state");
        fs::create_dir_all(&input).unwrap();
        fs::create_dir_all(&state).unwrap();
        TestBed {
            root,
            input,
            output,
            state,
        }
    }

    fn add_hwp(&self, name: &str) -> PathBuf {
        let path = self.input.join(name);
        fs::write(&path, b"dummy hwp payload").unwrap();
        path
    }

    fn write_config(&self, config: &serde_json::Value) -> PathBuf {
        let path = self.root.join("config.json");
        fs::write(&path, serde_json::to_string_pretty(config).unwrap()).unwrap();
        path
    }

    /// batch-convert 를 mock rhwp 와 함께 실행한다.
    fn run(&self, extra_args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(batch_convert_bin());
        cmd.arg("--input-dir")
            .arg(&self.input)
            .arg("--output-dir")
            .arg(&self.output)
            .arg("--rhwp-bin")
            .arg(mock_rhwp_bin())
            .env("MOCK_RHWP_STATE_DIR", &self.state);
        for arg in extra_args {
            cmd.arg(arg);
        }
        for (key, value) in envs {
            cmd.env(key, value);
        }
        cmd.output().expect("failed to run batch-convert")
    }

    fn argv_logs(&self) -> Vec<Vec<String>> {
        let mut logs = Vec::new();
        if let Ok(entries) = fs::read_dir(self.state.join("argv")) {
            for entry in entries.flatten() {
                let text = fs::read_to_string(entry.path()).unwrap();
                logs.push(text.lines().map(|s| s.to_string()).collect());
            }
        }
        logs
    }

    /// 하위 명령별 mock 호출 횟수.
    fn invocations(&self, subcommand: &str) -> usize {
        self.argv_logs()
            .iter()
            .filter(|log| log.first().map(String::as_str) == Some(subcommand))
            .count()
    }

    fn clear_argv_logs(&self) {
        let _ = fs::remove_dir_all(self.state.join("argv"));
    }

    /// 모든 mock 호출이 관측한 최대 동시 실행 수.
    fn max_concurrency(&self) -> usize {
        let mut max = 0;
        if let Ok(entries) = fs::read_dir(self.state.join("samples")) {
            for entry in entries.flatten() {
                let value: usize = fs::read_to_string(entry.path())
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                max = max.max(value);
            }
        }
        max
    }
}

impl Drop for TestBed {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// 요약 줄("Successful conversions: N" 등)에서 개수를 읽는다.
fn summary_count(stdout: &str, label: &str) -> usize {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(label))
        .unwrap_or_else(|| panic!("missing summary line {:?} in stdout:\n{}", label, stdout))
        .trim()
        .parse()
        .unwrap()
}

fn formats(pdf: bool, png: bool, svg: bool, text: bool) -> serde_json::Value {
    json!({ "pdf": pdf, "png": png, "svg": svg, "text": text })
}

fn pdf_only_config(behavior: serde_json::Value) -> serde_json::Value {
    json!({ "formats": formats(true, false, false, false), "behavior": behavior })
}

/// argv 로그에 `--flag value` 쌍이 있는지 검사.
fn has_flag_value(log: &[String], flag: &str, value: &str) -> bool {
    log.windows(2).any(|w| w[0] == flag && w[1] == value)
}

// ---------------------------------------------------------------------------
// --jobs 계약
// ---------------------------------------------------------------------------

#[test]
fn jobs_zero_is_rejected() {
    let bed = TestBed::new("jobs0");
    bed.add_hwp("a.hwp");

    let output = bed.run(&["--jobs", "0"], &[]);

    assert!(
        !output.status.success(),
        "--jobs 0 must fail, stdout:\n{}",
        stdout_of(&output)
    );
    let stderr = stderr_of(&output);
    assert!(stderr.contains("--jobs"), "stderr:\n{}", stderr);
    // 거부는 rhwp 호출 이전이어야 한다.
    assert_eq!(bed.invocations("export-pdf"), 0);
}

#[test]
fn jobs_one_limits_concurrency_to_one() {
    let bed = TestBed::new("jobs1");
    for i in 0..4 {
        bed.add_hwp(&format!("f{}.hwp", i));
    }
    let config = bed.write_config(&pdf_only_config(json!({})));

    let output = bed.run(&["--config", config.to_str().unwrap(), "--jobs", "1"], &[]);

    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));
    assert_eq!(bed.invocations("export-pdf"), 4);
    // 전용 풀의 worker 가 1개면 mock 이 자기 자신 외의 동시 실행을 볼 수 없다.
    assert_eq!(
        bed.max_concurrency(),
        1,
        "--jobs 1 must serialize rhwp invocations"
    );
    for i in 0..4 {
        assert!(bed.output.join("pdf").join(format!("f{}.pdf", i)).is_file());
    }
}

#[test]
fn jobs_four_allows_parallel_invocations() {
    let bed = TestBed::new("jobs4");
    for i in 0..4 {
        bed.add_hwp(&format!("f{}.hwp", i));
    }
    let config = bed.write_config(&pdf_only_config(json!({})));

    // mock 은 동시 실행 2를 관측할 때까지 (최대 15초) 기다린 뒤 표본을 남긴다 —
    // 겹침 여부를 벽시계 시간이 아니라 랑데부로 확정한다.
    let output = bed.run(
        &["--config", config.to_str().unwrap(), "--jobs", "4"],
        &[
            ("MOCK_RHWP_WAIT_FOR", "2"),
            ("MOCK_RHWP_WAIT_TIMEOUT_MS", "15000"),
        ],
    );

    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));
    assert_eq!(bed.invocations("export-pdf"), 4);
    assert!(
        bed.max_concurrency() >= 2,
        "--jobs 4 with 4 files must run at least 2 rhwp invocations concurrently"
    );
}

// ---------------------------------------------------------------------------
// behavior.overwrite / behavior.skip_existing
// ---------------------------------------------------------------------------

#[test]
fn overwrite_false_preserves_existing_outputs() {
    let bed = TestBed::new("no-overwrite");
    bed.add_hwp("a.hwp");
    bed.add_hwp("b.hwp");
    let existing = bed.output.join("pdf").join("a.pdf");
    fs::create_dir_all(existing.parent().unwrap()).unwrap();
    fs::write(&existing, "OLD").unwrap();
    let config = bed.write_config(&pdf_only_config(json!({ "overwrite": false })));

    let output = bed.run(&["--config", config.to_str().unwrap()], &[]);

    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));
    // 기존 산출물은 그대로, 없는 것만 새로 만든다.
    assert_eq!(fs::read_to_string(&existing).unwrap(), "OLD");
    assert!(bed.output.join("pdf").join("b.pdf").is_file());
    assert_eq!(bed.invocations("export-pdf"), 1);
    let stdout = stdout_of(&output);
    assert_eq!(summary_count(&stdout, "Successful conversions:"), 1);
    assert_eq!(summary_count(&stdout, "Skipped files:"), 1);
}

#[test]
fn overwrite_true_rewrites_existing_outputs() {
    let bed = TestBed::new("overwrite");
    bed.add_hwp("a.hwp");
    let existing = bed.output.join("pdf").join("a.pdf");
    fs::create_dir_all(existing.parent().unwrap()).unwrap();
    fs::write(&existing, "OLD").unwrap();
    // overwrite 기본값은 true.
    let config = bed.write_config(&pdf_only_config(json!({})));

    let output = bed.run(&["--config", config.to_str().unwrap()], &[]);

    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));
    assert_eq!(bed.invocations("export-pdf"), 1);
    let rewritten = fs::read_to_string(&existing).unwrap();
    assert_ne!(rewritten, "OLD", "overwrite=true must rewrite the output");
    assert!(rewritten.starts_with("mock-output"));
}

#[test]
fn skip_existing_skips_fully_converted_files() {
    let bed = TestBed::new("skip-existing");
    bed.add_hwp("a.hwp");
    bed.add_hwp("b.hwp");
    let config = bed.write_config(&json!({
        "formats": formats(true, false, false, true),
        "behavior": { "skip_existing": true }
    }));

    let first = bed.run(&["--config", config.to_str().unwrap()], &[]);
    assert!(first.status.success(), "stderr:\n{}", stderr_of(&first));
    assert_eq!(bed.invocations("export-pdf"), 2);
    assert_eq!(bed.invocations("export-text"), 2);

    bed.clear_argv_logs();
    let second = bed.run(&["--config", config.to_str().unwrap()], &[]);
    assert!(second.status.success(), "stderr:\n{}", stderr_of(&second));
    assert_eq!(bed.invocations("export-pdf"), 0);
    assert_eq!(bed.invocations("export-text"), 0);
    assert_eq!(summary_count(&stdout_of(&second), "Skipped files:"), 2);
}

// ---------------------------------------------------------------------------
// behavior.fail_fast / behavior.max_retries / behavior.collect_failed
// ---------------------------------------------------------------------------

#[test]
fn fail_fast_stops_after_first_failure() {
    let bed = TestBed::new("fail-fast");
    for i in 0..4 {
        bed.add_hwp(&format!("f{}.hwp", i));
    }
    let config = bed.write_config(&pdf_only_config(json!({
        "fail_fast": true,
        "max_retries": 0
    })));

    let output = bed.run(
        &["--config", config.to_str().unwrap(), "--jobs", "1"],
        &[("MOCK_RHWP_FAIL_MATCH", ".hwp")],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr:\n{}",
        stderr_of(&output)
    );
    // 첫 실패 후 남은 파일은 시도조차 하지 않는다.
    assert_eq!(bed.invocations("export-pdf"), 1);
    let stdout = stdout_of(&output);
    assert_eq!(summary_count(&stdout, "Failed conversions:"), 1);
    assert_eq!(summary_count(&stdout, "Skipped files:"), 3);
}

#[test]
fn max_retries_bounds_attempts_then_fails() {
    let bed = TestBed::new("retries-exhaust");
    bed.add_hwp("a.hwp");
    let config = bed.write_config(&pdf_only_config(json!({ "max_retries": 2 })));

    let output = bed.run(
        &["--config", config.to_str().unwrap(), "--jobs", "1"],
        &[("MOCK_RHWP_FAIL_MATCH", ".hwp")],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr:\n{}",
        stderr_of(&output)
    );
    // 총 시도 = 1 + max_retries.
    assert_eq!(bed.invocations("export-pdf"), 3);
    assert_eq!(summary_count(&stdout_of(&output), "Failed conversions:"), 1);
}

#[test]
fn max_retries_recovers_from_transient_failure() {
    let bed = TestBed::new("retries-recover");
    bed.add_hwp("a.hwp");
    let config = bed.write_config(&pdf_only_config(json!({ "max_retries": 2 })));

    let output = bed.run(
        &["--config", config.to_str().unwrap(), "--jobs", "1"],
        &[
            ("MOCK_RHWP_FAIL_MATCH", ".hwp"),
            ("MOCK_RHWP_FAIL_TIMES", "1"),
        ],
    );

    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));
    // 1회 실패 + 재시도 1회 성공 = 호출 2회.
    assert_eq!(bed.invocations("export-pdf"), 2);
    assert!(bed.output.join("pdf").join("a.pdf").is_file());
    assert_eq!(
        summary_count(&stdout_of(&output), "Successful conversions:"),
        1
    );
}

#[test]
fn collect_failed_copies_failed_inputs() {
    let bed = TestBed::new("collect-failed");
    bed.add_hwp("good.hwp");
    bed.add_hwp("bad.hwp");
    let config = bed.write_config(&pdf_only_config(json!({
        "collect_failed": true,
        "max_retries": 0
    })));

    let output = bed.run(
        &["--config", config.to_str().unwrap()],
        &[("MOCK_RHWP_FAIL_MATCH", "bad")],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr:\n{}",
        stderr_of(&output)
    );
    assert!(
        bed.output.join("failed").join("bad.hwp").is_file(),
        "failed input must be collected under <output>/failed/"
    );
    assert!(!bed.output.join("failed").join("good.hwp").exists());
    let stdout = stdout_of(&output);
    assert_eq!(summary_count(&stdout, "Successful conversions:"), 1);
    assert_eq!(summary_count(&stdout, "Failed conversions:"), 1);
}

// ---------------------------------------------------------------------------
// behavior.create_format_dirs / --dry-run
// ---------------------------------------------------------------------------

#[test]
fn create_format_dirs_false_uses_flat_layout() {
    let bed = TestBed::new("flat-layout");
    bed.add_hwp("doc.hwp");
    let config = bed.write_config(&json!({
        "formats": formats(true, false, false, true),
        "behavior": { "create_format_dirs": false }
    }));

    let output = bed.run(&["--config", config.to_str().unwrap()], &[]);

    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));
    assert!(bed.output.join("doc.pdf").is_file());
    assert!(bed.output.join("doc").join("doc.txt").is_file());
    assert!(!bed.output.join("pdf").exists());
    assert!(!bed.output.join("text").exists());
}

#[test]
fn dry_run_invokes_nothing_and_writes_nothing() {
    let bed = TestBed::new("dry-run");
    bed.add_hwp("a.hwp");
    let config = bed.write_config(&pdf_only_config(json!({})));

    let output = bed.run(&["--config", config.to_str().unwrap(), "--dry-run"], &[]);

    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));
    assert_eq!(bed.invocations("export-pdf"), 0);
    assert!(!bed.output.join("pdf").exists());
    assert_eq!(
        summary_count(&stdout_of(&output), "Successful conversions:"),
        1
    );
}

// ---------------------------------------------------------------------------
// 포맷 옵션 → rhwp 플래그 전달
// ---------------------------------------------------------------------------

#[test]
fn format_options_are_forwarded_as_rhwp_flags() {
    let bed = TestBed::new("format-options");
    bed.add_hwp("doc.hwp");
    let config = bed.write_config(&json!({
        "formats": formats(true, true, true, true),
        "pdf": { "backend": "direct", "profile": "print", "raster_dpi": 144 },
        "png": { "profile": "screen", "dpi": 300, "scale": 1.5, "max_dimension": 1568 },
        "svg": { "embed_fonts": true },
        "behavior": {}
    }));

    let output = bed.run(&["--config", config.to_str().unwrap()], &[]);
    assert!(output.status.success(), "stderr:\n{}", stderr_of(&output));

    let logs = bed.argv_logs();
    let log_for = |subcommand: &str| -> &Vec<String> {
        logs.iter()
            .find(|log| log.first().map(String::as_str) == Some(subcommand))
            .unwrap_or_else(|| panic!("no invocation of {}", subcommand))
    };

    let pdf = log_for("export-pdf");
    assert!(has_flag_value(pdf, "--backend", "direct"), "{:?}", pdf);
    assert!(has_flag_value(pdf, "--profile", "print"), "{:?}", pdf);
    assert!(has_flag_value(pdf, "--raster-dpi", "144"), "{:?}", pdf);

    let png = log_for("export-png");
    assert!(has_flag_value(png, "--profile", "screen"), "{:?}", png);
    assert!(has_flag_value(png, "--dpi", "300"), "{:?}", png);
    assert!(has_flag_value(png, "--scale", "1.5"), "{:?}", png);
    assert!(has_flag_value(png, "--max-dimension", "1568"), "{:?}", png);

    let svg = log_for("export-svg");
    assert!(svg.contains(&"--embed-fonts".to_string()), "{:?}", svg);
    assert!(
        !svg.contains(&"--profile".to_string()),
        "svg must not receive --profile unless configured: {:?}",
        svg
    );

    // export-text 는 추가 플래그 없이 <subcommand> <input> -o <output> 만 받는다.
    let text = log_for("export-text");
    assert_eq!(text.len(), 4, "{:?}", text);

    // 산출물 규약 (pdf 파일 + 포맷별 페이지 폴더).
    assert!(bed.output.join("pdf").join("doc.pdf").is_file());
    assert!(bed.output.join("png").join("doc").join("doc.png").is_file());
    assert!(bed.output.join("svg").join("doc").join("doc.svg").is_file());
    assert!(bed
        .output
        .join("text")
        .join("doc")
        .join("doc.txt")
        .is_file());
}

// ---------------------------------------------------------------------------
// config 계약 (unknown field / 잘못된 조합 거부)
// ---------------------------------------------------------------------------

#[test]
fn unknown_config_field_is_rejected() {
    let bed = TestBed::new("unknown-field");
    bed.add_hwp("a.hwp");
    // 제거된 과거 필드(pdf.compression)는 조용히 무시되지 않고 거부되어야 한다.
    let config = bed.write_config(&json!({
        "formats": formats(true, false, false, false),
        "pdf": { "compression": 9 }
    }));

    let output = bed.run(&["--config", config.to_str().unwrap()], &[]);

    assert!(!output.status.success());
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("compression"),
        "stderr must name the unknown field:\n{}",
        stderr
    );
    assert_eq!(bed.invocations("export-pdf"), 0);
}

#[test]
fn invalid_option_combination_is_rejected_before_batch() {
    let bed = TestBed::new("bad-combo");
    bed.add_hwp("a.hwp");
    // rhwp export-svg 는 --profile 과 --embed-fonts 동시 지정을 거부한다 —
    // 파일마다 실패시키는 대신 배치 시작 전에 막는다.
    let config = bed.write_config(&json!({
        "formats": formats(false, false, true, false),
        "svg": { "profile": "print", "embed_fonts": true }
    }));

    let output = bed.run(&["--config", config.to_str().unwrap()], &[]);

    assert!(!output.status.success());
    let stderr = stderr_of(&output);
    assert!(stderr.contains("svg.embed_fonts"), "stderr:\n{}", stderr);
    assert_eq!(bed.invocations("export-svg"), 0);
}
