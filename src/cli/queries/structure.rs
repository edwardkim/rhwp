//! 문서 개요·조문 계층을 만드는 비렌더 structure query 어댑터.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// [#3261] `export-structure --json`·`batch export-structure --json` 이 공유하는
/// 구조 봉투 레코드. `mode`/`nodeCount` 를 톱레벨로 올려 스윕 선별(jq select)이 싸다.
pub(crate) fn structure_json_value(
    file_path: &str,
    st: &rhwp::document_core::queries::structure::StructureDoc,
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "mode": st.mode,
            "nodeCount": st.node_count,
            "structure": st,
        }),
        "export-structure",
    )
}

/// `export-structure` — 문서 개요/조문 계층을 중첩 JSON 트리로 추출 (조문 DB화용).
pub(crate) fn export_structure(args: &[String]) -> i32 {
    use rhwp::document_core::queries::structure::{build_structure, StructureMode};

    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut mode = StructureMode::Auto;
    // [#3261] --json: 계약 봉투(schemaVersion·source)를 씌운 한 줄 JSON.
    // 기본 출력(무봉투 pretty JSON·-o 파일 저장)은 기존 소비자 계약이라 건드리지 않는다.
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--mode" => {
                i += 1;
                match args.get(i).and_then(|s| StructureMode::parse(s)) {
                    Some(m) => mode = m,
                    None => {
                        eprintln!("오류: --mode 는 auto|outline|clause");
                        return EXIT_USAGE;
                    }
                }
            }
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
            "사용법: rhwp export-structure <파일> [--mode auto|outline|clause] [-o out.json]"
        );
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let st = build_structure(doc.document(), mode);

    if json_mode {
        // [#3261] 봉투는 한 줄 — NDJSON(batch)과 같은 스키마로 단건/배치 동일 소비.
        let envelope = structure_json_value(file_path, &st);
        println!("{envelope}");
        return EXIT_OK;
    }

    let json = match serde_json::to_string_pretty(&st) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("오류: JSON 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    match out_path {
        Some(p) => match fs::write(&p, &json) {
            Ok(_) => {
                println!(
                    "구조 추출 완료: mode={} 노드={} → {}",
                    st.mode, st.node_count, p
                );
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 출력 쓰기 실패 - {}: {}", p, e);
                // [#2707] 출력 파일을 못 쓴 실행은 실패다.
                EXIT_RUNTIME
            }
        },
        None => {
            println!("{json}");
            EXIT_OK
        }
    }
}
