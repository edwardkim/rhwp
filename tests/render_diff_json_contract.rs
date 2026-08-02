//! [#3719 §6-2] `render-diff --json` 기계 계약 — L1 마지막 공백.
//!
//! 계약: `--json` 의 stdout 은 단건 JSON 봉투(배치는 NDJSON)뿐이고 `schemaVersion` 을
//! 포함한다. 필드 추가는 허용, 기존 필드의 변경·삭제는 본 테스트가 실패로 잡는다.
//!
//! ## 종료 코드가 이 파일의 핵심이다
//!
//! `render-diff` 는 게이트 명령이라 회귀를 찾으면 종전에 exit 1 을 냈다. 그런데 1 은
//! #2707 사전상 **런타임 실패**(읽기·파싱·렌더·쓰기)다 — 도구가 정상 동작하며 회귀를
//! *검출한* 것과 도구가 *실패한* 것이 같은 코드로 뭉개져 있었다. CI 는 둘을 구분해야
//! "렌더가 깨졌다"와 "파일을 못 읽었다"에 다르게 반응할 수 있다.
//!
//! 그래서 `--json` 모드만 회귀를 3(검증 단언 실패, `--verify` 계열과 같은 의미론)으로
//! 옮기고, 사람 모드는 1 그대로 둔다. 이미 1 을 실패로 읽는 CI 스크립트가 있기 때문이다.
//! 그 **무변경**도 아래에서 함께 고정한다 — 고정하지 않으면 다음 사람이 "일관성"을
//! 이유로 사람 모드까지 3 으로 옮겨 기존 소비자를 조용히 깨뜨린다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 자기 라운드트립이 PASS 하는 실제 샘플 (cli_json_contract.rs 와 동일 원천).
const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// 서로 다른 두 문서 — pair 모드에서 확실히 회귀(STRUCT_MISMATCH)를 낸다.
const PAIR_A: &str = "samples/tac-host-spacing.hwpx";
const PAIR_B: &str = "samples/issue2527_empty_linesegs.hwpx";
/// 자기 라운드트립이 임계를 넘는(OVER) 샘플 — 배치 회귀 집계용.
const OVER_SAMPLE: &str = "samples/hwp3-pagedef-1915.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
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

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

/// 종료 코드와 "실패 경로 stdout 0바이트"를 함께 본다.
fn assert_silent_failure(args: &[&str], want: i32) {
    let output = run(args);
    assert_eq!(
        output.status.code(),
        Some(want),
        "{}",
        describe(args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 0바이트여야 합니다.\n{}",
        describe(args, &output)
    );
}

// ── 단건 봉투 ──────────────────────────────────────────────────────────────

#[test]
fn render_diff_json_roundtrip_envelope_contract() {
    let src = sample(SAMPLE);
    let args = ["render-diff", src.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "단건 봉투는 한 줄이어야 합니다.\n{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["mode"], "roundtrip", "{v}");
    assert!(v["sourceA"].is_string(), "{v}");
    // 자기 라운드트립에는 비교 상대 파일이 없다 — 빈 문자열이 아니라 null 이어야
    // 소비자가 "경로가 비었다"와 "그런 축이 없다"를 구분한다.
    assert!(v["sourceB"].is_null(), "{v}");
    assert_eq!(v["via"], "hwpx", "{v}");
    assert!(v["pageFilter"].is_null(), "{v}");
    assert_eq!(v["threshold"], 1.0, "{v}");
    assert!(v["pageCountA"].as_u64().unwrap() >= 1, "{v}");
    assert_eq!(v["pageCountA"], v["pageCountB"], "{v}");
    assert_eq!(v["pageCountMismatch"], false, "{v}");
    assert!(v["maxDisp"].is_number() || v["maxDisp"].is_null(), "{v}");
    assert!(
        v["worstPage"].is_number() || v["worstPage"].is_null(),
        "{v}"
    );
    for key in ["overPages", "structPages", "hardStructPages"] {
        assert!(v[key].as_u64().is_some(), "{key} 는 숫자여야 한다: {v}");
    }
    assert_eq!(v["status"], "PASS", "{v}");
    assert_eq!(v["regression"], false, "{v}");

    let pages = v["pages"].as_array().expect("pages 는 배열");
    assert_eq!(
        pages.len() as u64,
        v["pageCountA"].as_u64().unwrap(),
        "필터 없는 pages 는 비교 페이지 수와 일치해야 한다: {v}"
    );
    let p0 = &pages[0];
    assert_eq!(p0["page"], 0, "{p0}");
    for key in ["nodeCountA", "nodeCountB"] {
        assert!(p0[key].as_u64().is_some(), "{key}: {p0}");
    }
    for key in ["maxDisp", "meanDisp"] {
        assert!(
            p0[key].is_number() || p0[key].is_null(),
            "{key} 는 숫자 또는 null: {p0}"
        );
    }
    assert!(p0["structureMismatch"].is_boolean(), "{p0}");
    assert!(p0["structTextrunPm1"].is_boolean(), "{p0}");
    assert!(p0["topDeltas"].is_array(), "{p0}");
    assert!(p0["typeDeltas"].is_array(), "{p0}");
}

#[test]
fn render_diff_json_pair_mode_reports_both_sources_and_no_via() {
    // pair 는 라운드트립이 아니다 — 경유 포맷이 존재하지 않으므로 via 는 null 이다.
    // "hwpx" 를 기본값으로 실어 보내면 소비자는 하지도 않은 변환을 했다고 읽는다.
    let a = sample(PAIR_A);
    let b = sample(PAIR_B);
    let args = [
        "render-diff",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["mode"], "pair", "{v}");
    assert!(v["sourceA"].is_string(), "{v}");
    assert!(v["sourceB"].is_string(), "{v}");
    assert!(v["via"].is_null(), "{v}");
}

#[test]
fn render_diff_json_page_filter_narrows_pages_and_is_echoed() {
    let a = sample(PAIR_A);
    let b = sample(PAIR_B);
    let args = [
        "render-diff",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "-p",
        "0",
        "--json",
    ];
    let output = run(&args);
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["pageFilter"], 0, "{v}");
    let pages = v["pages"].as_array().expect("pages 는 배열");
    assert_eq!(pages.len(), 1, "{v}");
    assert_eq!(pages[0]["page"], 0, "{v}");
}

// ── 종료 코드: 검출(3) 과 실패(1) 의 분리 ──────────────────────────────────

#[test]
fn render_diff_json_regression_exits_three_with_a_full_envelope() {
    let a = sample(PAIR_A);
    let b = sample(PAIR_B);
    let args = [
        "render-diff",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "회귀 검출은 3(검증 단언 실패)이어야 합니다 — 1 은 런타임 실패 전용입니다.\n{}",
        describe(&args, &output)
    );
    // 3 은 실패가 아니라 판정이다 — 봉투는 온전히 나와야 한다(0바이트 규약은 1·2 만).
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["regression"], true, "{v}");
    assert_ne!(v["status"], "PASS", "{v}");
    assert!(
        !v["pages"].as_array().expect("pages").is_empty(),
        "회귀를 낸 판정인데 근거 페이지가 비어 있습니다: {v}"
    );
}

#[test]
fn render_diff_human_mode_keeps_exit_one_for_regressions() {
    // **무변경 고정**: 사람 모드는 종전대로 1 이다. 이미 1 을 실패로 읽는 CI 스크립트가
    // 있으므로, 일관성을 이유로 3 으로 옮기면 기존 소비자가 조용히 깨진다.
    let a = sample(PAIR_A);
    let b = sample(PAIR_B);
    let args = ["render-diff", a.to_str().unwrap(), b.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "사람 모드 종료 코드는 바뀌면 안 됩니다.\n{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("status:"),
        "사람 모드는 사람용 표를 유지해야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn render_diff_json_missing_file_is_runtime_failure_with_silent_stdout() {
    // 읽기 실패는 회귀 검출이 아니다 — 3 이 아니라 1 이고 stdout 은 0바이트다.
    assert_silent_failure(&["render-diff", "없는파일-renderdiff.hwp", "--json"], 1);
    assert_silent_failure(
        &[
            "render-diff",
            "없는A-renderdiff.hwp",
            "없는B-renderdiff.hwp",
            "--json",
        ],
        1,
    );
}

#[test]
fn render_diff_json_usage_errors_are_exit_two_with_silent_stdout() {
    let src = sample(SAMPLE);
    let path = src.to_str().unwrap();
    // 미지 옵션
    assert_silent_failure(&["render-diff", path, "--nope", "--json"], 2);
    // 잘못된 --via 값
    assert_silent_failure(&["render-diff", path, "--via", "zip", "--json"], 2);
    // 비수치 -p / --max-disp
    assert_silent_failure(&["render-diff", path, "-p", "abc", "--json"], 2);
    assert_silent_failure(&["render-diff", path, "--max-disp", "xx", "--json"], 2);
    // 위치 인자 없음
    assert_silent_failure(&["render-diff", "--json"], 2);
}

#[test]
fn render_diff_page_filter_out_of_range_is_usage_error_not_an_empty_result() {
    // "필터가 안 맞았다"와 "차이가 없다"는 정반대 결론이다. 조용한 빈 결과로 내면
    // 소비자는 후자로 읽고 회귀를 통과시킨다 — 두 모드 모두 사용법 오류(2)로 만든다.
    let a = sample(PAIR_A);
    let b = sample(PAIR_B);
    assert_silent_failure(
        &[
            "render-diff",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "-p",
            "999",
            "--json",
        ],
        2,
    );
    assert_silent_failure(
        &[
            "render-diff",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "-p",
            "999",
        ],
        2,
    );
}

// ── 배치 NDJSON ────────────────────────────────────────────────────────────

/// 테스트용 임시 폴더 — 드롭 시 통째로 지운다.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("rhwp-render-diff-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("임시 폴더 생성");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn copy_in(&self, rel: &str) {
        let src = sample(rel);
        let name = src.file_name().expect("파일 이름");
        std::fs::copy(&src, self.0.join(name)).expect("샘플 복사");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn ndjson(args: &[&str], output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l).unwrap_or_else(|e| {
                panic!(
                    "NDJSON 한 줄이 JSON 이 아닙니다 ({e}): {l}\n{}",
                    describe(args, output)
                )
            })
        })
        .collect()
}

#[test]
fn render_diff_batch_json_streams_ndjson_and_keeps_failed_loads() {
    // 로드 실패 레코드를 스트림에서 빼면 입력 N건·출력 N-1건이 되고, 아무도 그
    // 한 건이 처리되지 않았음을 모른다 — 누락은 반드시 레코드로 남아야 한다.
    let dir = TempDir::new("mixed");
    dir.copy_in(OVER_SAMPLE);
    std::fs::write(dir.path().join("깨진문서.hwp"), b"not a document").expect("깨진 파일");
    let out = dir.path().join("out");

    let args = [
        "render-diff",
        "--batch",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    // 로드 실패가 있으면 "전건을 재봤다"고 말할 수 없다 — 회귀(3)보다 실패(1)가 우선한다.
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );

    let records = ndjson(&args, &output);
    assert_eq!(
        records.len(),
        2,
        "입력 2건은 모두 레코드를 남겨야 합니다: {records:?}"
    );
    for r in &records {
        assert_eq!(r["schemaVersion"], "1.0", "{r}");
        assert!(r["source"].is_string(), "{r}");
        assert!(r["status"].is_string(), "{r}");
        assert!(r["regression"].is_boolean(), "{r}");
        assert_eq!(r["via"], "hwpx", "{r}");
    }
    let failed: Vec<&serde_json::Value> = records
        .iter()
        .filter(|r| r.get("error").is_some())
        .collect();
    assert_eq!(
        failed.len(),
        1,
        "로드 실패 레코드가 정확히 1건이어야 합니다: {records:?}"
    );
    assert_eq!(failed[0]["status"], "LOAD_FAIL", "{}", failed[0]);
    // 측정 실패는 회귀 검출이 아니다 — 두 축이 겹치면 종료 코드를 가를 수 없다.
    assert_eq!(failed[0]["regression"], false, "{}", failed[0]);
}

#[test]
fn render_diff_batch_json_regression_only_exits_three() {
    let dir = TempDir::new("regression");
    dir.copy_in(OVER_SAMPLE);
    let out = dir.path().join("out");
    let args = [
        "render-diff",
        "--batch",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(3),
        "로드 실패 없이 회귀만 있으면 3 입니다.\n{}",
        describe(&args, &output)
    );
    let records = ndjson(&args, &output);
    assert_eq!(records.len(), 1, "{records:?}");
    assert_eq!(records[0]["regression"], true, "{}", records[0]);
    assert!(records[0].get("error").is_none(), "{}", records[0]);
}

#[test]
fn render_diff_batch_human_mode_keeps_exit_one() {
    // 배치 사람 모드도 무변경 — 하드 실패는 종전대로 1 이다.
    let dir = TempDir::new("human");
    dir.copy_in(OVER_SAMPLE);
    let out = dir.path().join("out");
    let args = [
        "render-diff",
        "--batch",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
}

#[test]
fn render_diff_batch_json_stdout_carries_no_human_summary() {
    // NDJSON stdout 에 요약·TSV 안내가 섞이면 스트림 파서가 그 줄에서 죽는다.
    let dir = TempDir::new("clean");
    dir.copy_in(OVER_SAMPLE);
    let out = dir.path().join("out");
    let args = [
        "render-diff",
        "--batch",
        dir.path().to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("render-diff 요약") && !stdout.contains("TSV 저장"),
        "요약·산출물 안내는 stderr 로 가야 합니다.\n{}",
        describe(&args, &output)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("render-diff 요약"),
        "요약이 통째로 사라지면 사람이 배치 결과를 못 읽습니다.\n{}",
        describe(&args, &output)
    );
}

// ── 자기서술 드리프트 가드 ─────────────────────────────────────────────────

fn capabilities() -> serde_json::Value {
    let args = ["capabilities"];
    parse_stdout_json(&args, &run(&args))
}

fn render_diff_command_entry() -> serde_json::Value {
    capabilities()["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "render-diff")
        .expect("render-diff 항목")
        .clone()
}

#[test]
fn capabilities_declares_render_diff_json_contract() {
    let entry = render_diff_command_entry();
    assert_eq!(entry["json"], true, "{entry}");
    let flags: Vec<&str> = entry["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    for want in ["--json", "--batch", "--via", "-p", "--max-disp", "-o"] {
        assert!(flags.contains(&want), "{want} 선언 누락: {entry}");
    }

    // 선언한 봉투 필드가 실제 봉투에 전부 있어야 한다 — 선언만 있고 나오지 않는
    // 필드는 코드 생성기가 만든 타입에 영영 null 로 남는다.
    let src = sample(SAMPLE);
    let args = ["render-diff", src.to_str().unwrap(), "--json"];
    let v = parse_stdout_json(&args, &run(&args));
    for field in entry["recordFields"].as_array().expect("recordFields") {
        let name = field.as_str().expect("recordFields 항목");
        assert!(
            v.get(name).is_some(),
            "선언한 봉투 필드 {name} 이 실제 출력에 없습니다: {v}"
        );
    }
}

#[test]
fn render_diff_declared_flags_are_actually_accepted() {
    // 선언한 플래그를 CLI 가 거부하면(미지 옵션 = 2) 매니페스트로 호출을 조립하는
    // 에이전트는 첫 호출에서 막힌다.
    let src = sample(SAMPLE);
    let path = src.to_str().unwrap();
    let dir = TempDir::new("flags");
    let out = dir.path().join("out");
    let out = out.to_str().unwrap().to_string();
    let cases: Vec<Vec<&str>> = vec![
        vec!["render-diff", path, "--json"],
        vec!["render-diff", path, "--via", "hwpx", "--json"],
        vec!["render-diff", path, "--via", "hwp", "--json"],
        vec!["render-diff", path, "-p", "0", "--json"],
        vec!["render-diff", path, "--max-disp", "2.5", "--json"],
        vec!["render-diff", path, "-o", &out, "--json"],
    ];
    for args in cases {
        let output = run(&args);
        assert_ne!(
            output.status.code(),
            Some(2),
            "선언한 플래그를 실제로 받지 않습니다.\n{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn exit_code_dictionary_names_render_diff_without_dropping_the_others() {
    let three = capabilities()["exitCodes"]["3"]
        .as_str()
        .expect("exitCodes.3")
        .to_string();
    assert!(
        three.contains("render-diff"),
        "exit 3 사전이 render-diff 표면을 빠뜨렸습니다: {three}"
    );
    // 기존 표면 서술을 지우면 cli_json_contract 의 사전 가드가 깨진다.
    for surface in ["convert", "edit", "run"] {
        assert!(
            three.contains(surface),
            "exit 3 사전에서 {surface} 표면이 사라졌습니다: {three}"
        );
    }
}

#[test]
fn mcp_manifest_registers_hwp_render_diff_fully_wired() {
    let args = ["capabilities", "--mcp"];
    let m = parse_stdout_json(&args, &run(&args));
    let tool = m["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_render_diff")
        .expect("hwp_render_diff 등재");

    assert_eq!(tool["cli"]["command"], "render-diff", "{tool}");
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    assert!(tool["inputSchema"]["properties"].is_object(), "{tool}");
    // 필수 인자는 배열로 반드시 선언한다 — 부재와 "필수 없음"은 다르다.
    let required: Vec<&str> = tool["inputSchema"]["required"]
        .as_array()
        .expect("required 배열")
        .iter()
        .filter_map(|r| r.as_str())
        .collect();
    assert_eq!(required, vec!["path"], "{tool}");

    // 선언한 모든 입력 속성이 CLI 에 닿아야 한다.
    let mut wired: Vec<String> = tool["cli"]["args"]
        .as_array()
        .expect("args")
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| s.starts_with('{') && s.ends_with('}') && s.len() > 2)
        .map(|s| s[1..s.len() - 1].to_string())
        .collect();
    for o in tool["cli"]["optionalArgs"]
        .as_array()
        .expect("optionalArgs")
    {
        wired.push(o["when"].as_str().expect("when").to_string());
    }
    for key in tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties")
        .keys()
    {
        assert!(wired.contains(key), "{key} 가 배선되지 않았습니다: {tool}");
    }

    // 도구가 광고하는 출력 필드는 자기서술 recordFields 와 같은 목록이어야 한다.
    let entry = render_diff_command_entry();
    assert_eq!(
        tool["outputFields"], entry["recordFields"],
        "MCP 출력 필드와 capabilities recordFields 가 어긋납니다"
    );
}

#[test]
fn help_documents_json_and_the_exit_three_rule() {
    let help = run(&["--help"]);
    let text = String::from_utf8_lossy(&help.stdout);
    let block: Vec<&str> = text
        .lines()
        .filter(|l| l.contains("render-diff") || l.contains("종료 코드 3"))
        .collect();
    let joined = block.join("\n");
    assert!(
        joined.contains("--json"),
        "help 에 --json 안내가 없습니다:\n{joined}"
    );
    assert!(
        joined.contains("종료 코드 3"),
        "help 가 exit 3 규약을 밝히지 않습니다:\n{joined}"
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
