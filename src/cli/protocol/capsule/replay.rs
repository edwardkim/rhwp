use crate::cli::protocol::*;

pub(crate) fn cmd_replay(args: &[String]) -> i32 {
    let mut plan_path: Option<&str> = None;
    let mut plan_inline: Option<&str> = None;
    let mut expected: Option<String> = None;
    let mut capsule_path: Option<String> = None;
    let mut parent_path: Option<String> = None;
    let mut sign_key_path: Option<String> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--plan-json" => {
                i += 1;
                match args.get(i) {
                    Some(v) => plan_inline = Some(v.as_str()),
                    None => {
                        eprintln!("오류: --plan-json 뒤에 계획 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--expect-output-sha256" => {
                i += 1;
                match args.get(i) {
                    Some(v) => expected = Some(v.trim().to_ascii_lowercase()),
                    None => {
                        eprintln!(
                            "오류: --expect-output-sha256 뒤에 64자리 16진 해시가 필요합니다."
                        );
                        return EXIT_USAGE;
                    }
                }
            }
            "--parent" => {
                i += 1;
                match args.get(i) {
                    Some(v) => parent_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: --parent 뒤에 부모 캡슐 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--sign-key" => {
                i += 1;
                match args.get(i) {
                    Some(v) => sign_key_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: --sign-key 뒤에 키 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--capsule" => {
                i += 1;
                match args.get(i) {
                    Some(v) => capsule_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: --capsule 뒤에 저장 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other if !other.starts_with("--") && plan_path.is_none() => plan_path = Some(other),
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
        i += 1;
    }
    if let Some(e) = expected.as_deref() {
        if e.len() != 64 || !e.bytes().all(|b| b.is_ascii_hexdigit()) {
            eprintln!("오류: --expect-output-sha256 값은 64자리 16진이어야 합니다: {e}");
            return EXIT_USAGE;
        }
    }
    if sign_key_path.is_some() && capsule_path.is_none() {
        // [#4509] 서명 대상은 캡슐 파일 바이트다 — 캡슐 없이 서명할 것이 없다.
        eprintln!("오류: --sign-key 는 --capsule 과 함께 사용합니다 (서명 대상 = 캡슐 파일).");
        return EXIT_USAGE;
    }
    let plan_text: String = match (plan_inline, plan_path) {
        (Some(inline), _) => inline.to_string(),
        (None, Some(path)) => match fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 계획을 읽을 수 없습니다 - {path}: {e}");
                return EXIT_RUNTIME;
            }
        },
        (None, None) => {
            eprintln!("사용법: rhwp replay <계획.json> [--plan-json <json>] [--expect-output-sha256 <hex>] [--json]");
            return EXIT_USAGE;
        }
    };
    let plan_sha = replay_sha256_hex(plan_text.as_bytes());
    let mut plan: serde_json::Value = match serde_json::from_str(&plan_text) {
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
    let plan_original = plan.clone();
    let (output_sha, steps, input_sha) = match replay_execute_to_temp(&mut plan, &plan_sha[..12]) {
        Ok(v) => v,
        Err((msg, code)) => {
            if json_mode {
                println!(
                    "{}",
                    provenance::marked(
                        serde_json::json!({ "schemaVersion": ENVELOPE_SCHEMA_VERSION, "error": msg }),
                        "replay",
                    )
                );
            } else {
                eprintln!("{msg} — 영수증 없음");
            }
            return code;
        }
    };
    let reproduced = expected.as_deref().map(|e| e == output_sha);
    let envelope = provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "mode": if expected.is_some() { "verify" } else { "attest" },
            "input": input,
            "inputSha256": input_sha,
            "planSha256": plan_sha,
            "outputSha256": output_sha,
            "toolVersion": rhwp::version(),
            "steps": steps,
            "reproduced": reproduced,
            "expectedOutputSha256": expected,
        }),
        "replay",
    );
    if let Some(cp) = capsule_path.as_deref() {
        // [#4393] 작업 캡슐 — 계획(원본 output 보존)+영수증의 자기완결 교환 형식.
        // [#4401] --parent 가 있으면 부모 캡슐 파일의 SHA-256 을 내장해 계보
        // 링크를 만든다 — 부모가 나중에 변조되면 lineage 가 이 해시로 폭로한다.
        let parent_link = match parent_path.as_deref() {
            Some(pp) => {
                let parent_abs = match fs::canonicalize(pp) {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!("오류: 부모 캡슐을 읽을 수 없습니다 - {pp}: {e}");
                        return EXIT_RUNTIME;
                    }
                };
                if paths_refer_to_same_file(std::path::Path::new(cp), &parent_abs) {
                    eprintln!(
                        "오류: --capsule과 --parent가 같은 기존 파일을 가리킵니다 — 부모 캡슐을 덮어쓰지 않습니다."
                    );
                    return EXIT_USAGE;
                }
                let bytes = match fs::read(&parent_abs) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!("오류: 부모 캡슐을 읽을 수 없습니다 - {pp}: {e}");
                        return EXIT_RUNTIME;
                    }
                };
                let capsule_dir = std::path::Path::new(cp)
                    .parent()
                    .filter(|path| !path.as_os_str().is_empty())
                    .unwrap_or(std::path::Path::new("."));
                let capsule_dir_abs = match fs::canonicalize(capsule_dir) {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!(
                            "오류: 캡슐 폴더를 확인할 수 없습니다 - {}: {e}",
                            capsule_dir.display()
                        );
                        return EXIT_RUNTIME;
                    }
                };
                let stored_parent = parent_abs
                    .strip_prefix(&capsule_dir_abs)
                    .map(std::path::PathBuf::from)
                    .unwrap_or(parent_abs);
                serde_json::json!({
                    "capsule": stored_parent.to_string_lossy(),
                    "sha256": replay_sha256_hex(&bytes),
                })
            }
            None => serde_json::Value::Null,
        };
        let capsule = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "kind": "workCapsule",
            "parent": parent_link,
            "plan": plan_original,
            "planText": plan_text,
            "receipt": envelope,
        });
        if let Err(e) = fs::write(
            cp,
            serde_json::to_string_pretty(&capsule).unwrap_or_default(),
        ) {
            eprintln!("오류: 캡슐 저장 실패 - {cp}: {e}");
            return EXIT_RUNTIME;
        }
        if let Some(kp) = sign_key_path.as_deref() {
            // [#4509] 분리 서명 — 방금 쓴 캡슐 "파일 바이트"를 봉인한다. 캡슐
            // 안에 서명을 넣으면 정규화 문제가 생기므로 사이드카가 규약이다.
            let (signing, key_id, _) = match capsule_sign::load_signing_key(kp) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("오류: {e}");
                    return EXIT_RUNTIME;
                }
            };
            let capsule_bytes = match fs::read(cp) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("오류: 서명 대상 캡슐 재독 실패 - {cp}: {e}");
                    return EXIT_RUNTIME;
                }
            };
            let capsule_sha = replay_sha256_hex(&capsule_bytes);
            let sidecar =
                capsule_sign::make_sidecar_json(&signing, &key_id, &capsule_sha, &capsule_bytes);
            let sc_path = capsule_sign::sidecar_path(cp);
            if let Err(e) = fs::write(
                &sc_path,
                serde_json::to_string_pretty(&sidecar).unwrap_or_default(),
            ) {
                eprintln!("오류: 서명 저장 실패 - {sc_path}: {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    if json_mode {
        println!("{envelope}");
    } else {
        println!("작업 영수증 — 입력 {input}");
        println!("  inputSha256:  {input_sha}");
        println!("  planSha256:   {plan_sha}");
        println!(
            "  outputSha256: {output_sha}  (steps {steps}, rhwp v{})",
            rhwp::version()
        );
        if let Some(r) = reproduced {
            println!("  reproduced:   {r}");
        }
    }
    match reproduced {
        Some(false) => 3, // #2707: 검증 단언 실패 — 주장된 산출과 재현 산출이 다르다.
        _ => EXIT_OK,
    }
}
