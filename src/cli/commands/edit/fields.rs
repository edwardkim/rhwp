//! Field targeting and form-fill command adapter.

use std::fs;
use std::path::Path;
use std::process;

use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use super::runtime::{edit_output_format, edit_serialize, edit_verify_report, EditOutputFormat};
use crate::{load_document, LoadError, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn parse_field_key(key: &str) -> (&str, usize) {
    let Some(open) = key.rfind('[') else {
        return (key, 0);
    };
    if !key.ends_with(']') {
        return (key, 0);
    }
    let inner = &key[open + 1..key.len() - 1];
    match inner.parse::<usize>() {
        Ok(n) => (&key[..open], n),
        // 색인으로 해석되지 않으면 이름의 일부로 둔다 — 대괄호가 든 이름을 깨뜨리지 않는다.
        Err(_) => (key, 0),
    }
}

/// [#3762] `export-ir-schema` — 공개 IR 의 JSON Schema 를 낸다 (M18 바인딩 착수 조건).
///
/// 문서를 입력으로 받지 않는다 — 스키마는 **타입의 자기서술**이지 특정 문서의
/// 속성이 아니다. capabilities 가 명령 표면을 설명하듯, 이 명령은 문서 모델을
/// 설명한다. 외부 바인딩 세대가 코드 생성의 단일 출처로 쓴다.
pub(super) fn edit_fill_fields(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut data_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    // [#3702] 저장 직후 자기검증 — 판정은 데이터, 차이 시 exit 3.
    let mut verify_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data" => {
                i += 1;
                match args.get(i) {
                    Some(v) => data_arg = Some(v),
                    None => {
                        eprintln!("오류: --data 뒤에 JSON 또는 @파일경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
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
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => file_path = Some(other),
        }
        i += 1;
    }

    let (Some(file_path), Some(data_arg)) = (file_path, data_arg) else {
        eprintln!("사용법: rhwp edit fill-fields <파일.hwp|파일.hwpx> --data <JSON|@파일> [-o <출력>] [--dry-run] [--json]");
        return EXIT_USAGE;
    };

    // `@경로` 면 파일에서 읽는다 — 대량 메일머지에서 셸 인용 지옥을 피한다.
    let data_text = if let Some(path) = data_arg.strip_prefix('@') {
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("오류: --data 파일을 읽을 수 없습니다 - {}: {}", path, e);
                return EXIT_RUNTIME;
            }
        }
    } else {
        data_arg.to_string()
    };

    let data: serde_json::Map<String, serde_json::Value> =
        match serde_json::from_str::<serde_json::Value>(&data_text) {
            Ok(serde_json::Value::Object(m)) => m,
            Ok(_) => {
                eprintln!("오류: --data 는 {{\"필드이름\":\"값\"}} 형식의 JSON 객체여야 합니다.");
                return EXIT_USAGE;
            }
            Err(e) => {
                eprintln!("오류: --data JSON 파싱 실패 - {}", e);
                return EXIT_USAGE;
            }
        };

    let outcome = match fill_fields_core(file_path, &data, out_path, dry_run, verify_mode) {
        Ok(o) => o,
        Err(message) => {
            eprintln!("오류: {message}");
            return EXIT_RUNTIME;
        }
    };
    let FillOutcome {
        envelope,
        output_path,
        verify_failed,
        ..
    } = outcome;

    if json_mode {
        println!("{envelope}");
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    let empty: Vec<serde_json::Value> = Vec::new();
    let filled = envelope["filled"].as_array().unwrap_or(&empty);
    let not_found: Vec<&str> = envelope["notFound"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    let confusable = envelope["confusable"].as_array().unwrap_or(&empty);

    if dry_run {
        println!("변경 예정: {} (필드 {}개)", file_path, filled.len());
    } else {
        println!(
            "채우기 완료: {} → {} (필드 {}개)",
            file_path,
            output_path,
            filled.len()
        );
    }
    for f in filled {
        println!(
            "  {} = {:?}",
            f["name"].as_str().unwrap_or(""),
            f["value"].as_str().unwrap_or("")
        );
    }
    if !not_found.is_empty() {
        println!("  문서에 없는 필드 이름: {}", not_found.join(", "));
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    for c in confusable {
        // 사람에게도 알린다 — 화면상 같은 이름이라 눈으로는 잡을 수 없는 축이다.
        eprintln!(
            "경고: '{}' 과(와) 화면상 구별되지 않는 이름의 누름틀이 문서에 함께 있습니다 \
             — 채운 칸이 의도한 칸인지 확인하세요.",
            c["name"].as_str().unwrap_or("")
        );
    }
    EXIT_OK
}

/// [#3719 §6-6] `edit fill-fields`(단건)와 `batch fill`(메일머지)이 공유하는 채움 결과.
pub(crate) struct FillOutcome {
    /// `edit fill-fields --json` 봉투 그대로. 배치 레코드는 여기에 `row` 만 더한다 —
    /// 소비자가 단건과 배치를 같은 코드로 읽게 하기 위함(기존 batch 축 규약).
    pub(crate) envelope: serde_json::Value,
    /// 산출 경로. `--dry-run` 이면 **만들 예정** 경로다(디스크에 파일은 없다).
    output_path: String,
    /// [#3383] 산출 형식 — 입력 형식을 따른다.
    pub(crate) output_format: EditOutputFormat,
    /// `--verify` 판정이 "차이 있음"인가. 단건은 exit 3, 배치는 집계 대상.
    verify_failed: bool,
}

/// [#3719 §6-6] 누름틀 채움의 **단 하나의** 구현. 단건 CLI 도 배치도 이 함수만 부른다.
///
/// 배치를 위해 새 편집 로직을 쓰지 않는다 — 채움 규칙(순번 지목·모호성 보고·혼동 이름
/// 경고·형식 보존·저장 직후 자기검증·changedPages)이 두 곳으로 갈라지면 단건으로 검증한
/// 서식이 배치에서 다르게 채워지고, 그 차이는 산출물 N개가 나온 뒤에야 드러난다.
///
/// 실패는 `Err(사람이 읽는 사유)` 다. 단건은 stderr + exit 1 로, 배치는 그 행의 `error`
/// 레코드로 바꾼다 — 프로세스를 끊지 않는 이유는 뒤 행이 남아 있기 때문이다.
pub(crate) fn fill_fields_core(
    file_path: &str,
    data: &serde_json::Map<String, serde_json::Value>,
    out_path: Option<String>,
    dry_run: bool,
    verify_mode: bool,
) -> Result<FillOutcome, String> {
    let bytes = fs::read(file_path)
        .map_err(|e| format!("파일을 읽을 수 없습니다 - {}: {}", file_path, e))?;
    let mut doc = load_document(&bytes).map_err(|e| match e {
        LoadError::NeedPassword => {
            "비밀번호가 필요한 암호 문서입니다 (--password <pw> 로 전달)".to_string()
        }
        LoadError::WrongPassword => {
            "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다".to_string()
        }
        LoadError::Other(msg) => format!("HWP 파싱 실패 - {}", msg),
    })?;

    // [#3476] 이름별 **개수**를 센다. 실제 제출 서식은 같은 항목 묶음을 여러 번 요구해
    // (규제영향분석서의 `피규제집단명` ×14 등) 이름만으로는 하나만 지목된다.
    let mut name_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    // [#3712] 같은 순회에서 문단 주소도 담는다 — changedPages 산출 근거.
    let mut name_locs: std::collections::HashMap<String, Vec<(usize, usize)>> =
        std::collections::HashMap::new();
    for fi in doc.collect_all_fields().iter() {
        if let Some(n) = fi.field.field_name() {
            *name_counts.entry(n.to_string()).or_insert(0) += 1;
            name_locs
                .entry(n.to_string())
                .or_default()
                .push((fi.location.section_index, fi.location.para_index));
        }
    }
    let mut changed_paras: Vec<(usize, usize)> = Vec::new();

    let mut filled: Vec<serde_json::Value> = Vec::new();
    let mut not_found: Vec<String> = Vec::new();
    // 이름만 준 키가 여러 곳에 해당하면 그 사실을 보고한다 — 침묵하면 소비자가
    // 불완전한 산출물을 완성본으로 판단한다.
    let mut ambiguous: Vec<serde_json::Value> = Vec::new();
    // [#3707] 바이트가 달라 위 개수 판정을 통과하지만 **화면상 구별되지 않는** 이름
    // 쌍은 별도 축이다. 지목한 이름에 그런 쌍둥이가 있으면 채우되 반드시 보고한다 —
    // 침묵하면 "엉뚱한 칸을 채우고 완벽한 성공을 보고"하는 상태가 된다.
    let all_names: Vec<String> = name_counts.keys().cloned().collect();
    let confusable_groups = rhwp::document_core::text_security::confusable_collisions(&all_names);
    let mut confusable: Vec<serde_json::Value> = Vec::new();

    for (key, value) in data {
        let value_str = match value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let (name, occurrence) = parse_field_key(key);
        let total = name_counts.get(name).copied().unwrap_or(0);

        // 이름이 없거나, 지정한 순번이 범위를 벗어나면 채우지 않고 보고한다.
        if total == 0 || occurrence >= total {
            not_found.push(key.clone());
            continue;
        }
        if occurrence == 0 && total > 1 && !key.contains('[') {
            ambiguous.push(serde_json::json!({
                "name": name,
                "matched": 1,
                "total": total,
            }));
        }
        if let Some((_, group)) = confusable_groups
            .iter()
            .find(|(_, g)| g.iter().any(|n| n == name))
        {
            let others: Vec<&String> = group.iter().filter(|n| *n != name).collect();
            confusable.push(serde_json::json!({
                "name": name,
                "lookalikes": others,
                "note": "화면상 구별되지 않는 이름의 누름틀이 이 문서에 함께 있습니다 — 채운 칸이 의도한 칸인지 확인하세요.",
            }));
        }

        if dry_run {
            // 파일을 건드리지 않고 무엇이 바뀔지만 기록한다.
            filled.push(
                serde_json::json!({ "name": name, "occurrence": occurrence, "value": value_str }),
            );
            continue;
        }
        // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
        doc.set_field_value_by_name_at(name, occurrence, &value_str)
            .map_err(|e| format!("필드 '{}' 설정 실패 - {}", key, e))?;
        if let Some(loc) = name_locs.get(name).and_then(|l| l.get(occurrence)) {
            changed_paras.push(*loc);
        }
        filled.push(
            serde_json::json!({ "name": name, "occurrence": occurrence, "value": value_str }),
        );
    }

    // [#3383] 입력 형식을 보존한다 — 기본 확장자도 산출 형식을 따른다.
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        // [#3469] 기본 산출물은 **입력 파일 옆**에 만든다. 종전에는 파일명만 써서
        // 현재 작업 디렉터리에 떨어졌는데, 임의 경로의 문서를 다루는 에이전트·MCP
        // 클라이언트에게는 산출물이 엉뚱한 곳에 생기는 셈이었다.
        let input = Path::new(file_path);
        let stem = input
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        let name = format!("{}_filled.{}", stem, out_format.ext());
        match input.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => {
                dir.join(name).to_string_lossy().to_string()
            }
            _ => name,
        }
    });

    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = edit_serialize(&mut doc, out_format)
            .map_err(|e| format!("{} 직렬화 실패 - {}", out_format.label().to_uppercase(), e))?;
        fs::write(&output_path, &out_bytes)
            .map_err(|e| format!("출력 쓰기 실패 - {}: {}", output_path, e))?;
        // [#3702] 저장 직후 자기검증 — 편집 후 IR ↔ 저장본 재파싱 IR.
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    // [#3712] 눈검증 대상 페이지 — 편집 반영 후 조판 기준. 확정 불가면 null(부분 목록 금지).
    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&changed_paras) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };

    let mut envelope = serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "source": file_path,
        "dryRun": dry_run,
        "changedPages": changed_pages,
        "filledCount": filled.len(),
        "filled": filled,
        "notFound": not_found,
        "ambiguous": ambiguous,
        "confusable": confusable,
    });
    if !dry_run {
        envelope["output"] = serde_json::Value::String(output_path.clone());
        envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
        envelope["verify"] = verify_report;
    }

    Ok(FillOutcome {
        envelope,
        output_path,
        output_format: out_format,
        verify_failed,
    })
}
