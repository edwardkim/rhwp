use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("rhwp-q7-{label}-{}-{sequence}", std::process::id()));
        fs::create_dir_all(&path).expect("temporary directory should be created");
        Self { path }
    }

    fn join(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp command should run")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn parse_json_line(output: &Output) -> Value {
    let stdout = stdout_text(output);
    let mut lines = stdout.lines();
    let value: Value = serde_json::from_str(lines.next().expect("one JSON line should exist"))
        .expect("stdout should contain valid JSON");
    assert!(
        lines.next().is_none(),
        "JSON output should be exactly one line"
    );
    value
}

#[test]
fn test_field_writes_reloadable_output_without_touching_input() {
    let input = sample("samples/field-01.hwp");
    let before = fs::read(&input).expect("field sample should be readable");
    let temp = TempDir::new("test-field");
    let output_path = temp.join("field-output.hwp");

    let result = run(&[
        "test-field",
        input.to_str().expect("sample path should be UTF-8"),
        output_path.to_str().expect("output path should be UTF-8"),
    ]);

    assert!(
        result.status.success(),
        "test-field failed: {}",
        stderr_text(&result)
    );
    assert!(result.stderr.is_empty(), "success should not write stderr");

    let stdout = stdout_text(&result);
    assert!(stdout.contains("=== 필드 목록 ("));
    assert!(stdout.contains("=== 필드 값 설정 ==="));
    assert!(stdout.contains("저장:"));
    assert!(output_path.is_file(), "output document should be written");

    let written = fs::read(&output_path).expect("written document should be readable");
    assert!(!written.is_empty(), "written document should not be empty");
    rhwp::parser::parse_document(&written).expect("written document should reload");

    let after = fs::read(&input).expect("field sample should remain readable");
    assert_eq!(before, after, "test-field must not modify its input");
}

#[test]
fn dump_anchors_human_output_is_deterministic_and_keeps_coordinate_fields() {
    let input = sample("samples/hwp3-sample.hwp");
    let input_arg = input.to_str().expect("sample path should be UTF-8");

    let first = run(&["dump-anchors", input_arg, "--all"]);
    let second = run(&["dump-anchors", input_arg, "--all"]);

    assert!(
        first.status.success(),
        "dump-anchors failed: {}",
        stderr_text(&first)
    );
    assert!(second.status.success());
    assert!(first.stderr.is_empty());
    assert!(second.stderr.is_empty());
    assert_eq!(
        first.stdout, second.stdout,
        "human output must be deterministic"
    );

    let stdout = stdout_text(&first);
    assert!(stdout.starts_with(&format!("== {}\n", input.display())));
    assert!(stdout.contains("s0 p0: chars="));
    assert!(stdout.contains("char_offsets="));
    assert!(stdout.contains("controls="));
    assert!(stdout.contains("ctrl_positions="));
}

#[test]
fn dump_carets_json_is_filtered_and_machine_clean() {
    let input = sample("samples/hwp3-sample.hwp");
    let result = run(&[
        "dump-carets",
        input.to_str().expect("sample path should be UTF-8"),
        "--json",
        "-s",
        "0",
        "-p",
        "0",
    ]);

    assert!(
        result.status.success(),
        "dump-carets failed: {}",
        stderr_text(&result)
    );
    assert!(result.stderr.is_empty());

    let value = parse_json_line(&result);
    assert_eq!(value["schemaVersion"], "1.0");
    assert_eq!(value["file"], input.display().to_string());
    let count = value["count"].as_u64().expect("count should be an integer");
    let carets = value["carets"]
        .as_array()
        .expect("carets should be an array");
    assert!(count > 0, "fixture should expose caret positions");
    assert_eq!(count as usize, carets.len());

    let mut previous_offset = None;
    for caret in carets {
        assert_eq!(caret["section"], 0);
        assert_eq!(caret["para"], 0);
        let offset = caret["offset"]
            .as_u64()
            .expect("offset should be an integer");
        if let Some(previous) = previous_offset {
            assert!(previous <= offset, "caret offsets should be ordered");
        }
        previous_offset = Some(offset);
        assert!(caret.get("pageIndex").is_some());
        assert!(caret.get("x").is_some());
        assert!(caret.get("y").is_some());
        assert!(caret.get("height").is_some());
    }
}

#[test]
fn dump_carets_json_failure_keeps_stdout_clean() {
    let temp = TempDir::new("missing-caret");
    let missing = temp.join("missing.hwp");
    let result = run(&[
        "dump-carets",
        missing.to_str().expect("missing path should be UTF-8"),
        "--json",
    ]);

    assert_eq!(result.status.code(), Some(1));
    assert!(
        result.stdout.is_empty(),
        "failure must not contaminate stdout"
    );
    assert!(
        !result.stderr.is_empty(),
        "failure should explain itself on stderr"
    );
}

#[test]
fn ir_sweep_json_preserves_identical_and_truncated_difference_contracts() {
    let same = sample("samples/hwp3-sample.hwp");
    let different = sample("samples/SO-SUEOP.hwp");
    let same_arg = same.to_str().expect("sample path should be UTF-8");
    let different_arg = different.to_str().expect("sample path should be UTF-8");

    let identical = run(&["ir-sweep", same_arg, same_arg, "--json"]);
    assert!(
        identical.status.success(),
        "identical sweep failed: {}",
        stderr_text(&identical)
    );
    assert!(identical.stderr.is_empty());
    let identical_json = parse_json_line(&identical);
    assert_eq!(identical_json["identical"], true);
    assert_eq!(identical_json["diffCount"], 0);
    assert_eq!(identical_json["truncated"], false);
    assert_eq!(identical_json["categories"], serde_json::json!({}));
    assert_eq!(identical_json["divergences"], serde_json::json!([]));

    let changed = run(&[
        "ir-sweep",
        same_arg,
        different_arg,
        "--json",
        "--max-lines",
        "1",
    ]);
    assert_eq!(
        changed.status.code(),
        Some(3),
        "JSON difference should use exit 3: {}",
        stderr_text(&changed)
    );
    assert!(changed.stderr.is_empty());
    let changed_json = parse_json_line(&changed);
    assert_eq!(changed_json["identical"], false);
    assert!(changed_json["diffCount"].as_u64().unwrap_or(0) > 1);
    assert_eq!(changed_json["truncated"], true);
    assert_eq!(
        changed_json["divergences"]
            .as_array()
            .expect("divergences should be an array")
            .len(),
        1
    );
    assert!(!changed_json["categories"]
        .as_object()
        .expect("categories should be an object")
        .is_empty());
}

#[test]
fn ir_sweep_text_mode_reports_differences_without_json_exit_semantics() {
    let left = sample("samples/hwp3-sample.hwp");
    let right = sample("samples/SO-SUEOP.hwp");
    let result = run(&[
        "ir-sweep",
        left.to_str().expect("sample path should be UTF-8"),
        right.to_str().expect("sample path should be UTF-8"),
        "--max-lines",
        "1",
    ]);

    assert!(
        result.status.success(),
        "text sweep should report differences with exit 0: {}",
        stderr_text(&result)
    );
    assert!(result.stderr.is_empty());
    let stdout = stdout_text(&result);
    assert!(stdout.contains("전수 비교 완료: 차이"));
    assert!(stdout.contains(" → "));
}
