//! [#3787 S4] `rhwp inspect unicode` 계약 테스트 — 유니코드 기만 탐지.
//!
//! ## 무엇을 고정하는가
//!
//! 문서 텍스트는 그대로 LLM 에게 간다. **화면에 보이는 것과 실제 바이트가 다르면**
//! 사람은 안전하다고 판단하는데 에이전트는 다른 걸 읽는다. 이 테스트는 그 어긋남을
//! 실제 HWP/HWPX 파일에 심어 놓고, CLI 가 그것을 근거와 함께 보고하는지 본다.
//!
//! ## 왜 합성 문서를 런타임에 만드는가
//!
//! 기만 문자를 담은 바이너리 픽스처를 저장소에 커밋하면, 그 파일 자체가 코드 리뷰·grep·
//! 에디터를 속이는 물건이 되어 저장소를 돌아다닌다. 그래서 픽스처는 **정상 샘플 +
//! `rhwp edit replace-text`** 로 매 실행 시 만든다 — 페이로드가 소스에 평문 이스케이프로
//! 남아 검토 가능하고, HWP5·HWPX 두 기록 경로가 문자를 보존하는지도 함께 검증된다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 파싱·편집·재기록이 모두 되는 정상 한국어 문서.
const SAMPLE: &str = "samples/2026_oss_rst.hwp";
/// 페이로드를 심을 자리 — 이 샘플 본문에 정확히 한 번 나온다.
const ANCHOR: &str = "제출 방법";

/// 네 축을 한 문단에 모두 심는 페이로드.
///
/// - `\u{200B}`×3 — 제로폭 연속(은닉 데이터 형태) → high
/// - `\u{202E}` … `\u{202C}` — 방향 오버라이드. 화면엔 `exe.doc`, 실제론 `cod.exe`
/// - `\u{0422}otal` — 키릴 Т 로 위장한 라틴 낱말 `Total`
/// - `\u{E0049}\u{E0067}…` — 태그 문자로 실어 나른 숨은 지시 `Ignore`
const PAYLOAD: &str = "제출\u{200B}\u{200B}\u{200B}방법 \u{202E}cod.exe\u{202C} \u{0422}otal\u{E0049}\u{E0067}\u{E006E}\u{E006F}\u{E0072}\u{E0065}";

fn rhwp_bin() -> String {
    env!("CARGO_BIN_EXE_rhwp").to_string()
}

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\n종료 코드: {:?}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

/// 기만 페이로드를 심은 문서를 만든다. `ext` 로 HWP5·HWPX 두 기록 경로를 모두 시험한다.
fn attack_document(tag: &str, ext: &str) -> PathBuf {
    let src = manifest(SAMPLE);
    let out =
        std::env::temp_dir().join(format!("rhwp-unicode-{}-{}.{ext}", std::process::id(), tag));
    let _ = std::fs::remove_file(&out);
    let args = [
        "edit",
        "replace-text",
        src.to_str().expect("샘플 경로"),
        "--find",
        ANCHOR,
        "--replace",
        PAYLOAD,
        "-o",
        out.to_str().expect("출력 경로"),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "픽스처 생성 실패\n{}",
        describe(&args, &output)
    );
    assert!(out.exists(), "픽스처가 만들어지지 않았습니다: {out:?}");
    out
}

fn inspect(path: &Path, extra: &[&str]) -> serde_json::Value {
    let p = path.to_str().expect("경로");
    let mut args: Vec<&str> = vec!["inspect", "unicode", p, "--json"];
    args.extend_from_slice(extra);
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    parse_stdout_json(&args, &output)
}

fn kinds_of(v: &serde_json::Value) -> Vec<String> {
    v["findings"]
        .as_array()
        .expect("findings 배열")
        .iter()
        .filter_map(|f| f["kind"].as_str().map(String::from))
        .collect()
}

// ── 봉투 계약 ──────────────────────────────────────────────────────────────

#[test]
fn clean_document_reports_empty_findings_not_a_missing_key() {
    // "검사했는데 깨끗함"과 "검사하지 않음"은 소비자가 반드시 구별할 수 있어야 한다.
    // 0건일 때 findings 키를 빼 버리면 그 구별이 사라진다.
    let src = manifest(SAMPLE);
    let v = inspect(&src, &[]);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    assert_eq!(v["kindFilter"], "all", "{v}");
    assert_eq!(v["clean"], true, "정상 샘플은 clean 이어야 한다: {v}");
    assert_eq!(v["findingCount"], 0, "{v}");
    assert_eq!(
        v["findings"].as_array().map(Vec::len),
        Some(0),
        "0건이면 findings 는 빈 배열이어야 한다: {v}"
    );
    assert!(
        v["scannedChars"].as_u64().unwrap_or(0) > 0,
        "0자를 훑고 clean 이라고 하면 공허한 통과다: {v}"
    );
    // 축별 집계는 0건이어도 전 축이 실려야 한다 — 소비자가 축 존재를 알 수 있다.
    for k in ["zero_width", "bidi_override", "tag_char", "confusable"] {
        assert_eq!(v["kindCounts"][k], 0, "{k} 집계 누락: {v}");
    }
}

#[test]
fn every_kind_is_detected_in_a_real_document() {
    for ext in ["hwp", "hwpx"] {
        let doc = attack_document("all", ext);
        let v = inspect(&doc, &[]);
        assert_eq!(v["clean"], false, "[{ext}] {v}");
        let kinds = kinds_of(&v);
        for expected in ["zero_width", "bidi_override", "tag_char", "confusable"] {
            assert!(
                kinds.iter().any(|k| k == expected),
                "[{ext}] {expected} 축이 실제 문서에서 탐지되지 않았습니다: {v}"
            );
        }
        assert_eq!(
            v["findingCount"].as_u64().map(|n| n as usize),
            Some(kinds.len()),
            "[{ext}] findingCount 가 실제 건수와 다릅니다: {v}"
        );
        let _ = std::fs::remove_file(&doc);
    }
}

#[test]
fn bidi_finding_shows_rendered_and_raw_disagreeing() {
    // 이 축의 전부는 "화면과 바이트가 어긋난다"이다. 차이를 못 보이면 보고가 공허하다.
    let doc = attack_document("bidi", "hwp");
    let v = inspect(&doc, &["--kind", "bidi"]);
    let findings = v["findings"].as_array().expect("findings");
    let rlo = findings
        .iter()
        .find(|f| f["codepoint"] == "U+202E")
        .unwrap_or_else(|| panic!("RLO 탐지 누락: {v}"));

    assert_eq!(rlo["kind"], "bidi_override", "{rlo}");
    assert_eq!(rlo["severity"], "high", "오버라이드는 높게: {rlo}");
    let rendered = rlo["rendered"].as_str().expect("rendered");
    let raw = rlo["raw"].as_str().expect("raw");
    assert_ne!(
        rendered, raw,
        "rendered 와 raw 가 같으면 보고가 공허하다: {rlo}"
    );
    assert!(
        rendered.contains("exe.doc"),
        "화면에 보이는 모습은 exe.doc 이어야 한다: {rlo}"
    );
    assert!(
        raw.contains("cod.exe"),
        "실제 순서는 cod.exe 여야 한다: {rlo}"
    );
    // 보고 채널이 다시 속지 않도록, 제어문자는 원문 그대로가 아니라 표기로 실린다.
    assert!(raw.contains("<U+202E>"), "{rlo}");
    assert!(
        !raw.contains('\u{202E}') && !rendered.contains('\u{202E}'),
        "봉투에 원문 방향 제어문자가 그대로 실렸습니다: {rlo}"
    );
    let _ = std::fs::remove_file(&doc);
}

#[test]
fn tag_chars_report_the_decoded_hidden_instruction() {
    let doc = attack_document("tag", "hwp");
    let v = inspect(&doc, &["--kind", "tag"]);
    let findings = v["findings"].as_array().expect("findings");
    assert_eq!(findings.len(), 1, "태그 열은 1건으로 묶는다: {v}");
    let f = &findings[0];
    assert_eq!(f["kind"], "tag_char", "{f}");
    assert_eq!(f["severity"], "high", "{f}");
    assert_eq!(f["runLength"], 6, "{f}");
    assert_eq!(
        f["hidden"], "Ignore",
        "숨은 지시를 복원해 보여야 사람이 판단할 수 있다: {f}"
    );
    assert!(
        !f["rendered"].as_str().expect("rendered").contains("Ignore"),
        "태그 문자는 화면에 렌더되지 않는다: {f}"
    );
    let _ = std::fs::remove_file(&doc);
}

#[test]
fn confusable_finding_folds_to_the_latin_lookalike() {
    let doc = attack_document("conf", "hwp");
    let v = inspect(&doc, &["--kind", "confusable"]);
    let findings = v["findings"].as_array().expect("findings");
    let f = findings
        .iter()
        .find(|f| f["codepoint"] == "U+0422")
        .unwrap_or_else(|| panic!("키릴 Т 탐지 누락: {v}"));
    assert_eq!(f["kind"], "confusable", "{f}");
    assert_eq!(
        f["rendered"], "Total",
        "라틴으로 접었을 때의 모습을 보여야 한다: {f}"
    );
    assert!(
        f["raw"].as_str().expect("raw").contains("<U+0422>"),
        "실제 글자의 정체를 지목해야 한다: {f}"
    );
    let _ = std::fs::remove_file(&doc);
}

#[test]
fn zero_width_run_is_one_finding_graded_high() {
    let doc = attack_document("zw", "hwp");
    let v = inspect(&doc, &["--kind", "zero-width"]);
    let findings = v["findings"].as_array().expect("findings");
    assert_eq!(findings.len(), 1, "연속 열은 1건으로 묶는다: {v}");
    let f = &findings[0];
    assert_eq!(f["codepoint"], "U+200B", "{f}");
    assert_eq!(f["runLength"], 3, "{f}");
    assert_eq!(f["severity"], "high", "다량 연속은 높게: {f}");
    assert_eq!(v["severityCounts"]["high"], 1, "{v}");
    let _ = std::fs::remove_file(&doc);
}

// ── --kind 필터 ────────────────────────────────────────────────────────────

#[test]
fn kind_filter_partitions_the_findings_exactly() {
    // 필터가 "걸러내는 척"만 하고 실제로는 전부 돌려주면(또는 엉뚱하게 빠뜨리면)
    // 에이전트는 축을 좁혔다고 믿은 채 잘못된 결론을 낸다.
    let doc = attack_document("filter", "hwp");
    let all = inspect(&doc, &[]);
    let all_kinds = kinds_of(&all);
    assert!(all_kinds.len() >= 4, "{all}");

    let mut partitioned = 0usize;
    for (flag, label) in [
        ("zero-width", "zero_width"),
        ("bidi", "bidi_override"),
        ("tag", "tag_char"),
        ("confusable", "confusable"),
    ] {
        let v = inspect(&doc, &["--kind", flag]);
        assert_eq!(v["kindFilter"], flag, "{v}");
        let ks = kinds_of(&v);
        assert!(!ks.is_empty(), "--kind {flag} 가 0건: {v}");
        assert!(
            ks.iter().all(|k| k == label),
            "--kind {flag} 에 다른 축이 새어 나왔습니다: {v}"
        );
        assert_eq!(
            ks.len(),
            all_kinds.iter().filter(|k| *k == label).count(),
            "--kind {flag} 의 건수가 전체 스캔의 부분집합과 다릅니다: {v}"
        );
        partitioned += ks.len();
    }
    assert_eq!(
        partitioned,
        all_kinds.len(),
        "축별 합이 전체와 다릅니다 — 어느 축에도 안 잡히는 탐지가 있습니다: {all}"
    );

    // 명시적 all 은 필터 없음과 같아야 한다.
    let explicit = inspect(&doc, &["--kind", "all"]);
    assert_eq!(explicit["findingCount"], all["findingCount"], "{explicit}");
    let _ = std::fs::remove_file(&doc);
}

#[test]
fn declared_kind_enum_is_accepted_by_the_cli() {
    // 드리프트 가드: MCP `inputSchema` 가 광고하는 enum 값은 CLI 가 실제로 받아야 한다.
    // 선언과 파서가 다른 목록을 들면, 스키마를 읽고 값을 고른 에이전트가 usage 오류를 맞는다.
    let mcp = parse_stdout_json(&["capabilities", "--mcp"], &run(&["capabilities", "--mcp"]));
    let tool = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_inspect_unicode")
        .unwrap_or_else(|| panic!("hwp_inspect_unicode 도구 누락: {mcp}"));
    let declared: Vec<&str> = tool["inputSchema"]["properties"]["kind"]["enum"]
        .as_array()
        .unwrap_or_else(|| panic!("kind enum 누락: {tool}"))
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(declared.contains(&"all"), "{tool}");
    assert!(declared.len() >= 5, "축 4종 + all: {tool}");

    let src = manifest(SAMPLE);
    for value in declared {
        let p = src.to_str().expect("경로");
        let args = ["inspect", "unicode", p, "--json", "--kind", value];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "선언된 enum 값 {value} 를 CLI 가 거부했습니다\n{}",
            describe(&args, &output)
        );
    }
}

#[test]
fn mcp_tool_declares_required_and_wires_every_property() {
    // 드리프트 가드: `required` 배열이 없으면 자동 등록 클라이언트가 스키마를 못 읽는다.
    // 선언한 속성이 argv 에 닿지 않으면 서버는 그 인자를 조용히 버리고 성공을 보고한다.
    let mcp = parse_stdout_json(&["capabilities", "--mcp"], &run(&["capabilities", "--mcp"]));
    let tool = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_inspect_unicode")
        .unwrap_or_else(|| panic!("hwp_inspect_unicode 누락: {mcp}"));

    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    let required = tool["inputSchema"]["required"]
        .as_array()
        .unwrap_or_else(|| panic!("required 배열 누락: {tool}"));
    assert!(required.iter().any(|r| r == "path"), "{tool}");
    assert_eq!(tool["cli"]["command"], "inspect", "{tool}");

    let mut wired: Vec<String> = tool["cli"]["args"]
        .as_array()
        .expect("cli.args")
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| s.starts_with('{') && s.ends_with('}'))
        .map(|s| s[1..s.len() - 1].to_string())
        .collect();
    if let Some(optional) = tool["cli"]["optionalArgs"].as_array() {
        for o in optional {
            if let Some(k) = o["when"].as_str() {
                wired.push(k.to_string());
            }
        }
    }
    for key in tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties")
        .keys()
    {
        assert!(
            wired.iter().any(|w| w == key),
            "{key} 가 선언만 되고 CLI 에 배선되지 않았습니다: {tool}"
        );
    }
}

#[test]
fn capabilities_and_help_both_carry_inspect() {
    // 드리프트 가드: 기계 자기서술(capabilities)과 사람용 help 가 같은 명령을 알아야 한다.
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let entry = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "inspect")
        .unwrap_or_else(|| panic!("capabilities 에 inspect 없음: {cap}"));
    assert_eq!(entry["json"], true, "{entry}");
    let flags: Vec<&str> = entry["flags"]
        .as_array()
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(
        flags.contains(&"--json") && flags.contains(&"--kind"),
        "{entry}"
    );

    let help = run(&["--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(
        help_text
            .lines()
            .any(|l| l.starts_with("  inspect unicode ")),
        "--help 에 inspect 줄이 없습니다"
    );
    // 선언한 플래그는 help 에도 실제로 보여야 한다.
    for flag in flags {
        assert!(help_text.contains(flag), "help 에 {flag} 안내 없음");
    }
}

// ── 실패 경로: stdout 은 반드시 0바이트 ────────────────────────────────────

#[test]
fn failures_keep_stdout_empty() {
    // 에이전트는 stdout 을 무조건 JSON 으로 파싱한다. 실패 경로가 한 바이트라도 흘리면
    // 파싱은 깨지고, 최악의 경우 절반쯤 쓰인 봉투가 성공으로 읽힌다.
    let src = manifest(SAMPLE);
    let p = src.to_str().expect("경로").to_string();
    let cases: Vec<(Vec<&str>, i32, &str)> = vec![
        (vec!["inspect"], 2, "하위 명령 누락"),
        (vec!["inspect", "유니코드"], 2, "알 수 없는 하위 명령"),
        (vec!["inspect", "unicode"], 2, "파일 경로 누락"),
        (
            vec!["inspect", "unicode", &p, "--kind"],
            2,
            "--kind 값 누락",
        ),
        (
            vec!["inspect", "unicode", &p, "--kind", "없는축"],
            2,
            "알 수 없는 --kind 값",
        ),
        (
            vec!["inspect", "unicode", &p, "--wat"],
            2,
            "알 수 없는 옵션",
        ),
        (vec!["inspect", "unicode", &p, &p], 2, "위치 인자 과다"),
        (
            vec!["inspect", "unicode", "없는파일.hwp", "--json"],
            1,
            "파일 없음은 런타임 실패",
        ),
    ];
    for (args, code, why) in cases {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{why}\n{}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "{why} — 실패 경로가 stdout 에 {}바이트를 흘렸습니다\n{}",
            output.stdout.len(),
            describe(&args, &output)
        );
        assert!(
            !output.stderr.is_empty(),
            "{why} — 안내가 stderr 로도 나오지 않았습니다\n{}",
            describe(&args, &output)
        );
    }
}

// ── 문서 무변경 ────────────────────────────────────────────────────────────

#[test]
fn scanning_does_not_touch_the_document() {
    // 검사 명령이 원본을 건드리면 그 자체가 사고다. 스캔 전후 바이트가 같아야 한다.
    let doc = attack_document("immutable", "hwp");
    let before = std::fs::read(&doc).expect("픽스처 읽기");
    for extra in [vec![], vec!["--kind", "bidi"], vec!["--kind", "all"]] {
        let _ = inspect(&doc, &extra);
    }
    // 사람용(비 --json) 경로도 같은 보장을 진다.
    let _ = run(&["inspect", "unicode", doc.to_str().expect("경로")]);
    let after = std::fs::read(&doc).expect("픽스처 재읽기");
    assert_eq!(
        before.len(),
        after.len(),
        "스캔이 파일 크기를 바꿨습니다: {doc:?}"
    );
    assert!(before == after, "스캔이 파일 내용을 바꿨습니다: {doc:?}");
    let _ = std::fs::remove_file(&doc);
}

// ── 오탐 0 ─────────────────────────────────────────────────────────────────

/// 실제 한국 공문서·시험지·서식에서 단 한 건도 나오면 안 되는 표본.
///
/// `exam_kor.hwp` 는 중세 국어 옛한글(PUA)을 담은 국어 시험지다 — 전수 스윕에서 **유일하게**
/// 걸렸던 파일이고, 그 24건이 전부 PUA 낱자에 잇댄 U+200B(조판 보조)였다. 규칙을 좁힌
/// 근거이자 그 좁힘이 되돌아가지 않게 붙잡아 두는 자물쇠라 목록 맨 앞에 둔다.
const CLEAN_CORPUS: &[&str] = &[
    "samples/exam_kor.hwp",
    "samples/2026_oss_rst.hwp",
    "samples/hwp3-sample.hwp",
    "samples/2022년 국립국어원 업무계획.hwp",
    "samples/2025 행정업무운영 편람(최종).hwp",
    "samples/2025 행정업무운영 편람(최종).hwpx",
    "samples/21868765_별표2_보건소_분장사무.hwp",
    "samples/21_언어_기출_편집가능본.hwp",
    "samples/3-09월_교육_통합_2022.hwp",
    "samples/3-09월_교육_통합_2022.hwpx",
    "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx",
    "samples/exam_math.hwp",
];

#[test]
fn ordinary_korean_documents_are_clean() {
    let mut checked = 0usize;
    let mut noisy: Vec<String> = Vec::new();
    for rel in CLEAN_CORPUS {
        let p = manifest(rel);
        if !p.exists() {
            continue;
        }
        let path = p.to_str().expect("경로");
        let args = ["inspect", "unicode", path, "--json"];
        let output = run(&args);
        if output.status.code() != Some(0) {
            // 파싱 자체가 안 되는 파일은 이 테스트의 대상이 아니다(다른 축의 문제).
            continue;
        }
        let v = parse_stdout_json(&args, &output);
        checked += 1;
        if v["clean"] != true {
            noisy.push(format!(
                "  - {rel}: {}건 {:?}",
                v["findingCount"],
                kinds_of(&v)
            ));
        }
    }
    assert!(
        checked >= 8,
        "표본을 거의 못 읽었습니다 — 이 가드가 공허하게 통과합니다 ({checked}건)"
    );
    assert!(
        noisy.is_empty(),
        "정상 한국어 문서에서 오탐 {}건:\n{}\n\
         정상 문서가 걸리면 규칙을 좁히세요 — 경보는 오탐 한 건이면 통째로 무시됩니다.",
        noisy.len(),
        noisy.join("\n"),
    );
}

#[test]
fn scan_cost_is_linear_in_document_size() {
    // 문서 전체를 훑는 명령이 2차식이면 대형 문서에서 조용히 못 쓰게 된다.
    // 같은 문서를 잘라 만든 2배 크기 사이에서 문자 수와 시간이 함께 선형인지 본다.
    let big = manifest("samples/2025 행정업무운영 편람(최종).hwp");
    let small = manifest(SAMPLE);
    if !big.exists() {
        eprintln!("대형 샘플 없음 — 건너뜀");
        return;
    }
    let measure = |p: &Path| -> (u64, f64) {
        let path = p.to_str().expect("경로");
        let args = ["inspect", "unicode", path, "--json"];
        let t = std::time::Instant::now();
        let output = run(&args);
        let secs = t.elapsed().as_secs_f64();
        let v = parse_stdout_json(&args, &output);
        (v["scannedChars"].as_u64().unwrap_or(0), secs)
    };
    let (small_chars, _) = measure(&small);
    let (big_chars, big_secs) = measure(&big);
    assert!(
        big_chars > small_chars * 10,
        "대형 표본이 충분히 크지 않습니다: {big_chars} vs {small_chars}"
    );
    // 상한만 건다 — CI 부하에 따라 절대 시간은 흔들리지만, 2차식이면 이 상한을 못 지킨다.
    assert!(
        big_secs < 30.0,
        "{big_chars}자 문서 스캔에 {big_secs:.1}초 — 선형이라면 나올 수 없는 시간입니다"
    );
}
