use crate::cli::protocol::*;

/// [#4549] 연합 번들 내보내기 — 계보 폐쇄집합+서명+머클 증명을 zip 하나로.
fn cmd_bundle_export(args: &[String]) -> i32 {
    let mut head: Option<&str> = None;
    let mut out: Option<&str> = None;
    let mut anchor_log_path: Option<&str> = None;
    let mut checkpoint_path: Option<&str> = None;
    let mut domain_path: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "-o" => {
                i += 1;
                out = args.get(i).map(String::as_str);
            }
            "--anchor-log" => {
                i += 1;
                anchor_log_path = args.get(i).map(String::as_str);
            }
            "--checkpoint" => {
                i += 1;
                checkpoint_path = args.get(i).map(String::as_str);
            }
            "--domain" => {
                i += 1;
                domain_path = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && head.is_none() => head = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(head), Some(out)) = (head, out) else {
        eprintln!("사용법: rhwp bundle export <머리캡슐> -o <x.lineage-bundle> [--anchor-log <로그> --checkpoint <cp.json>] [--domain <domain.json>] [--json]");
        return EXIT_USAGE;
    };
    let closure = match lineage_bundle::closure(head) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };
    let mut files: Vec<serde_json::Value> = Vec::new();
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut signatures = 0usize;
    for (name, path) in &closure {
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        };
        files.push(serde_json::json!({
            "path": format!("capsules/{name}"),
            "sha256": replay_sha256_hex(&bytes),
        }));
        entries.push((format!("capsules/{name}"), bytes));
        let sc_path = capsule_sign::sidecar_path(&path.to_string_lossy());
        if let Ok(sc) = fs::read(&sc_path) {
            files.push(serde_json::json!({
                "path": format!("signatures/{name}.sig.json"),
                "sha256": replay_sha256_hex(&sc),
            }));
            entries.push((format!("signatures/{name}.sig.json"), sc));
            signatures += 1;
        }
    }
    // 머클 증명 — 로그+체크포인트가 있으면 캡슐별 (로그 줄, 경로) 동봉.
    let mut proofs = 0usize;
    if let (Some(log_path), Some(cp_path)) = (anchor_log_path, checkpoint_path) {
        let log = match anchor_log::load(log_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("오류(로그 무결): {e}");
                return 3;
            }
        };
        let cp_bytes = match fs::read(cp_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("오류: 체크포인트를 읽을 수 없습니다 - {cp_path}: {e}");
                return EXIT_RUNTIME;
            }
        };
        let cp: serde_json::Value = match serde_json::from_slice(&cp_bytes) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 체크포인트 파싱 실패 - {e}");
                return EXIT_RUNTIME;
            }
        };
        let up_to = cp["upToSeq"].as_u64().unwrap_or(0) as usize;
        let log_text = fs::read_to_string(log_path).unwrap_or_default();
        let lines: Vec<&str> = log_text.lines().filter(|l| !l.trim().is_empty()).collect();
        let mut proof_list = Vec::new();
        for (name, path) in &closure {
            let sha = replay_sha256_hex(&fs::read(path).unwrap_or_default());
            if let Some(seq) = log
                .entries
                .iter()
                .position(|e| e["capsuleSha256"].as_str() == Some(sha.as_str()))
            {
                if seq <= up_to && up_to < log.line_hashes.len() {
                    let leaves = &log.line_hashes[..=up_to];
                    let path_json: Vec<serde_json::Value> = anchor_log::merkle_path(leaves, seq)
                        .into_iter()
                        .map(|(h, left)| serde_json::json!({ "sibling": h, "siblingIsLeft": left }))
                        .collect();
                    proof_list.push(serde_json::json!({
                        "capsule": name,
                        "seq": seq,
                        "line": lines.get(seq).copied().unwrap_or(""),
                        "path": path_json,
                    }));
                    proofs += 1;
                }
            }
        }
        let proofs_json = serde_json::json!({ "checkpoint": cp, "proofs": proof_list });
        let bytes = serde_json::to_vec_pretty(&proofs_json).unwrap_or_default();
        files.push(serde_json::json!({
            "path": "anchor/proofs.json",
            "sha256": replay_sha256_hex(&bytes),
        }));
        entries.push(("anchor/proofs.json".to_string(), bytes));
    }
    let mut domain_name = serde_json::Value::Null;
    if let Some(dp) = domain_path {
        let bytes = match fs::read(dp) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("오류: 도메인 파일을 읽을 수 없습니다 - {dp}: {e}");
                return EXIT_RUNTIME;
            }
        };
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            domain_name = v["domain"].clone();
        }
        files.push(serde_json::json!({
            "path": "domain.json",
            "sha256": replay_sha256_hex(&bytes),
        }));
        entries.push(("domain.json".to_string(), bytes));
    }
    let manifest = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "kind": lineage_bundle::BUNDLE_KIND,
        "head": format!("capsules/{}", closure[0].0),
        "domain": domain_name,
        "files": files,
    });
    let file = match fs::File::create(out) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("오류: 번들 생성 실패 - {out}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let mut zipw = zip::ZipWriter::new(file);
    if let Err(e) = lineage_bundle::zip_put(
        &mut zipw,
        "manifest.json",
        &serde_json::to_vec_pretty(&manifest).unwrap_or_default(),
    ) {
        eprintln!("오류: {e}");
        return EXIT_RUNTIME;
    }
    for (path, bytes) in &entries {
        if let Err(e) = lineage_bundle::zip_put(&mut zipw, path, bytes) {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    }
    if let Err(e) = zipw.finish() {
        eprintln!("오류: 번들 마감 실패 - {e}");
        return EXIT_RUNTIME;
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "bundle": out,
            "head": closure[0].0,
            "capsules": closure.len(),
            "signatures": signatures,
            "proofs": proofs,
            "domain": manifest["domain"],
        }),
        "bundle",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "번들 내보내기 — {out}: 캡슐 {} · 서명 {signatures} · 증명 {proofs}",
            closure.len()
        );
    }
    EXIT_OK
}

/// [#4549] 연합 번들 검증 — 5단(컨테이너·폐쇄집합·계보·서명·앵커) 오프라인 판정.
struct BundleVerifyOptions<'a> {
    bundle: &'a str,
    trust_domain: &'a str,
    json_mode: bool,
}

fn parse_bundle_verify_options(args: &[String]) -> Result<BundleVerifyOptions<'_>, i32> {
    let mut bundle: Option<&str> = None;
    let mut trust_domain: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--trust-domain" => {
                i += 1;
                trust_domain = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && bundle.is_none() => bundle = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
        }
        i += 1;
    }
    let (Some(bundle), Some(trust_domain)) = (bundle, trust_domain) else {
        eprintln!(
            "사용법: rhwp bundle verify <x.lineage-bundle> --trust-domain <domain.json> [--json]"
        );
        return Err(EXIT_USAGE);
    };
    Ok(BundleVerifyOptions {
        bundle,
        trust_domain,
        json_mode,
    })
}

fn cmd_bundle_verify(args: &[String]) -> i32 {
    let options = match parse_bundle_verify_options(args) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let BundleVerifyOptions {
        bundle,
        trust_domain,
        json_mode,
    } = options;
    let td_text = match fs::read_to_string(trust_domain) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: trust-domain 을 읽을 수 없습니다 - {trust_domain}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let (domain, keyring_value, checkpoints) = match lineage_bundle::parse_trust_domain(&td_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_USAGE;
        }
    };
    let ring = match capsule_sign::keyring_from_value(&keyring_value, trust_domain) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_USAGE;
        }
    };
    let map = match lineage_bundle::read_all(bundle) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };
    let mut broken_at = serde_json::Value::Null;
    let mut note = |ok: &mut bool, why: String, broken_at: &mut serde_json::Value| {
        if *ok {
            *ok = false;
            if broken_at.is_null() {
                *broken_at = serde_json::json!(why);
            }
        }
    };
    // ① 컨테이너 — 매니페스트의 전 파일 해시 대조.
    let mut container_ok = true;
    let manifest: serde_json::Value = match map
        .get("manifest.json")
        .and_then(|b| serde_json::from_slice(b).ok())
    {
        Some(m) => m,
        None => {
            eprintln!("오류: manifest.json 이 없거나 파싱 불가");
            return EXIT_RUNTIME;
        }
    };
    if manifest["kind"] != lineage_bundle::BUNDLE_KIND {
        note(
            &mut container_ok,
            "manifest kind 불일치".into(),
            &mut broken_at,
        );
    }
    for f in manifest["files"].as_array().cloned().unwrap_or_default() {
        let (Some(path), Some(sha)) = (f["path"].as_str(), f["sha256"].as_str()) else {
            note(
                &mut container_ok,
                "manifest files 항목 형식 오류".into(),
                &mut broken_at,
            );
            continue;
        };
        match map.get(path) {
            Some(bytes) if replay_sha256_hex(bytes) == sha => {}
            Some(_) => note(
                &mut container_ok,
                format!("{path}: 해시 불일치(운송 중 변조)"),
                &mut broken_at,
            ),
            None => note(
                &mut container_ok,
                format!("{path}: 번들에 없음"),
                &mut broken_at,
            ),
        }
    }
    // ② 폐쇄집합 + ③ 계보 걷기 (머리부터 부모 이름 해소).
    let mut closure_ok = true;
    let mut lineage_valid = true;
    let head_path = manifest["head"].as_str().unwrap_or("");
    let mut current = head_path.to_string();
    let mut recorded: Option<String> = None;
    let mut child_input: Option<String> = None;
    let mut capsule_names: Vec<String> = Vec::new();
    for _ in 0..1000 {
        let Some(bytes) = map.get(&current) else {
            note(
                &mut closure_ok,
                format!("{current}: 폐쇄집합에 없음(부모 누락)"),
                &mut broken_at,
            );
            break;
        };
        let file_sha = replay_sha256_hex(bytes);
        let Ok(capsule) = serde_json::from_slice::<serde_json::Value>(bytes) else {
            note(
                &mut lineage_valid,
                format!("{current}: 캡슐 파싱 실패"),
                &mut broken_at,
            );
            break;
        };
        if let Some(r) = recorded.as_deref() {
            if r != file_sha {
                note(
                    &mut lineage_valid,
                    format!("{current}: 부모 해시 불일치"),
                    &mut broken_at,
                );
                break;
            }
        }
        let out_sha = capsule["receipt"]["outputSha256"].as_str().unwrap_or("");
        if let Some(ci) = child_input.as_deref() {
            if !out_sha.is_empty() && out_sha != ci {
                note(
                    &mut lineage_valid,
                    format!("{current}: 계보 불변식 위반"),
                    &mut broken_at,
                );
                break;
            }
        }
        capsule_names.push(current.trim_start_matches("capsules/").to_string());
        let parent = &capsule["parent"];
        if parent.is_null() {
            break;
        }
        let (Some(pp), Some(psha)) = (parent["capsule"].as_str(), parent["sha256"].as_str()) else {
            note(
                &mut lineage_valid,
                format!("{current}: parent 형식 오류"),
                &mut broken_at,
            );
            break;
        };
        recorded = Some(psha.to_string());
        child_input = capsule["receipt"]["inputSha256"]
            .as_str()
            .map(str::to_string);
        let base = std::path::Path::new(pp)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| pp.to_string());
        current = format!("capsules/{base}");
    }
    // ④ 서명 — trust-domain 의 keyring 으로만 (동봉 keyring 불신, F2).
    let (mut sig_valid, mut sig_bad, mut unsigned) = (0u64, 0u64, 0u64);
    for name in &capsule_names {
        let cap_bytes = map
            .get(&format!("capsules/{name}"))
            .cloned()
            .unwrap_or_default();
        match map
            .get(&format!("signatures/{name}.sig.json"))
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(b).ok())
        {
            Some(sc) => {
                let v = capsule_sign::verify_sidecar(&sc, &cap_bytes, &ring);
                if v.verdict == "valid" {
                    sig_valid += 1;
                } else {
                    sig_bad += 1;
                    note(
                        &mut lineage_valid,
                        format!("{name}: 서명 {}(도메인 키링 기준)", v.verdict),
                        &mut broken_at,
                    );
                }
            }
            None => unsigned += 1,
        }
    }
    // ⑤ 앵커 — 동봉 증명의 루트가 도메인 선언 체크포인트와 일치해야 한다.
    let mut anchored = serde_json::Value::Null;
    if let Some(proofs) = map
        .get("anchor/proofs.json")
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(b).ok())
    {
        let bundle_root = proofs["checkpoint"]["merkleRoot"].as_str().unwrap_or("");
        let trusted = checkpoints
            .iter()
            .any(|c| c["merkleRoot"].as_str() == Some(bundle_root));
        let mut ok_count = 0u64;
        let mut bad = 0u64;
        for pr in proofs["proofs"].as_array().cloned().unwrap_or_default() {
            let line = pr["line"].as_str().unwrap_or("");
            let cap_name = pr["capsule"].as_str().unwrap_or("");
            let cap_sha = map
                .get(&format!("capsules/{cap_name}"))
                .map(|b| replay_sha256_hex(b))
                .unwrap_or_default();
            let line_entry: serde_json::Value =
                serde_json::from_str(line).unwrap_or(serde_json::Value::Null);
            let line_matches = line_entry["capsuleSha256"].as_str() == Some(cap_sha.as_str());
            let leaf = {
                use sha2::{Digest, Sha256};
                let mut h = Sha256::new();
                h.update(line.as_bytes());
                let d = h.finalize();
                let mut hex = String::with_capacity(64);
                for b in d {
                    use std::fmt::Write as _;
                    let _ = write!(hex, "{b:02x}");
                }
                hex
            };
            let path: Vec<(String, bool)> = pr["path"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter_map(|p| {
                    Some((
                        p["sibling"].as_str()?.to_string(),
                        p["siblingIsLeft"].as_bool()?,
                    ))
                })
                .collect();
            if trusted && line_matches && anchor_log::merkle_verify(&leaf, &path, bundle_root) {
                ok_count += 1;
            } else {
                bad += 1;
                note(
                    &mut lineage_valid,
                    format!(
                        "{cap_name}: 앵커 증명 실패(신뢰 체크포인트 {trusted}, 줄 일치 {line_matches})"
                    ),
                    &mut broken_at,
                );
            }
        }
        anchored = serde_json::json!({ "ok": ok_count, "bad": bad, "checkpointTrusted": trusted });
    }
    let all_ok = container_ok && closure_ok && lineage_valid && sig_bad == 0;
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "bundle": bundle,
            "trustDomain": domain,
            "containerOk": container_ok,
            "closureOk": closure_ok,
            "lineageValid": lineage_valid,
            "capsules": capsule_names.len(),
            "signed": { "valid": sig_valid, "invalid": sig_bad, "unsigned": unsigned },
            "anchored": anchored,
            "brokenAt": broken_at,
            "verdict": if all_ok { "ok" } else { "broken" },
        }),
        "bundle",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "번들 검증 — {bundle} @ {domain}: {}",
            envelope["verdict"].as_str().unwrap_or("?")
        );
    }
    if all_ok {
        EXIT_OK
    } else {
        3 // #2707: 검증 단언 실패 — 번들이 신뢰를 증명하지 못한다.
    }
}

/// [#4549] bundle 디스패치 — export·verify.
pub(crate) fn cmd_bundle(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("export") => cmd_bundle_export(&args[1..]),
        Some("verify") => cmd_bundle_verify(&args[1..]),
        _ => {
            eprintln!("사용법: rhwp bundle <export|verify> …");
            EXIT_USAGE
        }
    }
}
