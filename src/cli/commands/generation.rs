//! Structured-input document generation command adapters.

use std::fs;
use std::path::Path;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn build_from_ingest(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>");
        return EXIT_USAGE;
    }

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut media_dir: Option<&str> = None;
    // [#3600] --json: 생성 봉투를 stdout 순수 JSON 으로. 생성 동작 무변경.
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: -o 옵션에 값이 필요합니다");
                    return EXIT_USAGE;
                }
                output_path = Some(&args[i + 1]);
                i += 2;
            }
            "--media-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: --media-dir 옵션에 값이 필요합니다");
                    return EXIT_USAGE;
                }
                media_dir = Some(&args[i + 1]);
                i += 2;
            }
            "--json" => {
                json_mode = true;
                i += 1;
            }
            // [#3600] 미지 옵션 침묵 무시 제거 — #3349/#2551 계열 규약(즉시 exit 2)과 정합.
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if input_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
                i += 1;
            }
        }
    }

    let input = match input_path {
        Some(p) => p,
        None => {
            eprintln!("오류: 입력 ingest JSON 경로가 누락되었습니다");
            return EXIT_USAGE;
        }
    };
    let output = match output_path {
        Some(p) => p,
        None => {
            eprintln!("오류: -o <출력 경로> 가 누락되었습니다");
            return EXIT_USAGE;
        }
    };

    let bytes = match fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 입력 파일 읽기 실패 - {}: {}", input, e);
            return EXIT_RUNTIME;
        }
    };

    let ingest = match rhwp::parser::ingest::parse_ingest_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: ingest JSON 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(md) = media_dir {
        let p = Path::new(md);
        if !p.exists() {
            eprintln!(
                "경고: 미디어 디렉토리가 존재하지 않습니다 ({}). 본 단계는 이미지 placeholder로 처리됩니다.",
                md
            );
        }
    }

    let doc = rhwp::document_core::builders::exam_paper::build_exam_paper(&ingest);

    let hwpx_bytes = match rhwp::serializer::serialize_hwpx(&doc) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: HWPX 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    match fs::write(output, &hwpx_bytes) {
        Ok(_) => {
            let paragraph_count: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
            if json_mode {
                println!(
                    "{}",
                    provenance::marked(
                        serde_json::json!({
                            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                            "source": input,
                            "output": output,
                            "format": "hwpx",
                            "bytes": hwpx_bytes.len(),
                            "questionCount": ingest.questions.len(),
                            "paragraphCount": paragraph_count,
                        }),
                        "build-from-ingest",
                    )
                );
            } else {
                println!(
                    "저장 완료: {} ({}바이트, 문제 {}개, 문단 {}개)",
                    output,
                    hwpx_bytes.len(),
                    ingest.questions.len(),
                    paragraph_count
                );
            }
            EXIT_OK
        }
        Err(e) => {
            eprintln!("오류: 파일 저장 실패 - {}: {}", output, e);
            EXIT_RUNTIME
        }
    }
}

/// `rhwp scaffold <spec.json> [--format hwpx] [-o out.hwpx] [--json]`
///
/// 구조화된 명세(JSON)에서 유효한 HWPX 문서를 생성한다. `build-from-ingest` 와 같은
/// 생성(authoring) 계열이며, rhwp 의 읽기/편집 축과 직교한다 — 입력은 문서가 아니라
/// 호출자(사용자/에이전트)가 만든 계획서다. 지원 요소(제목·개요 제목·문단·단순 표)는
/// 모두 왕복 검증을 통과한 것만 방출한다 (`src/scaffold/` 참조).
pub(crate) fn run_scaffold(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp scaffold <spec.json> [--format hwpx] -o <out.hwpx> [--json]");
        return EXIT_USAGE;
    }

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut format: &str = "hwpx";
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: -o 옵션에 값이 필요합니다");
                    return EXIT_USAGE;
                }
                output_path = Some(&args[i + 1]);
                i += 2;
            }
            "--format" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: --format 옵션에 값이 필요합니다");
                    return EXIT_USAGE;
                }
                format = &args[i + 1];
                i += 2;
            }
            "--json" => {
                json_mode = true;
                i += 1;
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if input_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
                i += 1;
            }
        }
    }

    if !format.eq_ignore_ascii_case("hwpx") {
        eprintln!("오류: 지원하는 --format 은 hwpx 뿐입니다 (받음: {format})");
        return EXIT_USAGE;
    }

    let input = match input_path {
        Some(p) => p,
        None => {
            eprintln!("오류: 입력 spec JSON 경로가 누락되었습니다");
            return EXIT_USAGE;
        }
    };
    let output = match output_path {
        Some(p) => p,
        None => {
            eprintln!("오류: -o <출력 경로> 가 누락되었습니다");
            return EXIT_USAGE;
        }
    };

    let bytes = match fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 입력 파일 읽기 실패 - {}: {}", input, e);
            return EXIT_RUNTIME;
        }
    };

    let spec = match rhwp::scaffold::parse_scaffold_bytes(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: scaffold JSON 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let doc = rhwp::scaffold::build_scaffold(&spec);

    let hwpx_bytes = match rhwp::serializer::serialize_hwpx(&doc) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: HWPX 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    match fs::write(output, &hwpx_bytes) {
        Ok(_) => {
            let paragraph_count: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
            let table_count = spec
                .blocks
                .iter()
                .filter(|b| matches!(b, rhwp::scaffold::Block::Table { .. }))
                .count();
            if json_mode {
                println!(
                    "{}",
                    provenance::marked(
                        serde_json::json!({
                            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                            "source": input,
                            "output": output,
                            "format": "hwpx",
                            "bytes": hwpx_bytes.len(),
                            "blockCount": spec.blocks.len(),
                            "paragraphCount": paragraph_count,
                            "tableCount": table_count,
                        }),
                        "scaffold",
                    )
                );
            } else {
                println!(
                    "저장 완료: {} ({}바이트, 블록 {}개, 문단 {}개, 표 {}개)",
                    output,
                    hwpx_bytes.len(),
                    spec.blocks.len(),
                    paragraph_count,
                    table_count
                );
            }
            EXIT_OK
        }
        Err(e) => {
            eprintln!("오류: 파일 저장 실패 - {}: {}", output, e);
            EXIT_RUNTIME
        }
    }
}
