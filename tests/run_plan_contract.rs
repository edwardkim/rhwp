//! [#3703] 계획 실행기 `rhwp run` — 선언적 편집 계획의 정적 선검증·원자 실행·저널.
//! 계약: 선검증 실패 = exit 2 + 디스크 무변경, 성공 = 저널 봉투 + 단 한 번 저장,
//! 왕복 재독으로 편집 실적용 확인, MCP `hwp_run_plan` 선언.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SAMPLE: &str = "samples/field-01.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-runplan-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

/// 테스트 중단·assertion 실패에도 합성한 계획·HWPX·산출물을 남기지 않는다.
/// 회귀 입력은 실제 문서처럼 보일 수 있으므로 임시 파일의 생명주기를 명시한다.
struct TempFileGuard(PathBuf);

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp")
}

fn write_plan(tag: &str, plan: &serde_json::Value) -> PathBuf {
    let p = temp_path(tag, "json");
    std::fs::write(&p, serde_json::to_vec_pretty(plan).unwrap()).unwrap();
    p
}

/// 누름틀 두 개의 이름을 화면상 같은 Latin/Cyrillic 쌍으로 바꾼 임시 HWPX.
/// 공격용 fixture를 저장소에 남기지 않고 `run`의 JSON 저널 계약만 검증한다.
fn hwpx_with_field_names(tag: &str, first: &str, second: &str) -> TempFileGuard {
    let source = sample();
    let source_hwpx = TempFileGuard::new(temp_path(&format!("{tag}-source"), "hwpx"));
    let export = run(&[
        "export-hwpx",
        source.to_str().expect("입력 경로 UTF-8"),
        source_hwpx.path().to_str().expect("임시 경로 UTF-8"),
    ]);
    assert_eq!(
        export.status.code(),
        Some(0),
        "confusable 회귀용 HWPX 변환 실패: {}",
        String::from_utf8_lossy(&export.stderr)
    );

    let bytes = std::fs::read(source_hwpx.path()).expect("HWPX 읽기");
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let patched = TempFileGuard::new(temp_path(&format!("{tag}-patched"), "hwpx"));
    let mut zout = zip::ZipWriter::new(std::fs::File::create(patched.path()).expect("출력 zip"));
    for i in 0..zin.len() {
        let mut entry = zin.by_index(i).expect("zip 항목");
        let name = entry.name().expect("ZIP 엔트리 이름").to_string();
        let mut buf = Vec::new();
        std::io::copy(&mut entry, &mut buf).expect("zip 항목 읽기");
        if name == "Contents/section0.xml" {
            let section = String::from_utf8_lossy(&buf)
                .replace("name=\"회사명\"", &format!("name=\"{first}\""))
                .replace("name=\"작성자\"", &format!("name=\"{second}\""));
            buf = section.into_bytes();
        }
        zout.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )
        .expect("zip 항목 쓰기 시작");
        std::io::Write::write_all(&mut zout, &buf).expect("zip 항목 쓰기");
    }
    zout.finish().expect("zip 마감");
    patched
}

/// 선검증 실패는 실행 0 — exit 2 에 출력 파일이 아예 생기지 않아야 한다.
/// 저널은 어느 step 이 왜 불가한지 데이터로 말한다.
#[test]
fn prevalidation_failure_is_exit_2_with_no_output() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("preval", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "검증사"} },
            { "action": "fill_fields", "data": {"존재하지않는필드XYZ": "값"} },
        ],
    });
    let plan_path = write_plan("preval", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(2), "선검증 실패는 exit 2");
    assert!(!out.exists(), "실행 0 증명 — 출력 파일 부재");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    let invalid = v["invalid"].as_array().expect("invalid[]");
    assert_eq!(invalid.len(), 1, "{v}");
    assert_eq!(invalid[0]["step"], 1, "0-기반 step 지목: {v}");
    assert!(
        invalid[0]["reason"]
            .as_str()
            .unwrap_or("")
            .contains("존재하지않는필드XYZ"),
        "왜 불가한지: {v}"
    );
    let _ = std::fs::remove_file(&plan_path);
}

/// 중간 step 이 불가하면 앞 step 이 유효해도 디스크 무변경 (자연 트랜잭션).
#[test]
fn mid_plan_invalid_step_leaves_disk_unchanged() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("atomic", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "선행유효"} },
            { "action": "replace_text", "find": "이런문자열은문서에없다9999", "replace": "X" },
        ],
    });
    let plan_path = write_plan("atomic", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(2), "중간 불가 = 전체 불가");
    assert!(!out.exists(), "선행 step 도 디스크에 닿지 않는다");
    let _ = std::fs::remove_file(&plan_path);
}

/// `set_cell` 행·열은 HWP 격자 주소(u16)다. JSON u64를 무조건 캐스팅하면 65536이
/// 0으로 감겨 전혀 다른 셀을 고치므로, 선검증에서 이유와 함께 거부해야 한다.
#[test]
fn set_cell_u16_overflow_is_invalid_without_output() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/table-001.hwp");
    if !input.exists() {
        eprintln!("표 샘플 없음 — 건너뜀");
        return;
    }
    for (axis, row, col) in [("row", 65536, 0), ("col", 0, 65536)] {
        let out = TempFileGuard::new(temp_path(&format!("set-cell-u16-overflow-{axis}"), "hwp"));
        let plan = serde_json::json!({
            "planVersion": "1.0",
            "input": input.to_str().unwrap(),
            "output": out.path().to_str().unwrap(),
            "steps": [
                { "action": "set_cell", "table": 0, "row": row, "col": col, "text": "범위초과" },
            ],
        });
        let plan_path =
            TempFileGuard::new(write_plan(&format!("set-cell-u16-overflow-{axis}"), &plan));
        let args = [
            "run",
            plan_path.path().to_str().expect("계획 경로 UTF-8"),
            "--json",
        ];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{axis}=65536은 u16 범위 밖이므로 선검증에서 거부해야 한다. stdout: {} stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert!(
            !out.path().exists(),
            "{axis}=65536 선검증 실패는 다른 셀을 0으로 감아 저장하지 않아야 한다"
        );
        let journal: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("선검증 JSON 저널");
        let invalid = journal["invalid"].as_array().expect("invalid[]");
        assert_eq!(invalid.len(), 1, "{journal}");
        assert_eq!(invalid[0]["step"], 0, "{journal}");
        assert_eq!(invalid[0]["action"], "set_cell", "{journal}");
        assert!(
            invalid[0]["reason"]
                .as_str()
                .is_some_and(|reason| !reason.trim().is_empty()),
            "범위 초과의 사유를 invalid[]에 남겨야 한다: {journal}"
        );
    }
}

/// 계획 실행기의 `fill_fields` JSON 저널도 직접 `edit fill-fields --json`처럼
/// 화면상 같은 필드명 쌍을 confusable로 공개해야 한다. 그렇지 않으면 자동 실행이
/// 키릴 문자 하나가 다른 필드를 조용히 채운다.
#[test]
fn plan_fill_fields_journal_reports_confusable_twin() {
    let input = hwpx_with_field_names("plan-confusable", "Total", "\u{0422}otal");
    let out = TempFileGuard::new(temp_path("plan-confusable-output", "hwpx"));
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": input.path().to_str().unwrap(),
        "output": out.path().to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": { "Total": "999" } },
        ],
    });
    let plan_path = TempFileGuard::new(write_plan("plan-confusable", &plan));
    let args = [
        "run",
        plan_path.path().to_str().expect("계획 경로 UTF-8"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "계획 실행 실패. stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let journal: serde_json::Value = serde_json::from_slice(&output.stdout).expect("실행 저널");
    let steps = journal["steps"].as_array().expect("steps[]");
    assert_eq!(steps.len(), 1, "{journal}");
    let confusable = steps[0]["confusable"]
        .as_array()
        .expect("run fill_fields 저널은 confusable 배열을 항상 제공해야 한다");
    assert_eq!(confusable.len(), 1, "쌍둥이 필드 경고 누락: {journal}");
    assert_eq!(confusable[0]["name"], "Total", "{journal}");
    assert_eq!(
        confusable[0]["lookalikes"].as_array().map(Vec::len),
        Some(1),
        "키릴 동형자도 함께 기록해야 한다: {journal}"
    );
    assert!(out.path().exists(), "성공한 계획은 산출물을 한 번 저장한다");
}

/// 정상 계획: 저널이 step 별 결과와 verify 자기검증을 담고 exit 0.
#[test]
fn journal_reports_steps_and_verify() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("journal", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "계획실행사"} },
        ],
        "assertions": { "notFoundEmpty": true, "verify": true },
    });
    let plan_path = write_plan("journal", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_eq!(output.status.code(), Some(0), "{v}");
    assert!(out.exists(), "단언 통과 시에만 단 한 번 저장");
    let steps = v["steps"].as_array().expect("steps[]");
    assert_eq!(steps.len(), 1, "{v}");
    assert_eq!(steps[0]["action"], "fill_fields", "{v}");
    assert_eq!(steps[0]["filledCount"], 1, "{v}");
    assert_eq!(v["verify"]["identical"], true, "자기검증 동봉: {v}");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&plan_path);
}

/// 왕복 재독 — 산출물을 다시 읽어 계획의 편집이 실제 적용됐음을 확인한다.
#[test]
fn rerun_reread_confirms_edits_applied() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("reread", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [
            { "action": "fill_fields", "data": {"회사명": "왕복재독사"} },
        ],
        "assertions": { "verify": true },
    });
    let plan_path = write_plan("reread", &plan);
    let output = run(&["run", plan_path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let fields = run(&["fields", out.to_str().unwrap(), "--json"]);
    assert_eq!(fields.status.code(), Some(0));
    let fv: serde_json::Value = serde_json::from_slice(&fields.stdout).expect("fields");
    let text = fv.to_string();
    assert!(text.contains("왕복재독사"), "산출물 재독에 새 값: {fv}");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&plan_path);
}

/// capabilities --mcp 가 hwp_run_plan 도구를 선언한다 (에이전트 발견 가능성).
#[test]
fn capabilities_declares_run_plan_tool() {
    let output = run(&["capabilities", "--mcp"]);
    assert_eq!(output.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("caps");
    let names: Vec<&str> = v["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"hwp_run_plan"), "{names:?}");
}

/// mcp-serve hwp_run_plan — 인라인 계획 객체로 같은 엔진을 태우고 저널을 돌려준다.
#[test]
fn mcp_run_plan_returns_journal() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("mcp", "hwp");
    let plan = serde_json::json!({
        "planVersion": "1.0",
        "input": p.to_str().unwrap(),
        "output": out.to_str().unwrap(),
        "steps": [ { "action": "fill_fields", "data": {"회사명": "MCP계획사"} } ],
        "assertions": { "verify": true },
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"hwp_run_plan","arguments":{{"plan":{}}}}}}}"#,
        serde_json::to_string(&plan).unwrap()
    )
    .unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0);
    let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(v["result"]["isError"], false, "{v}");
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    let journal: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(journal["verify"]["identical"], true, "{journal}");
    assert!(out.exists());
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(&out);
}
