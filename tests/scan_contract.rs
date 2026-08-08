//! [#3918 승격 3호] `scan` — 코퍼스 발견·분류의 계약.
//!
//! `batch` 는 "경로 목록을 이미 갖고 있다"는 전제에서 시작한다. `scan` 은 그 앞
//! 단계다: 디렉터리를 재귀로 걸어 HWP 계열 파일을 찾고, 확장자 주장과 매직 감지를
//! 대조하고(`extMismatch`), `--probe` 면 실제로 열어 파싱 가능/암호 필요를 기록한다.
//!
//! 계약: 발견은 판정이 아니므로 성공 exit 0(게이트 코드 3 없음) / 실행 실패 stdout
//! 0 B + exit 1 / 조립 오류 exit 2(미지 옵션 침묵 무시 금지 포함). 봉투는 순수
//! JSON 하나, 파일 순서는 경로 오름차순으로 결정적이며, 문서 파생 가능 값
//! (`files[].probe.error`)은 출처 표지가 가린다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const HWP5_SAMPLE: &str = "samples/field-01.hwp";
const HWPX_SAMPLE: &str = "samples/tac-host-spacing.hwpx";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn stdout_json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("stdout 이 순수 JSON 이 아니다")
}

/// 시험용 코퍼스 — 트리 안에 정상 hwp5·정상 hwpx·확장자 거짓말(hwpx 인데 .hwp)·
/// 깨진 파일(.hwp 인데 쓰레기 바이트)·무관 텍스트 파일을 심는다.
///
/// `OnceLock` 인 이유: 테스트는 스레드 병렬로 돈다 — 각자 깔면 한쪽이 복사 중인
/// 반쪽짜리 파일을 다른 쪽 scan 이 읽어 매직 판정이 흔들린다(실측 플레이크).
static CORPUS: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

fn corpus() -> &'static Path {
    CORPUS.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("rhwp-scan-contract-{}", std::process::id()));
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).expect("코퍼스 폴더 생성 실패");
        for (from, to) in [
            (sample(HWP5_SAMPLE), dir.join("a-정상.hwp")),
            (sample(HWPX_SAMPLE), dir.join("b-거짓말.hwp")), // hwpx 매직인데 .hwp 확장자
            (sample(HWPX_SAMPLE), sub.join("c-하위.hwpx")),
        ] {
            std::fs::copy(from, &to).expect("표본 복사 실패");
        }
        // 매직이 어느 포맷도 아닌 쓰레기 — --probe 파싱 실패 경로의 고정 표본.
        std::fs::write(dir.join("d-깨짐.hwp"), b"HWP \xec\x95\x84\xeb\x8b\x98")
            .expect("깨진 파일 생성 실패");
        std::fs::write(dir.join("무관.txt"), "HWP 계열이 아니다").expect("잡음 파일 생성 실패");
        dir
    })
}

#[test]
fn discovery_envelope_is_deterministic_and_classifies_formats() {
    let dir = corpus();
    let out = run(&["scan", dir.to_str().unwrap(), "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["schemaVersion"], "1.0");
    assert_eq!(v["summary"]["total"], 4, "확장자 무관 파일은 세지 않는다");
    assert_eq!(v["summary"]["byFormat"]["hwp5"], 1);
    assert_eq!(v["summary"]["byFormat"]["hwpx"], 2);
    assert_eq!(v["summary"]["byFormat"]["unknown"], 1);
    assert_eq!(
        v["summary"]["extMismatch"], 2,
        "거짓말(.hwp↔hwpx)과 깨짐(.hwp↔unknown) 둘 다 불일치다"
    );
    assert_eq!(v["summary"]["probed"], false);

    // 결정성 — 경로 문자열 오름차순. 같은 트리는 언제나 같은 순서로 나온다.
    let paths: Vec<&str> = v["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "파일 순서가 경로 오름차순이 아니다");

    // 확장자 거짓말 파일: 주장은 hwp(hwp5/hwp3 겸용), 매직은 hwpx.
    let liar = v["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"].as_str().unwrap().contains("b-거짓말"))
        .expect("거짓말 파일 레코드가 없다");
    assert_eq!(liar["extFormat"], "hwp");
    assert_eq!(liar["magicFormat"], "hwpx");
    assert_eq!(liar["extMismatch"], true);
    assert!(
        liar["probe"].is_null(),
        "--probe 없이는 파싱을 시도하지 않는다"
    );
}

#[test]
fn probe_records_parse_result_and_page_count() {
    let s = sample(HWP5_SAMPLE);
    let out = run(&["scan", s.to_str().unwrap(), "--probe", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = stdout_json(&out);
    assert_eq!(v["summary"]["probed"], true);
    assert_eq!(v["summary"]["probeFailed"], 0);
    let probe = &v["files"][0]["probe"];
    assert_eq!(probe["parseOk"], true);
    assert_eq!(probe["needsPassword"], false);
    assert_eq!(probe["pageCount"], 3, "field-01 은 3쪽이다");
}

#[test]
fn probe_failure_is_data_not_a_gate() {
    let dir = corpus();
    let out = run(&["scan", dir.to_str().unwrap(), "--probe", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "발견은 판정이 아니다 — 파싱 실패도 exit 0 의 데이터다"
    );
    let v = stdout_json(&out);
    assert_eq!(v["summary"]["probeFailed"], 1);
    assert_eq!(v["summary"]["needsPassword"], 0);
    let broken = v["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"].as_str().unwrap().contains("d-깨짐"))
        .expect("깨진 파일 레코드가 없다");
    assert_eq!(broken["probe"]["parseOk"], false);
    assert!(
        broken["probe"]["error"]
            .as_str()
            .is_some_and(|e| !e.is_empty()),
        "실패 사유가 비어 있다: {broken}"
    );
}

#[test]
fn max_depth_one_stays_out_of_subfolders() {
    let dir = corpus();
    let out = run(&["scan", dir.to_str().unwrap(), "--max-depth", "1", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_eq!(v["summary"]["total"], 3, "sub/ 아래는 깊이 1 밖이다");
    assert!(v["files"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| !f["path"].as_str().unwrap().contains("하위")));
}

#[test]
fn limit_truncates_after_deterministic_sort() {
    let dir = corpus();
    let out = run(&["scan", dir.to_str().unwrap(), "--limit", "1", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_eq!(v["summary"]["total"], 1);
    assert_eq!(v["summary"]["truncated"], true);
    // 상한은 정렬 뒤에 적용된다 — 남는 것은 언제나 정렬 첫 항목이다.
    assert!(v["files"][0]["path"].as_str().unwrap().contains("a-정상"));
}

#[test]
fn provenance_marks_are_honest_per_invocation() {
    let dir = corpus();
    // probe 실패 메시지가 실린 호출 — 파서가 문서 바이트에서 만든 문자열이므로 표지가 붙는다.
    let out = run(&["scan", dir.to_str().unwrap(), "--probe", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_eq!(v["untrustedContent"], true);
    let fields: Vec<&str> = v["untrustedFields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(
        fields.contains(&"files[].probe.error"),
        "출처 지도의 선언이 봉투 표지에 나타나야 한다: {fields:?}"
    );

    // probe 없는 호출 — 문서를 열지 않았으므로 표지는 정직하게 false 다.
    let out = run(&["scan", dir.to_str().unwrap(), "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let v = stdout_json(&out);
    assert_eq!(
        v["untrustedContent"], false,
        "실리지 않은 필드를 광고하면 표지가 거짓말이 된다"
    );
    assert_eq!(v["untrustedFields"].as_array().map(Vec::len), Some(0));
}

#[test]
fn runtime_failure_keeps_stdout_empty() {
    let out = run(&["scan", "없는-폴더-scan-계약", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        out.stdout.is_empty(),
        "실행 실패는 stdout 0 B — 부분 봉투 금지"
    );
}

#[test]
fn usage_errors_are_exit_two() {
    // 경로 0개
    let out = run(&["scan", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    // 미지 옵션 침묵 무시 금지
    let dir = corpus();
    let out = run(&["scan", dir.to_str().unwrap(), "--bogus"]);
    assert_eq!(out.status.code(), Some(2));
    // --max-depth 값 누락
    let out = run(&["scan", dir.to_str().unwrap(), "--max-depth"]);
    assert_eq!(out.status.code(), Some(2));
    // --limit 에 0 은 무의미하다
    let out = run(&["scan", dir.to_str().unwrap(), "--limit", "0"]);
    assert_eq!(out.status.code(), Some(2));
}
