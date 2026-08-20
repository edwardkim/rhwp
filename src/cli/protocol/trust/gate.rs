use crate::cli::protocol::*;

/// [#4545] 정책 게이트 — 반입 판정의 기계화. 판정 재료는 자기 신고가
/// 아니라 재계산이며, 규칙이 참조하는 판정만 지연 계산한다(비용 회계).
struct GateOptions<'a> {
    target: &'a str,
    policy_path: &'a str,
    keyring_path: Option<&'a str>,
    anchor_log_path: Option<&'a str>,
    policy_keyring: Option<&'a str>,
    deep: bool,
    json_mode: bool,
}

fn parse_gate_options(args: &[String]) -> Result<GateOptions<'_>, i32> {
    let mut target: Option<&str> = None;
    let mut policy_path: Option<&str> = None;
    let mut keyring_path: Option<&str> = None;
    let mut anchor_log_path: Option<&str> = None;
    let mut policy_keyring: Option<&str> = None;
    let mut deep = false;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--deep" => deep = true,
            "--policy" => {
                i += 1;
                policy_path = args.get(i).map(String::as_str);
            }
            "--keyring" => {
                i += 1;
                keyring_path = args.get(i).map(String::as_str);
            }
            "--anchor-log" => {
                i += 1;
                anchor_log_path = args.get(i).map(String::as_str);
            }
            "--policy-keyring" => {
                i += 1;
                policy_keyring = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && target.is_none() => target = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
        }
        i += 1;
    }
    let (Some(target), Some(policy_path)) = (target, policy_path) else {
        eprintln!("사용법: rhwp gate <캡슐.json> --policy <policy.json> [--keyring <키링>] [--anchor-log <로그>] [--policy-keyring <키링>] [--deep] [--json]");
        return Err(EXIT_USAGE);
    };
    Ok(GateOptions {
        target,
        policy_path,
        keyring_path,
        anchor_log_path,
        policy_keyring,
        deep,
        json_mode,
    })
}

fn policy_signature_verdict(
    policy_keyring: Option<&str>,
    policy_path: &str,
    policy_text: &str,
) -> Result<serde_json::Value, i32> {
    let Some(keyring_path) = policy_keyring else {
        return Ok(serde_json::Value::Null);
    };
    let ring = capsule_sign::load_keyring(keyring_path).map_err(|e| {
        eprintln!("오류: {e}");
        EXIT_RUNTIME
    })?;
    let sidecar = fs::read_to_string(capsule_sign::sidecar_path(policy_path))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
    Ok(match sidecar {
        Some(sidecar) => {
            let verdict = capsule_sign::verify_sidecar(&sidecar, policy_text.as_bytes(), &ring);
            serde_json::json!(verdict.verdict == "valid")
        }
        None => serde_json::json!(false),
    })
}

fn load_gate_target(target: &str) -> Result<(Vec<u8>, serde_json::Value), i32> {
    let bytes = fs::read(target).map_err(|e| {
        eprintln!("오류: 대상을 읽을 수 없습니다 - {target}: {e}");
        EXIT_RUNTIME
    })?;
    let capsule = serde_json::from_slice(&bytes).map_err(|e| {
        eprintln!("오류: 캡슐 파싱 실패 - {target}: {e}");
        EXIT_RUNTIME
    })?;
    Ok((bytes, capsule))
}

fn load_gate_policy(policy_path: &str) -> Result<(String, policy_gate::Policy), i32> {
    let text = fs::read_to_string(policy_path).map_err(|e| {
        eprintln!("오류: 정책을 읽을 수 없습니다 - {policy_path}: {e}");
        EXIT_RUNTIME
    })?;
    let policy = policy_gate::parse(&text).map_err(|e| {
        eprintln!("오류(정책): {e}");
        EXIT_USAGE
    })?;
    Ok((text, policy))
}

pub(crate) fn cmd_gate(args: &[String]) -> i32 {
    let options = match parse_gate_options(args) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let GateOptions {
        target,
        policy_path,
        keyring_path,
        anchor_log_path,
        policy_keyring,
        deep,
        json_mode,
    } = options;
    let (policy_text, policy) = match load_gate_policy(policy_path) {
        Ok(policy) => policy,
        Err(code) => return code,
    };
    // 정책 자체의 서명 (M3, 4년 축 재사용) — 보고 필드.
    let policy_signed = match policy_signature_verdict(policy_keyring, policy_path, &policy_text) {
        Ok(verdict) => verdict,
        Err(code) => return code,
    };
    let (target_bytes, capsule) = match load_gate_target(target) {
        Ok(target) => target,
        Err(code) => return code,
    };
    let target_sha = replay_sha256_hex(&target_bytes);
    let needed = policy_gate::referenced_keys(&policy);
    let mut judgments: std::collections::BTreeMap<String, Option<serde_json::Value>> =
        std::collections::BTreeMap::new();
    // ── 계보 재계산 (lineageValid·lineageDepth) — 머리부터 뿌리까지 걷는다.
    if needed.contains("lineageValid") || needed.contains("lineageDepth") {
        let mut ok = true;
        let mut depth = 0u64;
        let mut current = std::path::PathBuf::from(target);
        let mut recorded: Option<String> = None;
        let mut child_input: Option<String> = None;
        for _ in 0..1000 {
            let Ok(bytes) = fs::read(&current) else {
                ok = false;
                break;
            };
            let file_sha = replay_sha256_hex(&bytes);
            let Ok(cap) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
                ok = false;
                break;
            };
            if cap["kind"] != "workCapsule" {
                ok = false;
                break;
            }
            if let Some(r) = recorded.as_deref() {
                if r != file_sha {
                    ok = false;
                    break;
                }
            }
            let out_sha = cap["receipt"]["outputSha256"].as_str().unwrap_or("");
            if let Some(ci) = child_input.as_deref() {
                if !out_sha.is_empty() && out_sha != ci {
                    ok = false;
                    break;
                }
            }
            depth += 1;
            let parent = &cap["parent"];
            if parent.is_null() {
                break;
            }
            let (Some(pp), Some(psha)) = (parent["capsule"].as_str(), parent["sha256"].as_str())
            else {
                ok = false;
                break;
            };
            recorded = Some(psha.to_string());
            child_input = cap["receipt"]["inputSha256"].as_str().map(str::to_string);
            let pp_path = std::path::PathBuf::from(pp);
            current = if pp_path.is_absolute() {
                pp_path
            } else {
                current
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join(pp_path)
            };
        }
        judgments.insert("lineageValid".into(), Some(serde_json::json!(ok)));
        judgments.insert("lineageDepth".into(), Some(serde_json::json!(depth)));
    }
    // ── 서명 재계산 (signerVerdict·signerKeyId).
    if needed.contains("signerVerdict") || needed.contains("signerKeyId") {
        match keyring_path {
            Some(kr) => match capsule_sign::load_keyring(kr) {
                Ok(ring) => {
                    let sc_path = capsule_sign::sidecar_path(target);
                    match fs::read_to_string(&sc_path)
                        .ok()
                        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                    {
                        Some(sc) => {
                            let v = capsule_sign::verify_sidecar(&sc, &target_bytes, &ring);
                            judgments
                                .insert("signerVerdict".into(), Some(serde_json::json!(v.verdict)));
                            judgments
                                .insert("signerKeyId".into(), Some(serde_json::json!(v.key_id)));
                        }
                        None => {
                            judgments.insert(
                                "signerVerdict".into(),
                                Some(serde_json::json!("unsigned")),
                            );
                            judgments.insert("signerKeyId".into(), Some(serde_json::Value::Null));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("오류: {e}");
                    return EXIT_RUNTIME;
                }
            },
            None => {
                judgments.insert("signerVerdict".into(), None);
                judgments.insert("signerKeyId".into(), None);
            }
        }
    }
    // ── 앵커 재계산 (anchoredOk).
    if needed.contains("anchoredOk") {
        match anchor_log_path {
            Some(path) => match anchor_log::load(path) {
                Ok(log) => {
                    let hit = log
                        .entries
                        .iter()
                        .any(|e| e["capsuleSha256"].as_str() == Some(target_sha.as_str()));
                    judgments.insert("anchoredOk".into(), Some(serde_json::json!(hit)));
                }
                Err(e) => {
                    eprintln!("오류(로그 무결): {e}");
                    return 3;
                }
            },
            None => {
                judgments.insert("anchoredOk".into(), None);
            }
        }
    }
    // ── 재현 재계산 (reproduced) — deep 요구.
    if needed.contains("reproduced") {
        if deep {
            let value = match validated_capsule_plan(&capsule) {
                Ok((validated_plan, _)) => {
                    let mut plan = validated_plan;
                    match replay_execute_to_temp(&mut plan, "gate") {
                        Ok((actual, _, _)) => Some(serde_json::json!(
                            capsule["receipt"]["outputSha256"].as_str() == Some(actual.as_str())
                        )),
                        Err(_) => Some(serde_json::json!(false)),
                    }
                }
                Err(_) => Some(serde_json::json!(false)),
            };
            judgments.insert("reproduced".into(), value);
        } else {
            // 재현 판정은 재실행 없이는 말할 수 없다 — 신고를 읽지 않는다.
            judgments.insert("reproduced".into(), None);
        }
    }
    let (allow, violations) = policy_gate::evaluate(&policy, &judgments);
    let evaluated: usize = policy.rules.iter().map(|r| r.require.len()).sum();
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "policy": policy.name,
            "policyPath": policy_path,
            "policySigned": policy_signed,
            "target": target,
            "targetSha256": target_sha,
            "verdict": if allow { "allow" } else { "deny" },
            "evaluated": evaluated,
            "violations": violations,
        }),
        "gate",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "게이트 — {target}: {} (평가 {evaluated}건)",
            envelope["verdict"].as_str().unwrap_or("?")
        );
    }
    if allow {
        EXIT_OK
    } else {
        3 // #2707: 반입 거부는 오류가 아니라 판정 데이터다.
    }
}
