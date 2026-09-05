//! 문서에 적용할 수 있는 행동을 결정론적으로 안내하는 read-only CLI 어댑터.

use std::fs;

use crate::{load_document, ENVELOPE_SCHEMA_VERSION, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

/// `rhwp explore <파일> [--json]` — 이 문서로 **무엇을 할 수 있는지** 어포던스 메뉴를 라우팅한다.
///
/// [#gym] 새 판정 로직이 아니라 기존 조회(`table_extract`·`field_query`·`structure`·
/// `chart_extract`·`explain::count_notes`·`injection_scan`·`hidden_text`)가 이미 센
/// 개수에서 유도한 결정론적 메뉴다 — 순위·근거·명령 매핑 로직은
/// `document_core::queries::explore` 에 있다. `explain`(문서가 무엇인지)·
/// `capabilities`(도구 일반)와 달리 explore 는 **이 문서로 무엇을 할 수 있는지**를
/// 라우팅한다. 정직한 휴리스틱이라 제안일 뿐 완전성을 보장하지 않는다. 암호 문서는
/// `load_document` 가 다른 명령과 같은 규약으로 처리한다.
pub(crate) fn explore_document(args: &[String]) -> i32 {
    use rhwp::document_core::queries::chart_extract::collect_charts;
    use rhwp::document_core::queries::explore::{build_menu, DocFacts, HONESTY_NOTE};
    use rhwp::document_core::queries::hidden_text::HiddenTextOptions;
    use rhwp::document_core::queries::injection_scan as scan;
    use rhwp::document_core::queries::structure::{build_structure, StructureMode};
    use rhwp::document_core::queries::table_extract::extract_tables;

    let mut json_mode = false;
    let mut file_path: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
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
    }
    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp explore <파일.hwp|파일.hwpx|파일.hml> [--json]");
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {file_path}: {e}");
            return EXIT_RUNTIME;
        }
    };

    let detected_format = rhwp::parser::detect_format(&data);
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let document = doc.document();

    let format_label = match detected_format {
        rhwp::parser::FileFormat::Hwp => "HWP5",
        rhwp::parser::FileFormat::Hwpx => "HWPX",
        rhwp::parser::FileFormat::Hwp3 => "HWP3",
        rhwp::parser::FileFormat::Hml => "HML",
        rhwp::parser::FileFormat::DrmProtected => "DRM",
        rhwp::parser::FileFormat::Empty => "빈 파일",
        rhwp::parser::FileFormat::Unknown => "알 수 없음",
    };

    // 기존 공개 조회를 각각 한 번씩만 호출해 개수를 모은다 — 탐지기 재구현·재파싱 없음.
    let tables = extract_tables(document);
    let merged_table_count = tables
        .iter()
        .filter(|g| g.cells.iter().any(|c| c.row_span > 1 || c.col_span > 1))
        .count();
    let structure = build_structure(document, StructureMode::Auto);
    let notes = rhwp::document_core::queries::explain::count_notes(document);
    let injection_options = scan::InjectionScanOptions {
        min_confidence: scan::Confidence::Low,
        include_fields: true,
        tool_names: Vec::new(),
    };
    let injection_signal_count = doc.scan_injection(&injection_options).len();
    let hidden = doc.detect_hidden_text(&HiddenTextOptions::default());

    let facts = DocFacts {
        format_label: format_label.to_string(),
        page_count: doc.page_count(),
        para_count: document.sections.iter().map(|s| s.paragraphs.len()).sum(),
        table_count: tables.len(),
        merged_table_count,
        field_count: doc.collect_all_fields().len(),
        chart_count: collect_charts(document).len(),
        structure_node_count: structure.node_count,
        footnote_count: notes.footnote_count,
        endnote_count: notes.endnote_count,
        injection_signal_count,
        hidden_text_count: hidden.hidden_text.len(),
        encrypted: document.header.encrypted,
    };

    let menu = build_menu(&facts);

    if json_mode {
        let menu_json: Vec<serde_json::Value> = menu
            .iter()
            .map(|a| {
                serde_json::json!({
                    "affordance": a.affordance,
                    "why": a.why,
                    "command": a.command,
                    "skill": a.skill,
                    "confidence": a.confidence,
                })
            })
            .collect();
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "format": format_label,
            "pageCount": facts.page_count,
            "encrypted": facts.encrypted,
            "affordanceCount": menu.len(),
            "menu": menu_json,
            "note": HONESTY_NOTE,
        });
        println!("{envelope}");
        return EXIT_OK;
    }

    // 기본 출력은 사람용 메뉴 — 기계 소비는 --json 이 담당한다.
    println!(
        "이 문서로 해 볼 수 있는 행동 ({format_label} 형식·{}쪽·어포던스 {}개):",
        facts.page_count,
        menu.len()
    );
    for (i, a) in menu.iter().enumerate() {
        println!(
            "  {}. [{}] {} — 스킬 {}",
            i + 1,
            a.confidence,
            a.affordance,
            a.skill
        );
        println!("      이유: {}", a.why);
        println!("      명령: {}", a.command);
    }
    println!();
    println!("{HONESTY_NOTE}");
    EXIT_OK
}
