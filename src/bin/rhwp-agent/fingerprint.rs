//! [#3918] `fingerprint` — 안정 지문·기준선·드리프트 게이트.
//!
//! `ir-diff` 는 "두 파일"을 비교한다. 이 명령은 "같은 파일의 어제와 오늘"을 지킨다:
//! 문서의 의미 지문(텍스트 해시·쪽수·문자수·문단수·표·필드 이름)을 산출해 기준선
//! 파일로 저장(`--write`)하고, 이후 실행에서 드리프트를 exit 3 으로 알린다(`--check`).
//!
//! # 의미 지문 vs 바이트 해시
//!
//! 기본 비교는 **의미 지문**만 본다 — 같은 내용을 재저장해 바이트가 달라져도
//! 드리프트가 아니다. 바이트 단위까지 잠그려면 `--strict`(fileHash·bytes 포함).

use crate::envelope::{
    envelope, format_token, hex_hash, load_core, page_texts, print_json, read_file, text_hash,
    EXIT_GATE, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::{json, Value};

/// 문서 하나의 지문 — `evidence` 도 같은 계산을 쓴다(전/후가 다른 잣대로 재지 않게).
pub struct DocFingerprint {
    pub source: String,
    pub bytes: u64,
    pub file_hash: String,
    pub format: &'static str,
    pub page_count: u32,
    pub char_count: u64,
    pub para_count: u64,
    pub table_count: u64,
    pub field_count: u64,
    pub field_names: Vec<String>,
    pub text_hash: String,
    /// 쪽별 (문자 수, 해시) — `--with-pages` 및 diff 재사용을 위한 원본 텍스트.
    pub pages: Vec<String>,
}

/// 의미 지문에 들어가는 키 — `--check` 비교와 `evidence` 의 변경 필드 판정이 공유한다.
pub const SEMANTIC_KEYS: &[&str] = &[
    "format",
    "pageCount",
    "charCount",
    "paraCount",
    "tableCount",
    "fieldCount",
    "fieldNames",
    "textHash",
];

/// 지문 계산. 실패는 (종료 코드, 메시지)로 — 적재 실패는 실행 오류(1)다.
pub fn compute(path: &str) -> Result<DocFingerprint, (i32, String)> {
    let data = read_file(path).map_err(|m| (EXIT_RUNTIME, m))?;
    let format = format_token(rhwp::parser::detect_format(&data));
    let core = load_core(&data).map_err(|fail| {
        if fail.needs_password {
            (
                EXIT_RUNTIME,
                format!("암호 문서입니다 - {path} (이 실험 표면은 아직 비밀번호를 받지 않습니다. 본 CLI 의 --password 를 쓰세요)"),
            )
        } else {
            (EXIT_RUNTIME, format!("문서를 열 수 없습니다 - {path}: {}", fail.message))
        }
    })?;

    let pages = page_texts(&core).map_err(|m| (EXIT_RUNTIME, format!("{path}: {m}")))?;
    let char_count: u64 = pages.iter().map(|p| p.chars().count() as u64).sum();

    let document = core.document();
    let para_count: u64 = document
        .sections
        .iter()
        .map(|s| s.paragraphs.len() as u64)
        .sum();
    let table_count =
        rhwp::document_core::queries::table_extract::extract_tables(document).len() as u64;

    // 필드 이름: 누름틀 고치기 이름(ctrl_data_name)이 있으면 그것, 없으면 command.
    // 빈 이름은 정체성 신호가 아니므로 버린다. 정렬·중복 제거로 순서 결정성을 만든다.
    let fields = core.collect_all_fields();
    let field_count = fields.len() as u64;
    let mut field_names: Vec<String> = fields
        .iter()
        .map(|f| {
            f.field
                .ctrl_data_name
                .clone()
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| f.field.command.clone())
        })
        .filter(|name| !name.is_empty())
        .collect();
    field_names.sort();
    field_names.dedup();

    Ok(DocFingerprint {
        source: path.to_string(),
        bytes: data.len() as u64,
        file_hash: hex_hash(&data),
        format,
        page_count: core.page_count(),
        char_count,
        para_count,
        table_count,
        field_count,
        field_names,
        text_hash: text_hash(&pages),
        pages,
    })
}

/// 지문의 봉투 본문 값 (envelope 공통 필드 제외).
pub fn payload(fp: &DocFingerprint, with_pages: bool) -> Value {
    let mut value = json!({
        "source": fp.source,
        "bytes": fp.bytes,
        "fileHash": fp.file_hash,
        "format": fp.format,
        "pageCount": fp.page_count,
        "charCount": fp.char_count,
        "paraCount": fp.para_count,
        "tableCount": fp.table_count,
        "fieldCount": fp.field_count,
        "fieldNames": fp.field_names,
        "textHash": fp.text_hash,
    });
    if with_pages {
        let pages: Vec<Value> = fp
            .pages
            .iter()
            .enumerate()
            .map(|(idx, text)| {
                json!({
                    "page": idx,
                    "chars": text.chars().count() as u64,
                    "hash": hex_hash(text.as_bytes()),
                })
            })
            .collect();
        value["pages"] = json!(pages);
    }
    value
}

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp-agent fingerprint <파일> [--json] [--with-pages] [--write <기준.json>] [--check <기준.json>] [--strict]";

    let mut json_mode = false;
    let mut with_pages = false;
    let mut strict = false;
    let mut write_to: Option<String> = None;
    let mut check_against: Option<String> = None;
    let mut file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--with-pages" => {
                with_pages = true;
                i += 1;
            }
            "--strict" => {
                strict = true;
                i += 1;
            }
            "--write" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --write 뒤에 기준선 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                };
                write_to = Some(value.clone());
                i += 2;
            }
            "--check" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --check 뒤에 기준선 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                };
                check_against = Some(value.clone());
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
            positional => {
                if file.is_some() {
                    eprintln!("오류: 파일은 하나만 지정할 수 있습니다 - {positional}");
                    return EXIT_USAGE;
                }
                file = Some(positional.to_string());
                i += 1;
            }
        }
    }

    let Some(path) = file else {
        eprintln!("오류: 대상 파일을 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if strict && check_against.is_none() {
        eprintln!("오류: --strict 는 --check 와 함께 써야 합니다.");
        return EXIT_USAGE;
    }

    let fp = match compute(&path) {
        Ok(fp) => fp,
        Err((code, message)) => {
            eprintln!("오류: {message}");
            return code;
        }
    };
    let mut body = payload(&fp, with_pages);

    // 기준선 저장 — 봉투 전체를 쓴다(사람이 열어봐도 무엇인지 알 수 있게).
    if let Some(dest) = &write_to {
        let baseline = envelope("fingerprint", body.clone(), &["fieldNames[]"]);
        let text = match serde_json::to_string_pretty(&baseline) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 기준선 직렬화 실패 - {e}");
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = std::fs::write(dest, text) {
            eprintln!("오류: 기준선을 쓸 수 없습니다 - {dest}: {e}");
            return EXIT_RUNTIME;
        }
        eprintln!("기준선 저장: {dest}");
    }

    // 드리프트 게이트.
    let mut exit = EXIT_OK;
    if let Some(base_path) = &check_against {
        let base_text = match std::fs::read_to_string(base_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 기준선을 읽을 수 없습니다 - {base_path}: {e}");
                return EXIT_RUNTIME;
            }
        };
        let base: Value = match serde_json::from_str(&base_text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 기준선이 JSON 이 아닙니다 - {base_path}: {e}");
                return EXIT_RUNTIME;
            }
        };

        let mut keys: Vec<&str> = SEMANTIC_KEYS.to_vec();
        if strict {
            keys.push("fileHash");
            keys.push("bytes");
        }
        let mut drift: Vec<Value> = Vec::new();
        for key in keys {
            let baseline_value = base.get(key).cloned().unwrap_or(Value::Null);
            let current_value = body.get(key).cloned().unwrap_or(Value::Null);
            if baseline_value != current_value {
                drift.push(json!({
                    "field": key,
                    "baseline": baseline_value,
                    "current": current_value,
                }));
            }
        }
        body["checkedAgainst"] = json!(base_path);
        body["strict"] = json!(strict);
        body["driftCount"] = json!(drift.len());
        body["ok"] = json!(drift.is_empty());
        if !drift.is_empty() {
            body["drift"] = json!(drift);
            exit = EXIT_GATE;
        }
    }

    if json_mode {
        // fieldNames·drift 값은 문서 파생이다. 실린 필드만 선언한다.
        let mut untrusted: Vec<&str> = vec!["fieldNames[]"];
        if body.get("drift").is_some() {
            untrusted.push("drift[].baseline");
            untrusted.push("drift[].current");
        }
        print_json(&envelope("fingerprint", body, &untrusted));
    } else {
        crate::outln!("rhwp-agent fingerprint — {}", fp.source);
        crate::outln!("  format      {}", fp.format);
        crate::outln!("  pageCount   {}", fp.page_count);
        crate::outln!("  charCount   {}", fp.char_count);
        crate::outln!("  paraCount   {}", fp.para_count);
        crate::outln!("  tableCount  {}", fp.table_count);
        crate::outln!("  fieldCount  {}", fp.field_count);
        crate::outln!("  textHash    {}", fp.text_hash);
        crate::outln!("  fileHash    {}", fp.file_hash);
        if let Some(count) = body.get("driftCount").and_then(Value::as_u64) {
            if count == 0 {
                crate::outln!("기준선 일치: 드리프트 없음");
            } else {
                crate::outln!("기준선 드리프트 {count}건:");
                if let Some(items) = body.get("drift").and_then(Value::as_array) {
                    for item in items {
                        crate::outln!(
                            "  {}: {} → {}",
                            item["field"].as_str().unwrap_or("?"),
                            item["baseline"],
                            item["current"],
                        );
                    }
                }
            }
        }
    }
    exit
}
