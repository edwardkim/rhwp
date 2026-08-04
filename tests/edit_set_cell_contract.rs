//! [#3381] `edit set-cell` 출력·안전 계약 회귀 테스트 (Stage 3 세 번째 조각).
//!
//! 편집 계약(#3329/#3373 과 동일): ① `--dry-run` 은 파일을 만들지 않는다
//! ② 실패 시 출력 파일을 쓰지 않는다 ③ 반영 여부는 **`export-tables` 재독**으로
//! 확인한다 — 좌표계가 같으므로 발견→편집→검증이 한 주소로 닫힌다.
//! ④ 병합으로 덮인 칸은 앵커 좌표 안내와 함께 exit 2.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::model::style::CharShape;
use rhwp::wasm_api::HwpDocument;

/// 실제 배포 정부 양식 — 누름틀 0·표 53(병합 포함). set-cell 의 실전 대상 그 자체다.
const SAMPLE: &str = "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_out(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-setcell-{tag}-{}-{}.hwp",
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

fn tables_of(path: &Path) -> serde_json::Value {
    let args = ["export-tables", path.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    parse_json(&args, &output)
}

/// 본문 최상위 표(containerPath 없음) 중 첫 표의 첫 셀을 고른다.
/// 전역 index 체계에서 머리말·글상자 안 표가 앞 번호를 차지할 수 있으므로
/// (실측: 본 양식의 표 0 은 머리말 안), 최상위 표의 실제 index 를 함께 돌려준다.
fn pick_cell(v: &serde_json::Value) -> (u64, u64, u64, String) {
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
        cell["text"].as_str().unwrap_or("").to_string(),
    )
}

/// 산출 HWP의 같은 격자 셀에 실제로 기록된 글자 모양을 읽는다.
fn cell_char_shape(path: &Path, table_no: u64, row: u64, col: u64) -> CharShape {
    let bytes = std::fs::read(path).expect("셀 글자모양 확인용 파일 읽기");
    let doc = HwpDocument::from_bytes(&bytes).expect("셀 글자모양 확인용 문서 파싱");
    let document = doc.document();
    let grids = extract_tables(document);
    let grid = grids
        .iter()
        .find(|grid| grid.index == table_no as usize && grid.container_path.is_empty())
        .expect("본문 최상위 표");
    let Control::Table(table) =
        &document.sections[grid.section].paragraphs[grid.paragraph].controls[grid.control]
    else {
        panic!("표 grid가 Table control을 가리키지 않음");
    };
    let cell = table
        .cells
        .iter()
        .find(|cell| cell.row as u64 == row && cell.col as u64 == col)
        .expect("대상 셀");
    let shape_id = cell
        .paragraphs
        .first()
        .and_then(|paragraph| paragraph.char_shapes.first())
        .expect("대상 셀 첫 문단 글자모양")
        .char_shape_id as usize;
    document.doc_info.char_shapes[shape_id].clone()
}

/// 핵심 루프 — 셀 기록 후 export-tables 재독으로 같은 좌표의 값을 대조한다.
#[test]
fn set_cell_applies_and_verifies_by_reread() {
    let sample = sample();
    let before = tables_of(&sample);
    let (tbl, row, col, old) = pick_cell(&before);
    let new_value = "실증테스트값";
    assert_ne!(old, new_value);

    let out = temp_out("apply");
    let (ts, rs, cs) = (tbl.to_string(), row.to_string(), col.to_string());
    let args = [
        "edit",
        "set-cell",
        sample.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        new_value,
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
    let v = parse_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(
        v["oldText"].as_str().map(|s| s.to_string()),
        Some(old),
        "{v}"
    );
    assert_eq!(v["newText"], new_value, "{v}");
    assert!(out.exists());

    // 재독 대조 — 같은 표·좌표의 셀 텍스트가 새 값이어야 한다.
    let after = tables_of(&out);
    let after_table = after["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|t| t["index"].as_u64() == Some(tbl))
        .expect("같은 index 표");
    let found = after_table["cells"]
        .as_array()
        .expect("cells")
        .iter()
        .find(|c| c["row"].as_u64() == Some(row) && c["col"].as_u64() == Some(col))
        .expect("좌표 셀");
    assert_eq!(found["text"], new_value, "재독 값 불일치: {found}");
    let _ = std::fs::remove_file(&out);
}

/// `--dry-run` 은 파일을 만들지 않고 old→new 를 예고한다.
#[test]
fn dry_run_reports_without_output() {
    let sample = sample();
    let before = tables_of(&sample);
    let (tbl, row, col, old) = pick_cell(&before);
    let out = temp_out("dry");
    let (ts, rs, cs) = (tbl.to_string(), row.to_string(), col.to_string());
    let args = [
        "edit",
        "set-cell",
        sample.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        "무엇이든",
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["dryRun"], true, "{v}");
    assert_eq!(
        v["oldText"].as_str().map(|s| s.to_string()),
        Some(old),
        "{v}"
    );
    assert!(v.get("output").is_none(), "{v}");
    assert!(!out.exists(), "dry-run 은 파일을 만들면 안 됩니다");
}

/// 병합으로 덮인 칸은 앵커 좌표를 안내하며 exit 2 (실물 양식에서 동적으로 찾는다).
#[test]
fn covered_cell_is_usage_error_with_anchor_hint() {
    let sample = sample();
    let v = tables_of(&sample);
    // colSpan > 1 인 앵커를 가진 최상위 표를 찾아, 덮인 칸(col+1)을 겨냥한다.
    let mut target: Option<(u64, u64, u64)> = None;
    for t in v["tables"].as_array().expect("tables") {
        if t.get("containerPath").is_some() {
            continue;
        }
        let idx = t["index"].as_u64().unwrap();
        for c in t["cells"].as_array().unwrap() {
            if c["colSpan"].as_u64().unwrap_or(1) > 1 {
                target = Some((
                    idx,
                    c["row"].as_u64().unwrap(),
                    c["col"].as_u64().unwrap() + 1,
                ));
                break;
            }
        }
        if target.is_some() {
            break;
        }
    }
    let (tbl, row, col) = target.expect("병합 셀이 있는 실물 양식이어야 합니다");
    let (ts, rs, cs) = (tbl.to_string(), row.to_string(), col.to_string());
    let args = [
        "edit",
        "set-cell",
        sample.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        "x",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("병합으로 덮인 칸"),
        "{}",
        describe(&args, &output)
    );
}

/// 필수 인자 누락·격자 밖 좌표·모델 u16 범위를 넘는 좌표는 사용법 오류(2)다.
#[test]
fn missing_args_and_out_of_range_are_usage_errors() {
    let sample = sample();
    let s = sample.to_str().unwrap();
    let (tbl, _, _, _) = pick_cell(&tables_of(&sample));
    let ts = tbl.to_string();
    for args in [
        vec![
            "edit", "set-cell", s, "--row", "0", "--col", "0", "--text", "a",
        ], // --table 누락
        vec![
            "edit", "set-cell", s, "--table", &ts, "--col", "0", "--text", "a",
        ], // --row 누락
        vec![
            "edit", "set-cell", s, "--table", &ts, "--row", "0", "--text", "a",
        ], // --col 누락
        vec![
            "edit", "set-cell", s, "--table", &ts, "--row", "0", "--col", "0",
        ], // --text 누락
        vec![
            "edit", "set-cell", s, "--table", &ts, "--row", "999", "--col", "0", "--text", "a",
        ], // 격자 밖
        vec![
            "edit", "set-cell", s, "--table", &ts, "--row", "65536", "--col", "0", "--text", "a",
        ], // u16 초과 행은 0으로 wrap하면 안 됨
        vec![
            "edit", "set-cell", s, "--table", &ts, "--row", "0", "--col", "65536", "--text", "a",
        ], // u16 초과 열은 0으로 wrap하면 안 됨
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
    }
}

/// [#3391] 기본은 검정 글씨로 기록한다 — 셀 안내문(파란 이탤릭) 스타일을 상속하지 않는다.
/// keepStyle 봉투 필드가 기본 false 이고, --keep-style 이면 true.
#[test]
fn default_black_style_and_keep_style_flag() {
    let sample = sample();
    let (tbl, row, col, _) = pick_cell(&tables_of(&sample));
    let (ts, rs, cs) = (tbl.to_string(), row.to_string(), col.to_string());
    let before_shape = cell_char_shape(&sample, tbl, row, col);

    // 기본: keepStyle=false
    let out = temp_out("black");
    let args = [
        "edit",
        "set-cell",
        sample.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        "검정값",
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
    let v = parse_json(&args, &output);
    assert_eq!(
        v["keepStyle"], false,
        "기본은 검정 기록(keepStyle=false): {v}"
    );
    assert!(out.exists());
    let after_shape = cell_char_shape(&out, tbl, row, col);
    assert_eq!(after_shape.text_color, 0, "기본 글자색은 검정");
    assert!(!after_shape.italic, "기본은 비이탤릭");
    assert!(!after_shape.bold, "기본은 비진하게");
    assert_eq!(after_shape.font_ids, before_shape.font_ids, "글꼴은 보존");
    assert_eq!(
        after_shape.base_size, before_shape.base_size,
        "글자 크기는 보존"
    );
    assert_eq!(after_shape.ratios, before_shape.ratios, "장평은 보존");
    assert_eq!(after_shape.spacings, before_shape.spacings, "자간은 보존");
    let _ = std::fs::remove_file(&out);

    // --keep-style: keepStyle=true
    let out2 = temp_out("keep");
    let args2 = [
        "edit",
        "set-cell",
        sample.to_str().unwrap(),
        "--table",
        &ts,
        "--row",
        &rs,
        "--col",
        &cs,
        "--text",
        "상속값",
        "--keep-style",
        "-o",
        out2.to_str().unwrap(),
        "--json",
    ];
    let output2 = run(&args2);
    assert_eq!(
        output2.status.code(),
        Some(0),
        "{}",
        describe(&args2, &output2)
    );
    let v2 = parse_json(&args2, &output2);
    assert_eq!(v2["keepStyle"], true, "{v2}");
    let _ = std::fs::remove_file(&out2);
}
