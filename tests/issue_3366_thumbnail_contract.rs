//! [#3366] `thumbnail` 종료 코드·파싱 계약 회귀 테스트.
//!
//! 계약: 알 수 없는 옵션·인자 없음·`-o` 값 누락·중복 positional 은 즉시 exit 2 (#2707),
//! 옵션은 파일 앞뒤 어디에 와도 동작한다 (#3349 규약). 종전에는 오타를 무시한 채
//! 산출물까지 만들고 exit 0 으로 끝났다.
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// PrvImage 를 실제로 가진 HWP5 샘플.
const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rhwp-3366-{label}-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 폴더");
    dir
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn run_in(current_dir: &Path, args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .current_dir(current_dir)
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({error}).\n{}",
            describe(args, output)
        )
    })
}

struct TestDir(PathBuf);

impl TestDir {
    fn new(label: &str) -> Self {
        Self(unique_temp_dir(label))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 종전 최악 사례 — 오타 옵션을 무시하고 산출물을 만들며 exit 0 이었다.
#[test]
fn unknown_option_is_usage_error_without_output() {
    let sample = sample_path();
    let dir = unique_temp_dir("unknown");
    let out = dir.join("t.png");
    let args = [
        "thumbnail",
        sample.to_str().unwrap(),
        "--no-such-option",
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("알 수 없는 옵션: --no-such-option"),
        "{}",
        describe(&args, &output)
    );
    assert!(
        !out.exists(),
        "사용법 오류 뒤에는 산출물을 만들면 안 됩니다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 인자 없음은 사용법 오류(2)다 — 종전 1.
#[test]
fn no_args_is_usage_error() {
    let args = ["thumbnail"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

/// `-o` 값 누락은 조용히 무시하지 않는다 — 종전 exit 0.
#[test]
fn output_without_value_is_usage_error() {
    let sample = sample_path();
    let args = ["thumbnail", sample.to_str().unwrap(), "-o"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

/// 옵션이 파일 앞에 와도 동작한다 (#3349 규약) — 종전에는 옵션이 파일 경로가 됐다.
#[test]
fn options_before_file_succeeds() {
    let sample = sample_path();
    let args = ["thumbnail", "--base64", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert!(
        !output.stdout.is_empty(),
        "base64 출력이 있어야 합니다.\n{}",
        describe(&args, &output)
    );
}

/// 중복 positional 은 즉시 사용법 오류다.
#[test]
fn duplicate_file_is_usage_error() {
    let sample = sample_path();
    let s = sample.to_str().unwrap();
    let args = ["thumbnail", s, s];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("입력 파일은 하나만"),
        "{}",
        describe(&args, &output)
    );
}

/// 정상 추출(파일 출력)은 종전과 동일하게 동작한다.
#[test]
fn normal_extraction_still_works() {
    let sample = sample_path();
    let dir = unique_temp_dir("ok");
    let out = dir.join("thumb.png");
    let args = [
        "thumbnail",
        sample.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    assert!(out.exists(), "썸네일 파일이 있어야 합니다");
    let _ = std::fs::remove_dir_all(&dir);
}

/// 썸네일이 없는 입력은 종전대로 런타임 실패(1)다 — HWP3 는 PrvImage 가 없다.
#[test]
fn missing_preview_is_runtime_error() {
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample.hwp");
    let args = ["thumbnail", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}

/// [#5511 Q1] 파일 모드는 parser가 돌려준 내장 이미지를 그대로 쓰고 입력은 고치지 않는다.
#[test]
fn file_mode_writes_exact_embedded_preview_and_preserves_input() {
    let sample = sample_path();
    let input_before = std::fs::read(&sample).expect("표본 읽기");
    let expected = rhwp::parser::extract_thumbnail_only(&input_before).expect("내장 썸네일");
    let dir = TestDir::new("exact-preview");
    let out = dir.path().join(format!("thumb.{}", expected.format));
    let args = [
        "thumbnail",
        sample.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];

    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let envelope = parse_json(&args, &output);
    assert_eq!(std::fs::read(&out).expect("썸네일 산출물"), expected.data);
    assert_eq!(
        std::fs::read(&sample).expect("표본 재읽기"),
        input_before,
        "thumbnail은 입력 문서를 변경하면 안 됩니다"
    );
    assert_eq!(envelope["format"], expected.format);
    assert_eq!(envelope["width"], expected.width);
    assert_eq!(envelope["height"], expected.height);
    assert_eq!(envelope["bytes"], expected.data.len());
    assert_eq!(envelope["output"], out.to_str().unwrap());
}

/// base64와 data URI도 파일 모드와 같은 내장 이미지 바이트를 전달하며 산출물을 만들지 않는다.
#[test]
fn encoded_modes_round_trip_exact_preview_without_file_output() {
    let sample = sample_path();
    let input = std::fs::read(&sample).expect("표본 읽기");
    let expected = rhwp::parser::extract_thumbnail_only(&input).expect("내장 썸네일");
    let dir = TestDir::new("encoded-preview");

    for (mode, field) in [("--base64", "base64"), ("--data-uri", "dataUri")] {
        let args = ["thumbnail", sample.to_str().unwrap(), mode, "--json"];
        let output = run_in(dir.path(), &args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            describe(&args, &output)
        );
        let envelope = parse_json(&args, &output);
        let encoded = envelope[field].as_str().expect("인코딩 필드");
        let encoded = encoded.rsplit_once(',').map_or(encoded, |(_, body)| body);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("base64 decode");
        assert_eq!(decoded, expected.data, "{mode} 바이트 동등성");
        assert!(envelope["output"].is_null(), "{envelope}");
    }

    assert_eq!(
        std::fs::read_dir(dir.path()).expect("격리 폴더").count(),
        0,
        "encoded 모드는 파일을 만들면 안 됩니다"
    );
}

/// 출력 경로 생략 시 현재 디렉터리 아래 `output/<stem>_thumb.<format>`을 만든다.
#[test]
fn default_output_path_is_relative_to_current_directory() {
    let sample = sample_path();
    let input = std::fs::read(&sample).expect("표본 읽기");
    let expected = rhwp::parser::extract_thumbnail_only(&input).expect("내장 썸네일");
    let dir = TestDir::new("default-output");
    let stem = sample.file_stem().unwrap().to_string_lossy();
    let relative = format!("output/{stem}_thumb.{}", expected.format);
    let args = ["thumbnail", sample.to_str().unwrap(), "--json"];

    let output = run_in(dir.path(), &args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let envelope = parse_json(&args, &output);
    assert_eq!(envelope["output"], relative);
    assert_eq!(
        std::fs::read(dir.path().join(&relative)).expect("기본 경로 산출물"),
        expected.data
    );
}

/// 디렉터리로 내려갈 수 없는 출력 경로는 runtime 실패이며 성공 봉투를 남기지 않는다.
#[test]
fn output_write_failure_is_runtime_error_without_stdout() {
    let sample = sample_path();
    let dir = TestDir::new("write-failure");
    let blocker = dir.path().join("not-a-directory");
    std::fs::write(&blocker, b"keep").expect("경로 차단 파일");
    let out = blocker.join("thumb.png");
    let args = [
        "thumbnail",
        sample.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];

    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("파일 저장 실패"),
        "{}",
        describe(&args, &output)
    );
    assert_eq!(std::fs::read(&blocker).expect("차단 파일"), b"keep");
}
