//! 배포 전 보안 축 — 구조 위협·본문 주입 신호. 문서를 고치지 않는다.

use crate::envelope::{
    envelope, load_core, one_file, page_texts, print_json, read_file, EXIT_GATE, EXIT_OK,
    EXIT_RUNTIME,
};
use rhwp::document_core::queries::injection_scan;
use rhwp::document_core::queries::threat_scan;
use serde_json::json;

pub fn run_threat_scan(args: &[String]) -> i32 {
    let usage = "rhwp-agent threat-scan <파일> [--json]";
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
    let report = threat_scan::scan_bytes(&opts.path, &data);
    let payload = json!({
        "source": report.source,
        "format": report.format,
        "findingCount": report.findings.len(),
        "clean": report.clean(),
        "truncated": report.truncated,
        "highestSeverity": report.highest_severity(),
        "notes": report.notes,
    });
    if opts.json {
        print_json(&envelope("threat-scan", payload, &[]));
    } else {
        crate::outln!(
            "clean={} findings={}",
            report.clean(),
            report.findings.len()
        );
    }
    if report.clean() {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}

pub fn run_injection_scan(args: &[String]) -> i32 {
    let usage = "rhwp-agent injection-scan <파일> [--json]";
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
    let pages = match page_texts(&core) {
        Ok(p) => p,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let tools = ["fill-fields", "replace-text", "run", "mcp-serve"]
        .iter()
        .map(|s| (*s).to_string())
        .collect::<Vec<_>>();
    let mut signals = Vec::new();
    for (i, text) in pages.iter().enumerate() {
        for sig in injection_scan::scan_text(text, &tools) {
            signals.push(json!({
                "page": i,
                "kind": sig.kind,
                "matched": sig.matched,
                "charOffset": sig.char_offset,
                "why": sig.why,
            }));
        }
    }
    let clean = signals.is_empty();
    let payload = json!({
        "source": opts.path,
        "signalCount": signals.len(),
        "clean": clean,
        "signals": signals,
    });
    if opts.json {
        print_json(&envelope("injection-scan", payload, &[]));
    } else {
        crate::outln!("clean={clean} signals={}", signals.len());
    }
    if clean {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}
