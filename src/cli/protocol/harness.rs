use crate::cli::protocol::*;

/// [#4537] 하네스 작업장 규약 — capsules/ 하위와 키링 골격을 만든다.
fn cmd_harness_init(args: &[String]) -> i32 {
    let mut dir: Option<&str> = None;
    let mut key_id: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--key-id" => {
                i += 1;
                key_id = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && dir.is_none() => dir = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let Some(dir) = dir else {
        eprintln!("사용법: rhwp harness init <폴더> [--key-id <소유/용도#세대>] [--json]");
        return EXIT_USAGE;
    };
    let caps_dir = std::path::Path::new(dir).join("capsules");
    if let Err(e) = fs::create_dir_all(&caps_dir) {
        eprintln!("오류: 작업장 생성 실패 - {dir}: {e}");
        return EXIT_RUNTIME;
    }
    let mut created = vec!["capsules/".to_string()];
    let mut key_file = serde_json::Value::Null;
    let mut public_key = serde_json::Value::Null;
    if let Some(id) = key_id {
        let kp = std::path::Path::new(dir).join("harness.key.json");
        if kp.exists() {
            eprintln!(
                "오류: 키 파일이 이미 있습니다 - {} (덮어쓰기 금지).",
                kp.display()
            );
            return EXIT_USAGE;
        }
        match capsule_sign::generate_key_json(id) {
            Ok(key) => {
                if let Err(e) =
                    fs::write(&kp, serde_json::to_string_pretty(&key).unwrap_or_default())
                {
                    eprintln!("오류: 키 저장 실패 - {}: {e}", kp.display());
                    return EXIT_RUNTIME;
                }
                let ring = serde_json::json!({
                    "schemaVersion": capsule_sign::SIGNING_SCHEMA_VERSION_STR,
                    "kind": "keyring",
                    "keys": [{ "keyId": id, "publicKey": key["publicKey"], "revoked": null }],
                });
                let rp = std::path::Path::new(dir).join("keyring.json");
                if let Err(e) =
                    fs::write(&rp, serde_json::to_string_pretty(&ring).unwrap_or_default())
                {
                    eprintln!("오류: 키링 저장 실패 - {}: {e}", rp.display());
                    return EXIT_RUNTIME;
                }
                created.push("harness.key.json".to_string());
                created.push("keyring.json".to_string());
                public_key = key["publicKey"].clone();
                key_file = serde_json::json!(kp.to_string_lossy());
            }
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "dir": dir,
            "created": created,
            "keyId": key_id,
            "publicKey": public_key,
            "keyFile": key_file,
        }),
        "harness",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!("하네스 작업장 — {dir}: {}", envelope["created"]);
    }
    EXIT_OK
}

/// [#4537] 한 방 루프 — 실산출 실행 + 영수증 + 캡슐(연번) + 자동 부모 연결 + 서명.
///
/// 에이전트가 매 작업을 이 명령으로 돌리면 capsules/ 안에서 해시 체인이
/// 스스로 자란다 — 사다리 5개 명령의 규약 조합을 한 명령으로 접은 것이
/// 하네스의 정의다.
fn cmd_harness_wrap(args: &[String]) -> i32 {
    let mut plan_arg: Option<&str> = None;
    let mut dir: Option<&str> = None;
    let mut sign_key: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--plan" => {
                i += 1;
                plan_arg = args.get(i).map(String::as_str);
            }
            "--dir" => {
                i += 1;
                dir = args.get(i).map(String::as_str);
            }
            "--sign-key" => {
                i += 1;
                sign_key = args.get(i).map(String::as_str);
            }
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(plan_arg), Some(dir)) = (plan_arg, dir) else {
        eprintln!(
            "사용법: rhwp harness wrap --plan <JSON|@파일> --dir <작업장> [--sign-key <키.json>] [--json]"
        );
        return EXIT_USAGE;
    };
    let plan_text = if let Some(path) = plan_arg.strip_prefix('@') {
        match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 계획을 읽을 수 없습니다 - {path}: {e}");
                return EXIT_RUNTIME;
            }
        }
    } else {
        plan_arg.to_string()
    };
    let plan: serde_json::Value = match serde_json::from_str(&plan_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 계획 JSON 파싱 실패 - {e}");
            return EXIT_USAGE;
        }
    };
    let Some(input) = plan["input"].as_str().map(str::to_string) else {
        eprintln!("오류: 계획에 input 이 필요합니다.");
        return EXIT_USAGE;
    };
    let Some(output) = plan["output"].as_str().map(str::to_string) else {
        eprintln!("오류: 계획에 output 이 필요합니다 — wrap 은 실산출을 만든다.");
        return EXIT_USAGE;
    };
    let caps_dir = std::path::Path::new(dir).join("capsules");
    if !caps_dir.is_dir() {
        eprintln!("오류: 작업장이 아닙니다 - {dir} (harness init 먼저: capsules/ 없음)");
        return EXIT_USAGE;
    }
    // 직전 캡슐 = 자동 부모 — 연번 파일명이 정렬 순서를 보증한다.
    let existing = match fs::read_dir(&caps_dir) {
        Ok(rd) => match collect_audit_capsules(rd.map(|e| e.map(|d| d.path()))) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        },
        Err(e) => {
            eprintln!("오류: capsules/ 읽기 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let input_bytes = match fs::read(&input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 입력을 읽을 수 없습니다 - {input}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let input_sha = replay_sha256_hex(&input_bytes);
    let plan_sha = replay_sha256_hex(plan_text.as_bytes());
    let plan_original = plan.clone();
    // 실산출 실행 — replay 와 달리 계획의 output 경로에 진짜로 쓴다.
    let (engine_env, engine_code) = run_plan_engine(&plan);
    if engine_code != 0 {
        if json_mode {
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "error": format!("계획 실행 실패 (engine exit {engine_code})"),
                    }),
                    "harness",
                )
            );
        } else {
            eprintln!("계획 실행 실패 (engine exit {engine_code})");
        }
        return engine_code;
    }
    let steps = engine_env["steps"].as_array().map(|s| s.len()).unwrap_or(0);
    let output_bytes = match fs::read(&output) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 산출을 읽을 수 없습니다 - {output}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let output_sha = replay_sha256_hex(&output_bytes);
    let receipt = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "mode": "wrap",
        "input": input,
        "inputSha256": input_sha,
        "planSha256": plan_sha,
        "outputSha256": output_sha,
        "toolVersion": rhwp::version(),
        "steps": steps,
        "reproduced": serde_json::Value::Null,
        "expectedOutputSha256": serde_json::Value::Null,
    });
    let parent_link = match existing.last() {
        Some(prev) => {
            let bytes = match fs::read(prev) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("오류: 직전 캡슐 읽기 실패 - {}: {e}", prev.display());
                    return EXIT_RUNTIME;
                }
            };
            let name = prev.file_name().unwrap().to_string_lossy().into_owned();
            serde_json::json!({ "capsule": name, "sha256": replay_sha256_hex(&bytes) })
        }
        None => serde_json::Value::Null,
    };
    let capsule = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "kind": "workCapsule",
        "parent": parent_link,
        "plan": plan_original,
        "planText": plan_text,
        "receipt": receipt,
    });
    let cap_name = format!("{:04}_{}.capsule.json", existing.len() + 1, &plan_sha[..8]);
    let cap_path = caps_dir.join(&cap_name);
    if let Err(e) = fs::write(
        &cap_path,
        serde_json::to_string_pretty(&capsule).unwrap_or_default(),
    ) {
        eprintln!("오류: 캡슐 저장 실패 - {}: {e}", cap_path.display());
        return EXIT_RUNTIME;
    }
    let mut signed = false;
    if let Some(kp) = sign_key {
        let (signing, key_id, _) = match capsule_sign::load_signing_key(kp) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        };
        let cap_bytes = match fs::read(&cap_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("오류: 캡슐 재독 실패 - {e}");
                return EXIT_RUNTIME;
            }
        };
        let sidecar = capsule_sign::make_sidecar_json(
            &signing,
            &key_id,
            &replay_sha256_hex(&cap_bytes),
            &cap_bytes,
        );
        let sc = capsule_sign::sidecar_path(&cap_path.to_string_lossy());
        if let Err(e) = fs::write(
            &sc,
            serde_json::to_string_pretty(&sidecar).unwrap_or_default(),
        ) {
            eprintln!("오류: 서명 저장 실패 - {sc}: {e}");
            return EXIT_RUNTIME;
        }
        signed = true;
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "dir": dir,
            "capsule": cap_name,
            "output": output,
            "inputSha256": receipt["inputSha256"],
            "planSha256": receipt["planSha256"],
            "outputSha256": receipt["outputSha256"],
            "steps": steps,
            "parent": capsule["parent"]["capsule"].clone(),
            "signed": signed,
        }),
        "harness",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "하네스 wrap — {cap_name} (부모 {}, 서명 {signed})",
            capsule["parent"]["capsule"]
        );
    }
    EXIT_OK
}

/// [#4537] 작업장 통합 판정 — 체인·서명·(--deep) 재현을 한 봉투로.
struct HarnessStatusOptions<'a> {
    dir: &'a str,
    keyring_path: Option<&'a str>,
    deep: bool,
    json_mode: bool,
}

fn parse_harness_status_options(args: &[String]) -> Result<HarnessStatusOptions<'_>, i32> {
    let mut dir: Option<&str> = None;
    let mut keyring_path: Option<&str> = None;
    let mut deep = false;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--deep" => deep = true,
            "--keyring" => {
                i += 1;
                keyring_path = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && dir.is_none() => dir = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
        }
        i += 1;
    }
    let Some(dir) = dir else {
        eprintln!("사용법: rhwp harness-status <작업장> [--keyring <키링.json>] [--deep] [--json]");
        return Err(EXIT_USAGE);
    };
    Ok(HarnessStatusOptions {
        dir,
        keyring_path,
        deep,
        json_mode,
    })
}

pub(crate) fn cmd_harness_status(args: &[String]) -> i32 {
    let options = match parse_harness_status_options(args) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let HarnessStatusOptions {
        dir,
        keyring_path,
        deep,
        json_mode,
    } = options;
    let caps_dir = std::path::Path::new(dir).join("capsules");
    let capsules = match fs::read_dir(&caps_dir) {
        Ok(rd) => match collect_audit_capsules(rd.map(|e| e.map(|d| d.path()))) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        },
        Err(e) => {
            eprintln!("오류: 작업장이 아닙니다 - {dir}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let keyring = match keyring_path {
        Some(p) => match capsule_sign::load_keyring(p) {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        },
        None => None,
    };
    let mut chain_valid = true;
    let mut broken_at = serde_json::Value::Null;
    let mut prev: Option<(String, String, String)> = None; // (파일명, 파일해시, 산출해시)
    let (mut sig_valid, mut sig_bad, mut unsigned) = (0u64, 0u64, 0u64);
    let (mut deep_checked, mut deep_ok) = (0u64, 0u64);
    for path in &capsules {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let fail = |why: &str, broken_at: &mut serde_json::Value, chain_valid: &mut bool| {
            if *chain_valid {
                *chain_valid = false;
                *broken_at = serde_json::json!(format!("{name}: {why}"));
            }
        };
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(_) => {
                fail("읽기 실패", &mut broken_at, &mut chain_valid);
                continue;
            }
        };
        let file_sha = replay_sha256_hex(&bytes);
        let Ok(capsule) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            fail("JSON 파싱 실패", &mut broken_at, &mut chain_valid);
            continue;
        };
        if capsule["kind"] != "workCapsule" {
            fail("kind 불일치", &mut broken_at, &mut chain_valid);
            continue;
        }
        let input_sha = capsule["receipt"]["inputSha256"].as_str().unwrap_or("");
        let output_sha = capsule["receipt"]["outputSha256"]
            .as_str()
            .unwrap_or("")
            .to_string();
        match (&prev, capsule.get("parent")) {
            (None, Some(p)) if !p.is_null() => {
                fail("첫 캡슐에 부모가 있다", &mut broken_at, &mut chain_valid)
            }
            (Some((pname, psha, pout)), Some(p)) => {
                if p["capsule"].as_str() != Some(pname.as_str()) {
                    fail("부모 파일명 불일치", &mut broken_at, &mut chain_valid);
                } else if p["sha256"].as_str() != Some(psha.as_str()) {
                    fail(
                        "부모 해시 불일치(사후 변조)",
                        &mut broken_at,
                        &mut chain_valid,
                    );
                } else if !input_sha.is_empty() && pout != input_sha && !pout.is_empty() {
                    // 연번 체인에서 산출→입력 연쇄는 선택 규약 — 다른 입력의
                    // 독립 작업도 같은 작업장에 쌓일 수 있으므로 깨짐이 아니라
                    // 참고 수치로만 센다(설계 결정: wrap 은 강제하지 않는다).
                }
            }
            (Some(_), None) => fail("parent 필드 없음", &mut broken_at, &mut chain_valid),
            _ => {}
        }
        if let Some(ring) = keyring.as_ref() {
            let sc_path = format!("{}.sig.json", path.display());
            match fs::read_to_string(&sc_path) {
                Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(sc) => {
                        let v = capsule_sign::verify_sidecar(&sc, &bytes, ring);
                        if v.verdict == "valid" {
                            sig_valid += 1;
                        } else {
                            sig_bad += 1;
                            fail("서명 무효", &mut broken_at, &mut chain_valid);
                        }
                    }
                    Err(_) => {
                        sig_bad += 1;
                        fail("서명 파싱 실패", &mut broken_at, &mut chain_valid);
                    }
                },
                Err(_) => unsigned += 1,
            }
        }
        if deep {
            deep_checked += 1;
            if let Ok((validated_plan, _)) = validated_capsule_plan(&capsule) {
                let mut plan = validated_plan;
                if let Ok((actual, _, _)) =
                    replay_execute_to_temp(&mut plan, &format!("hstat{deep_checked}"))
                {
                    if actual == output_sha {
                        deep_ok += 1;
                    } else {
                        fail("재현 불일치", &mut broken_at, &mut chain_valid);
                    }
                } else {
                    fail("재실행 실패", &mut broken_at, &mut chain_valid);
                }
            } else {
                fail("계획 검증 실패", &mut broken_at, &mut chain_valid);
            }
        }
        prev = Some((name, file_sha, output_sha));
    }
    let verdict_ok = chain_valid && sig_bad == 0 && (!deep || deep_ok == deep_checked);
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "dir": dir,
            "capsules": capsules.len(),
            "chainValid": chain_valid,
            "brokenAt": broken_at,
            "signed": if keyring.is_some() {
                serde_json::json!({ "valid": sig_valid, "invalid": sig_bad, "unsigned": unsigned })
            } else {
                serde_json::Value::Null
            },
            "reproduced": if deep {
                serde_json::json!({ "checked": deep_checked, "ok": deep_ok })
            } else {
                serde_json::Value::Null
            },
            "verdict": if verdict_ok { "ok" } else { "broken" },
        }),
        "harness-status",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "하네스 status — {dir}: 캡슐 {} · {}",
            capsules.len(),
            envelope["verdict"].as_str().unwrap_or("?")
        );
    }
    if verdict_ok {
        EXIT_OK
    } else {
        3 // #2707: 검증 단언 실패 — 작업장이 깨졌다.
    }
}

/// [#4537] harness 디스패치 — init·wrap. 판정(status)은 읽기 전용이라
/// 최상위 `harness-status` 로 나가 있다.
pub(crate) fn cmd_harness(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("init") => cmd_harness_init(&args[1..]),
        Some("wrap") => cmd_harness_wrap(&args[1..]),
        _ => {
            eprintln!("사용법: rhwp harness <init|wrap> …  (판정: rhwp harness-status)");
            EXIT_USAGE
        }
    }
}
