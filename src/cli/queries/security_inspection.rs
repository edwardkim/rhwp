//! 문서의 비가시 텍스트를 보고하는 read-only 보안 조회 CLI 어댑터.

use std::fs;

use crate::{
    load_document, load_document_core, provenance, ENVELOPE_SCHEMA_VERSION, EXIT_OK, EXIT_RUNTIME,
    EXIT_USAGE,
};

/// `inspect hidden-text` — 사람 눈에 안 보이는데 추출기는 읽어 가는 텍스트를 보고한다.
///
/// 탐지 건수가 0이 아니어도 종료 코드는 0이다 — 1은 런타임 실패 전용이고(#2707),
/// "위험 문서 발견"은 실패가 아니라 **정상적으로 얻어낸 판정 결과**다. 소비자는
/// `clean` 필드로 분기한다.
pub(crate) fn inspect_hidden_text(args: &[String]) -> i32 {
    use rhwp::document_core::queries::hidden_text::HiddenTextOptions;

    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    let mut opts = HiddenTextOptions::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--include-offpage" => opts.include_off_page = true,
            "--threshold-pt" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<f64>().ok()) {
                    // 상한은 CharShape.base_size 의 스펙 상한(4096pt)과 같다.
                    Some(n) if n.is_finite() && (0.0..=4096.0).contains(&n) => {
                        opts.threshold_pt = n
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
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다.");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp inspect hidden-text <파일.hwp|파일.hwpx> [--json] [--threshold-pt <N>] [--include-offpage]");
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

    let report = doc.detect_hidden_text(&opts);

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "thresholdPt": opts.threshold_pt,
            "includeOffPage": opts.include_off_page,
            "hiddenText": report.hidden_text,
            "hiddenCharCount": report.hidden_char_count,
            "clean": report.clean,
        });
        println!("{}", provenance::marked(envelope, "inspect"));
        return EXIT_OK;
    }

    // 기본 출력은 사람용 요약 — 기계 소비는 --json 이 담당한다.
    if report.clean {
        println!("은닉 텍스트 없음: {} (탐지 0건)", file_path);
        return EXIT_OK;
    }
    println!(
        "은닉 텍스트 {}건 (문자 {}개): {}",
        report.hidden_text.len(),
        report.hidden_char_count,
        file_path
    );
    for f in &report.hidden_text {
        let kind = match f.kind {
            rhwp::document_core::queries::hidden_text::HiddenKind::SameAsBackground => {
                "배경색과 같은 글자색"
            }
            rhwp::document_core::queries::hidden_text::HiddenKind::NearInvisible => "극소 글자",
            rhwp::document_core::queries::hidden_text::HiddenKind::ZeroSize => "0pt 글자",
            rhwp::document_core::queries::hidden_text::HiddenKind::OffPage => "쪽 밖 배치",
        };
        let page = f
            .page
            .map(|p| format!("{}쪽", p + 1))
            .unwrap_or_else(|| "미배치".to_string());
        println!(
            "  [{}] 구역{}:문단{} ({}) {}자: {}",
            kind, f.section, f.paragraph, page, f.char_count, f.excerpt
        );
    }
    EXIT_OK
}

fn inspect_unicode_scan_unit(
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
        let mut item = serde_json::json!({
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
            item["hidden"] = serde_json::Value::String(hidden);
        }
        out.push(item);
    }
}

/// `rhwp inspect unicode` — 화면에 보이는 것과 LLM 이 읽는 바이트가 어긋나는 지점을 찾는다.
///
/// 문서 텍스트는 그대로 LLM 에게 간다. 사람이 "안전한 문서"라고 판단한 근거는 **화면**인데,
/// 제로폭 문자·방향 오버라이드·태그 문자는 화면에 흔적을 남기지 않고 텍스트에만 남는다.
/// 그래서 이 명령의 산출은 `rendered`(보이는 모습)와 `raw`(실제 순서)를 **나란히** 낸다 —
/// 차이를 눈에 보이게 하지 못하면 보고는 공허하다.
///
/// 문서는 읽기만 한다. 저장 경로가 없고 IR 을 고치지 않는다.
pub(crate) fn inspect_unicode(args: &[String]) -> i32 {
    use rhwp::document_core::text_security as ts;
    use rhwp::model::control::Control;

    let mut file_path: Option<&str> = None;
    let mut json_mode = false;
    let mut kind_filter: Option<ts::DeceptionKind> = None;
    let mut kind_label = "all";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--kind" => {
                i += 1;
                let Some(value) = args.get(i) else {
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
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.is_none() {
                    file_path = Some(other);
                } else {
                    eprintln!("오류: 인자가 너무 많습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!("오류: 검사할 문서 경로를 지정해주세요.");
        eprintln!(
            "사용법: rhwp inspect unicode <파일.hwp|파일.hwpx> [--json] [--kind zero-width|bidi|tag|confusable|all]"
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
    let core = match load_document_core(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let document = core.document();

    let mut findings: Vec<serde_json::Value> = Vec::new();
    let mut scanned_chars = 0usize;

    // 코드포인트 1패스 — 문서를 한 번 훑고 끝낸다. 글자마다 정규식을 돌리지 않는다.
    for (si, section) in document.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            inspect_unicode_scan_unit(
                &mut findings,
                &mut scanned_chars,
                si,
                pi,
                "body",
                &para.text,
                kind_filter,
            );
            for (ci, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Table(table) => {
                        for (celli, cell) in table.cells.iter().enumerate() {
                            for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                                let loc = format!("cell[{ci}:{celli}].para[{cpi}]");
                                inspect_unicode_scan_unit(
                                    &mut findings,
                                    &mut scanned_chars,
                                    si,
                                    pi,
                                    &loc,
                                    &cp.text,
                                    kind_filter,
                                );
                                for nested in &cp.controls {
                                    if let Control::Equation(eq) = nested {
                                        inspect_unicode_scan_unit(
                                            &mut findings,
                                            &mut scanned_chars,
                                            si,
                                            pi,
                                            &format!("{loc}.equation"),
                                            &eq.script,
                                            kind_filter,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Control::Shape(shape) => {
                        if let Some(tb) = shape.as_ref().drawing().and_then(|d| d.text_box.as_ref())
                        {
                            for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                inspect_unicode_scan_unit(
                                    &mut findings,
                                    &mut scanned_chars,
                                    si,
                                    pi,
                                    &format!("textbox[{ci}].para[{tpi}]"),
                                    &tp.text,
                                    kind_filter,
                                );
                            }
                        }
                    }
                    Control::Equation(eq) => {
                        inspect_unicode_scan_unit(
                            &mut findings,
                            &mut scanned_chars,
                            si,
                            pi,
                            &format!("equation[{ci}]"),
                            &eq.script,
                            kind_filter,
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    let count_by = |key: &str, field: &str| {
        findings
            .iter()
            .filter(|f| f[field].as_str() == Some(key))
            .count()
    };
    let severity_counts = serde_json::json!({
        "high": count_by("high", "severity"),
        "medium": count_by("medium", "severity"),
        "low": count_by("low", "severity"),
    });
    let mut kind_counts = serde_json::Map::new();
    for k in ts::DeceptionKind::ALL {
        kind_counts.insert(
            k.label().to_string(),
            serde_json::Value::from(count_by(k.label(), "kind")),
        );
    }

    if json_mode {
        // 0건이면 findings: [] · clean: true — "검사했는데 깨끗함"과 "검사 안 함"은 다르다.
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "kindFilter": kind_label,
            "scannedChars": scanned_chars,
            "findings": findings,
            "findingCount": findings.len(),
            "clean": findings.is_empty(),
            "severityCounts": severity_counts,
            "kindCounts": serde_json::Value::Object(kind_counts),
        });
        println!("{}", provenance::marked(envelope, "inspect"));
        // 탐지 건수는 실행 실패가 아니다 — 1은 런타임 실패 전용이다(#2707).
        return EXIT_OK;
    }

    if findings.is_empty() {
        println!(
            "유니코드 기만 검사: {file_path} (축: {kind_label}, {scanned_chars}자) — 탐지 0건, 깨끗합니다"
        );
        return EXIT_OK;
    }
    println!(
        "유니코드 기만 검사: {file_path} (축: {kind_label}, {scanned_chars}자) — 탐지 {}건 (high {} · medium {} · low {})",
        findings.len(),
        severity_counts["high"],
        severity_counts["medium"],
        severity_counts["low"],
    );
    for f in &findings {
        let s = |k: &str| f[k].as_str().unwrap_or("");
        println!(
            "  [{}] {} {}  구역{}:문단{} {} +{}",
            s("severity"),
            s("kind"),
            s("codepoint"),
            f["section"],
            f["paragraph"],
            s("location"),
            f["charOffset"],
        );
        println!("      보이는 모습: {}", s("rendered"));
        println!("      실제 순서  : {}", s("raw"));
        if let Some(hidden) = f["hidden"].as_str() {
            println!("      숨은 내용  : {hidden}");
        }
        println!("      까닭       : {}", s("why"));
    }
    EXIT_OK
}
