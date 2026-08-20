use crate::cli::protocol::*;

/// [#4551] 가림 발급 — plan 문자열 잎 전부를 salt 커밋으로 치환한다.
fn cmd_disclose_redact(args: &[String]) -> i32 {
    let mut capsule: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut opening_out: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "-o" => {
                i += 1;
                out = args.get(i).map(String::as_str);
            }
            "--opening-out" => {
                i += 1;
                opening_out = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && capsule.is_none() => capsule = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(capsule), Some(out), Some(opening_out)) = (capsule, out, opening_out) else {
        eprintln!("사용법: rhwp disclose redact <캡슐.json> -o <가림.json> --opening-out <opening.json> [--json]");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(capsule) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 캡슐을 읽을 수 없습니다 - {capsule}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let original_sha = replay_sha256_hex(&bytes);
    let mut cap: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 캡슐 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    if cap["kind"] != "workCapsule" {
        eprintln!("오류: kind 가 workCapsule 이 아닙니다.");
        return EXIT_USAGE;
    }
    let plan_text = cap["planText"].as_str().unwrap_or_default().to_string();
    let mut plan = cap["plan"].clone();
    let mut openings: Vec<(String, String, String)> = Vec::new();
    if let Err(e) = disclose::redact_plan(&mut plan, "", "", &mut openings) {
        eprintln!("오류: {e}");
        return EXIT_RUNTIME;
    }
    cap["plan"] = plan;
    // planText 원문은 개봉 파일로 이사한다 — 가림본에 남기면 전부 샌다.
    cap["planText"] = serde_json::json!("(redacted — 개봉 파일 보유자만 복원 가능)");
    cap["planRedacted"] = serde_json::json!(true);
    cap["originalCapsuleSha256"] = serde_json::json!(original_sha);
    if let Err(e) = fs::write(out, serde_json::to_string_pretty(&cap).unwrap_or_default()) {
        eprintln!("오류: 가림 캡슐 저장 실패 - {out}: {e}");
        return EXIT_RUNTIME;
    }
    let opening_map: serde_json::Map<String, serde_json::Value> = openings
        .iter()
        .map(|(p, v, salt)| (p.clone(), serde_json::json!({ "value": v, "salt": salt })))
        .collect();
    let opening = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "kind": disclose::OPENING_KIND,
        "originalCapsuleSha256": original_sha,
        "planText": plan_text,
        "openings": opening_map,
    });
    if let Err(e) = fs::write(
        opening_out,
        serde_json::to_string_pretty(&opening).unwrap_or_default(),
    ) {
        eprintln!("오류: 개봉 파일 저장 실패 - {opening_out}: {e}");
        return EXIT_RUNTIME;
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "capsule": capsule,
            "redacted": out,
            "opening": opening_out,
            "committedFields": openings.len(),
            "originalCapsuleSha256": original_sha,
        }),
        "disclose",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "가림 발급 — {out}: 커밋 {}개 (개봉은 비밀 보관: {opening_out})",
            openings.len()
        );
    }
    EXIT_OK
}

/// [#4551] 부분 개봉 검증 — 필드 단위 커밋 대조.
fn cmd_disclose_verify(args: &[String]) -> i32 {
    let mut redacted: Option<&str> = None;
    let mut opening_path: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--opening" => {
                i += 1;
                opening_path = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && redacted.is_none() => redacted = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(redacted), Some(opening_path)) = (redacted, opening_path) else {
        eprintln!("사용법: rhwp disclose verify <가림.json> --opening <opening.json> [--json]");
        return EXIT_USAGE;
    };
    let cap: serde_json::Value = match fs::read(redacted)
        .map_err(|e| e.to_string())
        .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 가림 캡슐을 읽을 수 없습니다 - {redacted}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let opening: serde_json::Value = match fs::read(opening_path)
        .map_err(|e| e.to_string())
        .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 개봉 파일을 읽을 수 없습니다 - {opening_path}: {e}");
            return EXIT_RUNTIME;
        }
    };
    if opening["kind"] != disclose::OPENING_KIND {
        eprintln!("오류: 개봉 kind 가 {} 가 아닙니다.", disclose::OPENING_KIND);
        return EXIT_USAGE;
    }
    let plan = &cap["plan"];
    let mut verified: Vec<String> = Vec::new();
    let mut mismatched: Vec<String> = Vec::new();
    if let Some(map) = opening["openings"].as_object() {
        for (pointer, entry) in map {
            let (Some(value), Some(salt)) = (entry["value"].as_str(), entry["salt"].as_str())
            else {
                mismatched.push(format!("{pointer} (개봉 형식 오류)"));
                continue;
            };
            match disclose::committed_at(plan, pointer) {
                Some(committed) if disclose::commit(value, salt) == committed => {
                    verified.push(pointer.clone())
                }
                Some(_) => mismatched.push(pointer.clone()),
                None => mismatched.push(format!("{pointer} (커밋 잎 없음)")),
            }
        }
    }
    let total = disclose::committed_count(plan);
    let unopened = total.saturating_sub(verified.len() + mismatched.len());
    let ok = mismatched.is_empty();
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "redacted": redacted,
            "verifiedFields": verified,
            "mismatched": mismatched,
            "unopened": unopened,
            "verdict": if ok { "ok" } else { "mismatch" },
        }),
        "disclose",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "부분 개봉 — 검증 {} · 불일치 {} · 미개봉 {unopened}",
            verified.len(),
            mismatched.len()
        );
    }
    if ok {
        EXIT_OK
    } else {
        3 // #2707: 개봉이 커밋과 다르다 — 위조 또는 값 변경.
    }
}

/// [#4551] 전체 복원 — 바이트 단위 원본 재현 (원본 서명이 그대로 valid).
fn cmd_disclose_restore(args: &[String]) -> i32 {
    let mut redacted: Option<&str> = None;
    let mut opening_path: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--opening" => {
                i += 1;
                opening_path = args.get(i).map(String::as_str);
            }
            "-o" => {
                i += 1;
                out = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && redacted.is_none() => redacted = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(redacted), Some(opening_path), Some(out)) = (redacted, opening_path, out) else {
        eprintln!("사용법: rhwp disclose restore <가림.json> --opening <전체개봉.json> -o <복원.json> [--json]");
        return EXIT_USAGE;
    };
    let mut cap: serde_json::Value = match fs::read(redacted)
        .map_err(|e| e.to_string())
        .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 가림 캡슐을 읽을 수 없습니다 - {redacted}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let opening: serde_json::Value = match fs::read(opening_path)
        .map_err(|e| e.to_string())
        .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 개봉 파일을 읽을 수 없습니다 - {opening_path}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let expected_sha = cap["originalCapsuleSha256"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let Some(plan_text) = opening["planText"].as_str() else {
        eprintln!("오류: 전체 개봉에 planText 가 필요합니다 (부분 개봉으로는 복원 불가).");
        return EXIT_USAGE;
    };
    // 전체 커버리지 검사 — 커밋 잎마다 개봉이 있어야 한다.
    let total = disclose::committed_count(&cap["plan"]);
    let provided = opening["openings"]
        .as_object()
        .map(|m| m.len())
        .unwrap_or(0);
    if provided < total {
        eprintln!("오류: 개봉 {provided}/{total} — 전체 개봉이 아니면 복원할 수 없습니다.");
        return 3;
    }
    let plan: serde_json::Value = match serde_json::from_str(plan_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 개봉 planText 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    cap["plan"] = plan;
    cap["planText"] = serde_json::json!(plan_text);
    if let Some(map) = cap.as_object_mut() {
        map.remove("planRedacted");
        map.remove("originalCapsuleSha256");
    }
    let restored = serde_json::to_string_pretty(&cap).unwrap_or_default();
    if let Err(e) = fs::write(out, &restored) {
        eprintln!("오류: 복원 저장 실패 - {out}: {e}");
        return EXIT_RUNTIME;
    }
    let restored_sha = replay_sha256_hex(restored.as_bytes());
    let byte_identical = !expected_sha.is_empty() && restored_sha == expected_sha;
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "redacted": redacted,
            "restored": out,
            "restoredSha256": restored_sha,
            "originalCapsuleSha256": expected_sha,
            "byteIdentical": byte_identical,
        }),
        "disclose",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!("복원 — {out}: 바이트 동일 {byte_identical}");
    }
    if byte_identical {
        EXIT_OK
    } else {
        3 // #2707: 복원이 원본 바이트를 재현하지 못했다 — 개봉이 원본과 다르다.
    }
}

/// [#4551] disclose 디스패치 — redact·verify·restore.
pub(crate) fn cmd_disclose(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("redact") => cmd_disclose_redact(&args[1..]),
        Some("verify") => cmd_disclose_verify(&args[1..]),
        Some("restore") => cmd_disclose_restore(&args[1..]),
        _ => {
            eprintln!("사용법: rhwp disclose <redact|verify|restore> …");
            EXIT_USAGE
        }
    }
}
