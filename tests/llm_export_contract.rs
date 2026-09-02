//! `export-llm` 계약 — HWP/HWPX → LLM-ready RAG 청크.
//!
//! 실제 바이너리를 실제 샘플에 돌려 계약을 고정한다:
//! - 기본 산출은 NDJSON(한 줄당 청크 하나), `--format json` 은 단일 봉투.
//! - 청크마다 출처 앵커(headingPath/section/paragraph)와 **untrusted 표지**가 실린다.
//! - 표는 청크 안에서 Markdown 으로 선형화되어 자기완결이다(머리 행 보존·병합 주석).
//! - 같은 입력·옵션이면 바이트까지 같다(결정론).
//! - 청크 텍스트의 합이 문서 본문을 사실상 덮는다(무손실, export-text 대조).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

/// 본문만 있는 논문 샘플(제목 미검출 → 전량 서문). 예산 쪼개기·결정론·커버리지에 쓴다.
const PAPER: &str = "samples/hwp3-sample.hwp";
/// 중첩 제목 계층과 표가 풍부한 정부 편람. 제목 경로·표 선형화에 쓴다.
const MANUAL: &str = "samples/2025 행정업무운영 편람(최종).hwpx";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], out: &Output) -> String {
    format!(
        "명령: rhwp {}\n종료: {:?}\nstderr:\n{}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
    )
}

fn stdout_string(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("stdout UTF-8")
}

/// 문자열에서 영숫자만 이어붙인다 — 구두점·공백 표면 차이를 지운 내용 비교용.
fn alnum(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn has_hangul(s: &str) -> bool {
    s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

// ── NDJSON 기본 산출 ────────────────────────────────────────────────────────

#[test]
fn ndjson_is_default_and_every_line_is_a_marked_chunk() {
    let path = sample(PAPER);
    let path = path.to_str().unwrap();
    let args = ["export-llm", path];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));

    let body = stdout_string(&out);
    let lines: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(!lines.is_empty(), "청크가 최소 하나는 나와야 한다");

    for (i, line) in lines.iter().enumerate() {
        let v: Value = serde_json::from_str(line).expect("각 줄은 순수 JSON 객체");
        assert_eq!(v["schemaVersion"], "1.0");
        assert!(v["source"].as_str().unwrap().ends_with("hwp3-sample.hwp"));
        assert_eq!(v["chunkIndex"], i as i64, "chunkIndex 는 0부터 순차적");
        // 출처 표지 — 청크는 프롬프트 주입면이므로 항상 문서 파생으로 표지된다.
        assert_eq!(v["untrustedContent"], true, "{line}");
        let fields = v["untrustedFields"]
            .as_array()
            .expect("untrustedFields 배열");
        assert!(
            fields.iter().any(|f| f == "text"),
            "text 표지가 있어야 한다: {line}"
        );
        assert!(v["text"].as_str().is_some_and(|t| !t.is_empty()));
    }
}

#[test]
fn json_format_yields_a_single_envelope() {
    let path = sample(PAPER);
    let path = path.to_str().unwrap();
    let args = ["export-llm", path, "--format", "json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));

    let v: Value = serde_json::from_slice(&out.stdout).expect("단일 JSON 봉투");
    assert_eq!(v["schemaVersion"], "1.0");
    assert_eq!(v["maxTokens"], 512);
    assert_eq!(v["mode"], "auto");
    assert_eq!(v["tokenEstimator"], "cjk1-latin4-v1");
    let chunks = v["chunks"].as_array().expect("chunks 배열");
    assert_eq!(v["chunkCount"], chunks.len() as i64);
    assert!(!chunks.is_empty());
    assert_eq!(v["untrustedContent"], true);
    let fields = v["untrustedFields"].as_array().expect("untrustedFields");
    assert!(
        fields.iter().any(|f| f == "chunks[].text"),
        "봉투 표지에 chunks[].text 가 있어야 한다: {v}"
    );
}

// ── 결정론 ──────────────────────────────────────────────────────────────────

#[test]
fn output_is_byte_for_byte_deterministic() {
    let path = sample(PAPER);
    let path = path.to_str().unwrap();
    for format in [
        vec!["export-llm", path],
        vec!["export-llm", path, "--format", "json"],
    ] {
        let a = run(&format);
        let b = run(&format);
        assert_eq!(
            a.stdout, b.stdout,
            "같은 입력·옵션은 바이트까지 같아야 한다"
        );
    }
}

// ── 토큰 예산 ────────────────────────────────────────────────────────────────

#[test]
fn smaller_budget_produces_more_chunks() {
    let path = sample(PAPER);
    let path = path.to_str().unwrap();
    let count = |budget: &str| -> usize {
        let args = [
            "export-llm",
            path,
            "--format",
            "json",
            "--max-tokens",
            budget,
        ];
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        v["chunks"].as_array().unwrap().len()
    };
    // 예산이 작을수록 청크가 늘어난다 — 예산이 실제로 쪼갠다는 신호.
    assert!(
        count("100") > count("2000"),
        "작은 예산이 더 많은 청크를 내야 한다"
    );
}

#[test]
fn multi_unit_text_chunks_respect_the_budget() {
    // 여러 문단이 묶인(text 에 빈 줄 경계가 있는) text 청크는 예산을 넘지 않는다.
    // 단일 초대형 문단은 문단 경계를 지키느라 예산을 넘을 수 있다(정직한 예외).
    let path = sample(PAPER);
    let path = path.to_str().unwrap();
    let budget = 200i64;
    let args = [
        "export-llm",
        path,
        "--format",
        "json",
        "--max-tokens",
        "200",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    for c in v["chunks"].as_array().unwrap() {
        if c["kind"] == "text" && c["text"].as_str().unwrap().contains("\n\n") {
            assert!(
                c["tokenEstimate"].as_i64().unwrap() <= budget,
                "묶인 text 청크가 예산 초과: {c}"
            );
        }
    }
}

// ── 제목 경로 · 자기완결 표 ──────────────────────────────────────────────────

#[test]
fn nested_heading_paths_and_anchors_are_present() {
    let path = sample(MANUAL);
    let path = path.to_str().unwrap();
    let args = ["export-llm", path, "--format", "json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let chunks = v["chunks"].as_array().unwrap();

    // 중첩 제목 경로(예: ["제2장 …", "제1절 …"])가 실제로 나온다.
    let nested = chunks
        .iter()
        .any(|c| c["headingPath"].as_array().map(|p| p.len()).unwrap_or(0) >= 2);
    assert!(nested, "중첩 제목 경로가 있어야 한다");

    // 제목 경로가 있는 청크는 headingPath 를 문서 파생으로 표지하고 주소를 싣는다.
    for c in chunks {
        let hp = c["headingPath"].as_array().unwrap();
        if !hp.is_empty() {
            assert!(
                c["section"].is_number(),
                "제목 청크는 section 앵커를 실어야 한다"
            );
            assert!(c["paragraph"].is_number());
        }
    }
}

#[test]
fn tables_are_linearized_and_self_contained() {
    let path = sample(MANUAL);
    let path = path.to_str().unwrap();
    let args = ["export-llm", path, "--format", "json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let chunks = v["chunks"].as_array().unwrap();

    // 표를 품은 청크가 있고, 그 표는 청크 텍스트 안에서 Markdown 으로 선형화된다.
    let table_chunk = chunks
        .iter()
        .find(|c| {
            c["tables"].as_array().is_some_and(|t| !t.is_empty())
                && c["text"].as_str().unwrap().contains("| --- |")
        })
        .expect("Markdown 표를 품은 청크가 있어야 한다");
    let table_meta = &table_chunk["tables"][0];
    assert!(table_meta["rows"].is_number());
    assert!(table_meta["cols"].is_number());
    assert!(table_meta["index"].is_number());

    // 병합 셀은 청크 텍스트에 주석된다(문서 어딘가에 병합 표가 있다).
    let any_merge = chunks
        .iter()
        .any(|c| c["text"].as_str().unwrap().contains("[병합"));
    assert!(any_merge, "병합 셀 주석이 최소 한 번은 나와야 한다");
}

#[test]
fn split_tables_repeat_their_header() {
    // 작은 예산으로 큰 표를 강제로 쪼갠 뒤, 모든 파트가 머리 행을 되풀이하는지 본다.
    let path = sample(MANUAL);
    let path = path.to_str().unwrap();
    let args = ["export-llm", path, "--format", "json", "--max-tokens", "80"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let chunks = v["chunks"].as_array().unwrap();

    let mut saw_split = false;
    for c in chunks {
        for t in c["tables"].as_array().into_iter().flatten() {
            if t["partCount"].as_i64().unwrap_or(1) > 1 {
                saw_split = true;
                assert_eq!(
                    t["headerRepeated"], true,
                    "쪼개진 표 파트는 머리 행을 되풀이한다: {c}"
                );
            }
        }
    }
    assert!(saw_split, "예산 80 이면 큰 표가 쪼개져야 한다");
}

// ── 무손실(라운드트립) ──────────────────────────────────────────────────────

/// export-text 본문 토큰이 청크(headingPath + text)에 얼마나 담기는지.
fn coverage(path: &str) -> f64 {
    let text_out = run(&["export-text", "--json", path]);
    let tx: Value = serde_json::from_slice(&text_out.stdout).unwrap();
    let mut needles: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for page in tx["pages"].as_array().unwrap() {
        for raw in page["text"].as_str().unwrap_or("").split_whitespace() {
            let w = alnum(raw);
            let len = w.chars().count();
            if (len >= 2 && has_hangul(&w)) || len >= 4 {
                needles.insert(w);
            }
        }
    }
    let llm_out = run(&["export-llm", "--format", "json", path]);
    let llm: Value = serde_json::from_slice(&llm_out.stdout).unwrap();
    let mut hay = String::new();
    for c in llm["chunks"].as_array().unwrap() {
        for h in c["headingPath"].as_array().unwrap() {
            hay.push_str(&alnum(h.as_str().unwrap()));
        }
        hay.push_str(&alnum(c["text"].as_str().unwrap()));
    }
    let total = needles.len();
    if total == 0 {
        return 1.0;
    }
    let present = needles.iter().filter(|n| hay.contains(n.as_str())).count();
    present as f64 / total as f64
}

#[test]
fn chunks_cover_the_document_body() {
    // 본문만 있는 논문: 사실상 전량 커버.
    let paper = sample(PAPER);
    let paper_cov = coverage(paper.to_str().unwrap());
    assert!(paper_cov >= 0.97, "PAPER 커버리지 {paper_cov:.4} < 0.97");

    // 제목·표가 풍부한 편람: page-표면(머리말/꼬리말·쪽번호 반복) 차이로 100% 는 아니나
    // 본문 손실은 없다 — 보수적 하한을 건다.
    let manual = sample(MANUAL);
    let manual_cov = coverage(manual.to_str().unwrap());
    assert!(manual_cov >= 0.92, "MANUAL 커버리지 {manual_cov:.4} < 0.92");
}

// ── 사용법 · 런타임 오류 ────────────────────────────────────────────────────

#[test]
fn usage_and_runtime_errors_use_the_right_exit_codes() {
    // 인자 없음 → 사용법 오류(2).
    assert_eq!(run(&["export-llm"]).status.code(), Some(2));
    // 잘못된 --format → 2.
    let p = sample(PAPER);
    let p = p.to_str().unwrap();
    assert_eq!(
        run(&["export-llm", p, "--format", "xml"]).status.code(),
        Some(2)
    );
    // --max-tokens 0 → 2.
    assert_eq!(
        run(&["export-llm", p, "--max-tokens", "0"]).status.code(),
        Some(2)
    );
    // 잘못된 --mode → 2.
    assert_eq!(
        run(&["export-llm", p, "--mode", "bogus"]).status.code(),
        Some(2)
    );
    // 없는 파일 → 런타임 실패(1).
    assert_eq!(
        run(&["export-llm", "does-not-exist.hwp"]).status.code(),
        Some(1)
    );
}

#[test]
fn mode_option_is_accepted() {
    let p = sample(PAPER);
    let p = p.to_str().unwrap();
    for mode in ["auto", "outline", "clause"] {
        let args = ["export-llm", p, "--format", "json", "--mode", mode];
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    }
}
