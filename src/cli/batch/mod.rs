//! Ordered batch CLI orchestration and read-only record routing.
//!
//! The stream runtime owns ordering and backpressure. Query projections remain
//! separate from state-changing fill and convert command adapters.

use std::fs;
use std::path::Path;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{ConversionVerifyOptions, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) mod ordered;
mod query;

/// [#3238] batch — 파일 목록을 stdin(한 줄당 하나)으로 받아 한 프로세스에서 전건 처리하고
/// NDJSON 스트림을 stdout 으로 낸다. 건별 실패는 `error` 레코드로 스트림을 계속하되,
/// 하나라도 실패하면 [#2707] 계약대로 종료 코드 1 로 끝난다.
pub(crate) fn run(args: &[String]) -> i32 {
    use std::io::{BufRead, Write};

    const USAGE: &str = "사용법: <파일 목록> | rhwp batch <export-text|info|export-structure|export-tables|fields|search|extract-data|convert> --json [--mode auto|outline|clause] [--query <검색어>] [--kind date|amount|number|all] [--limit <N>] [--threads <N>] [convert: --out-dir <폴더> [--verify] [--verify-pages]]  (stdin: 한 줄당 파일 경로 하나)\n      rhwp batch fill --form <서식> --data <행.jsonl|행.csv> --out-dir <폴더> --json  (fill 만 stdin 을 읽지 않는다)";

    let subcommand = args.first().map(String::as_str);
    // [#3719 §6-6] fill 축은 **입력 축 자체가 다르다** — stdin 파일 목록이 아니라 서식 1 개와
    // 데이터 파일 1 개를 받고, 산출은 행 수만큼 나온다. 인자 문법이 다른 축과 겹치지 않으므로
    // 파싱부터 갈라 놓는다(경로 목록 읽기를 절대 타지 않게 하는 것이 요점이다).
    if subcommand == Some("fill") {
        return crate::cli::commands::batch_fill::run(&args[1..]);
    }
    let is_structure = subcommand == Some("export-structure");
    // [#3346] --query 는 search 축 전용이다 (--mode 가 export-structure 전용인 것과 같은 규약).
    let is_search = subcommand == Some("search");
    // [#3626] --out-dir·--verify·--verify-pages 는 convert 축 전용이다 (같은 규약).
    let is_convert = subcommand == Some("convert");
    // [#3830] --kind·--limit 는 extract-data 축 전용이다 (같은 규약).
    let is_extract_data = subcommand == Some("extract-data");
    if !matches!(
        subcommand,
        Some("export-text")
            | Some("info")
            | Some("export-structure")
            | Some("export-tables")
            | Some("fields")
            | Some("search")
            | Some("extract-data")
            | Some("convert")
    ) {
        match subcommand {
            Some(unknown) => eprintln!(
                "오류: batch 는 export-text·info·export-structure·export-tables·fields·search·extract-data·convert·fill 만 지원합니다 - {}",
                unknown
            ),
            None => eprintln!("오류: batch 서브커맨드를 지정해주세요."),
        }
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    let mut json_mode = false;
    let mut threads_opt: Option<usize> = None;
    let mut structure_mode = rhwp::document_core::queries::structure::StructureMode::Auto;
    let mut search_query: Option<String> = None;
    // [#3830] extract-data 축 전용 — 종류 필터·문서당 상한.
    let mut extract_kind = "all".to_string();
    let mut extract_limit: Option<usize> = None;
    // [#3626] convert 축 전용 — 목적지와 검증 게이트.
    let mut out_dir: Option<std::path::PathBuf> = None;
    // batch 레코드는 언제나 JSON 이므로 json 은 켠 채로 둔다 — verify/verify_pages 만 옵션.
    let mut verify_options = ConversionVerifyOptions {
        json: true,
        ..ConversionVerifyOptions::default()
    };
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--out-dir" => {
                // [#3626] --out-dir 는 convert 축 전용이다.
                if !is_convert {
                    eprintln!("오류: --out-dir 는 convert 에서만 사용할 수 있습니다.");
                    return EXIT_USAGE;
                }
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --out-dir 뒤에 폴더 경로가 필요합니다.");
                    return EXIT_USAGE;
                };
                if value.is_empty() || value.starts_with('-') {
                    eprintln!(
                        "오류: --out-dir 뒤에 플래그가 아닌 폴더 경로가 필요합니다 (이름이 - 로 시작하면 ./ 를 붙이세요)."
                    );
                    return EXIT_USAGE;
                }
                out_dir = Some(std::path::PathBuf::from(value));
                i += 2;
            }
            "--verify" | "--verify-pages" => {
                // 옵션 이름을 리터럴로 고정한다 — 인자에서 온 문자열을 그대로 찍으면
                // CodeQL cleartext-logging 대상이 된다(extract-pages 와 같은 규약).
                let opt: &'static str = if args[i] == "--verify" {
                    "--verify"
                } else {
                    "--verify-pages"
                };
                // [#3626] 검증 게이트는 파일을 쓰는 convert 축에서만 뜻이 있다.
                if !is_convert {
                    eprintln!("오류: {opt} 는 convert 에서만 사용할 수 있습니다.");
                    return EXIT_USAGE;
                }
                if opt == "--verify" {
                    verify_options.verify = true;
                } else {
                    verify_options.verify_pages = true;
                }
                i += 1;
            }
            "--query" => {
                // [#3346] --query 는 search 축 전용이다.
                if !is_search {
                    eprintln!("오류: --query 는 search 에서만 사용할 수 있습니다.");
                    return EXIT_USAGE;
                }
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --query 뒤에 검색어가 필요합니다.");
                    return EXIT_USAGE;
                };
                if value.is_empty() {
                    eprintln!("오류: --query 검색어가 비어 있습니다.");
                    return EXIT_USAGE;
                }
                search_query = Some(value.clone());
                i += 2;
            }
            "--kind" => {
                // [#3830] --kind 는 extract-data 축 전용이다.
                if !is_extract_data {
                    eprintln!("오류: --kind 는 extract-data 에서만 사용할 수 있습니다.");
                    return EXIT_USAGE;
                }
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --kind 뒤에 date|amount|number|all 이 필요합니다.");
                    return EXIT_USAGE;
                };
                match value.as_str() {
                    "all" => extract_kind = "all".to_string(),
                    v if rhwp::document_core::queries::extract_data::DataKind::parse(v)
                        .is_some() =>
                    {
                        extract_kind = v.to_string();
                    }
                    _ => {
                        eprintln!("오류: --kind 는 date|amount|number|all 중 하나여야 합니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--limit" => {
                // [#3830] --limit 는 extract-data 축 전용 — **문서마다** 적용되는 상한이다.
                if !is_extract_data {
                    eprintln!("오류: --limit 는 extract-data 에서만 사용할 수 있습니다.");
                    return EXIT_USAGE;
                }
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match value.parse::<usize>() {
                    Ok(n) if n >= 1 => extract_limit = Some(n),
                    _ => {
                        eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--mode" => {
                // [#3261] --mode 는 export-structure 축 전용이다.
                if !is_structure {
                    eprintln!("오류: --mode 는 export-structure 에서만 사용할 수 있습니다.");
                    return EXIT_USAGE;
                }
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --mode 뒤에 auto|outline|clause 가 필요합니다.");
                    return EXIT_USAGE;
                };
                match rhwp::document_core::queries::structure::StructureMode::parse(value) {
                    Some(m) => structure_mode = m,
                    None => {
                        eprintln!("오류: --mode 는 auto|outline|clause - {}", value);
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--threads" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: --threads 뒤에 스레드 수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match value.parse::<usize>() {
                    Ok(n) if n >= 1 => threads_opt = Some(n),
                    _ => {
                        eprintln!("오류: 스레드 수가 올바르지 않습니다 - {}", value);
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            other => {
                eprintln!("알 수 없는 옵션: {}", other);
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
        }
    }
    if !json_mode {
        eprintln!("오류: batch 는 현재 --json 출력만 지원합니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }

    let mode = match subcommand {
        Some("export-text") => BatchMode::ExportText,
        Some("info") => BatchMode::Info,
        Some("export-tables") => BatchMode::Tables,
        Some("fields") => BatchMode::Fields,
        Some("search") => {
            let Some(q) = search_query.as_deref() else {
                eprintln!("오류: batch search 는 --query <검색어> 가 필요합니다.");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            };
            BatchMode::Search { query: q }
        }
        Some("extract-data") => BatchMode::ExtractData {
            kind: extract_kind.as_str(),
            limit: extract_limit,
        },
        Some("convert") => {
            // [#3626] 목적지는 명시적이어야 한다. 읽기 전용 6축과 달리 이 축은 입력마다
            // 파일을 쓰는데, 경로는 stdin 에서 오므로 호출자가 산출물이 어디 생기는지
            // 명령줄만 보고 알 수 없으면 안 된다.
            let Some(dir) = out_dir.as_deref() else {
                eprintln!("오류: batch convert 는 --out-dir <폴더> 가 필요합니다.");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            };
            BatchMode::Convert {
                out_dir: dir,
                verify: verify_options,
            }
        }
        _ => BatchMode::Structure(structure_mode),
    };

    let stdin = std::io::stdin();
    let mut paths: Vec<String> = Vec::new();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                let path = l.trim().to_string();
                if !path.is_empty() {
                    paths.push(path);
                }
            }
            Err(e) => {
                eprintln!("오류: stdin 읽기 실패 - {}", e);
                return EXIT_RUNTIME;
            }
        }
    }

    // [#3626] 변환 축은 파일을 쓴다 — 읽기 전용 6축에 없던 사전 점검이 필요하다.
    // 산출 이름은 입력 파일 이름만 따르므로 서로 다른 폴더의 같은 이름이 한 경로로 겹친다.
    // 겹침을 레코드로 보고하며 진행하면 이미 절반이 변환된 산출 폴더가 남는다. 한 바이트도
    // 쓰기 전에 전건을 미리 계산해 잡고, 잡히면 사용법 오류로 끝낸다(부분 산출물 없음).
    if let BatchMode::Convert { out_dir, .. } = mode {
        let mut claimed: std::collections::HashMap<String, &str> =
            std::collections::HashMap::with_capacity(paths.len());
        for path in &paths {
            let candidate =
                crate::cli::commands::batch_convert::output_path(out_dir, Path::new(path));
            if let Some(first) = claimed.insert(
                crate::cli::commands::batch_convert::collision_key(&candidate),
                path.as_str(),
            ) {
                eprintln!(
                    "오류: 산출 경로가 겹칩니다 - {} ← {} · {}",
                    candidate.display(),
                    first,
                    path
                );
                eprintln!(
                    "      --out-dir 는 입력 파일 이름만 남기므로 서로 다른 폴더의 같은 이름을 구분할 수 없습니다. 입력을 나눠 실행하세요."
                );
                return EXIT_USAGE;
            }
        }
        if let Err(e) = fs::create_dir_all(out_dir) {
            eprintln!(
                "오류: 출력 폴더를 만들 수 없습니다 - {}: {}",
                out_dir.display(),
                e
            );
            return EXIT_RUNTIME;
        }
    }

    let threads = threads_opt
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        })
        .max(1);

    let started = std::time::Instant::now();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    let tally = ordered::stream_records(
        paths.len(),
        threads,
        |idx| batch_record(mode, &paths[idx]),
        &mut out,
    );

    if tally.aborted {
        return EXIT_RUNTIME;
    }
    if let Err(e) = out.flush() {
        eprintln!("오류: stdout 쓰기 실패 - {}", e);
        return EXIT_RUNTIME;
    }

    eprintln!(
        "batch: {}건 중 {} 성공, {} 실패 ({}ms, threads={})",
        tally.emitted,
        tally.emitted - tally.failed,
        tally.failed,
        started.elapsed().as_millis(),
        threads
    );
    if tally.verify_diff > 0 || tally.verify_pages_diff > 0 {
        eprintln!(
            "batch: 검증 판정 — verify 차이 {}건, verify-pages 불일치 {}건 (변환·저장 자체는 성공)",
            tally.verify_diff, tally.verify_pages_diff
        );
    }
    tally.exit_code()
}

/// [#3238] batch 가 처리하는 서브커맨드 축.
#[derive(Clone, Copy)]
enum BatchMode<'a> {
    ExportText,
    Info,
    /// [#3261] 문서 개요/조문 구조 — `export-structure --json` 과 스키마 공유.
    Structure(rhwp::document_core::queries::structure::StructureMode),
    /// [#3346] 표 격자 — `export-tables --json` 과 스키마 공유.
    Tables,
    /// [#3346] 누름틀 조사 — `fields --json` 과 스키마 공유.
    Fields,
    /// [#3346] 주소를 가진 검색 — `search --json` 과 스키마 공유.
    Search {
        query: &'a str,
    },
    /// [#3626] 편집 가능 HWP5 변환 저장 — `convert --json` 봉투와 스키마 공유.
    /// 읽기 전용인 다른 축과 달리 입력마다 파일을 쓰므로 목적지(`out_dir`)를 들고 다닌다.
    Convert {
        out_dir: &'a Path,
        verify: ConversionVerifyOptions,
    },
    /// [#3830] 날짜·금액·수량 추출 — `extract-data --json` 봉투와 스키마 공유.
    /// `limit` 은 **문서마다** 적용되는 상한이다(§6-10) — 전건을 이 축에서 훑어 상한을
    /// 적용하면 뒤쪽 문서가 조용히 0건이 되므로, 문서 하나를 처리하는 이 함수 내부에서
    /// 매 문서마다 독립적으로 절단한다.
    ExtractData {
        kind: &'a str,
        limit: Option<usize>,
    },
}

/// [#3238] 파일 하나를 처리해 NDJSON 레코드 하나를 만든다. 실패는 레코드로 보고하고
/// 스트림은 계속된다 — 프로세스 중단 없이 부분 실패를 종료 코드로 신호하기 위함.
///
/// 배치는 신뢰할 수 없는 대량 코퍼스를 훑는 용도라, 한 건의 파서 panic 이 배치 전체를
/// 죽여서는 안 된다. panic 도 해당 파일의 `error` 레코드로 격리한다.
fn batch_record(mode: BatchMode<'_>, path: &str) -> serde_json::Value {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match mode {
        BatchMode::ExportText => query::export_text_record(path),
        BatchMode::Info => query::info_record(path),
        BatchMode::Structure(structure_mode) => query::structure_record(path, structure_mode),
        BatchMode::Tables => query::tables_record(path),
        BatchMode::Fields => query::fields_record(path),
        BatchMode::Search { query } => query::search_record(path, query),
        BatchMode::Convert { out_dir, verify } => {
            crate::cli::commands::batch_convert::record(path, out_dir, verify)
        }
        BatchMode::ExtractData { kind, limit } => query::extract_data_record(path, kind, limit),
    })) {
        Ok(record) => record,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "원인 불명".to_string());
            fail_record(path, format!("내부 오류(panic): {}", message))
        }
    }
}

pub(crate) fn fail_record(path: &str, message: String) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": path,
            "error": message,
            "exitClass": "runtime",
        }),
        "batch",
    )
}
