//! [#3918] `scan` — 코퍼스 발견·분류.
//!
//! `batch` 는 "경로 목록을 이미 갖고 있다"는 전제에서 시작한다. 이 명령은 그 앞
//! 단계다: 디렉터리를 재귀로 걸어 HWP 계열 파일을 찾고, 확장자 주장과 매직 감지를
//! 대조하고, `--probe` 면 실제로 열어 파싱 가능/암호 필요/오류를 기록한다.
//! 출력을 그대로 `batch` 의 stdin 에 이어 붙일 수 있게 경로를 한 줄에 하나씩 뽑는
//! 소비 경로는 JSONL(`--jsonl` + jq)이 맡는다.
//!
//! 결정성: 파일 순서는 경로 문자열 오름차순으로 고정한다 — 같은 트리는 언제나
//! 같은 순서로 나온다(재현 가능한 코퍼스 목록).

use crate::envelope::{
    envelope, format_token, load_core, print_json, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// 확장자가 주장하는 포맷. `.hwp` 는 HWP5/HWP3 겸용 확장자라 "hwp"(모호)로 둔다.
fn ext_claim(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "hwp" => Some("hwp"),
        "hwpx" => Some("hwpx"),
        "hml" => Some("hml"),
        _ => None,
    }
}

/// 확장자 주장과 매직 감지가 어긋나는가. `.hwp` 는 hwp5·hwp3 둘 다 정상이다.
fn ext_mismatch(claim: &str, magic: &str) -> bool {
    match claim {
        "hwp" => !matches!(magic, "hwp5" | "hwp3"),
        other => other != magic,
    }
}

struct Options {
    json: bool,
    jsonl: bool,
    probe: bool,
    max_depth: Option<usize>,
    limit: Option<usize>,
}

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str =
        "사용법: rhwp-agent scan <경로...> [--json|--jsonl] [--probe] [--max-depth <N>] [--limit <N>]";

    let mut opts = Options {
        json: false,
        jsonl: false,
        probe: false,
        max_depth: None,
        limit: None,
    };
    let mut roots: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                opts.json = true;
                i += 1;
            }
            "--jsonl" => {
                opts.jsonl = true;
                i += 1;
            }
            "--probe" => {
                opts.probe = true;
                i += 1;
            }
            "--max-depth" => {
                match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => opts.max_depth = Some(n),
                    _ => {
                        eprintln!("오류: --max-depth 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--limit" => {
                match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => opts.limit = Some(n),
                    _ => {
                        eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
            path => {
                roots.push(path.to_string());
                i += 1;
            }
        }
    }

    if roots.is_empty() {
        eprintln!("오류: 검색할 경로를 하나 이상 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }
    if opts.json && opts.jsonl {
        eprintln!("오류: --json 과 --jsonl 은 동시에 쓸 수 없습니다.");
        return EXIT_USAGE;
    }

    // ① 대상 수집 — 루트마다 걷고, 전체를 경로 문자열로 정렬해 결정적 순서를 만든다.
    let mut files: Vec<PathBuf> = Vec::new();
    for root in &roots {
        let path = Path::new(root);
        if path.is_file() {
            files.push(path.to_path_buf());
            continue;
        }
        if !path.is_dir() {
            eprintln!("오류: 경로가 존재하지 않습니다 - {root}");
            return EXIT_RUNTIME;
        }
        if let Err(message) = walk(path, 1, opts.max_depth, &mut files) {
            eprintln!("오류: {message}");
            return EXIT_RUNTIME;
        }
    }
    files.sort_by_key(|p| p.to_string_lossy().to_string());
    files.dedup();

    let mut truncated = false;
    if let Some(limit) = opts.limit {
        if files.len() > limit {
            files.truncate(limit);
            truncated = true;
        }
    }

    // ② 파일별 레코드.
    let mut records: Vec<Value> = Vec::new();
    let mut by_format: std::collections::BTreeMap<String, u64> = Default::default();
    let mut mismatch_count = 0u64;
    let mut probe_failed = 0u64;
    let mut needs_password = 0u64;

    for file in &files {
        let record = match file_record(file, opts.probe) {
            Ok(r) => r,
            Err(message) => {
                eprintln!("오류: {message}");
                return EXIT_RUNTIME;
            }
        };
        let magic = record["magicFormat"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        *by_format.entry(magic).or_insert(0) += 1;
        if record["extMismatch"].as_bool() == Some(true) {
            mismatch_count += 1;
        }
        if record["probe"]["parseOk"].as_bool() == Some(false) {
            probe_failed += 1;
            if record["probe"]["needsPassword"].as_bool() == Some(true) {
                needs_password += 1;
            }
        }
        records.push(record);
    }

    let summary = json!({
        "total": records.len(),
        "byFormat": by_format,
        "extMismatch": mismatch_count,
        "probed": opts.probe,
        "probeFailed": if opts.probe { json!(probe_failed) } else { Value::Null },
        "needsPassword": if opts.probe { json!(needs_password) } else { Value::Null },
        "truncated": truncated,
    });

    // ③ 출력.
    // probe.error 는 파서 메시지라 문서 파생일 수 있다 — 실린 호출에만 선언한다.
    let untrusted: &[&str] = if opts.probe {
        &["files[].probe.error"]
    } else {
        &[]
    };

    if opts.jsonl {
        for record in &records {
            let mut line = record.clone();
            if let Some(map) = line.as_object_mut() {
                map.insert("record".into(), json!("file"));
                map.insert("schemaVersion".into(), json!(ENVELOPE_SCHEMA_VERSION));
            }
            match serde_json::to_string(&line) {
                Ok(s) => crate::outln!("{s}"),
                Err(e) => eprintln!("오류: JSON 직렬화 실패 - {e}"),
            }
        }
        let mut tail = summary.clone();
        if let Some(map) = tail.as_object_mut() {
            map.insert("record".into(), json!("summary"));
            map.insert("schemaVersion".into(), json!(ENVELOPE_SCHEMA_VERSION));
        }
        match serde_json::to_string(&tail) {
            Ok(s) => crate::outln!("{s}"),
            Err(e) => eprintln!("오류: JSON 직렬화 실패 - {e}"),
        }
        return EXIT_OK;
    }

    if opts.json {
        let payload = json!({
            "roots": roots,
            "files": records,
            "summary": summary,
        });
        print_json(&envelope("scan", payload, untrusted));
        return EXIT_OK;
    }

    // 사람용 텍스트.
    crate::outln!("rhwp-agent scan — {}개 파일", records.len());
    for record in &records {
        let mut notes: Vec<String> = Vec::new();
        if record["extMismatch"].as_bool() == Some(true) {
            notes.push("확장자 불일치".to_string());
        }
        if record["probe"]["needsPassword"].as_bool() == Some(true) {
            notes.push("암호 필요".to_string());
        } else if record["probe"]["parseOk"].as_bool() == Some(false) {
            notes.push("파싱 실패".to_string());
        }
        let notes = if notes.is_empty() {
            String::new()
        } else {
            format!("  [{}]", notes.join(", "))
        };
        crate::outln!(
            "  {}  {}  {}바이트{notes}",
            record["magicFormat"].as_str().unwrap_or("?"),
            record["path"].as_str().unwrap_or("?"),
            record["bytes"].as_u64().unwrap_or(0),
        );
    }
    crate::outln!(
        "합계: {} · 확장자 불일치 {}{}",
        records.len(),
        mismatch_count,
        if opts.probe {
            format!(" · 파싱 실패 {probe_failed} (암호 필요 {needs_password})")
        } else {
            String::new()
        }
    );
    EXIT_OK
}

/// 재귀 걷기 — 심볼릭 링크 디렉터리는 따라가지 않는다(순환 방지).
fn walk(
    dir: &Path,
    depth: usize,
    max_depth: Option<usize>,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("폴더를 읽을 수 없습니다 - {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("항목을 읽을 수 없습니다 - {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("파일 유형을 읽을 수 없습니다 - {}: {e}", path.display()))?;
        if file_type.is_dir() {
            if file_type.is_symlink() {
                continue;
            }
            if max_depth.map(|m| depth < m).unwrap_or(true) {
                walk(&path, depth + 1, max_depth, out)?;
            }
        } else if file_type.is_file() && ext_claim(&path).is_some() {
            out.push(path);
        }
    }
    Ok(())
}

fn file_record(path: &Path, probe: bool) -> Result<Value, String> {
    let display = path.to_string_lossy().to_string();
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("파일 정보를 읽을 수 없습니다 - {display}: {e}"))?;
    let claim = ext_claim(path).unwrap_or("hwp");

    // 매직 감지에는 앞부분이면 충분하지만, --probe 는 어차피 전체를 읽는다.
    let data =
        std::fs::read(path).map_err(|e| format!("파일을 읽을 수 없습니다 - {display}: {e}"))?;
    let magic = format_token(rhwp::parser::detect_format(&data));

    let probe_value = if probe {
        let started = std::time::Instant::now();
        match load_core(&data) {
            Ok(core) => json!({
                "parseOk": true,
                "needsPassword": false,
                "pageCount": core.page_count(),
                "ms": started.elapsed().as_millis() as u64,
            }),
            Err(fail) => json!({
                "parseOk": false,
                "needsPassword": fail.needs_password,
                "error": fail.message,
                "ms": started.elapsed().as_millis() as u64,
            }),
        }
    } else {
        Value::Null
    };

    let modified_unix = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Ok(json!({
        "path": display,
        "bytes": meta.len(),
        "modifiedUnix": modified_unix,
        "extFormat": claim,
        "magicFormat": magic,
        "extMismatch": ext_mismatch(claim, magic),
        "probe": probe_value,
    }))
}
