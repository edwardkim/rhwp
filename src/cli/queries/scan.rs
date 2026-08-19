//! Corpus discovery and format classification query adapter.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{load_document, LoadError, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

const SCAN_USAGE: &str =
    "사용법: rhwp scan <경로...> [--probe] [--max-depth <N>] [--limit <N>] [--json]";

struct ScanOptions {
    json_mode: bool,
    probe: bool,
    max_depth: Option<usize>,
    limit: Option<usize>,
    roots: Vec<String>,
}

struct ScanFileRecord {
    value: serde_json::Value,
    magic: &'static str,
    mismatch: bool,
    probe_failed: bool,
    locked: bool,
}

struct ScanRun {
    records: Vec<serde_json::Value>,
    by_format: std::collections::BTreeMap<String, u64>,
    mismatch_count: u64,
    probe_failed: u64,
    locked_count: u64,
    truncated: bool,
}

fn parse_scan_positive(args: &[String], index: usize, flag: &str) -> Result<usize, String> {
    match args
        .get(index)
        .and_then(|value| value.parse::<usize>().ok())
    {
        Some(value) if value >= 1 => Ok(value),
        _ => Err(format!("{flag} 뒤에 1 이상의 정수가 필요합니다.")),
    }
}

fn parse_scan_options(args: &[String]) -> Result<ScanOptions, String> {
    let mut options = ScanOptions {
        json_mode: false,
        probe: false,
        max_depth: None,
        limit: None,
        roots: Vec::new(),
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => options.json_mode = true,
            "--probe" => options.probe = true,
            "--max-depth" => {
                index += 1;
                options.max_depth = Some(parse_scan_positive(args, index, "--max-depth")?);
            }
            "--limit" => {
                index += 1;
                options.limit = Some(parse_scan_positive(args, index, "--limit")?);
            }
            other if other.starts_with('-') => {
                return Err(format!("알 수 없는 옵션입니다 - {other}"));
            }
            path => options.roots.push(path.to_string()),
        }
        index += 1;
    }
    if options.roots.is_empty() {
        return Err("검색할 경로를 하나 이상 지정해주세요.".to_string());
    }
    Ok(options)
}

/// 확장자가 주장하는 포맷. `.hwp` 는 HWP5/HWP3 겸용 확장자라 "hwp"(모호)로 둔다.
fn scan_ext_claim(path: &std::path::Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "hwp" => Some("hwp"),
        "hwpx" => Some("hwpx"),
        "hml" => Some("hml"),
        _ => None,
    }
}

/// 확장자 주장과 매직 감지가 어긋나는가. `.hwp` 는 hwp5·hwp3 둘 다 정상이다.
fn scan_ext_mismatch(claim: &str, magic: &str) -> bool {
    match claim {
        "hwp" => !matches!(magic, "hwp5" | "hwp3"),
        other => other != magic,
    }
}

/// `parser::FileFormat` → `info --json` 의 `format` 토큰 (verify 와 같은 지도).
fn scan_format_token(format: rhwp::parser::FileFormat) -> &'static str {
    use rhwp::parser::FileFormat;
    match format {
        FileFormat::Hwp => "hwp5",
        FileFormat::Hwpx => "hwpx",
        FileFormat::Hwp3 => "hwp3",
        FileFormat::Hml => "hml",
        FileFormat::DrmProtected => "drm-protected",
        FileFormat::Empty => "empty",
        FileFormat::Unknown => "unknown",
    }
}

/// 재귀 걷기 — 심볼릭 링크는 따라가지 않는다(순환 방지).
fn walk_scan_dir(
    dir: &std::path::Path,
    depth: usize,
    max_depth: Option<usize>,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| format!("폴더를 읽을 수 없습니다 - {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("항목을 읽을 수 없습니다 - {error}"))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            format!("파일 유형을 읽을 수 없습니다 - {}: {error}", path.display())
        })?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() && max_depth.map(|max| depth < max).unwrap_or(true) {
            walk_scan_dir(&path, depth + 1, max_depth, out)?;
        } else if file_type.is_file() && scan_ext_claim(&path).is_some() {
            out.push(path);
        }
    }
    Ok(())
}

fn collect_scan_files(options: &ScanOptions) -> Result<(Vec<std::path::PathBuf>, bool), String> {
    let mut files = Vec::new();
    for root in &options.roots {
        let path = std::path::Path::new(root);
        if path.is_file() {
            files.push(path.to_path_buf());
        } else if path.is_dir() {
            walk_scan_dir(path, 1, options.max_depth, &mut files)?;
        } else {
            return Err(format!("경로가 존재하지 않습니다 - {root}"));
        }
    }
    files.sort_by_key(|path| path.to_string_lossy().to_string());
    files.dedup();

    let truncated = options.limit.is_some_and(|limit| files.len() > limit);
    if let Some(limit) = options.limit {
        files.truncate(limit);
    }
    Ok((files, truncated))
}

fn scan_probe(data: &[u8], enabled: bool) -> (serde_json::Value, bool, bool) {
    if !enabled {
        return (serde_json::Value::Null, false, false);
    }
    let started = std::time::Instant::now();
    match load_document(data) {
        Ok(doc) => (
            serde_json::json!({
                "parseOk": true,
                "needsPassword": false,
                "pageCount": doc.page_count(),
                "ms": started.elapsed().as_millis() as u64,
            }),
            false,
            false,
        ),
        Err(fail) => {
            let (locked, message) = match fail {
                LoadError::NeedPassword => (true, "비밀번호가 필요한 암호 문서입니다".to_string()),
                LoadError::WrongPassword => (
                    true,
                    "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다".to_string(),
                ),
                LoadError::Other(message) => (false, message),
            };
            (
                serde_json::json!({
                    "parseOk": false,
                    "needsPassword": locked,
                    "error": message,
                    "ms": started.elapsed().as_millis() as u64,
                }),
                true,
                locked,
            )
        }
    }
}

fn scan_file_record(file: &std::path::Path, probe: bool) -> Result<ScanFileRecord, String> {
    let display = file.to_string_lossy().to_string();
    let meta = fs::metadata(file)
        .map_err(|error| format!("파일 정보를 읽을 수 없습니다 - {display}: {error}"))?;
    let data =
        fs::read(file).map_err(|error| format!("파일을 읽을 수 없습니다 - {display}: {error}"))?;
    let claim = scan_ext_claim(file).unwrap_or("hwp");
    let magic = scan_format_token(rhwp::parser::detect_format(&data));
    let mismatch = scan_ext_mismatch(claim, magic);
    let (probe_value, probe_failed, locked) = scan_probe(&data, probe);
    let modified_unix = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    Ok(ScanFileRecord {
        value: serde_json::json!({
            "path": display,
            "bytes": meta.len(),
            "modifiedUnix": modified_unix,
            "extFormat": claim,
            "magicFormat": magic,
            "extMismatch": mismatch,
            "probe": probe_value,
        }),
        magic,
        mismatch,
        probe_failed,
        locked,
    })
}

fn build_scan_run(
    files: &[std::path::PathBuf],
    options: &ScanOptions,
    truncated: bool,
) -> Result<ScanRun, String> {
    let mut run = ScanRun {
        records: Vec::new(),
        by_format: Default::default(),
        mismatch_count: 0,
        probe_failed: 0,
        locked_count: 0,
        truncated,
    };
    for file in files {
        let record = scan_file_record(file, options.probe)?;
        *run.by_format.entry(record.magic.to_string()).or_insert(0) += 1;
        run.mismatch_count += u64::from(record.mismatch);
        run.probe_failed += u64::from(record.probe_failed);
        // 암호로 잠긴 파일 **개수** — 자격증명이 아니다. 변수명에 password 를 쓰면
        // CodeQL cleartext-logging 이 요약 출력을 민감정보 기록으로 오탐한다.
        run.locked_count += u64::from(record.locked);
        run.records.push(record.value);
    }
    Ok(run)
}

fn scan_summary(run: &ScanRun, probed: bool) -> serde_json::Value {
    serde_json::json!({
        "total": run.records.len(),
        "byFormat": run.by_format,
        "extMismatch": run.mismatch_count,
        "probed": probed,
        "probeFailed": if probed { serde_json::json!(run.probe_failed) } else { serde_json::Value::Null },
        "needsPassword": if probed { serde_json::json!(run.locked_count) } else { serde_json::Value::Null },
        "truncated": run.truncated,
    })
}

fn write_scan_human(run: &ScanRun, probed: bool) {
    println!("rhwp scan — {}개 파일", run.records.len());
    for record in &run.records {
        let mut notes: Vec<&str> = Vec::new();
        if record["extMismatch"].as_bool() == Some(true) {
            notes.push("확장자 불일치");
        }
        if record["probe"]["needsPassword"].as_bool() == Some(true) {
            notes.push("암호 필요");
        } else if record["probe"]["parseOk"].as_bool() == Some(false) {
            notes.push("파싱 실패");
        }
        let notes = if notes.is_empty() {
            String::new()
        } else {
            format!("  [{}]", notes.join(", "))
        };
        println!(
            "  {}  {}  {}바이트{notes}",
            record["magicFormat"].as_str().unwrap_or("?"),
            record["path"].as_str().unwrap_or("?"),
            record["bytes"].as_u64().unwrap_or(0),
        );
    }
    println!(
        "합계: {} · 확장자 불일치 {}{}",
        run.records.len(),
        run.mismatch_count,
        if probed {
            format!(
                " · 파싱 실패 {} (암호 필요 {})",
                run.probe_failed, run.locked_count
            )
        } else {
            String::new()
        }
    );
}

pub(crate) fn run(args: &[String]) -> i32 {
    let options = match parse_scan_options(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("오류: {message}");
            eprintln!("{SCAN_USAGE}");
            return EXIT_USAGE;
        }
    };
    let (files, truncated) = match collect_scan_files(&options) {
        Ok(result) => result,
        Err(message) => {
            eprintln!("오류: {message}");
            return EXIT_RUNTIME;
        }
    };
    let run = match build_scan_run(&files, &options, truncated) {
        Ok(run) => run,
        Err(message) => {
            eprintln!("오류: {message}");
            return EXIT_RUNTIME;
        }
    };
    if options.json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "roots": options.roots,
            "files": run.records,
            "summary": scan_summary(&run, options.probe),
        });
        println!("{}", provenance::marked(envelope, "scan"));
    } else {
        write_scan_human(&run, options.probe);
    }
    EXIT_OK
}
