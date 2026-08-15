//! [#3918] `rhwp-agent` 실험 표면 계약 테스트.
//!
//! 고정하는 계약:
//! 1. **등재↔실행 왕복** — `capabilities --json` 의 전 명령이 실제로 디스패치되고,
//!    디스패치 가능한 명령은 전부 등재돼 있다("하위 명령 사각" 봉인).
//! 2. **봉투** — `--json` 의 stdout 은 순수 JSON 하나, `schemaVersion` "1.0",
//!    `untrustedContent`/`untrustedFields` 표지를 싣는다.
//! 3. **종료 코드** — 0/1/2 + 게이트 3 (fingerprint --check·diff-text·verify·pii-scan).
//! 4. **미지 입력 거부** — 미지 명령·미지 플래그는 침묵 무시 없이 exit 2 (#3884 계열).
//! 5. **PII 원문 비노출 기본** — `pii-scan` 봉투에 `--show-values` 없이 `raw` 금지.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 파싱까지 성공하는 실제 표본 (cli_json_contract.rs 와 같은 파일).
const SAMPLE_HWP3: &str = "samples/hwp3-sample.hwp";
/// 다른 내용·다른 포맷의 두 번째 표본 — diff·evidence 의 "다름" 축.
const SAMPLE_HWPX: &str = "samples/hwpx/form-01.hwpx";

/// nextest archive가 런타임에 주입하는 binary 경로를 우선한다(#3289).
fn agent_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-agent")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-agent").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(agent_bin())
        .args(args)
        .output()
        .expect("rhwp-agent 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp-agent {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// stdout 전체가 JSON 하나여야 한다는 계약 그대로 파싱한다.
fn stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

/// 봉투 공통 표지 검사.
fn assert_envelope(v: &serde_json::Value, command: &str, ctx: &str) {
    assert_eq!(v["schemaVersion"], "1.0", "{ctx}\n{v}");
    assert_eq!(v["tool"], "rhwp-agent", "{ctx}\n{v}");
    assert_eq!(v["command"], command, "{ctx}\n{v}");
    assert!(v["version"].is_string(), "{ctx}\n{v}");
    assert!(v["untrustedContent"].is_boolean(), "{ctx}\n{v}");
    assert!(v["untrustedFields"].is_array(), "{ctx}\n{v}");
    // 표지 정합: 목록이 비어 있지 않으면 내용 있음이 true 여야 한다 (반대도 성립).
    let has_fields = !v["untrustedFields"].as_array().unwrap().is_empty();
    assert_eq!(
        v["untrustedContent"].as_bool().unwrap(),
        has_fields,
        "untrustedContent 와 untrustedFields 가 어긋납니다: {ctx}\n{v}"
    );
}

/// 테스트 전용 임시 폴더 (테스트마다 별개 이름, 종료 시 최선 노력 정리).
struct TempDir(PathBuf);
impl TempDir {
    fn new(tag: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("rhwp_agent_contract_{tag}_{}", std::process::id()));
        // 이전 실행 잔재가 있으면 지우고 새로 만든다 — 파일 수 단정이 흔들리지 않게.
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("임시 폴더 생성 실패");
        TempDir(dir)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ── 1. 등재↔실행 왕복 ─────────────────────────────────────────────────────

#[test]
fn capabilities_lists_every_command_and_every_command_dispatches() {
    let args = ["capabilities", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "capabilities", "capabilities --json");
    assert_eq!(v["experimental"], true, "{v}");

    let commands = v["commands"].as_array().expect("commands 배열");
    let names: Vec<&str> = commands.iter().filter_map(|c| c["name"].as_str()).collect();
    // 명령 집합은 계약이다 — 추가는 이 목록을 함께 늘리면 되고, 삭제·개명은 깨져야 한다.
    assert_eq!(
        names,
        vec![
            "capabilities",
            "doctor",
            "scan",
            "fingerprint",
            "diff-text",
            "verify",
            "pii-scan",
            "chunk-plan",
            "context-cost",
            "evidence",
        ],
        "{v}"
    );
    for c in commands {
        assert!(!c["usage"].as_str().unwrap_or("").is_empty(), "{c}");
        assert!(!c["summary"].as_str().unwrap_or("").is_empty(), "{c}");
        assert!(c["flags"].is_array(), "{c}");
    }

    // 왕복: 등재된 명령은 전부 실제로 디스패치된다 — 인자 없이 불러도
    // "알 수 없는 명령" 경로로 빠지지 않아야 한다.
    for name in names {
        let output = run(&[name]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("알 수 없는 명령"),
            "등재된 명령이 디스패치되지 않습니다 - {name}\n{stderr}"
        );
        // 인자가 필요한 명령은 사용법 오류(2), 스스로 완결되는 명령은 성공(0)이다.
        let expected = match name {
            "capabilities" | "doctor" => Some(0),
            _ => Some(2),
        };
        assert_eq!(
            output.status.code(),
            expected,
            "명령 {name} 의 맨몸 호출 계약\n{stderr}"
        );
    }
}

#[test]
fn unknown_command_is_usage_error_with_hint() {
    let output = run(&["fingerprnt"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("알 수 없는 명령"), "{stderr}");
    assert!(
        stderr.contains("fingerprint"),
        "did-you-mean 힌트가 없습니다\n{stderr}"
    );
}

#[test]
fn unknown_flag_is_rejected_not_ignored() {
    let sample = sample(SAMPLE_HWP3);
    for args in [
        vec!["scan", ".", "--recursive"],
        vec!["fingerprint", sample.to_str().unwrap(), "--pretty"],
        vec!["pii-scan", sample.to_str().unwrap(), "--raw"],
    ] {
        let refs: Vec<&str> = args.clone();
        let output = run(&refs);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&refs, &output)
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("알 수 없는 옵션"),
            "{}",
            describe(&refs, &output)
        );
    }
}

// ── 2. doctor ─────────────────────────────────────────────────────────────

#[test]
fn doctor_passes_and_reports_checks() {
    let sample = sample(SAMPLE_HWP3);
    let args = ["doctor", "--json", "--sample", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "doctor", "doctor --json");
    assert_eq!(v["ok"], true, "{v}");
    let checks = v["checks"].as_array().expect("checks 배열");
    assert!(checks.len() >= 4, "{v}");
    assert!(checks.iter().all(|c| c["ok"] == true), "{v}");
}

// ── 3. fingerprint — 결정성·기준선·드리프트 게이트 ────────────────────────

#[test]
fn fingerprint_is_deterministic_and_check_gates_drift() {
    let tmp = TempDir::new("fingerprint");
    let sample = sample(SAMPLE_HWP3);
    let sample = sample.to_str().unwrap();

    // 결정성: 두 번 계산한 의미 지문이 같다.
    let args = ["fingerprint", sample, "--json"];
    let first = stdout_json(&args, &run(&args));
    let second = stdout_json(&args, &run(&args));
    for key in [
        "textHash",
        "pageCount",
        "charCount",
        "paraCount",
        "tableCount",
        "fieldCount",
    ] {
        assert_eq!(first[key], second[key], "지문이 실행마다 흔들립니다: {key}");
    }
    assert_envelope(&first, "fingerprint", "fingerprint --json");
    assert!(
        first["untrustedFields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "fieldNames[]"),
        "fieldNames 는 문서 파생인데 표지가 없습니다\n{first}"
    );

    // 기준선 저장 → 같은 파일 검사 = 드리프트 없음(0).
    let base = tmp.path("base.json");
    let base_str = base.to_str().unwrap();
    let output = run(&["fingerprint", sample, "--write", base_str, "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let output = run(&["fingerprint", sample, "--check", base_str, "--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "같은 파일인데 드리프트로 판정"
    );

    // 기준선 훼손 → exit 3 + 어긋난 필드가 지목된다.
    let mut tampered: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&base).unwrap()).unwrap();
    tampered["charCount"] = serde_json::json!(1);
    let tampered_path = tmp.path("tampered.json");
    std::fs::write(&tampered_path, serde_json::to_string(&tampered).unwrap()).unwrap();
    let args = [
        "fingerprint",
        sample,
        "--check",
        tampered_path.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_eq!(v["ok"], false, "{v}");
    let drifted: Vec<&str> = v["drift"]
        .as_array()
        .expect("drift 배열")
        .iter()
        .filter_map(|d| d["field"].as_str())
        .collect();
    assert_eq!(drifted, vec!["charCount"], "{v}");
}

// ── 4. diff-text — 같음/다름 게이트 ───────────────────────────────────────

#[test]
fn diff_text_same_file_exits_zero() {
    let sample = sample(SAMPLE_HWP3);
    let sample = sample.to_str().unwrap();
    let args = ["diff-text", sample, sample, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "diff-text", "diff-text 같은 파일");
    assert_eq!(v["identical"], true, "{v}");
    assert_eq!(v["added"], 0, "{v}");
    assert_eq!(v["removed"], 0, "{v}");
    // 같음 = 문서 파생 헝크가 없다 = 표지도 비어 있어야 한다.
    assert_eq!(v["untrustedContent"], false, "{v}");
}

#[test]
fn diff_text_different_files_gate_and_declare_hunks() {
    let a = sample(SAMPLE_HWP3);
    let b = sample(SAMPLE_HWPX);
    let args = [
        "diff-text",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_eq!(v["identical"], false, "{v}");
    assert!(
        v["added"].as_u64().unwrap() + v["removed"].as_u64().unwrap() > 0,
        "{v}"
    );
    assert!(v["hunkCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(
        v["untrustedFields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "hunks[].lines[].text"),
        "헝크 텍스트는 문서 본문인데 표지가 없습니다\n{v}"
    );
}

// ── 5. verify — 사후 검증 게이트 ──────────────────────────────────────────

#[test]
fn verify_pass_fail_and_usage_contracts() {
    let sample = sample(SAMPLE_HWP3);
    let sample = sample.to_str().unwrap();

    // 통과 축: 표본의 안정 성질만 건다 (표 개수 등 세부는 추출기 개선에 흔들릴 수 있다).
    let output = run(&[
        "verify",
        sample,
        "--expect-format",
        "hwp3",
        "--expect-min-pages",
        "1",
        "--expect-min-chars",
        "100",
        "--expect-min-tables",
        "1",
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 위반 축: exit 3 + 어느 단정이 왜 어긋났는지 봉투에 남는다.
    let args = ["verify", sample, "--expect-pages", "9999", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "verify", "verify 위반");
    assert_eq!(v["ok"], false, "{v}");
    assert_eq!(v["failed"], 1, "{v}");
    let assertion = &v["assertions"][0];
    assert_eq!(assertion["name"], "pages", "{v}");
    assert_eq!(assertion["ok"], false, "{v}");

    // 기대 0개는 판정이 아니라 사용법 오류다.
    let output = run(&["verify", sample]);
    assert_eq!(output.status.code(), Some(2));
    // 포맷 토큰 오타도 조용히 넘어가지 않는다.
    let output = run(&["verify", sample, "--expect-format", "hwp"]);
    assert_eq!(output.status.code(), Some(2));
    // 빈 필드명은 존재 여부 검사로 의미가 없으므로 사용법 오류여야 한다.
    let output = run(&["verify", sample, "--expect-field", "=value"]);
    assert_eq!(output.status.code(), Some(2));
}

// ── 6. pii-scan — 원문 비노출 기본 ────────────────────────────────────────

#[test]
fn pii_scan_never_carries_raw_without_opt_in() {
    let sample = sample(SAMPLE_HWP3);
    let args = ["pii-scan", sample.to_str().unwrap(), "--json"];
    let output = run(&args);
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "pii-scan", "pii-scan --json");
    assert_eq!(v["showValues"], false, "{v}");

    let total = v["total"].as_u64().expect("total");
    let findings = v["findings"].as_array().expect("findings");
    for f in findings {
        assert!(
            f.get("raw").is_none(),
            "--show-values 없이 원문이 실렸습니다\n{f}"
        );
        assert!(f["masked"].is_string(), "{f}");
    }
    // 게이트 계약: 발견 0 = 0, 1 이상 = 3.
    let expected = if total == 0 { 0 } else { 3 };
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        describe(&args, &output)
    );

    // counts 합계는 total 과 같다.
    let sum: u64 = v["counts"]
        .as_object()
        .unwrap()
        .values()
        .map(|n| n.as_u64().unwrap())
        .sum();
    assert_eq!(sum, total, "{v}");
}

// ── 7. chunk-plan — 안전 봉투·전체 커버 ───────────────────────────────────

#[test]
fn chunk_plan_covers_all_pages_with_safe_envelope() {
    let sample = sample(SAMPLE_HWP3);
    let sample = sample.to_str().unwrap();

    // 예산이 충분하면 구간 하나가 전 쪽을 덮는다.
    let args = ["chunk-plan", sample, "--max-chars", "10000000", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "chunk-plan", "chunk-plan 큰 예산");
    // 계획 봉투에는 문서 본문이 없어야 한다 — 표지 자체가 계약이다.
    assert_eq!(v["untrustedContent"], false, "{v}");
    assert_eq!(v["chunkCount"], 1, "{v}");
    let chunk = &v["chunks"][0];
    assert_eq!(chunk["pageFrom"], 1, "{v}");
    assert_eq!(chunk["pageTo"], v["pageCount"], "{v}");
    // 다음 실행은 셸 문자열이 아니라 구조화된 argv 다. 경로가 하나의 인자로
    // 유지돼 소비자가 shell eval 을 할 필요가 없어야 한다.
    assert_eq!(chunk["command"]["program"], "rhwp", "{v}");
    assert_eq!(
        chunk["command"]["args"],
        serde_json::json!([
            "digest",
            sample,
            "--pages",
            format!("1..{}", v["pageCount"].as_u64().unwrap()),
            "--json",
        ]),
        "{v}"
    );

    // 작은 예산: 구간들이 빈틈·겹침 없이 1..pageCount 를 잇는다.
    let args = ["chunk-plan", sample, "--max-chars", "2000", "--json"];
    let v = stdout_json(&args, &run(&args));
    let chunks = v["chunks"].as_array().unwrap();
    let mut next = 1u64;
    for c in chunks {
        assert_eq!(c["pageFrom"].as_u64().unwrap(), next, "{v}");
        next = c["pageTo"].as_u64().unwrap() + 1;
    }
    assert_eq!(next, v["pageCount"].as_u64().unwrap() + 1, "{v}");
}

// ── 8. scan — 분류·프로브·JSONL ───────────────────────────────────────────

#[test]
fn scan_classifies_mismatch_empty_and_parse_failure() {
    let tmp = TempDir::new("scan");
    // a.hwp: 진짜 HWP3 — 파싱 성공해야 한다.
    std::fs::copy(sample(SAMPLE_HWP3), tmp.path("a.hwp")).unwrap();
    // b.hwpx: 확장자만 hwpx 인 쓰레기 — 매직 unknown + 불일치 + 파싱 실패.
    std::fs::write(tmp.path("b.hwpx"), b"this is not a document").unwrap();
    // c.hwp: 빈 파일 — 매직 empty.
    std::fs::write(tmp.path("c.hwp"), b"").unwrap();

    let dir = tmp.0.to_str().unwrap().to_string();
    let args = ["scan", dir.as_str(), "--probe", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "scan", "scan --probe");

    let files = v["files"].as_array().expect("files 배열");
    assert_eq!(files.len(), 3, "{v}");
    // 경로 오름차순 결정성: a.hwp → b.hwpx → c.hwp.
    let names: Vec<String> = files
        .iter()
        .map(|f| {
            Path::new(f["path"].as_str().unwrap())
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names, vec!["a.hwp", "b.hwpx", "c.hwp"], "{v}");

    assert_eq!(files[0]["magicFormat"], "hwp3", "{v}");
    assert_eq!(files[0]["extMismatch"], false, "{v}");
    assert_eq!(files[0]["probe"]["parseOk"], true, "{v}");

    assert_eq!(files[1]["magicFormat"], "unknown", "{v}");
    assert_eq!(files[1]["extMismatch"], true, "{v}");
    assert_eq!(files[1]["probe"]["parseOk"], false, "{v}");

    assert_eq!(files[2]["magicFormat"], "empty", "{v}");

    let summary = &v["summary"];
    assert_eq!(summary["total"], 3, "{v}");
    assert!(summary["probeFailed"].as_u64().unwrap() >= 2, "{v}");
    // 프로브 오류 문자열이 실렸으므로 표지가 있어야 한다.
    assert!(
        v["untrustedFields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p == "files[].probe.error"),
        "{v}"
    );
}

#[test]
fn scan_jsonl_streams_records_then_summary() {
    let tmp = TempDir::new("jsonl");
    std::fs::copy(sample(SAMPLE_HWP3), tmp.path("only.hwp")).unwrap();
    let dir = tmp.0.to_str().unwrap().to_string();
    let args = ["scan", dir.as_str(), "--jsonl"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2, "파일 1 + 요약 1\n{stdout}");
    let first: serde_json::Value = serde_json::from_str(lines[0]).expect("NDJSON 레코드");
    let last: serde_json::Value = serde_json::from_str(lines[1]).expect("NDJSON 요약");
    assert_eq!(first["record"], "file", "{first}");
    assert_eq!(first["schemaVersion"], "1.0", "{first}");
    assert_eq!(last["record"], "summary", "{last}");
    assert_eq!(last["schemaVersion"], "1.0", "{last}");
    assert_eq!(last["total"], 1, "{last}");
}

// ── 9. evidence — 전/후 번들 ──────────────────────────────────────────────

#[test]
fn evidence_same_file_is_identical_and_different_files_report_changes() {
    let a = sample(SAMPLE_HWP3);
    let a = a.to_str().unwrap();

    let args = ["evidence", a, a, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_envelope(&v, "evidence", "evidence 같은 파일");
    assert_eq!(v["identical"], true, "{v}");
    assert_eq!(v["changed"].as_array().unwrap().len(), 0, "{v}");

    let b = sample(SAMPLE_HWPX);
    let args = ["evidence", a, b.to_str().unwrap(), "--json"];
    let output = run(&args);
    // 보고서이지 게이트가 아니다 — 달라도 0.
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = stdout_json(&args, &output);
    assert_eq!(v["identical"], false, "{v}");
    let changed: Vec<&str> = v["changed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["field"].as_str())
        .collect();
    assert!(changed.contains(&"textHash"), "{v}");
    assert!(v["textDiff"]["hunkCount"].as_u64().unwrap() >= 1, "{v}");

    // 마크다운 모드는 전/후 표를 담은 사람용 텍스트다 (stdout 계약: JSON 아님).
    let output = run(&["evidence", a, b.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0));
    let md = String::from_utf8_lossy(&output.stdout);
    assert!(md.contains("| 항목 | 전 | 후 |"), "{md}");
}
