use crate::cli::protocol::*;

/// [#4558] 공용 — 폴더 캡슐들의 축별 판정 재료를 한 번에 계산한다.
///
/// 반환: 캡슐별 (서명 verdict 문자열 옵션, anchored 옵션, lineage 유효 옵션,
/// 재현 성공 옵션). 옵션 `None` = 해당 축 재료 미지정(판정 밖).
#[allow(clippy::type_complexity)]
fn y10_axis_materials(
    nodes: &[audit_standard::CapsuleNode],
    keyring: Option<&std::collections::BTreeMap<String, capsule_sign::KeyEntry>>,
    anchored_set: Option<&std::collections::BTreeSet<String>>,
    deep: bool,
) -> Vec<(
    Option<String>,
    Option<bool>,
    Option<bool>,
    Option<Result<(), String>>,
)> {
    nodes
        .iter()
        .map(|node| {
            let signer = keyring.map(|kr| {
                let sidecar_file = capsule_sign::sidecar_path(&node.path.to_string_lossy());
                match fs::read(&sidecar_file)
                    .ok()
                    .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                {
                    Some(sc) => {
                        let bytes = fs::read(&node.path).unwrap_or_default();
                        capsule_sign::verify_sidecar(&sc, &bytes, kr)
                            .verdict
                            .to_string()
                    }
                    None => "unsigned".to_string(),
                }
            });
            let anchored = anchored_set.map(|set| set.contains(&node.file_sha256));
            let lineage_ok = Some(
                audit_standard::walk_ancestry(&node.path, &node.value)
                    .broken_at
                    .is_none(),
            );
            let reproduced = if deep {
                Some(y10_reproduce_one(&node.value))
            } else {
                None
            };
            (signer, anchored, lineage_ok, reproduced)
        })
        .collect()
}

#[allow(clippy::type_complexity)]
fn audit_reproduction(
    nodes: &[audit_standard::CapsuleNode],
    materials: &[(
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<Result<(), String>>,
    )],
    deep: bool,
) -> serde_json::Value {
    if !deep {
        return serde_json::Value::Null;
    }
    let mut reproduced = 0u64;
    let mut failures: Vec<serde_json::Value> = Vec::new();
    for (node, (_, _, _, result)) in nodes.iter().zip(materials) {
        match result.as_ref().expect("deep 재료") {
            Ok(()) => reproduced += 1,
            Err(error) => failures.push(serde_json::json!({
                "capsule": node.name, "reason": error,
            })),
        }
    }
    let attempted = nodes.len() as u64;
    serde_json::json!({
        "attempted": attempted,
        "reproduced": reproduced,
        "rate": if attempted == 0 { serde_json::Value::Null }
                else { serde_json::json!(reproduced as f64 / attempted as f64) },
        "failures": failures,
    })
}

#[allow(clippy::type_complexity)]
fn audit_attribution(
    materials: &[(
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<Result<(), String>>,
    )],
    enabled: bool,
) -> serde_json::Value {
    if !enabled {
        return serde_json::Value::Null;
    }
    let (mut signed, mut unsigned, mut valid, mut revoked) = (0u64, 0u64, 0u64, 0u64);
    for (signer, _, _, _) in materials {
        match signer.as_deref() {
            Some("unsigned") => unsigned += 1,
            Some(verdict) => {
                signed += 1;
                valid += u64::from(verdict == "valid");
                revoked += u64::from(verdict == "revoked");
            }
            None => unreachable!("keyring 지정 시 signer 는 항상 계산된다"),
        }
    }
    serde_json::json!({
        "signed": signed, "unsigned": unsigned,
        "validSignatures": valid, "revokedKeyUses": revoked,
    })
}

#[allow(clippy::type_complexity)]
fn audit_anchoring(
    capsule_count: usize,
    materials: &[(
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<Result<(), String>>,
    )],
    enabled: bool,
) -> serde_json::Value {
    if !enabled {
        return serde_json::Value::Null;
    }
    let anchored = materials
        .iter()
        .filter(|(_, anchored, _, _)| *anchored == Some(true))
        .count() as u64;
    serde_json::json!({
        "anchored": anchored,
        "unanchored": capsule_count as u64 - anchored,
    })
}

/// [#4558] 캡슐 하나의 deep 재현 — audit 와 같은 실행 코어 재사용.
fn y10_reproduce_one(capsule: &serde_json::Value) -> Result<(), String> {
    let (plan, _steps) = validated_capsule_plan(capsule)?;
    let mut plan = plan;
    let (out_sha, _n, input_sha) = replay_execute_to_temp(&mut plan, "y10").map_err(|(e, _)| e)?;
    let want_in = capsule["receipt"]["inputSha256"].as_str().unwrap_or("");
    let want_out = capsule["receipt"]["outputSha256"].as_str().unwrap_or("");
    if !want_in.is_empty() && want_in != input_sha {
        return Err("입력 해시 불일치(원본이 변했다)".to_string());
    }
    if want_out != out_sha {
        return Err("산출 해시 불일치(재현 실패)".to_string());
    }
    Ok(())
}

/// [#4558] 감사 보고 — 전 수치가 기존 축 검증의 기계 합산인 표준 보고서.
struct AuditReportOptions<'a> {
    dir: &'a str,
    policy_path: Option<&'a str>,
    keyring_path: Option<&'a str>,
    anchor_path: Option<&'a str>,
    sign_key: Option<&'a str>,
    out: &'a str,
    deep: bool,
    json_mode: bool,
}

fn parse_audit_report_options(args: &[String]) -> Result<AuditReportOptions<'_>, i32> {
    let mut dir: Option<&str> = None;
    let mut policy_path: Option<&str> = None;
    let mut keyring_path: Option<&str> = None;
    let mut anchor_path: Option<&str> = None;
    let mut sign_key: Option<&str> = None;
    let mut out: Option<&str> = None;
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
                anchor_path = args.get(i).map(String::as_str);
            }
            "--sign-key" => {
                i += 1;
                sign_key = args.get(i).map(String::as_str);
            }
            "-o" => {
                i += 1;
                out = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && dir.is_none() => dir = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
        }
        i += 1;
    }
    let (Some(dir), Some(out)) = (dir, out) else {
        eprintln!("사용법: rhwp audit-report <캡슐 폴더> -o <report.json> [--deep] [--keyring <k>] [--anchor-log <l>] [--policy <p>] [--sign-key <키>] [--json]");
        return Err(EXIT_USAGE);
    };
    Ok(AuditReportOptions {
        dir,
        policy_path,
        keyring_path,
        anchor_path,
        sign_key,
        out,
        deep,
        json_mode,
    })
}

pub(crate) fn cmd_audit_report(args: &[String]) -> i32 {
    let options = match parse_audit_report_options(args) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let AuditReportOptions {
        dir,
        policy_path,
        keyring_path,
        anchor_path,
        sign_key,
        out,
        deep,
        json_mode,
    } = options;
    let nodes = match audit_standard::collect(dir) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };
    let keyring = match keyring_path {
        Some(kp) => match capsule_sign::load_keyring(kp) {
            Ok(k) => Some(k),
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        },
        None => None,
    };
    let anchored_set: Option<std::collections::BTreeSet<String>> = match anchor_path {
        Some(lp) => match anchor_log::load(lp) {
            Ok(log) => Some(
                log.entries
                    .iter()
                    .filter_map(|e| e["capsuleSha256"].as_str().map(str::to_string))
                    .collect(),
            ),
            Err(e) => {
                eprintln!("오류: 앵커 로그 검증 실패 — {e}");
                return 3;
            }
        },
        None => None,
    };
    let materials = y10_axis_materials(&nodes, keyring.as_ref(), anchored_set.as_ref(), deep);

    // 계보 절 — 머리(자식 없는 노드)별 사슬 판정, graphs = 뿌리 수.
    let (heads, roots) = audit_standard::heads_and_roots(&nodes);
    let mut lineage_valid = 0u64;
    let mut lineage_broken: Vec<serde_json::Value> = Vec::new();
    for &h in &heads {
        let a = audit_standard::walk_ancestry(&nodes[h].path, &nodes[h].value);
        match a.broken_at {
            None => lineage_valid += 1,
            Some(at) => lineage_broken.push(serde_json::json!({
                "head": nodes[h].name, "brokenAt": at,
            })),
        }
    }

    // 재현 절 (--deep opt-in — 재현은 비싸다, 6년 게이트와 같은 문장).
    let reproduction = audit_reproduction(&nodes, &materials, deep);

    // 귀속 절 (--keyring opt-in).
    let attribution = audit_attribution(&materials, keyring.is_some());

    // 앵커 절 (--anchor-log opt-in).
    let anchoring = audit_anchoring(nodes.len(), &materials, anchored_set.is_some());

    // 게이트 절 (--policy opt-in) — 캡슐별 판정, 재료는 위 축들의 재사용.
    let gate: serde_json::Value = match policy_path {
        Some(pp) => {
            let text = match fs::read_to_string(pp) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("오류: 정책을 읽을 수 없습니다 - {pp}: {e}");
                    return EXIT_RUNTIME;
                }
            };
            let policy = match policy_gate::parse(&text) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("오류(정책): {e}");
                    return EXIT_USAGE;
                }
            };
            let policy_sha = settle::sha256_hex(text.as_bytes());
            let (mut passed, mut denied) = (0u64, 0u64);
            for (signer, anchored, lineage_ok, rep) in &materials {
                let mut judgments: std::collections::BTreeMap<String, Option<serde_json::Value>> =
                    std::collections::BTreeMap::new();
                judgments.insert(
                    "reproduced".to_string(),
                    rep.as_ref().map(|r| serde_json::json!(r.is_ok())),
                );
                judgments.insert(
                    "lineageValid".to_string(),
                    lineage_ok.map(|v| serde_json::json!(v)),
                );
                judgments.insert(
                    "signerVerdict".to_string(),
                    signer.as_ref().map(|v| serde_json::json!(v)),
                );
                judgments.insert(
                    "anchoredOk".to_string(),
                    anchored.map(|v| serde_json::json!(v)),
                );
                let (ok, _violations) = policy_gate::evaluate(&policy, &judgments);
                if ok {
                    passed += 1;
                } else {
                    denied += 1;
                }
            }
            serde_json::json!({
                "policySha256": policy_sha, "passed": passed, "denied": denied,
            })
        }
        None => serde_json::Value::Null,
    };

    // 도구 버전 절 — 캡슐 영수증의 기록 합산(없으면 "미기록", 정직 보고).
    let mut versions: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for node in &nodes {
        let v = node.value["receipt"]["version"]
            .as_str()
            .unwrap_or("미기록")
            .to_string();
        versions.insert(v);
    }
    let tool_versions = serde_json::json!({
        "rhwp": versions.iter().collect::<Vec<_>>(),
        "mixed": versions.len() > 1,
    });

    let mut report = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "kind": audit_standard::REPORT_KIND,
        "scope": { "root": dir, "capsules": nodes.len() },
        "reproduction": reproduction,
        "lineage": {
            "graphs": roots, "heads": heads.len(),
            "valid": lineage_valid, "broken": lineage_broken,
        },
        "attribution": attribution,
        "anchoring": anchoring,
        "gate": gate,
        "toolVersions": tool_versions,
    });
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
    if let Some((_, key_id)) = &signer {
        report["auditor"] = serde_json::json!({ "keyId": key_id });
    }
    let report_text = serde_json::to_string_pretty(&report).unwrap_or_default();
    if let Err(e) = fs::write(out, &report_text) {
        eprintln!("오류: 보고서 저장 실패 - {out}: {e}");
        return EXIT_RUNTIME;
    }
    if let Some((signing, key_id)) = &signer {
        let report_sha = settle::sha256_hex(report_text.as_bytes());
        let sidecar =
            capsule_sign::make_sidecar_json(signing, key_id, &report_sha, report_text.as_bytes());
        let sidecar_out = capsule_sign::sidecar_path(out);
        if let Err(e) = fs::write(
            &sidecar_out,
            serde_json::to_string_pretty(&sidecar).unwrap_or_default(),
        ) {
            eprintln!("오류: 보고서 서명 저장 실패 - {sidecar_out}: {e}");
            return EXIT_RUNTIME;
        }
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "report": out,
            "capsules": nodes.len(),
            "reproduction": report["reproduction"],
            "lineage": report["lineage"],
            "attribution": report["attribution"],
            "anchoring": report["anchoring"],
            "gate": report["gate"],
            "toolVersions": report["toolVersions"],
            "signed": signer.is_some(),
        }),
        "audit-report",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "감사 보고 — {out}: 캡슐 {} · 계보 {}/{} (서명 {})",
            nodes.len(),
            lineage_valid,
            heads.len(),
            signer.is_some()
        );
    }
    EXIT_OK
}

/// [#4558] 리콜 범위 — 오염 노드의 후손 폐쇄집합 + 정산 연결.
pub(crate) fn cmd_recall_scope(args: &[String]) -> i32 {
    let mut contaminated: Option<&str> = None;
    let mut among: Option<&str> = None;
    let mut ledger_path: Option<&str> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--contaminated" => {
                i += 1;
                contaminated = args.get(i).map(String::as_str);
            }
            "--among" => {
                i += 1;
                among = args.get(i).map(String::as_str);
            }
            "--ledger" => {
                i += 1;
                ledger_path = args.get(i).map(String::as_str);
            }
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(contaminated), Some(among)) = (contaminated, among) else {
        eprintln!("사용법: rhwp recall-scope --contaminated <캡슐|sha256> --among <폴더> [--ledger <원장>] [--json]");
        return EXIT_USAGE;
    };
    // 오염 정체성 = 파일 해시(64자리 16진이면 해시 그대로, 아니면 파일을 읽어 해시).
    let contaminated_sha =
        if contaminated.len() == 64 && contaminated.chars().all(|c| c.is_ascii_hexdigit()) {
            contaminated.to_lowercase()
        } else {
            match fs::read(contaminated) {
                Ok(b) => settle::sha256_hex(&b),
                Err(e) => {
                    eprintln!("오류: 오염 캡슐을 읽을 수 없습니다 - {contaminated}: {e}");
                    return EXIT_USAGE;
                }
            }
        };
    let nodes = match audit_standard::collect(among) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };
    let mut affected: Vec<serde_json::Value> = Vec::new();
    let mut affected_shas: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for node in &nodes {
        if node.file_sha256 == contaminated_sha {
            // 오염 노드 자신 — 회수 1호.
            affected_shas.insert(node.file_sha256.clone());
            affected.push(serde_json::json!({
                "capsule": node.name, "path": [node.name],
            }));
            continue;
        }
        let ancestry = audit_standard::walk_ancestry(&node.path, &node.value);
        if let Some(pos) = ancestry
            .ancestors
            .iter()
            .position(|(_, sha)| *sha == contaminated_sha)
        {
            // 경로 = 오염 조상 → … → 이 캡슐 (가까운 순 기록을 뒤집는다).
            let mut path: Vec<String> = ancestry.ancestors[..=pos]
                .iter()
                .map(|(n, _)| n.clone())
                .collect();
            path.reverse();
            path.push(node.name.clone());
            affected_shas.insert(node.file_sha256.clone());
            affected.push(serde_json::json!({ "capsule": node.name, "path": path }));
        }
    }
    let unaffected = nodes.len() - affected.len();
    let mut envelope = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "contaminated": contaminated_sha,
        "affected": affected,
        "unaffected": unaffected,
    });
    if let Some(lp) = ledger_path {
        match anchor_log::load_kind(lp, settle::LEDGER_KIND) {
            Ok(ledger) => {
                let claims: Vec<serde_json::Value> = ledger
                    .entries
                    .iter()
                    .filter(|e| {
                        e["capsuleSha256"]
                            .as_str()
                            .map(|sha| affected_shas.contains(sha))
                            .unwrap_or(false)
                    })
                    .map(|e| {
                        serde_json::json!({
                            "seq": e["seq"], "claimSha256": e["claimSha256"],
                            "verdict": e["verdict"],
                        })
                    })
                    .collect();
                envelope["claims"] = serde_json::json!(claims);
            }
            Err(e) => {
                eprintln!("오류: 원장 검증 실패 — {e}");
                return 3;
            }
        }
    }
    let envelope = provenance::marked(envelope, "recall-scope");
    if json_mode {
        println!("{envelope}");
    } else {
        println!("리콜 범위 — 영향 {} · 미영향 {unaffected}", affected.len());
    }
    EXIT_OK
}

/// [#4558] 적합성 자가진단 — L1~L5 누적 요건, 판정기 재사용(발명 0).
pub(crate) fn cmd_conformance(args: &[String]) -> i32 {
    let mut dir: Option<&str> = None;
    let mut level: Option<&str> = None;
    let mut keyring_path: Option<&str> = None;
    let mut anchor_path: Option<&str> = None;
    let mut policy_path: Option<&str> = None;
    let mut ledger_path: Option<&str> = None;
    let mut deep = false;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--deep" => deep = true,
            "--level" => {
                i += 1;
                level = args.get(i).map(String::as_str);
            }
            "--keyring" => {
                i += 1;
                keyring_path = args.get(i).map(String::as_str);
            }
            "--anchor-log" => {
                i += 1;
                anchor_path = args.get(i).map(String::as_str);
            }
            "--policy" => {
                i += 1;
                policy_path = args.get(i).map(String::as_str);
            }
            "--ledger" => {
                i += 1;
                ledger_path = args.get(i).map(String::as_str);
            }
            other if !other.starts_with("--") && dir.is_none() => dir = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    let (Some(dir), Some(level)) = (dir, level) else {
        eprintln!("사용법: rhwp conformance <캡슐 폴더> --level <L1..L5> [--deep] [--keyring] [--anchor-log] [--policy] [--ledger] [--json]");
        return EXIT_USAGE;
    };
    let want: u8 = match level {
        "L1" => 1,
        "L2" => 2,
        "L3" => 3,
        "L4" => 4,
        "L5" => 5,
        _ => {
            eprintln!("--level 은 L1..L5 만 받는다");
            return EXIT_USAGE;
        }
    };
    // 등급이 요구하는 재료의 선검사 — 없으면 판정이 아니라 사용법 오류다.
    if want >= 3 && (keyring_path.is_none() || anchor_path.is_none()) {
        eprintln!("L3 이상은 --keyring 과 --anchor-log 가 필요하다 (서명 귀속 + 앵커 운영이 요건)");
        return EXIT_USAGE;
    }
    if want >= 4 && policy_path.is_none() {
        eprintln!("L4 이상은 --policy 가 필요하다 (게이트 상시 배치가 요건)");
        return EXIT_USAGE;
    }
    if want >= 5 && ledger_path.is_none() {
        eprintln!("L5 는 --ledger 가 필요하다 (정산 원장 운영이 요건)");
        return EXIT_USAGE;
    }
    let nodes = match audit_standard::collect(dir) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("오류: {e}");
            return EXIT_RUNTIME;
        }
    };
    if nodes.is_empty() {
        eprintln!("오류: 캡슐이 없다 — 빈 폴더의 적합성은 판정 대상이 아니다");
        return EXIT_USAGE;
    }
    let mut checks: Vec<serde_json::Value> = Vec::new();
    let push = |checks: &mut Vec<serde_json::Value>, id: &str, ok: bool, detail: String| {
        checks.push(serde_json::json!({ "id": id, "ok": ok, "detail": detail }));
        ok
    };
    // L1 — 산출물마다 영수증 (receipt 3해시).
    let bad_receipt = nodes
        .iter()
        .filter(|n| {
            !(n.value["receipt"]["inputSha256"].is_string()
                && n.value["receipt"]["outputSha256"].is_string()
                && n.value["receipt"]["planSha256"].is_string())
        })
        .count();
    let mut achieved = push(
        &mut checks,
        "L1-영수증",
        bad_receipt == 0,
        format!("영수증 미비 {bad_receipt}/{}", nodes.len()),
    );
    // L2 — 계획 정합(감사 가능) + 계보 유효.
    if want >= 2 {
        let bad_plan = nodes
            .iter()
            .filter(|n| validated_capsule_plan(&n.value).is_err())
            .count();
        achieved &= push(
            &mut checks,
            "L2-감사가능",
            bad_plan == 0,
            format!("계획 정합 실패 {bad_plan}/{}", nodes.len()),
        );
        let broken = nodes
            .iter()
            .filter(|n| {
                audit_standard::walk_ancestry(&n.path, &n.value)
                    .broken_at
                    .is_some()
            })
            .count();
        achieved &= push(
            &mut checks,
            "L2-계보",
            broken == 0,
            format!("계보 파손 {broken}/{}", nodes.len()),
        );
        if deep {
            let failed = nodes
                .iter()
                .filter(|n| y10_reproduce_one(&n.value).is_err())
                .count();
            achieved &= push(
                &mut checks,
                "L2-재현(deep)",
                failed == 0,
                format!("재현 실패 {failed}/{}", nodes.len()),
            );
        }
    }
    // L3 — 서명 전건 valid + 앵커 전건 포함.
    if want >= 3 {
        let keyring = match capsule_sign::load_keyring(keyring_path.expect("선검사")) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("오류: {e}");
                return EXIT_RUNTIME;
            }
        };
        let anchored_set: std::collections::BTreeSet<String> =
            match anchor_log::load(anchor_path.expect("선검사")) {
                Ok(log) => log
                    .entries
                    .iter()
                    .filter_map(|e| e["capsuleSha256"].as_str().map(str::to_string))
                    .collect(),
                Err(e) => {
                    eprintln!("오류: 앵커 로그 검증 실패 — {e}");
                    return 3;
                }
            };
        let materials = y10_axis_materials(&nodes, Some(&keyring), Some(&anchored_set), false);
        let unsigned_or_bad = materials
            .iter()
            .filter(|(s, _, _, _)| s.as_deref() != Some("valid"))
            .count();
        achieved &= push(
            &mut checks,
            "L3-귀속",
            unsigned_or_bad == 0,
            format!("서명 미비/무효 {unsigned_or_bad}/{}", nodes.len()),
        );
        let unanchored = materials
            .iter()
            .filter(|(_, a, _, _)| *a != Some(true))
            .count();
        achieved &= push(
            &mut checks,
            "L3-앵커",
            unanchored == 0,
            format!("미앵커 {unanchored}/{}", nodes.len()),
        );
        // L4 — 게이트 전건 allow (재료는 위 축 재사용 — 판정기 발명 0).
        if want >= 4 {
            let text = match fs::read_to_string(policy_path.expect("선검사")) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("오류: 정책을 읽을 수 없습니다: {e}");
                    return EXIT_RUNTIME;
                }
            };
            let policy = match policy_gate::parse(&text) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("오류(정책): {e}");
                    return EXIT_USAGE;
                }
            };
            let mut denied = 0usize;
            for (node, (signer, anchored, _, _)) in nodes.iter().zip(&materials) {
                let lineage_ok = audit_standard::walk_ancestry(&node.path, &node.value)
                    .broken_at
                    .is_none();
                let mut judgments: std::collections::BTreeMap<String, Option<serde_json::Value>> =
                    std::collections::BTreeMap::new();
                judgments.insert(
                    "reproduced".to_string(),
                    if deep {
                        Some(serde_json::json!(y10_reproduce_one(&node.value).is_ok()))
                    } else {
                        None
                    },
                );
                judgments.insert(
                    "lineageValid".to_string(),
                    Some(serde_json::json!(lineage_ok)),
                );
                judgments.insert(
                    "signerVerdict".to_string(),
                    signer.as_ref().map(|v| serde_json::json!(v)),
                );
                judgments.insert(
                    "anchoredOk".to_string(),
                    anchored.map(|v| serde_json::json!(v)),
                );
                let (ok, _) = policy_gate::evaluate(&policy, &judgments);
                if !ok {
                    denied += 1;
                }
            }
            achieved &= push(
                &mut checks,
                "L4-게이트",
                denied == 0,
                format!("게이트 거부 {denied}/{}", nodes.len()),
            );
        }
    }
    // L5 — 정산 원장 무결·비어있지 않음. (8년 공개 "운영"은 기계 판정 밖 — 정직 명시.)
    if want >= 5 {
        let ledger_ok =
            match anchor_log::load_kind(ledger_path.expect("선검사"), settle::LEDGER_KIND) {
                Ok(l) => !l.entries.is_empty(),
                Err(_) => false,
            };
        achieved &= push(
            &mut checks,
            "L5-원장",
            ledger_ok,
            "원장 체인 무결 + 기입 1건 이상".to_string(),
        );
        checks.push(serde_json::json!({
            "id": "L5-공개(판정 밖)", "ok": serde_json::Value::Null,
            "detail": "선택적 공개 '운영'은 조직 절차라 기계 판정 밖 — 수동 확인 항목",
        }));
    }
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "level": level,
            "capsules": nodes.len(),
            "checks": checks,
            "achieved": achieved,
            "verdict": if achieved { "conformant" } else { "nonconformant" },
        }),
        "conformance",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "적합성 {level} — {} (캡슐 {})",
            if achieved {
                "conformant"
            } else {
                "nonconformant"
            },
            nodes.len()
        );
    }
    if achieved {
        EXIT_OK
    } else {
        3 // #2707: 판정 데이터 — 미달 항목은 checks 가 말한다.
    }
}
