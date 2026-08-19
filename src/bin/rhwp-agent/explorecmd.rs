//! 문서별 행동 메뉴. `explore::build_menu` 에 기존 조회 개수만 넘긴다.

use crate::envelope::{
    envelope, format_token, load_core, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME,
};
use rhwp::document_core::queries::chart_extract::collect_charts;
use rhwp::document_core::queries::explore::{build_menu, DocFacts, HONESTY_NOTE};
use rhwp::document_core::queries::hidden_text::HiddenTextOptions;
use rhwp::document_core::queries::injection_scan::{Confidence, InjectionScanOptions};
use rhwp::document_core::queries::structure::{build_structure, StructureMode};
use rhwp::document_core::queries::table_extract::extract_tables;
use serde_json::json;

pub fn run_explore(args: &[String]) -> i32 {
    let usage = "rhwp-agent explore <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let data = match read_file(&opts.path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let format = rhwp::parser::detect_format(&data);
    let core = match load_core(&data) {
        Ok(c) => c,
        Err(fail) => {
            eprintln!(
                "오류: 문서를 열 수 없습니다 - {}: {}",
                opts.path, fail.message
            );
            return EXIT_RUNTIME;
        }
    };
    let document = core.document();
    let tables = extract_tables(document);
    let merged_table_count = tables
        .iter()
        .filter(|g| g.cells.iter().any(|c| c.row_span > 1 || c.col_span > 1))
        .count();
    let structure = build_structure(document, StructureMode::Auto);
    let notes = rhwp::document_core::queries::explain::count_notes(document);
    let injection = core.scan_injection(&InjectionScanOptions {
        min_confidence: Confidence::Low,
        include_fields: true,
        tool_names: ["fill-fields", "replace-text", "run", "mcp-serve"]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    });
    let hidden = core.detect_hidden_text(&HiddenTextOptions::default());
    let format_label = match format_token(format) {
        "hwp5" => "HWP5",
        "hwpx" => "HWPX",
        "hwp3" => "HWP3",
        "hml" => "HML",
        "drm-protected" => "DRM",
        "empty" => "빈 파일",
        _ => "알 수 없음",
    };
    let facts = DocFacts {
        format_label: format_label.to_string(),
        page_count: core.page_count(),
        para_count: document.sections.iter().map(|s| s.paragraphs.len()).sum(),
        table_count: tables.len(),
        merged_table_count,
        field_count: core.collect_all_fields().len(),
        chart_count: collect_charts(document).len(),
        structure_node_count: structure.node_count,
        footnote_count: notes.footnote_count,
        endnote_count: notes.endnote_count,
        injection_signal_count: injection.len(),
        hidden_text_count: hidden.hidden_text.len(),
        encrypted: document.header.encrypted,
    };
    let menu = build_menu(&facts);
    let menu_json: Vec<serde_json::Value> = menu
        .iter()
        .map(|a| {
            json!({
                "affordance": a.affordance,
                "why": a.why,
                "command": a.command,
                "skill": a.skill,
                "confidence": a.confidence,
            })
        })
        .collect();
    let payload = json!({
        "source": opts.path,
        "format": format_label,
        "pageCount": facts.page_count,
        "encrypted": facts.encrypted,
        "fieldCount": facts.field_count,
        "tableCount": facts.table_count,
        "affordanceCount": menu.len(),
        "menu": menu_json,
        "note": HONESTY_NOTE,
    });
    if opts.json {
        print_json(&envelope("explore", payload, &[]));
    } else {
        crate::outln!(
            "{} format={} pages={} affordances={}",
            opts.path,
            format_label,
            facts.page_count,
            menu.len()
        );
        for a in &menu {
            crate::outln!("  [{}] {} — {}", a.confidence, a.affordance, a.command);
        }
    }
    EXIT_OK
}
