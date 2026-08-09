//! [R50] 편집 전후 render-diff 회귀 게이트 계약 테스트.
//!
//! 이미 있는 `rhwp render-diff <전> <후> --json`(pair 모드, 단일 출처:
//! `src/diagnostics/render_geom_diff.rs`)을 대표 편집 3종의 전후에 배선한다.
//! 편집 자체는 편집 페이지의 기하를 정당하게 바꾸므로, 게이트가 고정하는 것은
//! "편집이 얼마나 바뀌었나"가 아니라 **편집 전후 비교의 불변식**이다:
//!
//! ① 결정성 — 같은 비교를 반복하면 봉투가 완전히 동일하다 (게이트 전제).
//! ② 국소성 — 변화(변위·구조)는 편집이 닿은 페이지 1곳에만 나타나고,
//!    나머지 페이지는 maxDisp == 0.0 · 구조 불일치 없음. 쪽수도 불변.
//! ③ 상한 — 편집 페이지의 maxDisp 는 실측 기반 제안 임계 이내
//!    (set-cell 539.6px 실측 → 600px, fill-fields 152.0px 실측 → 200px).
//!    **임계 확정은 메인테이너 몫** — 근거·분포는
//!    `mydocs/report/edit_render_gate_r1_20260808.md`.
//! ④ 카나리 — 동폭 치환(회사→기관)은 기하 변화가 정확히 0 (maxDisp 0.0, PASS).
//! ⑤ red 실증 — 폭이 크게 다른 장문 치환은 OVER·exit 3 으로 잡힌다.
//!
//! red 주입 실증 (2026-08-08, 변이 후 복원):
//! - 카나리 치환을 장문(회사→주식회사법인등기부등본상호명)으로 바꾸면
//!   status=OVER, maxDisp=279.0, exit 3 — ④가 red 로 간다.
//! - 편집 페이지 상한을 실측 아래(--max-disp 1.0 기본)로 두면 set-cell 전후가
//!   OVER·exit 3 — ③이 red 로 간다 (본 파일에서는 --max-disp 600 으로 green).
//!
//! 실측 전량은 `python tools/measure_edit_render_gate.py` 로 재현된다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 실제 배포 정부 양식(30쪽, 표 53) — set-cell 계약 테스트와 같은 fixture.
const FORM_HWPX: &str = "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx";
/// 누름틀 3쪽 문서 — fill-fields/replace-text 계약 테스트와 같은 fixture.
const FIELD_HWP: &str = "samples/field-01.hwp";

/// render-diff `--json` 종료 코드 계약: 3 = 회귀 검출 (런타임 실패 1 과 구분).
const EXIT_REGRESSION: i32 = 3;

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_out(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-r50gate-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

/// 전후 pair diff — (종료코드, 봉투). stderr(레이아웃 진단)는 계약 채널이 아니다.
fn render_diff(a: &Path, b: &Path, extra: &[&str]) -> (i32, serde_json::Value) {
    let mut args = vec!["render-diff", a.to_str().unwrap(), b.to_str().unwrap()];
    args.extend_from_slice(extra);
    args.push("--json");
    let output = run(&args);
    let code = output.status.code().unwrap_or(-1);
    (code, parse_json(&args, &output))
}

/// 국소성 단언 — 변화(변위>0 또는 구조 불일치)가 있는 페이지 번호 목록.
fn changed_pages(v: &serde_json::Value) -> Vec<u64> {
    v["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .filter(|p| {
            p["maxDisp"].as_f64().expect("maxDisp") > 0.0
                || p["structureMismatch"].as_bool().expect("structureMismatch")
        })
        .map(|p| p["page"].as_u64().expect("page"))
        .collect()
}

/// 쪽수 불변 단언 — 편집이 페이지 수를 바꾸면 게이트 최강 신호다.
fn assert_page_count_stable(v: &serde_json::Value) {
    assert_eq!(v["pageCountMismatch"], false, "{v}");
    assert_eq!(v["pageCountA"], v["pageCountB"], "{v}");
}

/// export-tables 로 본문 최상위 표 첫 셀 좌표를 고른다 (set-cell 계약 테스트와 동일).
fn pick_top_level_cell(doc: &Path) -> (u64, u64, u64) {
    let args = ["export-tables", doc.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    let table = v["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|t| t.get("containerPath").is_none())
        .expect("본문 최상위 표");
    let cell = &table["cells"][0];
    (
        table["index"].as_u64().expect("index"),
        cell["row"].as_u64().expect("row"),
        cell["col"].as_u64().expect("col"),
    )
}

/// ① 결정성 — 자기 pair diff 는 변위 정확히 0 이고, 반복해도 봉투가 동일하다.
/// 이 성질이 깨지면 게이트는 hard 로 쓸 수 없다 (soft/관측 모드로 강등해야 한다).
#[test]
fn self_pair_diff_is_deterministic_zero() {
    let doc = sample(FIELD_HWP);
    let (code1, v1) = render_diff(&doc, &doc, &[]);
    let (code2, v2) = render_diff(&doc, &doc, &[]);
    assert_eq!(code1, 0, "{v1}");
    assert_eq!(code2, 0, "{v2}");
    assert_eq!(v1["status"], "PASS", "{v1}");
    assert_eq!(v1["maxDisp"].as_f64(), Some(0.0), "{v1}");
    assert_eq!(v1["regression"], false, "{v1}");
    // 봉투 전체 동일 — maxDisp 만이 아니라 페이지 상세까지 결정적이어야 한다.
    assert_eq!(v1, v2, "같은 비교의 봉투가 달라지면 게이트 판정이 흔들린다");
}

/// ②③ set-cell 전후 — 변화는 편집 페이지 1곳, 나머지 29쪽 변위 0, 쪽수 30 불변.
/// 편집 페이지 상한: 실측 539.6px → 제안 임계 600px 에서 PASS·exit 0.
/// 기본 임계(1.0px)에서는 OVER·exit 3 — red 경로가 살아 있음을 함께 고정한다.
#[test]
fn set_cell_before_after_is_localized_and_within_proposed_ceiling() {
    let doc = sample(FORM_HWPX);
    let (tbl, row, col) = pick_top_level_cell(&doc);
    let out = temp_out("setcell", "hwpx");
    let (ts, rs, cs) = (tbl.to_string(), row.to_string(), col.to_string());
    let args = [
        "edit",
        "set-cell",
        doc.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        "실증테스트값",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    // 기본 임계(1.0px): 편집 자체가 임계를 넘으므로 OVER·exit 3 (실측 539.6px).
    let (code, v) = render_diff(&doc, &out, &[]);
    assert_eq!(code, EXIT_REGRESSION, "{v}");
    assert_eq!(v["status"], "OVER", "{v}");
    assert_eq!(v["regression"], true, "{v}");
    assert_page_count_stable(&v);
    // 구조는 불변 — set-cell 은 셀 텍스트만 바꾼다 (실측: structPages 0).
    assert_eq!(v["structPages"].as_u64(), Some(0), "{v}");
    assert_eq!(v["hardStructPages"].as_u64(), Some(0), "{v}");
    // 국소성: 변화가 있는 페이지는 정확히 1곳 (편집이 닿은 페이지).
    let changed = changed_pages(&v);
    assert_eq!(changed.len(), 1, "변화 페이지 {changed:?}: {v}");
    assert_eq!(v["worstPage"].as_u64(), Some(changed[0]), "{v}");
    // 편집은 보여야 한다 — 변위 0 이면 편집이 반영되지 않은 것.
    assert!(v["maxDisp"].as_f64().expect("maxDisp") > 0.0, "{v}");

    // 제안 임계 600px (실측 539.6px + 여유): PASS·exit 0. 임계 확정은 메인테이너 몫.
    let (code600, v600) = render_diff(&doc, &out, &["--max-disp", "600"]);
    assert_eq!(code600, 0, "{v600}");
    assert_eq!(v600["status"], "PASS", "{v600}");
    assert_eq!(v600["regression"], false, "{v600}");
    let _ = std::fs::remove_file(&out);
}

/// ②③ fill-fields 전후 — 변화는 편집 페이지 1곳(구조: TextRun -2), 쪽수 3 불변,
/// 편집 페이지 maxDisp 는 실측 152.0px → 제안 상한 200px 이내.
/// 구조 불일치는 --max-disp 를 아무리 키워도 침묵하지 않는다 (임계 독립 하드 신호).
#[test]
fn fill_fields_before_after_struct_change_is_localized() {
    let doc = sample(FIELD_HWP);
    let out = temp_out("fill", "hwp");
    let args = [
        "edit",
        "fill-fields",
        doc.to_str().unwrap(),
        "--data",
        r#"{"회사명":"주식회사 검증"}"#,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let (code, v) = render_diff(&doc, &out, &[]);
    assert_eq!(code, EXIT_REGRESSION, "{v}");
    assert_eq!(v["status"], "STRUCT_MISMATCH", "{v}");
    assert_page_count_stable(&v);
    let changed = changed_pages(&v);
    assert_eq!(changed.len(), 1, "변화 페이지 {changed:?}: {v}");
    // 편집 페이지 변위 상한 — 실측 152.0px, 제안 200px (근거는 r1 보고서).
    let max_disp = v["maxDisp"].as_f64().expect("maxDisp");
    assert!(
        max_disp > 0.0 && max_disp <= 200.0,
        "편집 페이지 변위 {max_disp}px 가 제안 상한 200px 를 벗어남: {v}"
    );
    // 구조 변화의 정체 고정 — 누름틀 안내 run 이 실값 run 으로 합쳐지며 TextRun -2.
    let page = v["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .find(|p| p["page"].as_u64() == Some(changed[0]))
        .expect("변화 페이지");
    let deltas = page["typeDeltas"].as_array().expect("typeDeltas");
    assert_eq!(deltas.len(), 1, "{v}");
    assert_eq!(deltas[0]["nodeType"], "TextRun", "{v}");
    assert_eq!(deltas[0]["net"].as_i64(), Some(-2), "{v}");

    // 임계 독립성: --max-disp 100000 에서도 STRUCT_MISMATCH·exit 3 유지.
    let (code_hi, v_hi) = render_diff(&doc, &out, &["--max-disp", "100000"]);
    assert_eq!(code_hi, EXIT_REGRESSION, "{v_hi}");
    assert_eq!(v_hi["status"], "STRUCT_MISMATCH", "{v_hi}");
    let _ = std::fs::remove_file(&out);
}

/// ④ 카나리 — 동폭 치환(회사→기관 2자→2자)은 기하 변화가 정확히 0 이어야 한다.
/// 편집·레이아웃 어느 쪽이든 잡음이 생기면 이 0.0 이 먼저 깨진다.
///
/// red 주입 실증(변이 후 복원, 2026-08-08): --replace 를
/// "주식회사법인등기부등본상호명"(14자)으로 바꾸면 OVER·maxDisp 279.0·exit 3.
#[test]
fn same_width_replace_is_zero_disp_canary() {
    let doc = sample(FIELD_HWP);
    let out = temp_out("canary", "hwp");
    let args = [
        "edit",
        "replace-text",
        doc.to_str().unwrap(),
        "--find",
        "회사",
        "--replace",
        "기관",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let (code, v) = render_diff(&doc, &out, &[]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["status"], "PASS", "{v}");
    assert_eq!(v["regression"], false, "{v}");
    assert_eq!(v["maxDisp"].as_f64(), Some(0.0), "{v}");
    assert_page_count_stable(&v);
    assert!(changed_pages(&v).is_empty(), "{v}");
    let _ = std::fs::remove_file(&out);
}

/// ⑤ red 경로 상시 실증 — 폭이 크게 다른 장문 치환은 기본 임계에서 반드시
/// OVER·exit 3 으로 잡힌다. 게이트가 "항상 green" 으로 퇴화하지 않았음을 고정한다.
#[test]
fn disruptive_edit_is_caught_as_regression() {
    let doc = sample(FIELD_HWP);
    let out = temp_out("red", "hwp");
    let args = [
        "edit",
        "replace-text",
        doc.to_str().unwrap(),
        "--find",
        "회사",
        "--replace",
        "주식회사법인등기부등본상호명",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let (code, v) = render_diff(&doc, &out, &[]);
    assert_eq!(code, EXIT_REGRESSION, "{v}");
    assert_eq!(v["status"], "OVER", "{v}");
    assert_eq!(v["regression"], true, "{v}");
    // 파괴적 편집도 국소성은 유지 (실측: page 0 한정, maxDisp 279.0).
    assert_page_count_stable(&v);
    assert!(v["maxDisp"].as_f64().expect("maxDisp") > 1.0, "{v}");
    let _ = std::fs::remove_file(&out);
}
