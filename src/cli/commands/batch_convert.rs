//! State-changing batch conversion command adapter.

use std::fs;
use std::path::Path;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::cli::batch::fail_record;
use crate::{paths_refer_to_same_file, ConversionVerifyOptions};

/// [#3626] `batch convert` 의 산출 경로 — `<out-dir>/<입력 파일이름>.hwp`.
///
/// stdin 은 한 줄에 경로 하나뿐이라 출력 경로를 함께 받을 자리가 없다. 목적지는
/// `--out-dir` 하나이고 이름은 입력 파일 이름을 따른다. 이름 겹침은 batch orchestration이
/// 한 바이트도 쓰기 전에 전건 사전 점검으로 잡는다.
pub(crate) fn output_path(out_dir: &Path, input: &Path) -> std::path::PathBuf {
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    out_dir.join(format!("{stem}.hwp"))
}

/// batch convert 는 macOS/Windows 기본 파일시스템에서도 안전해야 한다. 따라서
/// 대소문자만 다른 두 입력이 같은 산출물을 덮어쓰는 일을 모든 호스트에서 미리
/// 금지한다. Linux 에서도 이 보수적 규약을 공유해야 OS를 바꾼 재실행이 달라지지 않는다.
pub(crate) fn collision_key(output: &Path) -> String {
    output.to_string_lossy().to_lowercase()
}

/// [#3626] 검증 판정 봉투가 "차이 있음"인가. 필드가 없거나 null 이면 판정 자체가 없다.
pub(crate) fn verdict_differs(record: &serde_json::Value, key: &str) -> bool {
    record
        .get(key)
        .and_then(|v| v.get("identical"))
        .and_then(|v| v.as_bool())
        == Some(false)
}

/// [#3626] `batch convert --json` 의 파일당 레코드 — 단건 `convert --json` 봉투와 같은
/// 스키마다. 쪽수 불일치면 IR 비교를 하지 않고 `verify: null` 로 두는 단락(short-circuit)
/// 까지 단건과 같다.
///
/// 다른 것은 끝내는 방식뿐이다. 단건은 검증 차이에서 `process::exit(3|4)` 로 프로세스를
/// 끊지만 배치는 뒤 파일이 남아 있어 끊을 수 없다. 그래서 판정은 레코드에만 담고
/// (`ir-diff --json` 과 같은 "판정은 데이터" 규약) `run_batch` 가 전건을 모아 집계한다.
///
/// 재파싱 실패는 "판정 불가"가 아니라 **열 수 없는 산출물**이므로, 단건이 3/4 로 끝내는
/// 것과 달리 배치가 가진 `error` 레코드 채널로 보고한다(→ 최종 exit 1). 배치에는 단건에
/// 없는 실패 채널이 있고, 이쪽이 소비자에게 더 정확하다.
pub(crate) fn record(
    path: &str,
    out_dir: &Path,
    verify_options: ConversionVerifyOptions,
) -> serde_json::Value {
    let input_path = Path::new(path);
    let output_path = output_path(out_dir, input_path);
    // 사전 점검은 산출물끼리의 겹침만 본다. "산출 경로가 곧 그 입력"(--out-dir 이 입력
    // 폴더이고 입력이 이미 .hwp)은 파일 동일성 판정이 필요하므로 여기서 막는다 —
    // 단건 convert/export-hwpx 의 "원본을 덮어쓰지 않는다" 가드와 같은 규약.
    if paths_refer_to_same_file(input_path, &output_path) {
        return fail_record(
            path,
            "입력과 출력 경로가 같습니다. 원본을 덮어쓰지 않습니다.".to_string(),
        );
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    // [#3505] --verify 비교 강도를 정하려면 원본 포맷을 알아야 한다 (대상은 항상 HWP5).
    let source_format = rhwp::parser::detect_format(&data);
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {:?}", e)),
    };

    let page_count_before = if verify_options.verify_pages {
        Some(doc.page_count())
    } else {
        None
    };
    let was_distribution = doc.document().header.distribution;
    if let Err(e) = doc.convert_to_editable_native() {
        return fail_record(path, format!("변환 실패: {:?}", e));
    }
    let export_snapshot = doc.prepare_hwp_export_snapshot();
    let verify_expected = verify_options
        .verify
        .then(|| export_snapshot.document().clone());

    let bytes = match export_snapshot.serialize() {
        Ok(b) => b,
        Err(e) => return fail_record(path, format!("직렬화 실패: {:?}", e)),
    };
    if let Err(e) = fs::write(&output_path, &bytes) {
        // [#2707] 출력 파일이 아예 안 만들어졌는데 성공 레코드를 내던 부류의 경로.
        return fail_record(
            path,
            format!("파일 저장 실패 - {}: {}", output_path.display(), e),
        );
    }

    let bytes_len = bytes.len();
    let envelope = |verify: serde_json::Value, verify_pages: serde_json::Value| {
        provenance::marked(
            serde_json::json!({
                "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                "source": path,
                "output": output_path.display().to_string(),
                "format": "hwp5",
                "bytes": bytes_len,
                "wasDistribution": was_distribution,
                // batch 는 비밀번호 옵션을 받지 않는다(run_batch 가드) — 늘 false 다.
                "passwordProtected": false,
                "verify": verify,
                "verifyPages": verify_pages,
            }),
            "convert",
        )
    };

    if !verify_options.enabled() {
        return envelope(serde_json::Value::Null, serde_json::Value::Null);
    }

    let reloaded = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("검증 실패: 저장된 HWP 재파싱 실패 - {:?}", e)),
    };

    let mut verify_pages_report = serde_json::Value::Null;
    if let Some(before) = page_count_before {
        let after = reloaded.page_count();
        verify_pages_report = serde_json::json!({
            "before": before, "after": after, "identical": before == after,
        });
        if before != after {
            // 단건 convert 와 같은 단락 — 쪽수가 다르면 IR 비교까지 가지 않는다.
            return envelope(serde_json::Value::Null, verify_pages_report);
        }
    }

    let mut verify_report = serde_json::Value::Null;
    if verify_options.verify {
        let expected = verify_expected
            .as_ref()
            .expect("verify expected snapshot must exist");
        let diff = rhwp::serializer::hwpx::roundtrip::diff_documents(expected, reloaded.document());
        // [#3505, #3930] 출처별로 대상 포맷에 표현 자리가 없는 항목만 걷어낸다.
        let diff = match source_format {
            rhwp::parser::FileFormat::Hwp => diff,
            rhwp::parser::FileFormat::Hwpx => {
                rhwp::serializer::hwpx::roundtrip::strip_hwpx_to_hwp_noise(diff)
            }
            _ => rhwp::serializer::hwpx::roundtrip::strip_cross_format_noise(diff),
        };
        verify_report = serde_json::json!({
            "identical": diff.is_empty(), "diffCount": diff.differences.len(),
        });
    }

    envelope(verify_report, verify_pages_report)
}
