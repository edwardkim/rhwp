use super::supervisor::ExecutionLimits;
use rhwp::parser::pdf::{PdfImportOverrides, PdfImportPolicy, PdfImportProfile};
use serde::Deserialize;
use std::io::Read;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceFile {
    #[serde(default)]
    policy: PdfImportOverrides,
    #[serde(default)]
    execution: ExecutionOverrides,
}

fn present<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    u64::deserialize(d).map(Some)
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExecutionOverrides {
    #[serde(default, deserialize_with = "present")]
    wall_timeout_ms: Option<u64>,
    #[serde(default, deserialize_with = "present")]
    cpu_seconds: Option<u64>,
    #[serde(default, deserialize_with = "present")]
    address_space_bytes: Option<u64>,
    #[serde(default, deserialize_with = "present")]
    output_bytes: Option<u64>,
    #[serde(default, deserialize_with = "present")]
    diagnostics_bytes: Option<u64>,
}

pub(super) fn resolve(
    profile: &str,
    path: Option<&str>,
) -> Result<(PdfImportProfile, PdfImportPolicy, ExecutionLimits), String> {
    let profile: PdfImportProfile = profile.parse().map_err(|e| format!("{e}"))?;
    let policy = PdfImportPolicy::for_profile(profile);
    let mut execution = match profile {
        PdfImportProfile::Standard => ExecutionLimits::default(),
        PdfImportProfile::Constrained => ExecutionLimits::constrained(),
    };
    let Some(path) = path else {
        return Ok((profile, policy, execution));
    };
    // This is host configuration, not PDF content. Bound it independently and
    // reject malformed/duplicate keys before opening the input document.
    let file = std::fs::File::open(path)
        .map_err(|_| "PDF_RESOURCE_POLICY: 자원 설정 파일을 열 수 없습니다.".to_owned())?;
    let mut bytes = Vec::new();
    file.take(64 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "PDF_RESOURCE_POLICY: 자원 설정 파일을 읽을 수 없습니다.".to_owned())?;
    if bytes.len() > 64 * 1024 {
        return Err("PDF_RESOURCE_POLICY: 자원 설정 파일은 64 KiB 이하여야 합니다.".into());
    }
    let shape: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| "PDF_RESOURCE_POLICY: 잘못된 자원 설정 JSON입니다.".to_owned())?;
    if !shape.is_object()
        || shape
            .get("policy")
            .is_some_and(|v| !v.is_object() || v.get("backend").is_some_and(|b| !b.is_object()))
        || shape.get("execution").is_some_and(|v| !v.is_object())
    {
        return Err("PDF_RESOURCE_POLICY: 설정은 JSON 객체여야 합니다.".into());
    }
    // Parse the original bytes again so duplicate keys are not hidden by Value.
    let limits: ResourceFile = serde_json::from_slice(&bytes)
        .map_err(|_| "PDF_RESOURCE_POLICY: 잘못된 자원 설정 JSON입니다.".to_owned())?;
    let policy = limits.policy.apply_to(policy).map_err(|e| e.to_string())?;
    for (target, value) in [
        (
            &mut execution.wall_timeout_ms,
            limits.execution.wall_timeout_ms,
        ),
        (&mut execution.cpu_seconds, limits.execution.cpu_seconds),
        (
            &mut execution.address_space_bytes,
            limits.execution.address_space_bytes,
        ),
        (&mut execution.output_bytes, limits.execution.output_bytes),
        (
            &mut execution.diagnostics_bytes,
            limits.execution.diagnostics_bytes,
        ),
    ] {
        if let Some(value) = value {
            if value == 0 || value > *target {
                return Err(
                    "PDF_RESOURCE_POLICY: 실행 한도는 양수이며 선택한 프로필 이하여야 합니다."
                        .into(),
                );
            }
            *target = value;
        }
    }
    execution
        .validate()
        .map_err(|_| "PDF_RESOURCE_POLICY: 잘못된 실행 한도입니다.".to_owned())?;
    Ok((profile, policy, execution))
}
