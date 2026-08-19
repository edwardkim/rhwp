//! 배포 전 보안 축 — 구조 위협·본문 주입 신호. 문서를 고치지 않는다.

use crate::envelope::{
    envelope, load_core, one_file, page_texts, print_json, read_file, EXIT_GATE, EXIT_OK,
    EXIT_RUNTIME, EXIT_USAGE,
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

pub fn run_hidden_text(args: &[String]) -> i32 {
    use rhwp::document_core::queries::hidden_text::HiddenTextOptions;

    let usage = "rhwp-agent hidden-text <파일> [--json] [--threshold-pt <N>] [--include-offpage]";
    let mut json_mode = false;
    let mut include_off_page = false;
    let mut path: Option<String> = None;
    let mut opts = HiddenTextOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--include-offpage" => {
                include_off_page = true;
                opts.include_off_page = true;
                i += 1;
            }
            "--threshold-pt" => {
                let parsed = args.get(i + 1).and_then(|v| v.parse::<f64>().ok());
                match parsed {
                    Some(n) if n.is_finite() && (0.0..=4096.0).contains(&n) => {
                        opts.threshold_pt = n;
                        i += 2;
                    }
                    _ => {
                        eprintln!(
                            "오류: --threshold-pt 뒤에 0 이상 4096 이하의 실수가 필요합니다."
                        );
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다.");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let data = match read_file(&path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let core = match load_core(&data) {
        Ok(c) => c,
        Err(fail) => {
            eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
            return EXIT_RUNTIME;
        }
    };
    let report = core.detect_hidden_text(&opts);
    let payload = json!({
        "source": path,
        "thresholdPt": opts.threshold_pt,
        "includeOffPage": include_off_page,
        "hiddenText": report.hidden_text,
        "hiddenCharCount": report.hidden_char_count,
        "clean": report.clean,
    });
    if json_mode {
        print_json(&envelope("hidden-text", payload, &["hiddenText[].excerpt"]));
    } else {
        crate::outln!(
            "clean={} findings={} chars={}",
            report.clean,
            report.hidden_text.len(),
            report.hidden_char_count
        );
    }
    if report.clean {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}

pub fn run_unicode_scan(args: &[String]) -> i32 {
    use rhwp::document_core::text_security as ts;
    use rhwp::model::control::Control;

    let usage =
        "rhwp-agent unicode-scan <파일> [--json] [--kind zero-width|bidi|tag|confusable|all]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut kind_filter: Option<ts::DeceptionKind> = None;
    let mut kind_label = "all";
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--kind" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!(
                        "오류: --kind 뒤에 축 이름이 필요합니다 (zero-width|bidi|tag|confusable|all)."
                    );
                    return EXIT_USAGE;
                };
                if value == "all" {
                    kind_filter = None;
                    kind_label = "all";
                } else if let Some(k) = ts::DeceptionKind::from_filter(value) {
                    kind_filter = Some(k);
                    kind_label = k.filter_name();
                } else {
                    eprintln!("오류: 알 수 없는 --kind 값입니다 - {value}");
                    eprintln!("가능한 값: zero-width, bidi, tag, confusable, all");
                    return EXIT_USAGE;
                }
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다.");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let data = match read_file(&path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let core = match load_core(&data) {
        Ok(c) => c,
        Err(fail) => {
            eprintln!("오류: 문서를 열 수 없습니다 - {path}: {}", fail.message);
            return EXIT_RUNTIME;
        }
    };

    let mut findings = Vec::new();
    let mut scanned_chars = 0usize;
    let document = core.document();
    for (si, section) in document.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            push_unicode_findings(
                &mut findings,
                &mut scanned_chars,
                si,
                pi,
                "body",
                &para.text,
                kind_filter,
            );
            for ctrl in &para.controls {
                match ctrl {
                    Control::Table(table) => {
                        for cell in &table.cells {
                            for cell_para in &cell.paragraphs {
                                push_unicode_findings(
                                    &mut findings,
                                    &mut scanned_chars,
                                    si,
                                    pi,
                                    "table",
                                    &cell_para.text,
                                    kind_filter,
                                );
                            }
                        }
                    }
                    Control::Shape(shape) => {
                        if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                            for tb_para in &tb.paragraphs {
                                push_unicode_findings(
                                    &mut findings,
                                    &mut scanned_chars,
                                    si,
                                    pi,
                                    "textbox",
                                    &tb_para.text,
                                    kind_filter,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let clean = findings.is_empty();
    let payload = json!({
        "source": path,
        "kind": kind_label,
        "findingCount": findings.len(),
        "scannedChars": scanned_chars,
        "clean": clean,
        "findings": findings,
    });
    if json_mode {
        print_json(&envelope(
            "unicode-scan",
            payload,
            &[
                "findings[].excerpt",
                "findings[].rendered",
                "findings[].raw",
            ],
        ));
    } else {
        crate::outln!(
            "clean={clean} findings={} scannedChars={scanned_chars}",
            findings.len()
        );
    }
    if clean {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}

fn push_unicode_findings(
    out: &mut Vec<serde_json::Value>,
    scanned_chars: &mut usize,
    section: usize,
    paragraph: usize,
    location: &str,
    text: &str,
    only: Option<rhwp::document_core::text_security::DeceptionKind>,
) {
    use rhwp::document_core::text_security as ts;

    *scanned_chars += text.chars().count();
    for f in ts::scan_deception(text, only) {
        let mut item = json!({
            "kind": f.kind.label(),
            "codepoint": ts::format_codepoint(f.codepoint),
            "severity": f.severity.label(),
            "section": section,
            "paragraph": paragraph,
            "location": location,
            "charOffset": f.char_offset,
            "runLength": f.run_length,
            "excerpt": f.excerpt,
            "rendered": f.rendered,
            "raw": f.raw,
            "why": f.kind.why(),
        });
        if let Some(hidden) = f.hidden {
            item["hidden"] = json!(hidden);
        }
        out.push(item);
    }
}
