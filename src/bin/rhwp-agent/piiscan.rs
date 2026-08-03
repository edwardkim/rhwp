//! [#3918] `pii-scan` — 공개 전 개인정보 게이트 (읽기 전용).
//!
//! 판정 코어는 `edit redact` 와 동일한 [`DocumentCore::scan_pii`] 다 — 두 표면이
//! 다른 잣대로 재면 "스캔은 깨끗한데 redact 는 지우는" 어긋남이 생기므로 코어를
//! 공유한다. 이 명령이 더하는 것은 **게이트 계약**이다:
//!
//! - 읽기 전용: 문서를 절대 바꾸지 않는다.
//! - 종료 코드: 발견 0건 = 0, 1건 이상 = 3 (배포 파이프라인이 그대로 분기).
//! - 원문 비노출 기본: 봉투에는 **마스킹 값만** 싣는다. redact 계열에서 원문
//!   PII 가 최민감했던 교훈(#3885) — 게이트 로그는 CI·이슈에 남기 마련이라
//!   기본값이 안전해야 한다. 원문은 `--show-values` 옵트인.

use crate::envelope::{
    envelope, load_core, print_json, read_file, EXIT_GATE, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use rhwp::document_core::queries::pii_scan::PiiKind;
use serde_json::{json, Value};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp-agent pii-scan <파일> [--json] [--kind ssn,card,phone,email|all] [--show-values] [--limit <N>]";

    let mut json_mode = false;
    let mut show_values = false;
    let mut limit = 100usize;
    let mut kinds: Vec<PiiKind> = PiiKind::all().to_vec();
    let mut file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--show-values" => {
                show_values = true;
                i += 1;
            }
            "--limit" => {
                match args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => limit = n,
                    _ => {
                        eprintln!("오류: --limit 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
                i += 2;
            }
            "--kind" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!(
                        "오류: --kind 뒤에 종류 목록이 필요합니다 (ssn,card,phone,email|all)."
                    );
                    return EXIT_USAGE;
                };
                if value == "all" {
                    kinds = PiiKind::all().to_vec();
                } else {
                    let mut parsed = Vec::new();
                    for token in value.split(',') {
                        match PiiKind::parse(token) {
                            Some(kind) => parsed.push(kind),
                            None => {
                                eprintln!("오류: 알 수 없는 --kind 값입니다 - {token} (ssn|card|phone|email|all)");
                                return EXIT_USAGE;
                            }
                        }
                    }
                    if parsed.is_empty() {
                        eprintln!("오류: --kind 목록이 비어 있습니다.");
                        return EXIT_USAGE;
                    }
                    kinds = parsed;
                }
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
            positional => {
                if file.is_some() {
                    eprintln!("오류: 파일은 하나만 지정할 수 있습니다 - {positional}");
                    return EXIT_USAGE;
                }
                file = Some(positional.to_string());
                i += 1;
            }
        }
    }

    let Some(path) = file else {
        eprintln!("오류: 대상 파일을 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };

    let data = match read_file(&path) {
        Ok(d) => d,
        Err(message) => {
            eprintln!("오류: {message}");
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

    // `edit redact` 의 기본 마스킹 문자와 같은 '*' — 마스킹 결과는 원문과 문자 수가 같다.
    let findings = core.scan_pii(&kinds, '*');
    let total = findings.len();
    let truncated = total > limit;

    let mut counts: std::collections::BTreeMap<&str, u64> = Default::default();
    for finding in &findings {
        *counts.entry(finding.kind).or_insert(0) += 1;
    }

    if json_mode {
        let items: Vec<Value> = findings
            .iter()
            .take(limit)
            .map(|f| {
                let mut item = json!({
                    "kind": f.kind,
                    "masked": f.masked,
                    "section": f.section,
                    "paragraph": f.paragraph,
                    "page": f.page,
                    "charOffset": f.char_offset,
                });
                if show_values {
                    item["raw"] = json!(f.raw);
                }
                item
            })
            .collect();
        let payload = json!({
            "source": path,
            "kinds": kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
            "total": total,
            "counts": counts,
            "truncated": truncated,
            "showValues": show_values,
            "findings": items,
        });
        let untrusted: &[&str] = if total == 0 {
            &[]
        } else if show_values {
            &["findings[].masked", "findings[].raw"]
        } else {
            &["findings[].masked"]
        };
        print_json(&envelope("pii-scan", payload, untrusted));
    } else {
        crate::outln!("rhwp-agent pii-scan — {path}");
        if show_values {
            eprintln!("주의: --show-values — 원문 개인정보가 출력됩니다. 로그에 남기지 마세요.");
        }
        for f in findings.iter().take(limit) {
            let value = if show_values { &f.raw } else { &f.masked };
            let page = f
                .page
                .map(|p| format!("{p}쪽"))
                .unwrap_or_else(|| "미배치".to_string());
            crate::outln!(
                "  {} {} (구역 {}, 문단 {}, {page}, 오프셋 {})",
                f.kind,
                value,
                f.section,
                f.paragraph,
                f.char_offset
            );
        }
        if truncated {
            crate::outln!("({total}건 중 {limit}건만 표시 — --limit 로 조절)");
        }
        crate::outln!(
            "결과: {}",
            if total == 0 {
                "발견 없음".to_string()
            } else {
                format!("{total}건 발견")
            }
        );
    }

    if total == 0 {
        EXIT_OK
    } else {
        EXIT_GATE
    }
}
