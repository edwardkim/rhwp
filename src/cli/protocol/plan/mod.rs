use crate::cli::protocol::*;

mod condition;
mod execution;

pub(crate) use execution::run_plan_engine;

pub(crate) fn cmd_run_plan(args: &[String]) -> i32 {
    let mut plan_path: Option<&str> = None;
    let mut plan_inline: Option<&str> = None;
    let mut json_mode = false;
    // [#3721] 선검증만 돌리고 디스크는 건드리지 않는다 — 계획을 제출 전에 검사.
    let mut dry_run = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--dry-run" => dry_run = true,
            "--plan-json" => {
                i += 1;
                match args.get(i) {
                    Some(v) => plan_inline = Some(v.as_str()),
                    None => {
                        eprintln!("오류: --plan-json 뒤에 계획 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other if !other.starts_with("--") && plan_path.is_none() => plan_path = Some(other),
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let plan_text = match (plan_inline, plan_path) {
        (Some(inline), _) => inline.to_string(),
        (None, Some(path)) => match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 계획 파일을 읽을 수 없습니다 - {}: {}", path, e);
                return EXIT_RUNTIME;
            }
        },
        (None, None) => {
            eprintln!("사용법: rhwp run <계획.json> [--json] [--dry-run]  (파일 대신 --plan-json '<JSON>')");
            return EXIT_USAGE;
        }
    };
    let mut plan: serde_json::Value = match serde_json::from_str(&plan_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 계획 JSON 파싱 실패 - {}", e);
            return EXIT_USAGE;
        }
    };
    // 플래그는 계획서 필드를 덮어쓴다 — 의도의 단일 출처는 계획서이고, CLI 는 그 편의 입구다.
    // (계획서가 dryRun 을 실을 수 있으므로 MCP hwp_run_plan 은 인자 추가 없이 같은 계약을 얻는다.)
    if dry_run {
        if let Some(obj) = plan.as_object_mut() {
            obj.insert("dryRun".to_string(), serde_json::Value::Bool(true));
        }
    }
    let (journal, code) = run_plan_engine(&plan);
    if json_mode {
        println!("{}", journal);
    } else if code == EXIT_OK && journal["dryRun"] == true {
        let preview_all = journal["preview"].as_array().cloned().unwrap_or_default();
        // [#3719 §6-8] 건너뛸 step 은 "실행 가능"에 넣지 않는다 — dry-run 이 예고하는
        // 실행 개수와 run(실제 실행)이 보고할 적용 개수가 같은 말을 해야 한다.
        let skipped_count = preview_all.iter().filter(|s| s["skipped"] == true).count();
        println!(
            "검사 통과: {} step 실행 가능{} (디스크 무변경, 산출 예정 {})",
            preview_all.len() - skipped_count,
            if skipped_count == 0 {
                String::new()
            } else {
                format!(" · {} step 건너뜀 예정", skipped_count)
            },
            journal["output"].as_str().unwrap_or("-")
        );
        for step in &preview_all {
            println!("  - {}", preview_line(step));
        }
    } else if code == EXIT_OK {
        // [#3719 §6-8] 건너뛴 step 을 적용한 것과 같이 세면 "다 됐다"는 보고가 거짓이 된다.
        let skipped: Vec<&serde_json::Value> = journal["steps"]
            .as_array()
            .map(|steps| steps.iter().filter(|s| s["skipped"] == true).collect())
            .unwrap_or_default();
        let total = journal["steps"].as_array().map(|s| s.len()).unwrap_or(0);
        println!(
            "완료: {} step 적용{}, 산출 {}",
            total - skipped.len(),
            if skipped.is_empty() {
                String::new()
            } else {
                format!(" · {} step 건너뜀", skipped.len())
            },
            journal["output"].as_str().unwrap_or("-")
        );
        for step in &skipped {
            println!(
                "  - step {} 건너뜀: {}",
                step["step"].as_u64().unwrap_or(0),
                step["reason"].as_str().unwrap_or("")
            );
        }
        if let Some(steps) = journal["steps"].as_array() {
            for step in steps {
                if let Some(confusable) = step["confusable"].as_array() {
                    for item in confusable {
                        eprintln!(
                            "경고: '{}' 과(와) 화면상 구별되지 않는 이름의 누름틀이 문서에 함께 있습니다 — 채운 칸이 의도한 칸인지 확인하세요.",
                            item["name"].as_str().unwrap_or("")
                        );
                    }
                }
            }
        }
    } else {
        // 사람 모드에서도 판정 근거는 저널 그대로 남긴다 — 달리 설명할 출처가 없다.
        eprintln!("{}", journal);
    }
    code
}

/// [#3721] dry-run 미리보기 한 줄 — 사람 모드에서 "무엇이 얼마나 바뀌나"를 읽게 한다.
fn preview_line(step: &serde_json::Value) -> String {
    let idx = step["step"].as_u64().unwrap_or(0);
    // [#3719 §6-8] 건너뛸 step 은 다른 필드가 비어 있으므로 액션별 분기보다 먼저 본다.
    if step["skipped"] == true {
        return format!(
            "step {} 건너뜀 예정: {}",
            idx,
            step["reason"].as_str().unwrap_or("")
        );
    }
    match step["action"].as_str().unwrap_or("") {
        "fill_fields" => format!(
            "step {}: 누름틀 {}칸 채움",
            idx,
            step["targets"].as_array().map(|a| a.len()).unwrap_or(0)
        ),
        "replace_text" => format!(
            "step {}: '{}' {}건 중 {}건 치환",
            idx,
            step["find"].as_str().unwrap_or(""),
            step["matches"].as_u64().unwrap_or(0),
            step["willReplace"].as_u64().unwrap_or(0)
        ),
        "set_checkbox" => format!(
            "step {}: 빈 체크박스 {}개 중 {}번째 표시",
            idx,
            step["available"].as_u64().unwrap_or(0),
            step["occurrence"].as_u64().unwrap_or(0)
        ),
        "set_cell" => format!(
            "step {}: 표 {} ({},{}) 기록 — 현재값 {:?}",
            idx,
            step["table"].as_u64().unwrap_or(0),
            step["row"].as_u64().unwrap_or(0),
            step["col"].as_u64().unwrap_or(0),
            step["currentText"].as_str().unwrap_or("")
        ),
        other => format!("step {}: {}", idx, other),
    }
}
