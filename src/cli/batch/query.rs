//! Read-only per-document projections used by the ordered batch stream.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use super::fail_record;
use crate::cli::queries::structure::structure_json_value;
use crate::{
    collect_field_records, extract_data_json_value, fields_json_value, info_json_value,
    search_json_value, tables_json_value,
};

pub(super) fn export_text_record(path: &str) -> serde_json::Value {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {}", e)),
    };

    let page_count = doc.page_count();
    let mut text = String::new();
    for page_num in 0..page_count {
        match doc.extract_page_text_native(page_num) {
            Ok(t) => {
                text.push_str(&t);
                if !t.ends_with('\n') {
                    text.push('\n');
                }
            }
            Err(e) => {
                return fail_record(path, format!("페이지 {} 텍스트 추출 실패: {}", page_num, e))
            }
        }
    }

    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": path,
            "pageCount": page_count,
            "text": text,
        }),
        "export-text",
    )
}

/// [#3830] `batch extract-data --json` 의 파일당 레코드 — 단건 `extract-data --json`
/// 봉투(`extract_data_json_value` 공유)와 같은 스키마다. 추출 로직은 새로 만들지 않고
/// `DocumentCore::extract_data` 를 그대로 부른다(`extract_data_command` 와 동일한 절차).
///
/// [§6-10] `limit` 은 **이 문서 하나**에 대한 상한이다 — 배치 전체에 걸친 전역 상한이
/// 아니다. 전역 상한으로 읽으면 앞선 문서가 한도를 다 써버려 뒤 문서가 조용히 0건으로
/// 보고되고, 소비자는 "그 문서에 값이 없다"와 "한도를 이미 다 썼다"를 구별할 수 없다.
/// 그래서 문서마다 독립적으로 전수 추출 후 절단한다 — 단건 `extract-data` 와 같은 규약.
pub(super) fn extract_data_record(
    path: &str,
    kind_arg: &str,
    limit: Option<usize>,
) -> serde_json::Value {
    use rhwp::document_core::queries::extract_data::DataKind;

    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {}", e)),
    };

    let selected: Vec<DataKind> = if kind_arg == "all" {
        DataKind::ALL.to_vec()
    } else {
        DataKind::parse(kind_arg).into_iter().collect()
    };

    let all_items = doc.extract_data(&selected);
    let total_item_count = all_items.len();
    let mut counts = serde_json::Map::new();
    for kind in &selected {
        let n = all_items.iter().filter(|it| it.kind == *kind).count();
        counts.insert(kind.as_str().to_string(), serde_json::json!(n));
    }
    let counts = serde_json::Value::Object(counts);

    let items: Vec<_> = match limit {
        Some(n) => all_items.into_iter().take(n).collect(),
        None => all_items,
    };

    extract_data_json_value(path, kind_arg, &items, total_item_count, &counts)
}

/// [#3261] `batch export-structure --json` 의 파일당 레코드 — `export-structure --json`
/// 봉투(`structure_json_value` 공유)와 같은 스키마다.
pub(super) fn structure_record(
    path: &str,
    mode: rhwp::document_core::queries::structure::StructureMode,
) -> serde_json::Value {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {}", e)),
    };
    let st = rhwp::document_core::queries::structure::build_structure(doc.document(), mode);
    structure_json_value(path, &st)
}

/// [#3346] `batch export-tables --json` 의 파일당 레코드 — `export-tables --json` 봉투와
/// 같은 스키마(`tables_json_value` 공유)다.
pub(super) fn tables_record(path: &str) -> serde_json::Value {
    use rhwp::document_core::queries::table_extract::extract_tables;
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {}", e)),
    };
    let tables = extract_tables(doc.document());
    tables_json_value(path, &tables)
}

/// [#3346] `batch fields --json` 의 파일당 레코드 — `fields --json` 봉투와 같은 스키마.
pub(super) fn fields_record(path: &str) -> serde_json::Value {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {}", e)),
    };
    let fields = collect_field_records(&doc);
    fields_json_value(path, &fields)
}

/// [#3346] `batch search --json` 의 파일당 레코드 — `search --json` 봉투와 같은 스키마.
///
/// 대량 코퍼스에서 한 문서가 매치를 수만 건 쏟아내면 스트림이 부풀므로, 배치 경로는
/// 파일당 매치 상한을 둔다(단건 `search --limit` 과 같은 취지).
pub(super) fn search_record(path: &str, query: &str) -> serde_json::Value {
    const BATCH_MATCH_LIMIT: usize = 1000;
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {}", e)),
    };
    // 단건 `search --limit`와 동일하게 전체 매치 수를 먼저 관찰하고, NDJSON 크기만
    // 배치 상한으로 자른다. 그래야 단건·배치가 같은 envelope 계약을 공유한다.
    let all_matches = doc.grep(query, true, None);
    let total_match_count = all_matches.len();
    let matches: Vec<_> = all_matches.into_iter().take(BATCH_MATCH_LIMIT).collect();
    search_json_value(path, query, true, &matches, total_match_count)
}

/// [#3238] `batch info --json` 의 파일당 레코드 — `info --json` 과 같은 스키마
/// (`info_json_value` 공유)라 소비자가 단건/배치를 같은 코드로 읽는다.
pub(super) fn info_record(path: &str) -> serde_json::Value {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파일을 읽을 수 없습니다: {}", e)),
    };
    let file_size = data.len();
    let detected_format = rhwp::parser::detect_format(&data);
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => return fail_record(path, format!("파싱 실패: {}", e)),
    };
    info_json_value(path, file_size, detected_format, &doc)
}
