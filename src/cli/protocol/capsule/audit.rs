use crate::cli::protocol::*;

/// [#4393] 에이전트 노동 감사 — 작업 캡슐(*.capsule.json) 폴더를 전수 재실행해
/// 재현율을 회계한다. 개별 영수증(replay)이 작업 하나의 증명이라면, audit 은
/// 조직 규모의 "에이전트가 한 일" 전체에 대한 회계감사다. 불일치 1건 = exit 3.
pub(crate) fn cmd_audit(args: &[String]) -> i32 {
    let mut dir: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            other if !other.starts_with("--") && dir.is_none() => dir = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let Some(dir) = dir else {
        eprintln!("사용법: rhwp audit <캡슐 폴더> [--json]  (대상: *.capsule.json)");
        return EXIT_USAGE;
    };
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("오류: 폴더를 읽을 수 없습니다 - {dir}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let capsules =
        match collect_audit_capsules(entries.map(|entry| entry.map(|entry| entry.path()))) {
            Ok(capsules) => capsules,
            Err(e) => {
                eprintln!("오류: {dir} 감사 대상을 전수 열거할 수 없습니다 - {e}");
                return EXIT_RUNTIME;
            }
        };
    if capsules.is_empty() {
        eprintln!("오류: {dir} 에 *.capsule.json 이 없습니다 — 감사 대상 없음.");
        return EXIT_USAGE;
    }
    let mut reproduced_count = 0usize;
    let mut failed: Vec<serde_json::Value> = Vec::new();
    for (idx, path) in capsules.iter().enumerate() {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        let fail = |reason: String| serde_json::json!({ "capsule": name, "error": reason });
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                failed.push(fail(format!("읽기 실패: {e}")));
                continue;
            }
        };
        let capsule: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                failed.push(fail(format!("JSON 파싱 실패: {e}")));
                continue;
            }
        };
        if capsule["kind"] != "workCapsule" {
            failed.push(fail("kind 가 workCapsule 이 아님".into()));
            continue;
        }
        let Some(expected) = capsule["receipt"]["outputSha256"]
            .as_str()
            .filter(|value| is_sha256_hex(value))
        else {
            failed.push(fail(
                "receipt.outputSha256 가 없거나 64자리 16진이 아님".into(),
            ));
            continue;
        };
        let Some(expected_input) = capsule["receipt"]["inputSha256"]
            .as_str()
            .filter(|value| is_sha256_hex(value))
        else {
            failed.push(fail(
                "receipt.inputSha256 가 없거나 64자리 16진이 아님".into(),
            ));
            continue;
        };
        let (mut plan, expected_steps) = match validated_capsule_plan(&capsule) {
            Ok(value) => value,
            Err(error) => {
                failed.push(fail(error));
                continue;
            }
        };
        match replay_execute_to_temp(&mut plan, &format!("audit{idx}")) {
            Ok((actual, actual_steps, actual_input)) => {
                if actual_input != expected_input {
                    failed.push(serde_json::json!({
                        "capsule": name,
                        "kind": "inputSha256",
                        "expected": expected_input,
                        "actual": actual_input,
                    }));
                } else if actual_steps as u64 != expected_steps {
                    failed.push(serde_json::json!({
                        "capsule": name,
                        "kind": "steps",
                        "expected": expected_steps,
                        "actual": actual_steps,
                    }));
                } else if actual == expected {
                    reproduced_count += 1;
                } else {
                    failed.push(serde_json::json!({
                        "capsule": name,
                        "expected": expected,
                        "actual": actual,
                    }));
                }
            }
            Err((msg, _code)) => failed.push(fail(msg)),
        }
    }
    let total = capsules.len();
    let rate = reproduced_count as f64 / total as f64;
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "root": dir,
            "total": total,
            "reproduced": reproduced_count,
            "failed": failed,
            "reproducedRate": rate,
        }),
        "audit",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!("에이전트 노동 감사 — {dir}");
        println!(
            "  캡슐 {total} · 재현 {reproduced_count} · 실패 {} · 재현율 {:.1}%",
            total - reproduced_count,
            rate * 100.0
        );
        for f in &failed {
            println!("  [FAIL] {}", f["capsule"].as_str().unwrap_or("?"));
        }
    }
    if failed.is_empty() {
        EXIT_OK
    } else {
        3 // #2707: 검증 단언 실패 — 재현되지 않은 작업이 있다.
    }
}
