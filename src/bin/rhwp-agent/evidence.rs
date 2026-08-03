//! [#3918] `evidence` — 전/후 증빙 번들.
//!
//! PR 증빙·작업 보고가 요구하는 "전/후 무엇이 얼마나 바뀌었나"를 한 번에 만든다:
//! 두 문서의 지문(`fingerprint` 와 같은 계산)을 비교해 변경 필드를 뽑고, 텍스트
//! diff(`diff-text` 와 같은 엔진) 요약과 표본 헝크를 붙인다. 사람용 마크다운이
//! 기본이고 `--json` 이 기계용이다. 게이트가 아니라 **보고서**이므로 두 문서가
//! 달라도 exit 0 이다(같은지 판정만 원하면 `diff-text` 를 쓴다).

use crate::difftext::{diff_lines, lines_of};
use crate::envelope::{envelope, print_json, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};
use crate::fingerprint::{compute, payload, SEMANTIC_KEYS};
use serde_json::{json, Value};

/// 봉투에 싣는 표본 헝크 수 — 전체 diff 는 `diff-text` 가 담당한다.
const SAMPLE_HUNKS: usize = 3;

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp-agent evidence <전.hwp> <후.hwp> [--json|--md] [-o <파일>]";

    let mut json_mode = false;
    let mut md_mode = false;
    let mut out: Option<String> = None;
    let mut files: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--md" => {
                md_mode = true;
                i += 1;
            }
            "-o" | "--output" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: -o 뒤에 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                };
                out = Some(value.clone());
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
            positional => {
                files.push(positional.to_string());
                i += 1;
            }
        }
    }

    if files.len() != 2 {
        eprintln!("오류: 파일 두 개(전·후)를 지정해주세요.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }
    if json_mode && md_mode {
        eprintln!("오류: --json 과 --md 는 동시에 쓸 수 없습니다.");
        return EXIT_USAGE;
    }

    let before = match compute(&files[0]) {
        Ok(fp) => fp,
        Err((code, message)) => {
            eprintln!("오류: {message}");
            return code;
        }
    };
    let after = match compute(&files[1]) {
        Ok(fp) => fp,
        Err((code, message)) => {
            eprintln!("오류: {message}");
            return code;
        }
    };

    // 변경 필드: fingerprint --check 와 같은 의미 지문 잣대.
    let before_payload = payload(&before, false);
    let after_payload = payload(&after, false);
    let mut changed: Vec<Value> = Vec::new();
    for key in SEMANTIC_KEYS {
        let b = before_payload.get(*key).cloned().unwrap_or(Value::Null);
        let a = after_payload.get(*key).cloned().unwrap_or(Value::Null);
        if b != a {
            changed.push(json!({ "field": key, "before": b, "after": a }));
        }
    }

    // 텍스트 diff — diff-text 와 같은 엔진, 문맥 1줄.
    let lines_a = lines_of(&before.pages);
    let lines_b = lines_of(&after.pages);
    let diff = diff_lines(&lines_a, &lines_b, 1);
    let identical = diff.identical() && changed.is_empty();

    if json_mode {
        let sample: Vec<Value> = diff
            .hunks
            .iter()
            .take(SAMPLE_HUNKS)
            .map(|h| {
                json!({
                    "aStart": h.a_start,
                    "bStart": h.b_start,
                    "lines": h
                        .lines
                        .iter()
                        .map(|(op, text)| json!({"op": op.to_string(), "text": text}))
                        .collect::<Vec<_>>(),
                })
            })
            .collect();
        let body = json!({
            "before": before_payload,
            "after": after_payload,
            "identical": identical,
            "changed": changed,
            "textDiff": {
                "added": diff.added,
                "removed": diff.removed,
                "coarse": diff.coarse,
                "hunkCount": diff.hunks.len(),
                "sampleHunks": sample,
            },
        });
        let mut untrusted: Vec<&str> = vec!["before.fieldNames[]", "after.fieldNames[]"];
        if !changed.is_empty() {
            untrusted.push("changed[].before");
            untrusted.push("changed[].after");
        }
        if !diff.hunks.is_empty() {
            untrusted.push("textDiff.sampleHunks[].lines[].text");
        }
        let envelope_value = envelope("evidence", body, &untrusted);
        if let Some(dest) = &out {
            let text = match serde_json::to_string_pretty(&envelope_value) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("오류: JSON 직렬화 실패 - {e}");
                    return EXIT_RUNTIME;
                }
            };
            if let Err(e) = std::fs::write(dest, text) {
                eprintln!("오류: 파일을 쓸 수 없습니다 - {dest}: {e}");
                return EXIT_RUNTIME;
            }
            eprintln!("증빙 저장: {dest}");
        } else {
            print_json(&envelope_value);
        }
        return EXIT_OK;
    }

    // ── 마크다운 (기본) ────────────────────────────────────────────────────
    let mut md = String::new();
    md.push_str("## 전/후 증빙\n\n");
    md.push_str(&format!(
        "- 전: `{}`\n- 후: `{}`\n\n",
        before.source, after.source
    ));
    md.push_str("| 항목 | 전 | 후 |\n|---|---|---|\n");
    let rows: &[(&str, String, String)] = &[
        ("포맷", before.format.to_string(), after.format.to_string()),
        (
            "쪽수",
            before.page_count.to_string(),
            after.page_count.to_string(),
        ),
        (
            "문자 수",
            before.char_count.to_string(),
            after.char_count.to_string(),
        ),
        (
            "문단 수",
            before.para_count.to_string(),
            after.para_count.to_string(),
        ),
        (
            "표 개수",
            before.table_count.to_string(),
            after.table_count.to_string(),
        ),
        (
            "필드 개수",
            before.field_count.to_string(),
            after.field_count.to_string(),
        ),
        (
            "텍스트 해시",
            short(&before.text_hash),
            short(&after.text_hash),
        ),
    ];
    for (label, b, a) in rows {
        let mark = if b == a { "" } else { " ◀" };
        md.push_str(&format!("| {label} | {b} | {a}{mark} |\n"));
    }
    md.push('\n');
    if identical {
        md.push_str("두 문서의 의미 지문과 텍스트가 **같습니다**.\n");
    } else {
        md.push_str(&format!(
            "텍스트 diff 요약: **+{} -{}** (헝크 {}개{})\n\n",
            diff.added,
            diff.removed,
            diff.hunks.len(),
            if diff.coarse {
                ", 규모 예산 초과 — 개괄 diff"
            } else {
                ""
            }
        ));
        for hunk in diff.hunks.iter().take(SAMPLE_HUNKS) {
            md.push_str(&format!(
                "```diff\n@@ -{} +{} @@\n",
                hunk.a_start, hunk.b_start
            ));
            for (op, text) in &hunk.lines {
                md.push_str(&format!("{op}{text}\n"));
            }
            md.push_str("```\n");
        }
        if diff.hunks.len() > SAMPLE_HUNKS {
            md.push_str(&format!(
                "\n(표본 {SAMPLE_HUNKS}개만 표시 — 전체는 `rhwp-agent diff-text` 참고, 총 {}개)\n",
                diff.hunks.len()
            ));
        }
    }

    if let Some(dest) = &out {
        if let Err(e) = std::fs::write(dest, &md) {
            eprintln!("오류: 파일을 쓸 수 없습니다 - {dest}: {e}");
            return EXIT_RUNTIME;
        }
        eprintln!("증빙 저장: {dest}");
    } else {
        crate::outp!("{md}");
    }
    EXIT_OK
}

fn short(hash: &str) -> String {
    hash.chars().take(12).collect()
}
