use crate::cli::protocol::*;

/// [#4553] 청구 발급 — 명세서·캡슐·게이트 봉투를 3해시로 고정한다.
fn cmd_settle_propose(args: &[String]) -> i32 {
    let mut workorder: Option<&str> = None;
    let mut capsule: Option<&str> = None;
    let mut gate_env: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut sign_key: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--workorder" => {
                i += 1;
                workorder = args.get(i).map(String::as_str);
            }
            "--capsule" => {
                i += 1;
                capsule = args.get(i).map(String::as_str);
            }
            "--gate-envelope" => {
                i += 1;
                gate_env = args.get(i).map(String::as_str);
            }
            "-o" => {
                i += 1;
                out = args.get(i).map(String::as_str);
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
    let (Some(workorder), Some(capsule), Some(gate_env), Some(out)) =
        (workorder, capsule, gate_env, out)
    else {
        eprintln!("사용법: rhwp settle propose --workorder <wo.json> --capsule <c.json> --gate-envelope <g.json> -o <claim.json> [--sign-key <키>] [--json]");
        return EXIT_USAGE;
    };
    let read = |p: &str, what: &str| -> Result<Vec<u8>, i32> {
        fs::read(p).map_err(|e| {
            eprintln!("오류: {what}을(를) 읽을 수 없습니다 - {p}: {e}");
            EXIT_RUNTIME
        })
    };
    let wo_bytes = match read(workorder, "명세서") {
        Ok(b) => b,
        Err(c) => return c,
    };
    let cap_bytes = match read(capsule, "캡슐") {
        Ok(b) => b,
        Err(c) => return c,
    };
    let gate_bytes = match read(gate_env, "게이트 봉투") {
        Ok(b) => b,
        Err(c) => return c,
    };
    // 검수 기준 없는 명세서는 발급 단계에서 거부 — 분쟁을 산문으로 되돌리지 않는다.
    let wo = match settle::parse_workorder(&String::from_utf8_lossy(&wo_bytes)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_USAGE;
        }
    };
    let wo_sha = settle::sha256_hex(&wo_bytes);
    let cap_sha = settle::sha256_hex(&cap_bytes);
    let gate_sha = settle::sha256_hex(&gate_bytes);
    let signer = match sign_key {
        Some(k) => match capsule_sign::load_signing_key(k) {
            Ok((signing, key_id, _)) => Some((signing, key_id)),
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        },
        None => None,
    };
    let mut claim = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "kind": settle::CLAIM_KIND,
        "workorderId": wo["workorderId"],
        "workorderSha256": wo_sha,
        "capsuleSha256": cap_sha,
        "gateEnvelopeSha256": gate_sha,
        // 주장 필드 — 시점 증명은 원장 체크포인트 공표의 몫(5년 축 동형).
        "claimedAt": capsule_sign::rfc3339_utc_now(),
    });
    if let Some((_, key_id)) = &signer {
        claim["claimant"] = serde_json::json!({ "keyId": key_id });
    }
    let claim_text = serde_json::to_string_pretty(&claim).unwrap_or_default();
    if let Err(e) = fs::write(out, &claim_text) {
        eprintln!("오류: 청구 저장 실패 - {out}: {e}");
        return EXIT_RUNTIME;
    }
    if let Some((signing, key_id)) = &signer {
        let claim_sha = settle::sha256_hex(claim_text.as_bytes());
        let sidecar =
            capsule_sign::make_sidecar_json(signing, key_id, &claim_sha, claim_text.as_bytes());
        let sidecar_out = capsule_sign::sidecar_path(out);
        if let Err(e) = fs::write(
            &sidecar_out,
            serde_json::to_string_pretty(&sidecar).unwrap_or_default(),
        ) {
            eprintln!("오류: 청구 서명 저장 실패 - {sidecar_out}: {e}");
            return EXIT_RUNTIME;
        }
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "claim": out,
            "workorderSha256": wo_sha,
            "capsuleSha256": cap_sha,
            "gateEnvelopeSha256": gate_sha,
            "signed": signer.is_some(),
        }),
        "settle",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!("청구 발급 — {out}: 3해시 고정 (서명 {})", signer.is_some());
    }
    EXIT_OK
}

/// [#4553] 청구 검증 — 3해시 대조 + 서명·이중 청구 opt-in 축.
fn cmd_settle_verify(args: &[String]) -> i32 {
    let mut claim_path: Option<&str> = None;
    let mut workorder: Option<&str> = None;
    let mut capsule: Option<&str> = None;
    let mut gate_env: Option<&str> = None;
    let mut keyring_path: Option<&str> = None;
    let mut ledger_path: Option<&str> = None;
    let mut sig_path: Option<String> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--workorder" => {
                i += 1;
                workorder = args.get(i).map(String::as_str);
            }
            "--capsule" => {
                i += 1;
                capsule = args.get(i).map(String::as_str);
            }
            "--gate-envelope" => {
                i += 1;
                gate_env = args.get(i).map(String::as_str);
            }
            "--keyring" => {
                i += 1;
                keyring_path = args.get(i).map(String::as_str);
            }
            "--ledger" => {
                i += 1;
                ledger_path = args.get(i).map(String::as_str);
            }
            "--sig" => {
                i += 1;
                sig_path = args.get(i).map(String::from);
            }
            other if !other.starts_with("--") && claim_path.is_none() => claim_path = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(claim_path), Some(workorder), Some(capsule), Some(gate_env)) =
        (claim_path, workorder, capsule, gate_env)
    else {
        eprintln!("사용법: rhwp settle verify <claim.json> --workorder <wo> --capsule <c> --gate-envelope <g> [--keyring <k>] [--ledger <l>] [--sig <서명>] [--json]");
        return EXIT_USAGE;
    };
    let claim_bytes = match fs::read(claim_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 청구를 읽을 수 없습니다 - {claim_path}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let claim: serde_json::Value = match serde_json::from_slice(&claim_bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 청구 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    if claim["kind"] != settle::CLAIM_KIND {
        eprintln!("오류: kind 가 {} 가 아닙니다.", settle::CLAIM_KIND);
        return EXIT_USAGE;
    }
    let sha_of = |p: &str| fs::read(p).map(|b| settle::sha256_hex(&b));
    let check = |p: &str, pinned: &serde_json::Value| -> bool {
        matches!((sha_of(p), pinned.as_str()), (Ok(actual), Some(exp)) if actual == exp)
    };
    let workorder_ok = check(workorder, &claim["workorderSha256"]);
    let capsule_ok = check(capsule, &claim["capsuleSha256"]);
    let gate_ok = check(gate_env, &claim["gateEnvelopeSha256"]);
    // 게이트 봉투의 verdict 재확인 — 해시가 맞아도 판정이 allow 가 아니면 검수 미통과다.
    let gate_verdict: serde_json::Value = fs::read(gate_env)
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .map(|v| v["verdict"].clone())
        .unwrap_or(serde_json::Value::Null);
    let mut envelope = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "claim": claim_path,
        "workorderOk": workorder_ok,
        "capsuleOk": capsule_ok,
        "gateOk": gate_ok,
        "gateVerdict": gate_verdict,
    });
    let mut ok = workorder_ok && capsule_ok && gate_ok && gate_verdict == "allow";
    if let Some(kr_path) = keyring_path {
        let keyring = match capsule_sign::load_keyring(kr_path) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        };
        // 청구 서명 — 사이드카 부재는 false (청구 귀속은 이 축의 본질).
        let sidecar_file = sig_path.unwrap_or_else(|| capsule_sign::sidecar_path(claim_path));
        let signer_ok = match fs::read(&sidecar_file)
            .ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        {
            Some(sc) => {
                capsule_sign::verify_sidecar(&sc, &claim_bytes, &keyring).verdict == "valid"
            }
            None => false,
        };
        // 명세서 서명 — 사이드카 부재는 null(미서명 보고), 있으면 판정.
        let wo_sidecar = capsule_sign::sidecar_path(workorder);
        let workorder_signer_ok: serde_json::Value = match fs::read(&wo_sidecar)
            .ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        {
            Some(sc) => match fs::read(workorder) {
                Ok(wo_bytes) => serde_json::json!(
                    capsule_sign::verify_sidecar(&sc, &wo_bytes, &keyring).verdict == "valid"
                ),
                Err(_) => serde_json::json!(false),
            },
            None => serde_json::Value::Null,
        };
        ok = ok && signer_ok && workorder_signer_ok != serde_json::json!(false);
        envelope["signerOk"] = serde_json::json!(signer_ok);
        envelope["workorderSignerOk"] = workorder_signer_ok;
    }
    if let Some(lp) = ledger_path {
        match anchor_log::load_kind(lp, settle::LEDGER_KIND) {
            Ok(ledger) => {
                let dup =
                    settle::find_accepted(&ledger, claim["capsuleSha256"].as_str().unwrap_or(""))
                        .is_some();
                envelope["ledgerOk"] = serde_json::json!(true);
                envelope["duplicate"] = serde_json::json!(dup);
                ok = ok && !dup;
            }
            Err(e) => {
                eprintln!("경고: 원장 검증 실패 — {e}");
                envelope["ledgerOk"] = serde_json::json!(false);
                envelope["duplicate"] = serde_json::Value::Null;
                ok = false;
            }
        }
    }
    envelope["verdict"] = serde_json::json!(if ok { "ok" } else { "rejected" });
    let envelope = provenance::marked(envelope, "settle");
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "청구 검증 — 명세서 {workorder_ok} · 캡슐 {capsule_ok} · 게이트 {gate_ok} → {}",
            if ok { "ok" } else { "rejected" }
        );
    }
    if ok {
        EXIT_OK
    } else {
        3 // #2707: 판정 데이터 — 어떤 축이 무너졌는지는 봉투가 말한다.
    }
}

/// [#4553] 원장 기입 — 이중 청구 전역 검사 후 append-only 등재.
fn cmd_settle_record(args: &[String]) -> i32 {
    let mut claim_path: Option<&str> = None;
    let mut ledger_path: Option<&str> = None;
    let mut verdict = "accepted";
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--ledger" => {
                i += 1;
                ledger_path = args.get(i).map(String::as_str);
            }
            "--verdict" => {
                i += 1;
                verdict = match args.get(i).map(String::as_str) {
                    Some(v @ ("accepted" | "rejected")) => v,
                    _ => {
                        eprintln!("--verdict 는 accepted|rejected 만 받는다");
                        return EXIT_USAGE;
                    }
                };
            }
            other if !other.starts_with("--") && claim_path.is_none() => claim_path = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(claim_path), Some(ledger_path)) = (claim_path, ledger_path) else {
        eprintln!("사용법: rhwp settle record <claim.json> --ledger <ledger.ndjson> [--verdict accepted|rejected] [--json]");
        return EXIT_USAGE;
    };
    let claim_bytes = match fs::read(claim_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 청구를 읽을 수 없습니다 - {claim_path}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let claim: serde_json::Value = match serde_json::from_slice(&claim_bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 청구 파싱 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    if claim["kind"] != settle::CLAIM_KIND {
        eprintln!("오류: kind 가 {} 가 아닙니다.", settle::CLAIM_KIND);
        return EXIT_USAGE;
    }
    let Some(capsule_sha) = claim["capsuleSha256"].as_str().filter(|s| !s.is_empty()) else {
        eprintln!("오류: 청구에 capsuleSha256 이 없습니다.");
        return EXIT_USAGE;
    };
    // 깨진 원장에는 기입하지 않는다 — 5년 앵커 add 와 같은 문장.
    let ledger = match anchor_log::load_kind(ledger_path, settle::LEDGER_KIND) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("오류: 원장이 깨져 있어 기입을 거부합니다 — {e}");
            return 3;
        }
    };
    if verdict == "accepted" {
        if let Some(seq) = settle::find_accepted(&ledger, capsule_sha) {
            let envelope = provenance::marked(
                serde_json::json!({
                    "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                    "ledger": ledger_path,
                    "capsuleSha256": capsule_sha,
                    "duplicate": true,
                    "existingSeq": seq,
                }),
                "settle",
            );
            if json_mode {
                println!("{envelope}");
            } else {
                println!("이중 청구 — 같은 캡슐이 seq {seq} 에 이미 accepted (기입 거부)");
            }
            return 3; // #2707: 판정 데이터 — P3 이중 청구.
        }
    }
    let claim_sha = settle::sha256_hex(&claim_bytes);
    let line = settle::make_ledger_line(
        &ledger,
        &claim_sha,
        capsule_sha,
        verdict,
        &capsule_sign::rfc3339_utc_now(),
    );
    let mut text = String::new();
    if !ledger.entries.is_empty() {
        // 기존 파일 끝에 개행이 보장되지 않으므로 원문을 다시 읽어 이어붙인다.
        text = fs::read_to_string(ledger_path).unwrap_or_default();
        if !text.ends_with('\n') && !text.is_empty() {
            text.push('\n');
        }
    }
    text.push_str(&line);
    text.push('\n');
    if let Err(e) = fs::write(ledger_path, text) {
        eprintln!("오류: 원장 저장 실패 - {ledger_path}: {e}");
        return EXIT_RUNTIME;
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "ledger": ledger_path,
            "seq": ledger.entries.len(),
            "claimSha256": claim_sha,
            "capsuleSha256": capsule_sha,
            "verdict": verdict,
            "duplicate": false,
        }),
        "settle",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "원장 기입 — {ledger_path} seq {} ({verdict})",
            ledger.entries.len()
        );
    }
    EXIT_OK
}

/// [#4553] settle 디스패치 — propose·verify·record.
pub(crate) fn cmd_settle(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("propose") => cmd_settle_propose(&args[1..]),
        Some("verify") => cmd_settle_verify(&args[1..]),
        Some("record") => cmd_settle_record(&args[1..]),
        _ => {
            eprintln!("사용법: rhwp settle <propose|verify|record> …");
            EXIT_USAGE
        }
    }
}
