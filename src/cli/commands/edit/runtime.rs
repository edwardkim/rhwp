//! Shared serialization, verification, write, and edit-CAS contracts.

use std::fs;
use std::path::Path;
use std::process;

use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::cli::integrity::sha256_hex_of;
use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// `edit` 계열 산출 형식 (#3383).
///
/// 종전에는 세 하위 명령이 모두 `export_hwp_native()` 로 HWP5 를 강제 산출했다. 그래서
/// ① HWPX 입력이 조용히 `.hwp` 로 바뀌고(형식 미보존) ② 어댑터 없는 native 경로라
/// HWPX→HWP IR 매핑(#178)조차 타지 않아 산출물에서 차트·이미지가 유실됐다.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditOutputFormat {
    Hwp,
    Hwpx,
}

impl EditOutputFormat {
    /// 기본 산출 파일의 확장자(점 제외).
    pub(crate) fn ext(self) -> &'static str {
        match self {
            EditOutputFormat::Hwp => "hwp",
            EditOutputFormat::Hwpx => "hwpx",
        }
    }

    /// JSON 봉투의 `outputFormat` 값. **`info --json` 의 `format` 과 같은 어휘**를 쓴다 —
    /// 확장자(`hwp`)가 아니라 형식 이름(`hwp5`)이라야 두 봉투를 그대로 대조할 수 있다.
    pub(crate) fn label(self) -> &'static str {
        match self {
            EditOutputFormat::Hwp => "hwp5",
            EditOutputFormat::Hwpx => "hwpx",
        }
    }
}

/// 입력 형식과 사용자가 지정한 `-o` 경로로 `edit` 산출 형식을 정한다 (#3383).
///
/// 기본은 **입력 형식 보존**이다 — HWPX 입력은 HWPX 로, 그 외(HWP5/HWP3)는 HWP5 로.
/// 예외는 하나뿐이다: HWPX 입력에 사용자가 `-o ….hwp` 를 명시한 경우. 이때는 지정한
/// **경로를 그대로 존중해** HWP5 로 저장하되(기존 스크립트 호환), 형식이 바뀐다는 사실과
/// 손실 가능성을 stderr 로 알린다(이슈 제안 2의 과도기 경고).
///
/// 반대 방향(HWP 입력에 `-o ….hwpx`)은 `edit` 의 책임이 아니다 — 형식 변환은
/// `rhwp export-hwpx` 가 담당한다. 여기서는 경고만 하고 형식을 바꾸지 않는다.
pub(crate) fn edit_output_format(
    input_bytes: &[u8],
    explicit_out: Option<&str>,
) -> EditOutputFormat {
    let source_is_hwpx = matches!(
        rhwp::parser::detect_format(input_bytes),
        rhwp::parser::FileFormat::Hwpx
    );
    let explicit_ext = explicit_out.and_then(|path| {
        Path::new(path)
            .extension()
            .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
    });

    match (source_is_hwpx, explicit_ext.as_deref()) {
        (true, Some("hwp")) => {
            eprintln!(
                "경고: 입력은 HWPX 인데 출력 확장자가 .hwp 라 HWP5 로 저장합니다 — \
                 형식 변환 과정에서 차트·이미지 등이 유실될 수 있습니다 \
                 (형식을 보존하려면 -o 를 생략하거나 .hwpx 로 지정하세요)."
            );
            EditOutputFormat::Hwp
        }
        (true, _) => EditOutputFormat::Hwpx,
        (false, Some("hwpx")) => {
            eprintln!(
                "경고: 입력이 HWPX 가 아니므로 HWP5 로 저장합니다 — 지정한 출력 확장자(.hwpx)와 \
                 실제 형식이 다릅니다 (HWPX 로 변환하려면 `rhwp export-hwpx` 를 쓰세요)."
            );
            EditOutputFormat::Hwp
        }
        (false, _) => EditOutputFormat::Hwp,
    }
}

/// 결정된 형식으로 편집 결과를 직렬화한다 (#3383).
///
/// HWP5 산출은 반드시 **어댑터 경유**(`export_hwp_with_adapter`)다. HWPX 출처 IR 을 HWP
/// 호환 형태로 옮기는 #178 어댑터를 건너뛰면 한컴 호환성과 이미지·차트가 깨진다.
/// [#3702] 편집 저장본 자기검증 — 편집 후 IR 과 저장본 재파싱 IR 을 내부 대조한다.
/// 반환: (verify 봉투 값, exit 3 여부). 비교기는 diff_documents 재사용(신규 로직 없음).
/// HWPX 소스→HWP5 산출은 #3505/#3930 출처 전용 노이즈 제거를 승계한다.
pub(crate) fn edit_verify_report(
    doc: &rhwp::wasm_api::HwpDocument,
    out_bytes: &[u8],
    source_is_hwpx: bool,
) -> (serde_json::Value, bool) {
    let reloaded = match rhwp::wasm_api::HwpDocument::from_bytes(out_bytes) {
        Ok(d) => d,
        Err(e) => {
            return (
                serde_json::json!({ "identical": false, "diffCount": null, "reparseError": e.to_string() }),
                true,
            );
        }
    };
    let diff =
        rhwp::serializer::hwpx::roundtrip::diff_documents(doc.document(), reloaded.document());
    let diff = if source_is_hwpx {
        rhwp::serializer::hwpx::roundtrip::strip_hwpx_to_hwp_noise(diff)
    } else {
        diff
    };
    if diff.is_empty() {
        (
            serde_json::json!({ "identical": true, "diffCount": 0 }),
            false,
        )
    } else {
        (
            serde_json::json!({ "identical": false, "diffCount": diff.differences.len() }),
            true,
        )
    }
}

pub(crate) fn edit_serialize(
    doc: &mut rhwp::wasm_api::HwpDocument,
    format: EditOutputFormat,
) -> Result<Vec<u8>, String> {
    match format {
        EditOutputFormat::Hwpx => doc.export_hwpx_native(),
        EditOutputFormat::Hwp => doc.export_hwp_with_adapter(),
    }
    .map_err(|e| e.to_string())
}

/// `edit_serialize` 와 같은 바이트를 내되 **IR 을 건드리지 않는다**.
///
/// 무상태 CLI 는 저장 직후 프로세스가 끝나므로 어댑터가 살아 있는 IR 을 정규화해도
/// 관측되지 않는다. 세션 핸들은 다르다 — 도구 계약이 "핸들은 저장 후에도 열려 있다"
/// 이므로 저장은 스냅숏이어야 한다. 그래서 세션 경로만 이쪽을 쓰고 CLI 의 `&mut`
/// 경로는 그대로 둔다(CLI 에 문서 1회 clone 비용을 지우지 않는다).
pub(crate) fn edit_serialize_snapshot(
    doc: &rhwp::wasm_api::HwpDocument,
    format: EditOutputFormat,
) -> Result<Vec<u8>, String> {
    match format {
        EditOutputFormat::Hwpx => doc.export_hwpx_native(),
        EditOutputFormat::Hwp => doc.export_hwp_with_adapter_snapshot(),
    }
    .map_err(|e| e.to_string())
}

/// 기대 해시가 주어졌을 때만 검사한다. 형식 오류는 exit 2, 불일치는 exit 3 을
/// 돌려주고 봉투/진단을 직접 낸다. `None` 이면 통과.
pub(crate) fn check_expect_sha256(
    expect: Option<&str>,
    bytes: &[u8],
    source: &str,
    json_mode: bool,
) -> Option<i32> {
    let expect = expect?;
    let normalized = expect.trim().to_ascii_lowercase();
    if normalized.len() != 64 || !normalized.bytes().all(|b| b.is_ascii_hexdigit()) {
        eprintln!("오류: --expect-sha256 값은 64자리 16진이어야 합니다: {expect}");
        return Some(EXIT_USAGE);
    }
    let actual = sha256_hex_of(bytes);
    if actual == normalized {
        return None;
    }
    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": source,
            "preconditionFailed": {
                "kind": "inputSha256",
                "expected": normalized,
                "actual": actual,
            },
            "error": "입력 문서가 기대 해시와 다릅니다 — 다른 에이전트/사람이 먼저 바꿨을 수 있습니다. 문서를 다시 읽고 계획을 재수립하세요 (#3905 CAS).",
        });
        println!("{envelope}");
    } else {
        eprintln!("검증 실패: 입력 해시 불일치 (기대 {normalized} / 실제 {actual}) — 저장하지 않았습니다.");
    }
    Some(3) // #2707: 검증 단언 실패
}

/// 편집 명령 공통 저장·봉투. 호출부가 코어 변이를 끝낸 뒤에만 부른다.
pub(crate) fn finish_edit_write(
    doc: &mut rhwp::wasm_api::HwpDocument,
    bytes: &[u8],
    file_path: &str,
    out_path: Option<String>,
    suffix: &str,
    dry_run: bool,
    json_mode: bool,
    verify_mode: bool,
    mut extra: serde_json::Value,
    changed_paras: &[(usize, usize)],
    dry_msg: &str,
    ok_msg: &str,
) -> i32 {
    let out_format = edit_output_format(bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_{}.{}", stem, suffix, out_format.ext())
    });
    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = match edit_serialize(doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = fs::write(&output_path, &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }
    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(changed_paras) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };
    if json_mode {
        extra["schemaVersion"] = serde_json::json!(ENVELOPE_SCHEMA_VERSION);
        extra["source"] = serde_json::json!(file_path);
        extra["dryRun"] = serde_json::json!(dry_run);
        extra["changedPages"] = changed_pages;
        if !dry_run {
            extra["output"] = serde_json::Value::String(output_path.clone());
            extra["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            extra["verify"] = verify_report;
        }
        println!("{}", extra);
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }
    if dry_run {
        println!("{dry_msg}");
    } else {
        println!("{ok_msg} → {output_path}");
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}
