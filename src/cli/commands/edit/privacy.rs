//! Privacy redaction and metadata sanitization command adapters.

use std::fs;
use std::path::Path;
use std::process;

use rhwp::document_core::queries::pii_scan::PiiKind;
use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use super::runtime::{edit_output_format, edit_serialize, edit_verify_report, EditOutputFormat};
use crate::{atomic_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

// ─── [#3719 §6-11] 공개 전 정리 — edit redact / edit sanitize ───

/// `-o` 도 `--in-place` 도 없이 원본을 덮어쓰려 할 때의 거부 메시지.
///
/// 마스킹은 되돌릴 수 없다. "실수로 원본을 잃는" 경로를 아예 만들지 않기 위해,
/// 산출 경로를 **명시하지 않으면 실행하지 않는다**(다른 edit 명령의 `_replaced` 류
/// 기본 이름조차 만들지 않는다 — 어디에 무엇이 생겼는지 모른 채로 두지 않기 위해).
const REDACT_DESTINATION_REQUIRED: &str = "오류: 마스킹은 되돌릴 수 없습니다. \
     산출 경로를 -o <출력> 으로 지정하거나, 원본을 덮어쓸 의도라면 --in-place 를 \
     명시하세요 (먼저 --dry-run 으로 무엇이 지워질지 확인하기를 권합니다).";

struct RedactArgs<'a> {
    file_path: &'a str,
    out_path: Option<String>,
    kinds: Vec<PiiKind>,
    mask_char: char,
    dry_run: bool,
    json_mode: bool,
    verify_mode: bool,
    in_place: bool,
    no_raw: bool,
}

/// `edit redact` — 개인정보를 찾아 자릿수를 유지한 채 마스킹한다.
///
/// 탐지는 [`rhwp::document_core::queries::pii_scan`] 의 읽기 전용 판정을 쓰고, 실제
/// 변경은 검증된 치환 경로(`replace_all_native`)를 재사용한다 — 새 편집 로직이 없다.
/// 되돌릴 수 없는 작업이라 ① `--dry-run` 이 권장 흐름이고 ② 산출 경로를 명시하지
/// 않으면 exit 2 로 거부한다.
pub(super) fn edit_redact(args: &[String]) -> i32 {
    match parse_redact_args(args) {
        Ok(parsed) => execute_redact(parsed),
        Err(code) => code,
    }
}

fn parse_redact_args(args: &[String]) -> Result<RedactArgs<'_>, i32> {
    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut kinds: Vec<PiiKind> = Vec::new();
    let mut mask_char: char = '*';
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut in_place = false;
    let mut no_raw = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--kind" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: --kind 뒤에 ssn|phone|email|card|all 이 필요합니다.");
                    return Err(EXIT_USAGE);
                };
                for token in value.split(',').map(str::trim).filter(|t| !t.is_empty()) {
                    if token == "all" {
                        kinds.extend(PiiKind::all());
                        continue;
                    }
                    match PiiKind::parse(token) {
                        Some(k) => kinds.push(k),
                        None => {
                            eprintln!(
                                "오류: 알 수 없는 --kind 값 - {token} (ssn|phone|email|card|all)"
                            );
                            return Err(EXIT_USAGE);
                        }
                    }
                }
            }
            "--mask" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: --mask 뒤에 마스킹 문자 한 글자가 필요합니다.");
                    return Err(EXIT_USAGE);
                };
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    // 두 글자 이상이면 자릿수 보존이 깨진다 — 조용히 자르지 않고 거부한다.
                    (Some(c), None) if !c.is_alphanumeric() => mask_char = c,
                    (Some(_), None) => {
                        eprintln!("오류: --mask 는 영숫자가 아닌 문자여야 합니다 (예: * # ●).");
                        return Err(EXIT_USAGE);
                    }
                    _ => {
                        eprintln!("오류: --mask 는 정확히 한 글자여야 합니다 (자릿수 보존).");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--in-place" => in_place = true,
            "--dry-run" => dry_run = true,
            "--verify" => verify_mode = true,
            "--json" => json_mode = true,
            "--no-raw" => no_raw = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return Err(EXIT_USAGE);
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp edit redact <파일.hwp|파일.hwpx> [--kind ssn|phone|email|card|all] [--mask <문자>] [--dry-run] [--no-raw] [--verify] [-o <출력>|--in-place] [--json]"
        );
        return Err(EXIT_USAGE);
    };
    if kinds.is_empty() {
        kinds.extend(PiiKind::all());
    }
    kinds.sort_unstable();
    kinds.dedup();

    if out_path.is_some() && in_place {
        eprintln!("오류: -o 와 --in-place 는 함께 쓸 수 없습니다 (산출 경로가 모호합니다).");
        return Err(EXIT_USAGE);
    }
    // 원본 보호 — 산출 경로가 없는 실제 실행은 거부한다(--dry-run 은 아무것도 쓰지 않음).
    if !dry_run && out_path.is_none() && !in_place {
        eprintln!("{REDACT_DESTINATION_REQUIRED}");
        return Err(EXIT_USAGE);
    }
    // -o 로 원본을 지목한 경우도 같은 사고다 — 의도를 --in-place 로 말하게 한다.
    if let Some(out) = out_path.as_deref() {
        if !in_place && same_existing_path(file_path, out) {
            eprintln!("{REDACT_DESTINATION_REQUIRED}");
            return Err(EXIT_USAGE);
        }
    }

    Ok(RedactArgs {
        file_path,
        out_path,
        kinds,
        mask_char,
        dry_run,
        json_mode,
        verify_mode,
        in_place,
        no_raw,
    })
}

fn execute_redact(args: RedactArgs<'_>) -> i32 {
    let RedactArgs {
        file_path,
        out_path,
        kinds,
        mask_char,
        dry_run,
        json_mode,
        verify_mode,
        in_place,
        no_raw,
    } = args;

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let findings = doc.scan_pii(&kinds, mask_char);
    let changed_paras: Vec<(usize, usize)> = {
        let mut v: Vec<(usize, usize)> =
            findings.iter().map(|f| (f.section, f.paragraph)).collect();
        v.sort_unstable();
        v.dedup();
        v
    };

    // 치환은 값 단위 전량이다. 긴 값을 먼저 바꿔야 짧은 값이 긴 값의 부분열일 때
    // 원문을 깨뜨리지 않는다.
    let mut targets: Vec<(String, String)> = Vec::new();
    for f in &findings {
        if !targets.iter().any(|(raw, _)| *raw == f.raw) {
            targets.push((f.raw.clone(), f.masked.clone()));
        }
    }
    targets.sort_by(|a, b| b.0.chars().count().cmp(&a.0.chars().count()));

    let mut redacted_count = 0usize;
    if !dry_run {
        for (raw, masked) in &targets {
            match doc.replace_all_native(raw, masked, true) {
                Ok(result) => {
                    redacted_count += serde_json::from_str::<serde_json::Value>(&result)
                        .ok()
                        .and_then(|v| v["count"].as_u64())
                        .unwrap_or(0) as usize;
                }
                Err(e) => {
                    // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
                    eprintln!("오류: 마스킹 실패 - {:?}", e);
                    return EXIT_RUNTIME;
                }
            }
        }
    }

    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = match (&out_path, in_place) {
        (Some(p), _) => p.clone(),
        (None, true) => file_path.to_string(),
        // 여기 도달하려면 dry-run 이다 — 산출 경로를 쓰지 않는다.
        (None, false) => String::new(),
    };

    // 탐지 0건이면 무변경이다 — 산출물을 만들지 않는다(원본을 그대로 두는 편이 안전하다).
    let wrote_output = !dry_run && redacted_count > 0;
    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if wrote_output {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
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
        if let Err(e) = atomic_file::write_atomically(Path::new(&output_path), &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    let changed_pages = if wrote_output {
        match doc.pages_covering_paragraphs(&changed_paras) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    } else {
        serde_json::Value::Null
    };

    if json_mode {
        // --no-raw: findings[].raw(원문 개인정보)를 봉투에서 아예 뺀다. `null`로 채우지
        // 않는 이유 — 이 코드베이스는 "선택적으로 없을 수 있는 필드"를 스키마 차원에서
        // 생략으로 표현한다(PiiFinding.page 의 skip_serializing_if 가 같은 관례). raw 를
        // null 로 두면 소비자가 "탐지는 됐지만 값이 비었다"와 "일부러 뺐다"를 구분할
        // 근거가 없어지고, jq 같은 파이프라인에서 null 이 그대로 로그에 찍혀 새 유출
        // 경로가 될 수 있다. 필드 자체가 없으면 그 위험이 구조적으로 사라진다.
        let mut findings_value =
            serde_json::to_value(&findings).unwrap_or(serde_json::Value::Array(Vec::new()));
        if no_raw {
            if let serde_json::Value::Array(items) = &mut findings_value {
                for item in items.iter_mut() {
                    if let serde_json::Value::Object(obj) = item {
                        obj.remove("raw");
                    }
                }
            }
        }
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "kinds": kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
            "mask": mask_char.to_string(),
            "dryRun": dry_run,
            "inPlace": in_place,
            "noRaw": no_raw,
            "findingCount": findings.len(),
            "findings": findings_value,
            "redactedCount": redacted_count,
            "changedPages": changed_pages,
        });
        if wrote_output {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        // [#3885] findings[].raw 는 마스킹 전 원문 — 개인정보 그 자체다. 가장 민감한
        // 값을 싣는 봉투가 출처 표지 없이 나가면 S1 계약("표지는 항상 실린다")이
        // 정확히 그 지점에서 무너진다. --no-raw 면 raw 경로가 봉투에 없으므로
        // 표지도 masked 만 선언한다(실재 경로 필터).
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "마스킹 예정: {} — 탐지 {}건 (원문 {}개). 실제 적용은 -o 또는 --in-place.",
            file_path,
            findings.len(),
            targets.len()
        );
        for f in &findings {
            // --no-raw 는 --json 뿐 아니라 이 사람용 출력에도 적용한다 — 콘솔 로그·
            // 터미널 스크롤백도 유출 경로이므로 절반만 가려서는 목적을 달성하지 못한다.
            let shown_raw: &str = if no_raw {
                "(생략됨, --no-raw)"
            } else {
                &f.raw
            };
            println!(
                "  [{}] {} → {} (구역 {}, 문단 {}, 쪽 {})",
                f.kind,
                shown_raw,
                f.masked,
                f.section,
                f.paragraph,
                f.page
                    .map(|p| (p + 1).to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
        }
    } else if redacted_count == 0 {
        println!("마스킹 0건: {} — 탐지 없음 (출력 파일 미생성)", file_path);
    } else {
        println!(
            "마스킹 완료: {} → {} — {}건",
            file_path, output_path, redacted_count
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

/// 두 경로가 **이미 존재하는 같은 파일**을 가리키는지. 판정 불가면 `false`.
///
/// 산출 경로는 대개 존재하지 않으므로 정규화가 실패하는 것이 정상이다. 여기서
/// 잡으려는 것은 `-o` 로 원본 자신을 지목한 경우 하나뿐이다.
fn same_existing_path(a: &str, b: &str) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// `\u{5}HwpSummaryInformation` 에서 지울 속성 — `(PID, 봉투 필드 이름)`.
///
/// PID 는 HWP5 사양의 `HWPPIDSI_*` 다. 본문과 무관한 작성자·이력 메타만 고른다.
const SUMMARY_TARGETS: [(u32, &str); 11] = [
    (0x02, "title"),
    (0x03, "subject"),
    (0x04, "author"),
    (0x05, "keywords"),
    (0x06, "comments"),
    (0x08, "lastSavedBy"),
    (0x09, "revisionNumber"),
    (0x0B, "lastPrintedAt"),
    (0x0C, "createdAt"),
    (0x0D, "lastSavedAt"),
    (0x14, "dateString"),
];

/// FILETIME(1601-01-01 UTC 기준 100ns) → `YYYY-MM-DDTHH:MM:SSZ`.
///
/// 감사 기록용이다 — 무엇을 지웠는지 사람이 읽을 수 있어야 "조용히 지우지 않았다"가
/// 성립한다.
fn filetime_to_iso(ft: u64) -> String {
    const SECS_1601_TO_1970: i64 = 11_644_473_600;
    let secs = (ft / 10_000_000) as i64 - SECS_1601_TO_1970;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Howard Hinnant, civil_from_days (proleptic Gregorian).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + i64::from(m <= 2);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// `\u{5}HwpSummaryInformation`(OLE 속성 집합)에서 작성자·이력 메타를 지운다.
///
/// **바이트 길이를 바꾸지 않는다** — 속성 오프셋 표가 절대 위치를 담고 있어 크기를
/// 줄이면 나머지 속성이 전부 어긋난다. 문자열은 `cch=1`(NUL 하나)로 만들고 남은
/// 자리를 0으로 덮으며, FILETIME 은 0(미설정)으로 만든다.
///
/// 반환: `(필드 이름, 지우기 전 값)` 목록. 형식을 해석하지 못하면 빈 목록(무변경).
fn sanitize_summary_information(data: &mut [u8]) -> Vec<(String, String)> {
    fn u32_at(d: &[u8], off: usize) -> Option<u32> {
        d.get(off..off + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    let mut removed: Vec<(String, String)> = Vec::new();
    if data.len() < 48 || data[0] != 0xFE || data[1] != 0xFF {
        return removed;
    }
    let Some(section_off) = u32_at(data, 44).map(|v| v as usize) else {
        return removed;
    };
    let Some(count) = u32_at(data, section_off + 4).map(|v| v as usize) else {
        return removed;
    };
    // 병적으로 큰 개수는 해석을 포기한다(손상 파일에서 헛돌지 않게).
    if count > 4096 || section_off + 8 + count * 8 > data.len() {
        return removed;
    }

    for idx in 0..count {
        let pair = section_off + 8 + idx * 8;
        let (Some(pid), Some(rel)) = (u32_at(data, pair), u32_at(data, pair + 4)) else {
            continue;
        };
        let Some((_, field)) = SUMMARY_TARGETS.iter().find(|(p, _)| *p == pid) else {
            continue;
        };
        let abs = section_off + rel as usize;
        let Some(vt) = u32_at(data, abs) else {
            continue;
        };
        match vt {
            // VT_LPWSTR — UTF-16LE, cch 는 종단 NUL 을 포함한 문자 수.
            0x1F => {
                let Some(cch) = u32_at(data, abs + 4).map(|v| v as usize) else {
                    continue;
                };
                let start = abs + 8;
                let Some(raw) = data.get(start..start + cch * 2) else {
                    continue;
                };
                let units: Vec<u16> = raw
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .take_while(|u| *u != 0)
                    .collect();
                if units.is_empty() {
                    continue;
                }
                removed.push((field.to_string(), String::from_utf16_lossy(&units)));
                data[start..start + cch * 2].fill(0);
                data[abs + 4..abs + 8].copy_from_slice(&1u32.to_le_bytes());
            }
            // VT_FILETIME.
            0x40 => {
                let Some(raw) = data.get(abs + 4..abs + 12) else {
                    continue;
                };
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(raw);
                let value = u64::from_le_bytes(bytes);
                if value == 0 {
                    continue;
                }
                removed.push((field.to_string(), filetime_to_iso(value)));
                data[abs + 4..abs + 12].fill(0);
            }
            _ => {}
        }
    }
    removed
}

/// HWPX `Contents/content.hpf` 의 `<opf:metadata>` 블록을 중립 블록으로 바꾼다.
///
/// 이 블록은 직렬화기가 원본에서 그대로 splice 하는 유일한 저작자 정보 경로다
/// (`serializer::hwpx::content::write_content_hpf`). 지우지 않으면 HWPX 산출물에
/// 작성자·작성일이 그대로 남는다. 반환: 지우기 전 블록(있었을 때만).
fn sanitize_hwpx_metadata(entry: &mut Vec<u8>) -> Option<String> {
    const NEUTRAL: &str =
        "<opf:metadata><opf:title/><opf:language>ko</opf:language></opf:metadata>";
    let text = String::from_utf8(entry.clone()).ok()?;
    let open = text.find("<opf:metadata>")?;
    let close = text[open..].find("</opf:metadata>")? + open + "</opf:metadata>".len();
    let before = text[open..close].to_string();
    if before == NEUTRAL {
        return None;
    }
    let mut rebuilt = String::with_capacity(text.len());
    rebuilt.push_str(&text[..open]);
    rebuilt.push_str(NEUTRAL);
    rebuilt.push_str(&text[close..]);
    *entry = rebuilt.into_bytes();
    Some(before)
}

/// 본문 문단 텍스트를 공백·제어문자를 뺀 한 줄로 잇는다 (미리보기 대조용).
///
/// `serializer::cfb_writer::build_preview_text` 와 같은 범위(본문 문단만, 표·글상자 제외).
fn body_text_signature(document: &rhwp::model::document::Document) -> String {
    const MAX: usize = 4000;
    let mut out = String::new();
    for section in &document.sections {
        for para in &section.paragraphs {
            out.extend(
                para.text
                    .chars()
                    .filter(|c| !c.is_whitespace() && !c.is_control()),
            );
            if out.chars().count() >= MAX {
                return out;
            }
        }
    }
    out
}

/// 미리보기 텍스트가 **지금 본문**의 앞부분과 같은지.
///
/// 같으면 유출이 아니라 본문의 파생물이다(저장 시 어차피 같은 값이 다시 만들어진다).
/// 다르면 예전 판의 잔재 — 본문에서 지운 문장이 미리보기에만 남아 있는 전형적 사고다.
fn preview_text_is_current(preview: &str, body_signature: &str) -> bool {
    let stripped: String = preview
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect();
    stripped.is_empty() || body_signature.starts_with(&stripped)
}

/// `edit sanitize` — 문서 메타데이터를 제거한다 (본문은 건드리지 않는다).
///
/// 작성자·회사·최종수정자·작성일과 미리보기(PrvText/PrvImage)를 지운다. 무엇을
/// 지웠는지 `removed[]` 로 남긴다 — 조용히 지우면 감사할 수 없다.
pub(super) fn edit_sanitize(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut keep_preview = false;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--keep-preview" => keep_preview = true,
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp edit sanitize <파일.hwp|파일.hwpx> [--keep-preview] [-o <출력>] [--json]"
        );
        return EXIT_USAGE;
    };

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let out_format = edit_output_format(&bytes, out_path.as_deref());
    // HWPX 원본의 `/HwpSummaryInformation` 은 **파일에 없던** 계약 fallback 상수다
    // (`parser::hwpx::contract_streams`). HWPX 로 저장하면 산출물에도 실리지 않으므로
    // 손대지 않는다 — 없던 것을 지웠다고 보고하면 감사 기록이 거짓이 된다. HWP5 로
    // 변환할 때만 실제 산출물에 들어가므로 그때는 지운다.
    let source_is_hwpx = matches!(
        rhwp::parser::detect_format(&bytes),
        rhwp::parser::FileFormat::Hwpx
    );
    let touch_summary = !(source_is_hwpx && out_format == EditOutputFormat::Hwpx);

    let mut removed: Vec<(String, String)> = Vec::new();
    {
        let document = doc.document_mut();

        // ① OLE 요약 정보 (HWP5 원본 · HWPX→HWP5 변환 계약 스트림).
        if touch_summary {
            for (path, data) in document.extra_streams.iter_mut() {
                if !path
                    .trim_start_matches(['/', '\u{5}'])
                    .eq_ignore_ascii_case("HwpSummaryInformation")
                {
                    continue;
                }
                removed.extend(sanitize_summary_information(data));
            }
        }

        // ② HWPX 저작자 메타(content.hpf 의 opf:metadata splice 경로).
        for (path, entry) in document.hwpx_aux_entries.iter_mut() {
            if path != "Contents/content.hpf" {
                continue;
            }
            if let Some(before) = sanitize_hwpx_metadata(entry) {
                removed.push(("hwpx.metadata".to_string(), before));
            }
        }

        // ③ 미리보기 — 예전 판의 잔재가 남는 자리다. 본문에서 이미 지운 문장이
        //    미리보기에만 남아 공개되는 사고가 이 명령의 존재 이유 중 하나다.
        //    지금 본문과 같은 미리보기는 파생물이므로 보고하지 않는다(저장 시 재생성).
        let body_signature = body_text_signature(document);

        if let Some(preview) = document.preview.as_mut() {
            let stale = preview
                .text
                .as_deref()
                .is_some_and(|t| !preview_text_is_current(t, &body_signature));
            if stale {
                if let Some(text) = preview.text.take() {
                    removed.push((
                        "preview.text".to_string(),
                        text.chars().take(60).collect::<String>(),
                    ));
                }
            }
            if !keep_preview {
                if let Some(image) = preview.image.take() {
                    removed.push((
                        "preview.image".to_string(),
                        format!("{:?} {} bytes", image.format, image.data.len()),
                    ));
                }
            }
        }
        if document
            .preview
            .as_ref()
            .is_some_and(|p| p.text.is_none() && p.image.is_none())
        {
            document.preview = None;
        }

        // HWPX 컨테이너의 미리보기 — ZIP 엔트리(HWPX 산출용)와 계약 스트림
        // (HWPX→HWP5 변환용)은 같은 것의 두 표현이므로 함께 지우고 한 번만 보고한다.
        let hwpx_preview_text = document
            .hwpx_aux_entry("Preview/PrvText.txt")
            .and_then(|b| std::str::from_utf8(b).ok())
            .map(str::to_string);
        let drop_hwpx_text = hwpx_preview_text
            .as_deref()
            .is_some_and(|t| !preview_text_is_current(t, &body_signature));
        if drop_hwpx_text {
            if let Some(text) = hwpx_preview_text {
                removed.push((
                    "preview.text".to_string(),
                    text.chars().take(60).collect::<String>(),
                ));
            }
        }
        // 직렬화기는 엔트리가 없으면 빈 자리표시자를 넣는다. 이미 자리표시자면
        // 지울 것이 없다 — 반복 실행이 매번 "지웠다"고 보고하지 않게 한다.
        let drop_hwpx_image = !keep_preview
            && document
                .hwpx_aux_entry("Preview/PrvImage.png")
                .is_some_and(|b| b != rhwp::serializer::hwpx::static_assets::PRV_IMAGE_PNG);
        if drop_hwpx_image {
            if let Some(bytes) = document.hwpx_aux_entry("Preview/PrvImage.png") {
                removed.push((
                    "preview.image".to_string(),
                    format!("Png {} bytes", bytes.len()),
                ));
            }
        }
        document.hwpx_aux_entries.retain(|(path, _)| {
            !(path == "Preview/PrvText.txt" && drop_hwpx_text)
                && !(path == "Preview/PrvImage.png" && drop_hwpx_image)
        });
        document.extra_streams.retain(|(path, _)| {
            !(path == "/PrvText" && drop_hwpx_text) && !(path == "/PrvImage" && !keep_preview)
        });
    }

    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_sanitized.{}", stem, out_format.ext())
    });

    let out_bytes = match edit_serialize(&mut doc, out_format) {
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
    if let Err(e) = atomic_file::write_atomically(Path::new(&output_path), &out_bytes) {
        eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
        return EXIT_RUNTIME;
    }

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "keepPreview": keep_preview,
            "removedCount": removed.len(),
            "removed": removed
                .iter()
                .map(|(field, before)| serde_json::json!({ "field": field, "before": before }))
                .collect::<Vec<_>>(),
            "output": output_path,
            "outputFormat": out_format.label(),
        });
        // [#3885] removed[].before 는 지워진 문서 속성 원문이다 — 제목·작성자에
        // 더해 preview.text 는 본문 첫 화면 발췌라 문서 문장이 통째로 실린다.
        println!("{}", provenance::marked(envelope, "edit"));
        return EXIT_OK;
    }

    println!(
        "메타 제거 완료: {} → {} — {}건",
        file_path,
        output_path,
        removed.len()
    );
    for (field, before) in &removed {
        println!("  {field}: {before}");
    }
    EXIT_OK
}
