use crate::cli::protocol::*;

/// [#4543] 앵커 등재 — 캡슐 해시를 append-only 로그 끝에 더한다.
///
/// 등재 전에 로그 전체의 자기 무결(줄 해시 체인)을 검사한다 — 깨진 로그에
/// append 하는 것은 변조 위에 도장을 찍는 일이라 거부한다(exit 3).
fn cmd_anchor_add(args: &[String]) -> i32 {
    let mut capsule: Option<&str> = None;
    let mut log_path: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--log" => {
                i += 1;
                log_path = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && capsule.is_none() => capsule = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(capsule), Some(log_path)) = (capsule, log_path) else {
        eprintln!("사용법: rhwp anchor add <캡슐.json> --log <anchor.ndjson> [--json]");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(capsule) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 캡슐을 읽을 수 없습니다 - {capsule}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let capsule_sha = replay_sha256_hex(&bytes);
    let log = match anchor_log::load(log_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("오류(로그 무결): {e}");
            return 3; // #2707: 깨진 로그에는 등재하지 않는다.
        }
    };
    let line = anchor_log::make_entry_line(&log, &capsule_sha, &capsule_sign::rfc3339_utc_now());
    let mut data = String::new();
    if !log.entries.is_empty() {
        data.push('\n');
    }
    data.push_str(&line);
    use std::io::Write as _;
    let appended = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .and_then(|mut f| f.write_all(data.as_bytes()));
    if let Err(e) = appended {
        eprintln!("오류: 로그 append 실패 - {log_path}: {e}");
        return EXIT_RUNTIME;
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "log": log_path,
            "capsuleSha256": capsule_sha,
            "seq": log.entries.len(),
        }),
        "anchor",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!("앵커 등재 — seq {} ← {capsule}", log.entries.len());
    }
    EXIT_OK
}

/// [#4543] 머클 체크포인트 — 로그 전체의 루트를 산출한다.
///
/// 공표는 도구 밖 운영 절차다 — 봉투는 루트 산출까지만 책임진다.
fn cmd_anchor_checkpoint(args: &[String]) -> i32 {
    let mut log_path: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--log" => {
                i += 1;
                log_path = args.get(i).map(String::as_str);
            }
            "-o" => {
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
    let Some(log_path) = log_path else {
        eprintln!(
            "사용법: rhwp anchor checkpoint --log <anchor.ndjson> [-o <체크포인트.json>] [--json]"
        );
        return EXIT_USAGE;
    };
    let log = match anchor_log::load(log_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("오류(로그 무결): {e}");
            return 3;
        }
    };
    let Some(root) = anchor_log::merkle_root(&log.line_hashes) else {
        eprintln!("오류: 빈 로그에는 체크포인트가 없습니다 - {log_path}");
        return EXIT_USAGE;
    };
    let checkpoint = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "kind": anchor_log::CHECKPOINT_KIND,
        "upToSeq": log.entries.len() - 1,
        "merkleRoot": root,
    });
    if let Some(out) = out {
        if let Err(e) = fs::write(
            out,
            serde_json::to_string_pretty(&checkpoint).unwrap_or_default(),
        ) {
            eprintln!("오류: 체크포인트 저장 실패 - {out}: {e}");
            return EXIT_RUNTIME;
        }
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "log": log_path,
            "upToSeq": log.entries.len() - 1,
            "merkleRoot": root,
            "entries": log.entries.len(),
        }),
        "anchor",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!("체크포인트 — upToSeq {} root {root}", log.entries.len() - 1);
    }
    EXIT_OK
}

/// [#4543] 앵커 검증 — 캡슐이 로그에 있고, 체크포인트에 포함되는가.
fn cmd_anchor_verify(args: &[String]) -> i32 {
    let mut capsule: Option<&str> = None;
    let mut log_path: Option<&str> = None;
    let mut checkpoint_path: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--log" => {
                i += 1;
                log_path = args.get(i).map(String::as_str);
            }
            "--checkpoint" => {
                i += 1;
                checkpoint_path = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && capsule.is_none() => capsule = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(capsule), Some(log_path)) = (capsule, log_path) else {
        eprintln!("사용법: rhwp anchor verify <캡슐.json> --log <anchor.ndjson> [--checkpoint <cp.json>] [--json]");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(capsule) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 캡슐을 읽을 수 없습니다 - {capsule}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let capsule_sha = replay_sha256_hex(&bytes);
    let (log, chain_ok, chain_err) = match anchor_log::load(log_path) {
        Ok(l) => (Some(l), true, serde_json::Value::Null),
        Err(e) => (None, false, serde_json::json!(e)),
    };
    let seq = log.as_ref().and_then(|l| {
        l.entries
            .iter()
            .position(|e| e["capsuleSha256"].as_str() == Some(capsule_sha.as_str()))
    });
    let mut in_checkpoint = serde_json::Value::Null;
    let mut merkle_path_json = serde_json::Value::Null;
    if let (Some(log), Some(seq), Some(cp_path)) = (log.as_ref(), seq, checkpoint_path) {
        match fs::read_to_string(cp_path)
            .map_err(|e| e.to_string())
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).map_err(|e| e.to_string()))
        {
            Ok(cp) => {
                let up_to = cp["upToSeq"].as_u64().map(|v| v as usize);
                let root = cp["merkleRoot"].as_str().unwrap_or("");
                match up_to {
                    Some(up_to) if seq <= up_to && up_to < log.line_hashes.len() => {
                        let leaves = &log.line_hashes[..=up_to];
                        let path = anchor_log::merkle_path(leaves, seq);
                        let ok = anchor_log::merkle_verify(&log.line_hashes[seq], &path, root);
                        in_checkpoint = serde_json::json!(ok);
                        merkle_path_json = serde_json::json!(path
                            .iter()
                            .map(|(h, left)| serde_json::json!({ "sibling": h, "siblingIsLeft": left }))
                            .collect::<Vec<_>>());
                    }
                    _ => in_checkpoint = serde_json::json!(false),
                }
            }
            Err(e) => {
                eprintln!("오류: 체크포인트를 읽을 수 없습니다 - {cp_path}: {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    let logged = seq.is_some();
    let ok = chain_ok && logged && in_checkpoint != serde_json::json!(false);
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "capsule": capsule,
            "log": log_path,
            "capsuleSha256": capsule_sha,
            "logChainOk": chain_ok,
            "logChainError": chain_err,
            "logged": logged,
            "seq": seq,
            "inCheckpoint": in_checkpoint,
            "merklePath": merkle_path_json,
        }),
        "anchor",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "앵커 검증 — {capsule}: logged {logged} · chain {chain_ok} · checkpoint {in_checkpoint}"
        );
    }
    if ok {
        EXIT_OK
    } else {
        3 // #2707: 검증 단언 실패 — 앵커가 시점을 증명하지 못한다.
    }
}

/// [#4543] anchor 디스패치 — add·checkpoint·verify.
pub(crate) fn cmd_anchor(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("add") => cmd_anchor_add(&args[1..]),
        Some("checkpoint") => cmd_anchor_checkpoint(&args[1..]),
        Some("verify") => cmd_anchor_verify(&args[1..]),
        _ => {
            eprintln!("사용법: rhwp anchor <add|checkpoint|verify> …");
            EXIT_USAGE
        }
    }
}
