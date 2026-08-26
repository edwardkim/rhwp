use crate::cli::protocol::*;

use super::condition::evaluate_step_condition;

fn validate_plan_steps(
    steps: &[serde_json::Value],
    doc: &rhwp::wasm_api::HwpDocument,
    name_counts: &std::collections::HashMap<String, usize>,
    name_values: &std::collections::HashMap<String, Vec<String>>,
) -> (
    Vec<serde_json::Value>,
    Vec<serde_json::Value>,
    Vec<Option<String>>,
) {
    let mut invalid: Vec<serde_json::Value> = Vec::new();
    // [#3721] 선검증이 이미 계산한 값을 미리보기로 모은다 — dry-run 은 이걸 그대로 낸다.
    // (실행 모드에서는 쓰이지 않지만, 판정자와 미리보기가 같은 계산이라 어긋날 수 없다.)
    let mut preview: Vec<serde_json::Value> = Vec::new();

    // [#3719 §6-8] 조건부 step — 조건은 **입력 문서 기준으로 실행 전에 한 번** 판정한다.
    // 실행 중에 다시 보면 선검증이 통과시킨 step 이 실행에서 조건을 잃는(또는 그 반대)
    // 상태가 생겨, "무엇이 왜 안 바뀌었는지"가 저널만 봐서는 재구성되지 않는다.
    // 판정 결과는 Some(사유) = 건너뜀, None = 실행.
    let mut skip_reasons: Vec<Option<String>> = Vec::with_capacity(steps.len());
    for step in steps.iter() {
        match step.get("if") {
            None => skip_reasons.push(None),
            Some(condition) => {
                match evaluate_step_condition(condition, &doc, &name_counts, &name_values) {
                    Ok(reason) => skip_reasons.push(reason),
                    Err(_) => {
                        // 문법 오류는 아래 선검증 루프에서 다시 판정해 invalid 에 담는다
                        // (사유 메시지를 한 곳에서만 만들기 위함) — 여기서는 자리만 채운다.
                        skip_reasons.push(None);
                    }
                }
            }
        }
    }

    for (idx, step) in steps.iter().enumerate() {
        let action = step["action"].as_str().unwrap_or("");
        // [#3719 §6-8] 조건 문법 오류는 계획 자체가 무효다 — invalid 로 즉시 보고한다.
        if let Some(condition) = step.get("if") {
            if let Err(message) =
                evaluate_step_condition(condition, &doc, &name_counts, &name_values)
            {
                invalid
                    .push(serde_json::json!({ "step": idx, "action": action, "reason": message }));
                continue;
            }
        }
        // 조건이 거짓인 step 은 **실행 가능성 검사를 면제**한다. 없는 필드를 채우는
        // step 이라도 애초에 실행되지 않으므로 위반이 아니다 — 여기서 걸러 내지 않으면
        // 조건절은 "쓸 수는 있으나 쓰면 계획이 통과하지 않는" 장식이 된다.
        if let Some(reason) = &skip_reasons[idx] {
            preview.push(serde_json::json!({
                "step": idx, "action": action, "skipped": true, "reason": reason,
            }));
            continue;
        }
        match action {
            "fill_fields" => {
                let Some(data) = step["data"].as_object() else {
                    invalid.push(serde_json::json!({ "step": idx, "action": action,
                        "reason": "data 는 {\"필드이름\":\"값\"} 객체여야 합니다" }));
                    continue;
                };
                let mut targets: Vec<serde_json::Value> = Vec::new();
                for (key, value) in data.iter() {
                    let (name, occurrence) = parse_field_key(key);
                    let total = name_counts.get(name).copied().unwrap_or(0);
                    if total == 0 || occurrence >= total {
                        invalid.push(serde_json::json!({ "step": idx, "action": action,
                            "reason": format!("필드 '{}' 이(가) 없거나 순번이 범위 밖입니다 (동명 {}개)", key, total) }));
                        continue;
                    }
                    targets.push(serde_json::json!({
                        "name": name, "occurrence": occurrence, "sameNameCount": total,
                        "value": value.as_str().map(|v| v.to_string())
                            .unwrap_or_else(|| value.to_string()),
                    }));
                }
                preview.push(
                    serde_json::json!({ "step": idx, "action": action, "targets": targets }),
                );
            }
            "replace_text" => {
                let Some(find) = step["find"].as_str().filter(|s| !s.is_empty()) else {
                    invalid.push(serde_json::json!({ "step": idx, "action": action,
                        "reason": "find (비어 있지 않은 문자열)가 필요합니다" }));
                    continue;
                };
                if !step["replace"].is_string() {
                    invalid.push(serde_json::json!({ "step": idx, "action": action,
                        "reason": "replace (문자열)가 필요합니다" }));
                    continue;
                }
                let case_sensitive = step["caseSensitive"].as_bool().unwrap_or(true);
                let count = doc.grep(find, case_sensitive, None).len();
                match step["occurrence"].as_u64() {
                    Some(n) if (n as usize) >= count => {
                        invalid.push(serde_json::json!({ "step": idx, "action": action,
                            "reason": format!("occurrence {} 이(가) 범위 밖입니다 ('{}' 일치 {}건)", n, find, count) }));
                    }
                    None if count == 0 => {
                        invalid.push(serde_json::json!({ "step": idx, "action": action,
                            "reason": format!("'{}' 일치 0건 — 치환할 곳이 없습니다", find) }));
                    }
                    // occurrence 지정이면 1건만, 아니면 전건 — 실행 분기와 같은 규칙.
                    occurrence => preview.push(serde_json::json!({
                        "step": idx, "action": action, "find": find,
                        "matches": count,
                        "willReplace": if occurrence.is_some() { 1 } else { count },
                    })),
                }
            }
            "set_checkbox" => {
                let Some(n) = step["occurrence"].as_u64() else {
                    invalid.push(serde_json::json!({ "step": idx, "action": action,
                        "reason": "occurrence (0 기준 순번)가 필요합니다" }));
                    continue;
                };
                let count = doc.grep("□", true, None).len();
                if (n as usize) >= count {
                    invalid.push(serde_json::json!({ "step": idx, "action": action,
                        "reason": format!("occurrence {} 이(가) 범위 밖입니다 (빈 체크박스 □ {}건)", n, count) }));
                } else {
                    preview.push(serde_json::json!({ "step": idx, "action": action,
                        "occurrence": n, "available": count }));
                }
            }
            "set_cell" => {
                let (Some(t), Some(r), Some(c), Some(text)) = (
                    step["table"].as_u64(),
                    step["row"].as_u64(),
                    step["col"].as_u64(),
                    step["text"].as_str(),
                ) else {
                    invalid.push(serde_json::json!({ "step": idx, "action": action,
                        "reason": "table·row·col (정수)과 text (문자열)가 필요합니다" }));
                    continue;
                };
                if text.chars().any(|ch| matches!(ch, '\r' | '\n' | '\t')) {
                    invalid.push(serde_json::json!({ "step": idx, "action": action,
                        "reason": "text 에 줄바꿈·탭은 넣을 수 없습니다 (한 줄 값 기록)" }));
                    continue;
                }
                let table = match usize::try_from(t) {
                    Ok(value) => value,
                    Err(_) => {
                        invalid.push(serde_json::json!({ "step": idx, "action": action,
                            "reason": format!("table {} 이(가) 이 플랫폼의 인덱스 범위를 벗어났습니다", t) }));
                        continue;
                    }
                };
                let row = match u16::try_from(r) {
                    Ok(value) => value,
                    Err(_) => {
                        invalid.push(serde_json::json!({ "step": idx, "action": action,
                            "reason": format!("row {} 이(가) 0..65535 범위를 벗어났습니다", r) }));
                        continue;
                    }
                };
                let col = match u16::try_from(c) {
                    Ok(value) => value,
                    Err(_) => {
                        invalid.push(serde_json::json!({ "step": idx, "action": action,
                            "reason": format!("col {} 이(가) 0..65535 범위를 벗어났습니다", c) }));
                        continue;
                    }
                };
                match resolve_table_cell(doc.document(), table, row, col) {
                    Err(e) => {
                        let (CellResolveError::Usage(msg) | CellResolveError::Runtime(msg)) = e;
                        invalid.push(
                            serde_json::json!({ "step": idx, "action": action, "reason": msg }),
                        );
                    }
                    Ok((.., current)) => preview.push(serde_json::json!({
                        "step": idx, "action": action,
                        "table": table, "row": row, "col": col,
                        "currentText": current, "newText": text,
                    })),
                }
            }
            "" => {
                invalid.push(serde_json::json!({ "step": idx, "reason": "action 이 필요합니다" }))
            }
            other => invalid.push(serde_json::json!({ "step": idx, "action": other,
                "reason": format!("알 수 없는 action: {} (fill_fields·replace_text·set_cell·set_checkbox)", other) })),
        }
    }
    (invalid, preview, skip_reasons)
}

fn execute_plan_steps(
    steps: &[serde_json::Value],
    doc: &mut rhwp::wasm_api::HwpDocument,
    name_counts: &std::collections::HashMap<String, usize>,
    name_locs: &std::collections::HashMap<String, Vec<(usize, usize)>>,
    skip_reasons: &[Option<String>],
) -> Result<(Vec<serde_json::Value>, Vec<(usize, usize)>), String> {
    // `edit fill-fields`·세션 경로와 같은 text-security 판정이다. 계획 실행만
    // 이 경고를 누락하면 선언적 경로가 화면상 같은 필드 이름을 침묵 속에 통과시킨다.
    let all_names: Vec<String> = name_counts.keys().cloned().collect();
    let confusable_groups = rhwp::document_core::text_security::confusable_collisions(&all_names);
    let mut journal_steps: Vec<serde_json::Value> = Vec::new();
    let mut changed_paras: Vec<(usize, usize)> = Vec::new();
    for (idx, step) in steps.iter().enumerate() {
        let action = step["action"].as_str().unwrap_or("");
        // [#3719 §6-8] 건너뛴 step 도 저널에 남긴다. 조용히 사라지면 소비자는 "왜 그
        // 칸이 안 바뀌었는지"를 알 방법이 없다 — 조건이 거짓이었다는 사실 자체가 결과다.
        if let Some(reason) = &skip_reasons[idx] {
            journal_steps.push(serde_json::json!({
                "step": idx, "action": action, "skipped": true, "reason": reason,
            }));
            continue;
        }
        match action {
            "fill_fields" => {
                let data = step["data"].as_object().expect("선검증 통과");
                let mut filled: Vec<serde_json::Value> = Vec::new();
                let mut ambiguous: Vec<serde_json::Value> = Vec::new();
                let mut confusable: Vec<serde_json::Value> = Vec::new();
                for (key, value) in data {
                    let value_str = match value {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    let (name, occurrence) = parse_field_key(key);
                    let total = name_counts.get(name).copied().unwrap_or(0);
                    if occurrence == 0 && total > 1 && !key.contains('[') {
                        ambiguous.push(
                            serde_json::json!({ "name": name, "matched": 1, "total": total }),
                        );
                    }
                    if let Some((_, group)) = confusable_groups
                        .iter()
                        .find(|(_, group)| group.iter().any(|candidate| candidate == name))
                    {
                        let others: Vec<&String> = group
                            .iter()
                            .filter(|candidate| *candidate != name)
                            .collect();
                        confusable.push(serde_json::json!({
                            "name": name,
                            "lookalikes": others,
                            "note": "화면상 구별되지 않는 이름의 누름틀이 이 문서에 함께 있습니다 — 채운 칸이 의도한 칸인지 확인하세요.",
                        }));
                    }
                    if let Err(e) = doc.set_field_value_by_name_at(name, occurrence, &value_str) {
                        return Err(format!("step {}: 필드 '{}' 설정 실패 - {}", idx, key, e));
                    }
                    if let Some(loc) = name_locs.get(name).and_then(|l| l.get(occurrence)) {
                        changed_paras.push(*loc);
                    }
                    filled.push(serde_json::json!({
                        "name": name, "occurrence": occurrence, "value": value_str,
                    }));
                }
                journal_steps.push(serde_json::json!({
                    "step": idx, "action": "fill_fields",
                    "filledCount": filled.len(), "filled": filled,
                    "notFound": [], "ambiguous": ambiguous, "confusable": confusable,
                }));
            }
            "replace_text" => {
                let find = step["find"].as_str().expect("선검증 통과");
                let replace = step["replace"].as_str().expect("선검증 통과");
                let case_sensitive = step["caseSensitive"].as_bool().unwrap_or(true);
                {
                    // [#3712] 치환 전 매치 주소 — 문자열 치환은 문단 인덱스를 밀지 않는다.
                    let all = doc.grep(find, case_sensitive, None);
                    match step["occurrence"].as_u64() {
                        Some(n) => {
                            if let Some(m) = all.get(n as usize) {
                                changed_paras.push((m.section, m.paragraph));
                            }
                        }
                        None => changed_paras.extend(all.iter().map(|m| (m.section, m.paragraph))),
                    }
                }
                let result = match step["occurrence"].as_u64() {
                    Some(n) => doc.replace_nth_native(find, replace, case_sensitive, n as usize),
                    None => doc.replace_all_native(find, replace, case_sensitive),
                };
                let count = match result {
                    Ok(r) => serde_json::from_str::<serde_json::Value>(&r)
                        .ok()
                        .and_then(|v| v["count"].as_u64())
                        .unwrap_or(0),
                    Err(e) => return Err(format!("step {}: 치환 실패 - {:?}", idx, e)),
                };
                journal_steps.push(serde_json::json!({
                    "step": idx, "action": "replace_text",
                    "find": find, "replacedCount": count,
                }));
            }
            "set_checkbox" => {
                let n = step["occurrence"].as_u64().expect("선검증 통과") as usize;
                if let Some(m) = doc.grep("□", true, None).get(n) {
                    changed_paras.push((m.section, m.paragraph));
                }
                let count = match doc.replace_nth_native("□", "☑", true, n) {
                    Ok(r) => serde_json::from_str::<serde_json::Value>(&r)
                        .ok()
                        .and_then(|v| v["count"].as_u64())
                        .unwrap_or(0),
                    Err(e) => return Err(format!("step {}: 체크박스 기록 실패 - {:?}", idx, e)),
                };
                journal_steps.push(serde_json::json!({
                    "step": idx, "action": "set_checkbox",
                    "occurrence": n, "replacedCount": count,
                }));
            }
            "set_cell" => {
                let t = usize::try_from(step["table"].as_u64().expect("선검증 통과"))
                    .expect("선검증 통과");
                let r =
                    u16::try_from(step["row"].as_u64().expect("선검증 통과")).expect("선검증 통과");
                let c =
                    u16::try_from(step["col"].as_u64().expect("선검증 통과")).expect("선검증 통과");
                let text = step["text"].as_str().expect("선검증 통과");
                let keep_style = step["keepStyle"].as_bool().unwrap_or(false);
                // 앞 step 의 편집으로 좌표가 밀릴 수 있어 실행 시점에 재해석한다.
                let (sec, para, ctrl, cell_idx, para_lens, old_text) =
                    match resolve_table_cell(doc.document(), t, r, c) {
                        Ok(v) => v,
                        Err(CellResolveError::Usage(m) | CellResolveError::Runtime(m)) => {
                            return Err(format!("step {}: {}", idx, m));
                        }
                    };
                for (pi, len) in para_lens.iter().enumerate() {
                    if *len == 0 {
                        continue;
                    }
                    if let Err(e) = doc.delete_text_in_cell(
                        sec as u32,
                        para as u32,
                        ctrl as u32,
                        cell_idx as u32,
                        pi as u32,
                        0,
                        *len as u32,
                    ) {
                        return Err(format!(
                            "step {}: 셀 비우기 실패(문단 {}) - {:?}",
                            idx, pi, e
                        ));
                    }
                }
                if !text.is_empty() {
                    if let Err(e) = doc.insert_text_in_cell(
                        sec as u32,
                        para as u32,
                        ctrl as u32,
                        cell_idx as u32,
                        0,
                        0,
                        text,
                    ) {
                        return Err(format!("step {}: 셀 쓰기 실패 - {:?}", idx, e));
                    }
                    if !keep_style
                        && !recolor_cell_text_black(doc.document_mut(), sec, para, ctrl, cell_idx)
                    {
                        eprintln!("경고: step {} 셀 글자색을 검정으로 바꾸지 못했습니다.", idx);
                    }
                }
                changed_paras.push((sec, para));
                journal_steps.push(serde_json::json!({
                    "step": idx, "action": "set_cell",
                    "table": t, "row": r, "col": c, "oldText": old_text,
                }));
            }
            _ => unreachable!("선검증이 막는다"),
        }
    }

    Ok((journal_steps, changed_paras))
}

/// 계획 실행 본체 — (저널, 종료 코드). CLI 와 MCP `hwp_run_plan` 이 같은 판정을 공유한다.
pub(crate) fn run_plan_engine(plan: &serde_json::Value) -> (serde_json::Value, i32) {
    fn usage(reason: &str) -> (serde_json::Value, i32) {
        (
            provenance::marked(
                serde_json::json!({ "schemaVersion": ENVELOPE_SCHEMA_VERSION, "error": reason }),
                "run",
            ),
            EXIT_USAGE,
        )
    }
    fn fail(reason: String) -> (serde_json::Value, i32) {
        (
            provenance::marked(
                serde_json::json!({ "schemaVersion": ENVELOPE_SCHEMA_VERSION, "error": reason }),
                "run",
            ),
            EXIT_RUNTIME,
        )
    }

    if plan["planVersion"].as_str() != Some("1.0") {
        return usage("planVersion \"1.0\" 이 필요합니다");
    }
    let Some(input) = plan["input"].as_str() else {
        return usage("input (원본 문서 경로)이 필요합니다");
    };
    let Some(output) = plan["output"].as_str() else {
        return usage("output (산출 경로)이 필요합니다");
    };
    let steps = match plan["steps"].as_array() {
        Some(s) if !s.is_empty() => s,
        _ => return usage("steps 는 비어 있지 않은 배열이어야 합니다"),
    };
    let assert_verify = plan["assertions"]["verify"].as_bool().unwrap_or(false);
    // notFoundEmpty 는 선검증이 구조적으로 보장한다 — 계약 표기로 저널에 남긴다.
    let assert_not_found_empty = plan["assertions"]["notFoundEmpty"]
        .as_bool()
        .unwrap_or(true);
    // [#4378 R22] preconditions.inputSha256 — 형식은 여기서(usage), 대조는 읽기 직후.
    // 키가 있는데 타입이 잘못된 경우를 "전제조건 없음"으로 낮추면 CAS 경계가
    // fail-open 된다. 생략만 허용하고, 명시된 값은 반드시 문자열이어야 한다.
    let expected_input_sha = match plan.get("preconditions") {
        None => None,
        Some(serde_json::Value::Object(preconditions)) => match preconditions.get("inputSha256") {
            None => {
                return usage("preconditions 객체에는 inputSha256 하나가 반드시 필요합니다");
            }
            Some(serde_json::Value::String(raw)) => {
                if preconditions.len() != 1 {
                    return usage("preconditions 에는 inputSha256 외 속성을 둘 수 없습니다");
                }
                let normalized = raw.trim().to_ascii_lowercase();
                if normalized.len() != 64 || !normalized.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return usage("preconditions.inputSha256 은 64자리 16진이어야 합니다");
                }
                Some(normalized)
            }
            Some(_) => {
                return usage("preconditions.inputSha256 은 문자열이어야 합니다");
            }
        },
        Some(_) => return usage("preconditions 는 객체여야 합니다"),
    };

    let _cas_lock = match expected_input_sha.as_ref() {
        Some(_) => {
            if let Err(e) = cas_test_synchronize_before_lock() {
                return fail(e);
            }
            match CasPathLock::acquire(Path::new(input)) {
                Ok(lock) => Some(lock),
                Err(e) => {
                    return fail(format!(
                        "입력 문서 CAS 잠금을 얻을 수 없습니다 - {input}: {e}"
                    ))
                }
            }
        }
        None => None,
    };
    let bytes = match fs::read(input) {
        Ok(d) => d,
        Err(e) => return fail(format!("입력을 읽을 수 없습니다 - {}: {}", input, e)),
    };
    // [#4378 R23] 입력 지문 — CAS 대조(있으면)와 성공 저널의 `inputSha256` 이 같은
    // 값을 공유한다. R22 가 세운 해시 함수(`sha256_hex_of`)를 그대로 재사용한다 —
    // 저널이 계획서와 다른 해시를 쓰면 사슬(R23)이 끊긴다.
    let input_sha256 = sha256_hex_of(&bytes);
    // [#4378 R22] CAS — 계획이 세워진 시점의 문서가 아니면 실행 0·저장 0 으로
    // 거절한다(#3905 M1: 두 exit 0 이 편집 하나를 지우는 경합의 차단기).
    //
    // 판정 코드는 **3**(#2707 "판정" 계열)이다 — 사용법 오류(2)가 아니다. 계획서는
    // 문법도 의미도 옳고 틀린 것은 세상 쪽이라, 이건 실패가 아니라 판정이다. 같은
    // 이유로 `invalid[]` 는 비워 둔다: "invalid 가 비어 있지 않으면 exit 2"(정적
    // 선검증 위반) 불변식을 CAS 가 흔들면 소비자의 분기표가 깨진다. 단발 경로
    // (`edit ... --expect-sha256`, R24)가 이미 내는 `preconditionFailed{kind,
    // expected, actual}` 와 **같은 모양·같은 코드**여서, CAS 판정은 진입점과
    // 무관하게 하나다.
    let precondition_failure = |expected: &str, actual: String| {
        // 재계획 힌트 — 같은 의도를 **새 지문으로** 다시 선검증하는 실행 가능한 호출.
        // `--dry-run` 이라 디스크를 건드리지 않는다: step 이 바뀐 문서에서도 성립하면
        // 통과하고(그때 `--dry-run` 만 빼고 다시 부르면 된다), 성립하지 않으면
        // `invalid[]` 로 "진짜 재계획이 필요하다"를 알려 준다. 기대 해시를 실제
        // 해시로 갈아 끼운 계획을 그대로 실어, 소비자가 계획을 재조립하지 않게 한다.
        let mut replan = plan.clone();
        if let Some(obj) = replan.as_object_mut() {
            // dryRun 은 아래 argv 의 `--dry-run` 이 싣는다 — 같은 뜻을 두 곳에 두면
            // 통과 후 재실행할 때 계획 본문에서 지우는 걸 잊는 함정이 된다.
            obj.remove("dryRun");
            obj.insert(
                "preconditions".to_string(),
                serde_json::json!({ "inputSha256": actual }),
            );
        }
        (
            provenance::marked(
                serde_json::json!({
                    "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                    "planVersion": "1.0",
                    "input": input,
                    "output": output,
                    // 정적 선검증은 통과했다 — 계획이 무효한 게 아니다.
                    "invalid": [],
                    "preconditionFailed": {
                        "kind": "inputSha256",
                        "expected": expected,
                        "actual": actual,
                    },
                    // `name` = 명령, `arguments` = 그 뒤에 그대로 붙일 argv 조각.
                    "nextCall": {
                        "name": "run",
                        "arguments": [
                            "--plan-json", replan.to_string(), "--dry-run", "--json",
                        ],
                        "why": "계획 수립 후 입력 문서가 바뀌었습니다. 같은 의도를 새 지문으로 다시 선검증하세요 — 통과하면 --dry-run 만 빼고 그대로 실행하고, invalid 가 나오면 문서를 다시 읽고 재계획하세요.",
                    },
                    "error": "입력 문서가 계획의 기대 해시와 다릅니다 — 계획 수립 후 문서가 바뀌었습니다. 실행 0·저장 0. nextCall 로 재계획하세요 (#3905 CAS).",
                }),
                "run",
            ),
            3, // #2707: 판정(verify 단언 실패와 같은 계열) — 사용법 오류가 아니다
        )
    };
    if let Some(expected) = expected_input_sha.as_deref() {
        if input_sha256 != expected {
            return precondition_failure(expected, input_sha256.clone());
        }
        cas_test_mark_checked_and_wait();
    }
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => return fail(format!("HWP 파싱 실패 - {}", e)),
    };

    // 1) 정적 선검증 — 실행 0. 위반을 전부 모아 한 번에 보고한다(하나 고치면 다음
    //    위반이 나오는 두더지잡기 방지). 판정자는 실행이 쓰는 바로 그 함수들이다.
    let mut name_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    // [#3712] 같은 순회에서 문단 주소도 담는다 — 저널 changedPages 산출 근거.
    let mut name_locs: std::collections::HashMap<String, Vec<(usize, usize)>> =
        std::collections::HashMap::new();
    // [#3719 §6-8] 조건절 fieldEquals 가 볼 **현재 값**. 같은 순회에서 담아 두면
    // 조건 판정이 문서를 다시 훑지 않는다(동명 필드는 선언 순서 = 순번 순서).
    let mut name_values: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for fi in doc.collect_all_fields().iter() {
        if let Some(n) = fi.field.field_name() {
            *name_counts.entry(n.to_string()).or_insert(0) += 1;
            name_locs
                .entry(n.to_string())
                .or_default()
                .push((fi.location.section_index, fi.location.para_index));
            name_values
                .entry(n.to_string())
                .or_default()
                .push(fi.value.clone());
        }
    }
    let (invalid, preview, skip_reasons) =
        validate_plan_steps(steps, &doc, &name_counts, &name_values);
    if !invalid.is_empty() {
        return (
            provenance::marked(
                serde_json::json!({
                    "schemaVersion": ENVELOPE_SCHEMA_VERSION, "planVersion": "1.0",
                    "input": input, "output": output, "invalid": invalid,
                }),
                "run",
            ),
            EXIT_USAGE,
        );
    }

    // [#3721] dry-run — 선검증만 하고 여기서 끝낸다. 실행도, 저장도 없다.
    // 계획을 *제출 전에* 검사하는 가장 싼 안전장치이고, 미리보기는 위에서 판정자가
    // 이미 계산한 값 그대로라 "검사 결과와 실제 실행이 다를" 여지가 없다.
    if plan["dryRun"].as_bool().unwrap_or(false) {
        return (
            serde_json::json!({
                "schemaVersion": ENVELOPE_SCHEMA_VERSION, "planVersion": "1.0", "dryRun": true,
                "input": input, "output": output,
                "preview": preview, "invalid": [],
                "assertions": { "notFoundEmpty": assert_not_found_empty, "verify": assert_verify },
            }),
            EXIT_OK,
        );
    }

    // 2) 원자 실행 — 전 step 을 인메모리 IR 에만 적용한다. 디스크는 아직 무변경이라
    //    어느 step 이 실패해도 반편집 문서가 남지 않는다.
    let (journal_steps, changed_paras) =
        match execute_plan_steps(steps, &mut doc, &name_counts, &name_locs, &skip_reasons) {
            Ok(result) => result,
            Err(error) => return fail(error),
        };
    // 3) 사후 단언 → 단 한 번 저장. 단언 실패 시 디스크 무변경 — 자연 트랜잭션.
    // [#3712] 눈검증 대상 페이지 — 편집 반영 후 조판 기준. 확정 불가면 null.
    let changed_pages = match doc.pages_covering_paragraphs(&changed_paras) {
        Some(pages) => serde_json::json!(pages),
        None => serde_json::Value::Null,
    };
    let out_format = edit_output_format(&bytes, Some(output));
    let out_bytes = match edit_serialize(&mut doc, out_format) {
        Ok(b) => b,
        Err(e) => return fail(format!("{} 직렬화 실패 - {}", out_format.label(), e)),
    };
    // [#4378 R23] 산출 지문 — 다음 계획의 `preconditions.inputSha256`(또는 다음
    // 저널의 `inputSha256`)과 대조하면 저널만으로 편집 사슬을 재구성할 수 있다.
    // 이 값은 실제로 디스크에 쓰는 바이트(`out_bytes`)의 해시다 — 재파싱 후
    // 해시를 다시 재는 것이 아니라 "무엇을 썼는가"를 직접 지문 찍는다.
    let output_sha256 = sha256_hex_of(&out_bytes);
    let mut verify_report = serde_json::Value::Null;
    if assert_verify {
        let cross = out_format == EditOutputFormat::Hwp
            && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
        let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
        verify_report = report;
        if failed {
            return (
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION, "planVersion": "1.0",
                        "input": input, "output": output,
                        "steps": journal_steps, "verify": verify_report,
                        "error": "verify 단언 실패 — 디스크 무변경",
                    }),
                    "run",
                ),
                3,
            );
        }
    }
    if let Some(expected) = expected_input_sha.as_deref() {
        let latest = match fs::read(input) {
            Ok(bytes) => bytes,
            Err(e) => {
                return fail(format!(
                    "저장 직전 입력을 다시 읽을 수 없습니다 - {input}: {e}"
                ))
            }
        };
        let actual = sha256_hex_of(&latest);
        if actual != expected {
            return precondition_failure(expected, actual);
        }
    }
    if let Err(e) = fs::write(output, &out_bytes) {
        return fail(format!("출력 파일을 쓸 수 없습니다 - {}: {}", output, e));
    }
    (
        provenance::marked(
            serde_json::json!({
                "schemaVersion": ENVELOPE_SCHEMA_VERSION, "planVersion": "1.0",
                "input": input, "output": output, "outputFormat": out_format.label(),
                "steps": journal_steps, "verify": verify_report,
                "changedPages": changed_pages,
                // [#4378 R23] 지문 체인 — 앞 실행의 outputSha256 = 뒤 실행의
                // inputSha256 이면 저널만으로 편집 사슬을 재구성할 수 있다.
                "inputSha256": input_sha256,
                "outputSha256": output_sha256,
                "assertions": { "notFoundEmpty": assert_not_found_empty, "verify": assert_verify },
            }),
            "run",
        ),
        EXIT_OK,
    )
}
