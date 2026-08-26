//! 처음 보는 문서를 결정론적 규칙 문장으로 설명하는 read-only CLI 어댑터.

use std::fs;

use crate::{
    collect_field_records, load_document, provenance, ENVELOPE_SCHEMA_VERSION, EXIT_OK,
    EXIT_RUNTIME, EXIT_USAGE,
};

/// [#3828] `explain --json` 봉투의 표 항목 — `export-tables` 격자에서 텍스트를 빼고
/// 크기·병합 여부만 남긴다. 셀 내용을 싣지 않으므로 이 필드들은 전부 엔진값이다
/// (`src/provenance.rs` 의 `explain` 항목이 그 근거를 명시한다).
fn explain_table_summary(
    grid: &rhwp::document_core::queries::table_extract::TableGrid,
) -> serde_json::Value {
    let has_merged_cells = grid.cells.iter().any(|c| c.row_span > 1 || c.col_span > 1);
    serde_json::json!({
        "index": grid.index,
        "rows": grid.rows,
        "cols": grid.cols,
        "hasMergedCells": has_merged_cells,
    })
}

/// [#3828] 표 하나를 사람 문장 조각으로 만든다 — "표 1(3×4, 병합 셀 있음)".
/// 1 기준 번호를 쓰는 이유는 `export-tables` 의 0 기준 `index` 를 그대로 읽는 사람이
/// "0번 표"라는 어색한 표현을 안 보게 하려는 것뿐이고, JSON 쪽 `index` 는 0 기준을
/// 그대로 유지해 `export-tables`·`hwp_table_to_csv` 의 표 번호와 어긋나지 않는다.
fn explain_table_phrase(t: &serde_json::Value) -> String {
    let human_no = t["index"].as_u64().unwrap_or(0) + 1;
    let rows = t["rows"].as_u64().unwrap_or(0);
    let cols = t["cols"].as_u64().unwrap_or(0);
    if t["hasMergedCells"] == true {
        format!("표 {human_no}({rows}×{cols}, 병합 셀 있음)")
    } else {
        format!("표 {human_no}({rows}×{cols})")
    }
}

/// [#3828] `explain`·`explain --json` 이 공유하는 사람 문장 조립.
///
/// 결정론적 템플릿 조립이다 — 네 조회(`info`·`export-structure`·`export-tables`·
/// `fields`)와 각주/미주 집계가 이미 확정한 값을 문장으로 옮길 뿐, 여기서 새로
/// 판정하는 값은 없다. "부분 목록 금지"(#3719) 원칙에 따라 확신 없는 값은 만들지
/// 않는다 — 표·필드 이름은 있는 그대로 전부 나열하고, 축약·상위 N개 자르기를 하지
/// 않는다.
fn explain_summary(
    format_label: &str,
    page_count: u32,
    para_count: usize,
    tables: &[serde_json::Value],
    field_names: &[String],
    footnote_count: usize,
    endnote_count: usize,
    encrypted: bool,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "이 문서는 {format_label} 형식, {page_count}쪽, 문단 {para_count}개다."
    ));

    if tables.is_empty() {
        lines.push("표는 없다.".to_string());
    } else {
        let phrases: Vec<String> = tables.iter().map(explain_table_phrase).collect();
        lines.push(format!(
            "표가 {}개 있다 — {}.",
            tables.len(),
            phrases.join(", ")
        ));
    }

    if field_names.is_empty() {
        lines.push("누름틀은 없다.".to_string());
    } else {
        lines.push(format!(
            "누름틀이 {}개 있다 — 이름: {}.",
            field_names.len(),
            field_names.join(", ")
        ));
    }

    if footnote_count == 0 && endnote_count == 0 {
        lines.push("각주와 미주는 모두 없다.".to_string());
    } else {
        lines.push(format!(
            "각주가 {footnote_count}개, 미주가 {endnote_count}개 있다."
        ));
    }

    lines.push(if encrypted {
        "암호로 보호돼 있다.".to_string()
    } else {
        "암호로 보호돼 있지 않다.".to_string()
    });

    lines.join("\n")
}

/// [#3828] `explain --json` 이 내는 계약 봉투. `capabilities --mcp` 의 `hwp_explain`
/// 도구와 CLI `explain --json`이 이 함수 하나를 공유한다.
fn explain_json_value(
    file_path: &str,
    format_label: &str,
    page_count: u32,
    para_count: usize,
    tables: Vec<serde_json::Value>,
    field_names: Vec<String>,
    footnote_count: usize,
    endnote_count: usize,
    encrypted: bool,
) -> serde_json::Value {
    let summary = explain_summary(
        format_label,
        page_count,
        para_count,
        &tables,
        &field_names,
        footnote_count,
        endnote_count,
        encrypted,
    );
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "format": format_label,
            "pageCount": page_count,
            "paragraphCount": para_count,
            "tables": tables,
            "fields": field_names,
            "footnoteCount": footnote_count,
            "endnoteCount": endnote_count,
            "encrypted": encrypted,
            "summary": summary,
        }),
        "explain",
    )
}

/// `rhwp explain <파일> [--json]` — 처음 보는 문서를 결정론적 규칙 문장으로 설명한다.
///
/// [#3828] 새 판정 로직이 아니라 기존 조회(`info`·`export-structure`·`export-tables`·
/// `fields`)가 이미 계산한 값의 조합이다 — LLM 을 쓰지 않는다. 암호 문서는
/// `load_document` 가 다른 명령과 같은 규약(비밀번호 없으면 `EXIT_USAGE`, 틀리면
/// `EXIT_RUNTIME`)으로 거부하므로 explain 도 자동으로 그 규약을 따른다.
pub(crate) fn explain_document(args: &[String]) -> i32 {
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
        eprintln!("사용법: rhwp explain <파일.hwp|파일.hwpx|파일.hml> [--json]");
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
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
    let page_count = doc.page_count();
    let para_count: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();

    use rhwp::document_core::queries::table_extract::extract_tables;
    let tables: Vec<serde_json::Value> = extract_tables(document)
        .iter()
        .map(explain_table_summary)
        .collect();

    let field_records = collect_field_records(&doc);
    let field_names: Vec<String> = field_records
        .iter()
        .map(|f| f["name"].as_str().unwrap_or("").to_string())
        .collect();

    let notes = rhwp::document_core::queries::explain::count_notes(document);
    let encrypted = document.header.encrypted;

    if json_mode {
        let envelope = explain_json_value(
            file_path,
            format_label,
            page_count,
            para_count,
            tables,
            field_names,
            notes.footnote_count,
            notes.endnote_count,
            encrypted,
        );
        println!("{envelope}");
        return EXIT_OK;
    }

    let summary = explain_summary(
        format_label,
        page_count,
        para_count,
        &tables,
        &field_names,
        notes.footnote_count,
        notes.endnote_count,
        encrypted,
    );
    println!("{summary}");
    EXIT_OK
}
