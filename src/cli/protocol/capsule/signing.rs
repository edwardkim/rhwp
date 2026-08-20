use crate::cli::protocol::*;

/// [#4509] 서명키 발급 — Ed25519 키 파일. 비밀키가 담기므로 기존 파일을
/// 덮어쓰지 않는다(잃어버린 키는 재발급하면 되지만, 덮어쓴 키는 복구 불능).
pub(crate) fn cmd_keygen(args: &[String]) -> i32 {
    let mut key_id: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--key-id" => {
                i += 1;
                key_id = args.get(i).map(String::as_str);
            }
            "--out" => {
                i += 1;
                out = args.get(i).map(String::as_str);
            }
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(key_id), Some(out)) = (key_id, out) else {
        eprintln!("사용법: rhwp keygen --key-id <소유/용도#세대> --out <키.json> [--json]");
        return EXIT_USAGE;
    };
    if std::path::Path::new(out).exists() {
        eprintln!("오류: 키 파일이 이미 있습니다 - {out} (덮어쓰기 금지 — 새 경로를 쓰세요).");
        return EXIT_USAGE;
    }
    let key = match capsule_sign::generate_key_json(key_id) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };
    if let Err(e) = fs::write(out, serde_json::to_string_pretty(&key).unwrap_or_default()) {
        eprintln!("오류: 키 저장 실패 - {out}: {e}");
        return EXIT_RUNTIME;
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "keyId": key_id,
            "publicKey": key["publicKey"],
            "keyFile": out,
        }),
        "keygen",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!("서명키 발급 — {key_id}");
        println!("  keyFile:   {out}  (비밀키 포함 — 보관 책임은 소유자에게)");
        println!(
            "  publicKey: {}",
            envelope["publicKey"].as_str().unwrap_or("")
        );
    }
    EXIT_OK
}

/// [#4509] 캡슐 서명 단건 검증 — 분리 서명을 캡슐 파일 바이트·키 등록부와
/// 대조한다. 판정은 봉투 데이터(verdict)이고 유효하지 않으면 exit 3 이다.
pub(crate) fn cmd_verify_signature(args: &[String]) -> i32 {
    let mut capsule: Option<&str> = None;
    let mut sig: Option<String> = None;
    let mut keyring_path: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--sig" => {
                i += 1;
                sig = args.get(i).cloned();
            }
            "--keyring" => {
                i += 1;
                keyring_path = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && capsule.is_none() => capsule = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(capsule), Some(keyring_path)) = (capsule, keyring_path) else {
        eprintln!(
            "사용법: rhwp verify-signature <캡슐.json> --keyring <키링.json> [--sig <서명.json>] [--json]"
        );
        return EXIT_USAGE;
    };
    let capsule_bytes = match fs::read(capsule) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 캡슐을 읽을 수 없습니다 - {capsule}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let sig_path = sig.unwrap_or_else(|| capsule_sign::sidecar_path(capsule));
    let sig_text = match fs::read_to_string(&sig_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 서명 파일을 읽을 수 없습니다 - {sig_path}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let keyring = match capsule_sign::load_keyring(keyring_path) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };
    let capsule_sha = replay_sha256_hex(&capsule_bytes);
    // 서명 파일 파싱 실패는 IO 가 아니라 판정 데이터다 — 위조·손상 서명을
    // 오류로 숨기지 않고 verdict:malformed 로 폭로한다.
    let (verdict_json, exit_valid) = match serde_json::from_str::<serde_json::Value>(&sig_text) {
        Ok(sidecar) => {
            let sha_matches = sidecar["capsuleSha256"] == serde_json::json!(capsule_sha);
            let v = capsule_sign::verify_sidecar(&sidecar, &capsule_bytes, &keyring);
            let ok = v.verdict == "valid" && sha_matches;
            (
                serde_json::json!({
                    "capsuleShaMatches": sha_matches,
                    "signatureOk": v.signature_ok,
                    "keyId": v.key_id,
                    "keyKnown": v.key_known,
                    "revoked": v.revoked,
                    "verdict": v.verdict,
                }),
                ok,
            )
        }
        Err(_) => (
            serde_json::json!({
                "capsuleShaMatches": false,
                "signatureOk": serde_json::Value::Null,
                "keyId": serde_json::Value::Null,
                "keyKnown": false,
                "revoked": serde_json::Value::Null,
                "verdict": "malformed",
            }),
            false,
        ),
    };
    let mut body = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "capsule": capsule,
        "sigPath": sig_path,
        "capsuleSha256": capsule_sha,
    });
    for (k, v) in verdict_json.as_object().unwrap() {
        body[k] = v.clone();
    }
    let envelope = provenance::marked(body, "verify-signature");
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "캡슐 서명 — {capsule}: {}",
            envelope["verdict"].as_str().unwrap_or("?")
        );
    }
    if exit_valid {
        EXIT_OK
    } else {
        3 // #2707: 검증 단언 실패 — 서명이 귀속을 증명하지 못한다.
    }
}
