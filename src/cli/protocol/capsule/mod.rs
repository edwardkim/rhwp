use super::*;

mod audit;
mod lineage;
mod replay;
mod signing;

pub(crate) use audit::cmd_audit;
pub(crate) use lineage::cmd_lineage;
pub(crate) use replay::cmd_replay;
pub(crate) use signing::{cmd_keygen, cmd_verify_signature};

/// [#4391] 작업 영수증 — 계획을 **임시 산출**로 재실행해 (입력·계획·산출) SHA-256
/// 3종을 발급(attest)하거나, 기대 산출 해시와 대조해 타인의 작업 주장을
/// 재현 검증(verify)한다. 전제는 실측된 바이트 결정론(같은 계획 = 같은 산출)이고,
/// 사용자 파일은 절대 건드리지 않는다 — 계획의 output 은 임시 경로로 대체된다.
pub(crate) fn replay_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub(crate) fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

pub(crate) struct ReplayScratchDir(pub(crate) std::path::PathBuf);

impl Drop for ReplayScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn replay_scratch_dir(tag: &str) -> Result<ReplayScratchDir, String> {
    #[cfg(unix)]
    use std::os::unix::fs::DirBuilderExt;

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    for attempt in 0..128_u16 {
        let candidate = std::env::temp_dir().join(format!(
            "rhwp-replay-{}-{nonce:x}-{tag}-{attempt}",
            std::process::id()
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        match builder.create(&candidate) {
            Ok(()) => return Ok(ReplayScratchDir(candidate)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("사용 가능한 임시 폴더 이름이 없습니다".to_string())
}

/// 해시한 입력 바이트를 임시 파일에 고정하고, 엔진에는 그 스냅샷만 넘긴다.
pub(crate) fn with_replay_input_snapshot<T>(
    plan: &mut serde_json::Value,
    input_bytes: &[u8],
    scratch_dir: &std::path::Path,
    execute: impl FnOnce(&serde_json::Value) -> T,
) -> Result<T, String> {
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let input = plan["input"]
        .as_str()
        .ok_or_else(|| "계획에 input 이 필요합니다".to_string())?;
    let ext = std::path::Path::new(input)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("hwp");
    let snapshot = scratch_dir.join(format!("input.{ext}"));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&snapshot).map_err(|e| e.to_string())?;
    file.write_all(input_bytes).map_err(|e| e.to_string())?;
    drop(file);
    let original_input = plan["input"].clone();
    plan["input"] = serde_json::json!(snapshot.to_string_lossy());
    let result = execute(plan);
    plan["input"] = original_input;
    Ok(result)
}

pub(crate) fn validated_capsule_plan(
    capsule: &serde_json::Value,
) -> Result<(serde_json::Value, u64), String> {
    let plan_text = capsule
        .get("planText")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "planText 없음".to_string())?;
    let expected_plan_sha = capsule["receipt"]["planSha256"]
        .as_str()
        .filter(|value| is_sha256_hex(value))
        .ok_or_else(|| "receipt.planSha256 가 없거나 64자리 16진이 아님".to_string())?;
    let actual_plan_sha = replay_sha256_hex(plan_text.as_bytes());
    if actual_plan_sha != expected_plan_sha {
        return Err("planText 와 receipt.planSha256 불일치".to_string());
    }
    let plan: serde_json::Value =
        serde_json::from_str(plan_text).map_err(|e| format!("planText JSON 파싱 실패: {e}"))?;
    if !plan.is_object() {
        return Err("planText 계획 객체 없음".to_string());
    }
    if capsule.get("plan") != Some(&plan) {
        return Err("plan 과 planText 불일치".to_string());
    }
    let steps = capsule["receipt"]["steps"]
        .as_u64()
        .ok_or_else(|| "receipt.steps 가 음이 아닌 정수가 아님".to_string())?;
    let plan_steps = plan["steps"]
        .as_array()
        .ok_or_else(|| "planText.steps/plan.steps 가 배열이 아님".to_string())?
        .len() as u64;
    if steps != plan_steps {
        return Err(
            "receipt.steps 와 planText.steps 길이 불일치 (plan.steps 길이와 receipt.steps 불일치)"
                .to_string(),
        );
    }
    Ok((plan, steps))
}

/// [#4393] replay·audit 공용 실행 코어 — 계획을 **임시 산출**로 실행해 (산출
/// SHA-256, step 수, 입력 SHA-256)를 얻는다. 임시 파일은 성공·실패 모두
/// 정리한다. 계획의 output 은 이 함수가 임시 경로로 덮어쓴다(호출자는 필요 시
/// 사전 clone).
pub(crate) fn replay_execute_to_temp(
    plan: &mut serde_json::Value,
    tag: &str,
) -> Result<(String, usize, String), (String, i32)> {
    let Some(input) = plan["input"].as_str() else {
        return Err(("계획에 input 이 필요합니다".to_string(), EXIT_USAGE));
    };
    let input_bytes = fs::read(input).map_err(|e| {
        (
            format!("입력을 읽을 수 없습니다 - {input}: {e}"),
            EXIT_RUNTIME,
        )
    })?;
    let input_sha = replay_sha256_hex(&input_bytes);
    let scratch = replay_scratch_dir(tag).map_err(|e| {
        (
            format!("재실행 전용 임시 폴더를 만들 수 없습니다 - {e}"),
            EXIT_RUNTIME,
        )
    })?;
    let ext = plan["output"]
        .as_str()
        .and_then(|o| std::path::Path::new(o).extension().and_then(|e| e.to_str()))
        .unwrap_or("hwp")
        .to_string();
    let temp_out = scratch.0.join(format!("output.{ext}"));
    plan["output"] = serde_json::json!(temp_out.to_string_lossy());
    let (engine_env, engine_code) =
        with_replay_input_snapshot(plan, &input_bytes, &scratch.0, run_plan_engine).map_err(
            |e| {
                (
                    format!("재실행 입력 스냅샷을 만들 수 없습니다 - {e}"),
                    EXIT_RUNTIME,
                )
            },
        )?;
    if engine_code != 0 {
        return Err((
            format!("계획 재실행 실패 (engine exit {engine_code})"),
            engine_code,
        ));
    }
    let bytes = match fs::read(&temp_out) {
        Ok(b) => b,
        Err(e) => {
            return Err((
                format!("재실행 산출을 읽을 수 없습니다 - {e}"),
                EXIT_RUNTIME,
            ));
        }
    };
    let steps = engine_env["steps"].as_array().map(|s| s.len()).unwrap_or(0);
    Ok((replay_sha256_hex(&bytes), steps, input_sha))
}

pub(crate) fn collect_audit_capsules(
    entries: impl IntoIterator<Item = std::io::Result<std::path::PathBuf>>,
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut capsules = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| format!("폴더 항목 읽기 실패: {e}"))?;
        let is_capsule = path
            .file_name()
            .map(|name| name.to_string_lossy().ends_with(".capsule.json"))
            .unwrap_or(false);
        if is_capsule {
            capsules.push(path);
        }
    }
    capsules.sort();
    Ok(capsules)
}
