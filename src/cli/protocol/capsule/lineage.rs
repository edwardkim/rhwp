use crate::cli::protocol::*;

/// [#4401] 작업 계보 — 캡슐 해시 체인을 머리부터 거슬러 검증한다.
///
/// 3중 판정: ① 부모 파일 무결(자식이 기록한 부모 파일 SHA-256 과 실물 대조 —
/// 사후 변조는 여기서 폭로된다) ② 계보 불변식(부모의 산출 해시 == 자식의 입력
/// 해시 — "이전 작업의 산출이 다음 작업의 입력"이라는 연대기의 정의) ③ `--deep`
/// 이면 링크마다 재실행 재현까지. 판정은 봉투 데이터(valid·brokenAt·links[])이고
/// 깨진 체인은 exit 3 이다.
struct LineageOptions {
    head: String,
    deep: bool,
    keyring_path: Option<String>,
    anchor_log_path: Option<String>,
    json_mode: bool,
}

struct LineageTrace {
    links: Vec<serde_json::Value>,
    valid: bool,
    broken_at: Option<String>,
}

fn parse_lineage_options(args: &[String]) -> Result<LineageOptions, i32> {
    let mut head: Option<String> = None;
    let mut deep = false;
    let mut keyring_path = None;
    let mut anchor_log_path = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--deep" => deep = true,
            "--keyring" => {
                i += 1;
                match args.get(i) {
                    Some(v) => keyring_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: --keyring 뒤에 키 등록부 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--anchor-log" => {
                i += 1;
                match args.get(i) {
                    Some(v) => anchor_log_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: --anchor-log 뒤에 로그 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            other if !other.starts_with("--") && head.is_none() => head = Some(other.to_string()),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
        }
        i += 1;
    }
    let Some(head) = head else {
        eprintln!("사용법: rhwp lineage <캡슐.json> [--deep] [--keyring <키링.json>] [--anchor-log <로그>] [--json]");
        return Err(EXIT_USAGE);
    };
    Ok(LineageOptions {
        head,
        deep,
        keyring_path,
        anchor_log_path,
        json_mode,
    })
}

fn load_lineage_keyring(
    path: Option<&str>,
) -> Result<Option<std::collections::BTreeMap<String, capsule_sign::KeyEntry>>, i32> {
    let Some(path) = path else {
        return Ok(None);
    };
    capsule_sign::load_keyring(path).map(Some).map_err(|e| {
        eprintln!("오류: {e}");
        EXIT_RUNTIME
    })
}

fn load_anchored_set(
    path: Option<&str>,
) -> Result<Option<std::collections::BTreeSet<String>>, i32> {
    let Some(path) = path else {
        return Ok(None);
    };
    anchor_log::load(path)
        .map(|log| {
            Some(
                log.entries
                    .iter()
                    .filter_map(|entry| entry["capsuleSha256"].as_str().map(str::to_string))
                    .collect(),
            )
        })
        .map_err(|e| {
            eprintln!("오류(로그 무결): {e}");
            EXIT_RUNTIME
        })
}

fn trace_lineage(
    head: &str,
    deep: bool,
    keyring: Option<&std::collections::BTreeMap<String, capsule_sign::KeyEntry>>,
    anchored_set: Option<&std::collections::BTreeSet<String>>,
) -> Result<LineageTrace, i32> {
    let mut links: Vec<serde_json::Value> = Vec::new();
    let mut valid = true;
    let mut broken_at: Option<String> = None;
    let mut current = std::path::PathBuf::from(head);
    // 자식이 기록한 (부모 파일 해시, 자식 입력 해시) — 다음 링크에서 대조한다.
    let mut recorded_parent_sha: Option<String> = None;
    let mut child_input_sha: Option<String> = None;
    let mut guard = 0usize;
    loop {
        guard += 1;
        let name = current.display().to_string();
        if guard > 1000 {
            valid = false;
            broken_at = Some(name);
            links.push(serde_json::json!({ "error": "체인 길이 1000 초과 — 순환 의심" }));
            break;
        }
        let bytes = match fs::read(&current) {
            Ok(b) => b,
            Err(e) => {
                if links.is_empty() {
                    eprintln!("오류: 캡슐을 읽을 수 없습니다 - {name}: {e}");
                    return Err(EXIT_RUNTIME);
                }
                valid = false;
                broken_at = Some(name.clone());
                links.push(serde_json::json!({ "capsule": name, "error": format!("부모 캡슐 읽기 실패: {e}") }));
                break;
            }
        };
        let file_sha = replay_sha256_hex(&bytes);
        let capsule: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                valid = false;
                broken_at = Some(name.clone());
                links.push(
                    serde_json::json!({ "capsule": name, "error": format!("JSON 파싱 실패: {e}") }),
                );
                break;
            }
        };
        if capsule["kind"] != "workCapsule" {
            valid = false;
            broken_at = Some(name.clone());
            links.push(
                serde_json::json!({ "capsule": name, "error": "kind 가 workCapsule 이 아님" }),
            );
            break;
        }
        let Some(input_sha) = capsule["receipt"]["inputSha256"]
            .as_str()
            .filter(|value| is_sha256_hex(value))
            .map(str::to_string)
        else {
            valid = false;
            broken_at = Some(name.clone());
            links.push(serde_json::json!({
                "capsule": name,
                "error": "receipt.inputSha256 가 없거나 64자리 16진이 아님",
            }));
            break;
        };
        let Some(output_sha) = capsule["receipt"]["outputSha256"]
            .as_str()
            .filter(|value| is_sha256_hex(value))
            .map(str::to_string)
        else {
            valid = false;
            broken_at = Some(name.clone());
            links.push(serde_json::json!({
                "capsule": name,
                "error": "receipt.outputSha256 가 없거나 64자리 16진이 아님",
            }));
            break;
        };
        let (validated_plan, expected_steps) = match validated_capsule_plan(&capsule) {
            Ok(value) => value,
            Err(error) => {
                valid = false;
                broken_at = Some(name.clone());
                links.push(serde_json::json!({ "capsule": name, "error": error }));
                break;
            }
        };
        let Some(parent) = capsule.get("parent") else {
            valid = false;
            broken_at = Some(name.clone());
            links.push(serde_json::json!({
                "capsule": name,
                "error": "parent 필드 없음",
            }));
            break;
        };
        let parent_link = if parent.is_null() {
            None
        } else {
            let Some(pp) = parent["capsule"].as_str() else {
                valid = false;
                broken_at = Some(name.clone());
                links.push(serde_json::json!({ "capsule": name, "error": "parent.capsule 없음" }));
                break;
            };
            let Some(parent_sha) = parent["sha256"]
                .as_str()
                .filter(|value| is_sha256_hex(value))
            else {
                valid = false;
                broken_at = Some(name.clone());
                links.push(serde_json::json!({
                    "capsule": name,
                    "error": "parent.sha256 가 없거나 64자리 16진이 아님",
                }));
                break;
            };
            Some((pp.to_string(), parent_sha.to_string()))
        };
        let parent_ok = recorded_parent_sha.as_deref().map(|r| r == file_sha);
        let lineage_ok = child_input_sha.as_deref().map(|ci| output_sha == ci);
        let reproduced = if deep {
            let mut plan = validated_plan;
            match replay_execute_to_temp(&mut plan, &format!("lineage{guard}")) {
                Ok((actual, actual_steps, actual_input)) => Some(
                    actual == output_sha
                        && actual_input == input_sha
                        && actual_steps as u64 == expected_steps,
                ),
                Err(_) => Some(false),
            }
        } else {
            None
        };
        let mut link = serde_json::json!({
            "capsule": name,
            "inputSha256": input_sha,
            "outputSha256": output_sha,
            "parentOk": parent_ok,
            "lineageOk": lineage_ok,
            "reproduced": reproduced,
        });
        let mut signer_broken = false;
        if let Some(ring) = keyring.as_ref() {
            // 사이드카 없음 = null(미서명 — 강제는 게이트의 몫), 있는데 무효·
            // 미등록·폐기·기형 = false(깨진 계보). 읽기 실패는 없음으로 본다.
            let sc_path = format!("{}.sig.json", current.display());
            let (signer_ok, key_id) = match fs::read_to_string(&sc_path) {
                Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(sc) => {
                        let v = capsule_sign::verify_sidecar(&sc, &bytes, ring);
                        if v.verdict != "valid" {
                            signer_broken = true;
                        }
                        (
                            serde_json::json!(v.verdict == "valid"),
                            serde_json::json!(v.key_id),
                        )
                    }
                    Err(_) => {
                        signer_broken = true;
                        (serde_json::json!(false), serde_json::Value::Null)
                    }
                },
                Err(_) => (serde_json::Value::Null, serde_json::Value::Null),
            };
            link["signerOk"] = signer_ok;
            link["keyId"] = key_id;
        }
        if let Some(set) = anchored_set.as_ref() {
            // 미등재 = false 이되 체인을 깨지 않는다 — 등재 강제는 게이트(6년
            // 축)의 직무다. 판정 데이터만 싣는다.
            link["anchoredOk"] = serde_json::json!(set.contains(&file_sha));
        }
        links.push(link);
        if parent_ok == Some(false)
            || lineage_ok == Some(false)
            || reproduced == Some(false)
            || signer_broken
        {
            valid = false;
            broken_at = Some(name);
            break;
        }
        let Some((pp, parent_sha)) = parent_link else {
            break;
        };
        recorded_parent_sha = Some(parent_sha);
        child_input_sha = Some(input_sha);
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
    Ok(LineageTrace {
        links,
        valid,
        broken_at,
    })
}

fn print_lineage(head: &str, trace: &LineageTrace, json_mode: bool) {
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "head": head,
            "depth": trace.links.len(),
            "valid": trace.valid,
            "brokenAt": trace.broken_at,
            "links": trace.links,
        }),
        "lineage",
    );
    if json_mode {
        println!("{envelope}");
    } else {
        println!(
            "작업 계보 — {head}: 깊이 {} · {}",
            envelope["depth"],
            if trace.valid { "유효" } else { "깨짐" }
        );
        if let Some(broken_at) = envelope["brokenAt"].as_str() {
            println!("  brokenAt: {broken_at}");
        }
    }
}

pub(crate) fn cmd_lineage(args: &[String]) -> i32 {
    let options = match parse_lineage_options(args) {
        Ok(options) => options,
        Err(code) => return code,
    };
    // [#4509] 서명 판정은 opt-in — --keyring 없으면 signerOk 축 자체가 봉투에
    // 실리지 않아 기존 소비자가 깨지지 않는다.
    let keyring = match load_lineage_keyring(options.keyring_path.as_deref()) {
        Ok(keyring) => keyring,
        Err(code) => return code,
    };
    // [#4543] 앵커 판정도 opt-in — 로그의 등재 해시 집합을 한 번만 만든다.
    let anchored_set = match load_anchored_set(options.anchor_log_path.as_deref()) {
        Ok(anchored_set) => anchored_set,
        Err(code) => return code,
    };
    let trace = match trace_lineage(
        &options.head,
        options.deep,
        keyring.as_ref(),
        anchored_set.as_ref(),
    ) {
        Ok(trace) => trace,
        Err(code) => return code,
    };
    print_lineage(&options.head, &trace, options.json_mode);
    if trace.valid {
        EXIT_OK
    } else {
        3 // #2707: 검증 단언 실패 — 연대기가 깨졌다.
    }
}
