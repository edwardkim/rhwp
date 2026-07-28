//! [#3469] `ir-diff` 가 표 셀 안의 변경을 감지하는지 고정한다.
//!
//! `ir-diff` 는 단독 진단 도구가 아니라 **변환 검증 게이트**다 —
//! `convert --verify` / `export-hwpx --verify` 가 IR 차이를 exit 3 으로 신호하는 근거가
//! 이 비교다. 한국 문서는 표가 본체이므로, 표 셀 안이 보이지 않으면 변환이 표의 모든
//! 텍스트를 손상시켜도 게이트가 통과한다.
//!
//! 글상자는 #1807 이 같은 구멍(#1795 "소거망 구멍")을 이미 막았다. 표도 같아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 본문 내용이 전부 표 셀 안에 있는 실물 보도자료 서식 (누름틀 12개).
const SAMPLE_FORM: &str = "samples/20250130-hongbo.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-irdiff-cells-{tag}-{}-{}.hwp",
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

/// 누름틀 값을 바꿔 **표 셀 안의 텍스트만** 달라진 산출물을 만든다.
fn make_edited_copy(tag: &str) -> Option<PathBuf> {
    let src = sample(SAMPLE_FORM);
    if !src.exists() {
        return None;
    }
    let out = temp_path(tag);
    let args = [
        "edit",
        "fill-fields",
        src.to_str().unwrap(),
        "--data",
        r#"{"기관명":"가상광역시 상수도사업본부","제목명":"가상시, 폭염 대비 비상급수 체계 가동"}"#,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "편집 산출물 생성 실패\n{}",
        describe(&args, &output)
    );
    assert!(out.exists(), "산출물이 생성되지 않았습니다");
    Some(out)
}

#[test]
fn ir_diff_detects_text_change_inside_table_cells() {
    // 이 테스트가 본 수정의 존재 이유다. 편집으로 표 셀 텍스트가 바뀌었는데
    // ir-diff 가 identical 을 보고하면, --verify 게이트가 표 손상을 통과시킨다.
    let Some(edited) = make_edited_copy("detect") else {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    };
    let src = sample(SAMPLE_FORM);

    // 먼저 실제로 본문이 달라졌음을 독립적으로 확인한다(ir-diff 를 믿지 않는다).
    let text_args = ["export-text", src.to_str().unwrap(), "--json"];
    let before_text = run(&text_args);
    let after_text = run(&["export-text", edited.to_str().unwrap(), "--json"]);
    let bv: serde_json::Value =
        serde_json::from_slice(&before_text.stdout).expect("before export-text");
    let av: serde_json::Value =
        serde_json::from_slice(&after_text.stdout).expect("after export-text");
    assert_ne!(
        bv["pages"][0]["text"], av["pages"][0]["text"],
        "전제 조건 실패: 본문이 실제로 달라져야 합니다"
    );

    // 본론: ir-diff 가 그 차이를 봐야 한다.
    let args = [
        "ir-diff",
        src.to_str().unwrap(),
        edited.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ir-diff --json 이 순수 JSON 이어야 합니다");

    assert_eq!(
        v["identical"],
        false,
        "표 셀 텍스트가 달라졌는데 identical 로 보고했습니다 — --verify 게이트가 뚫립니다.\n{}",
        describe(&args, &output)
    );
    assert!(
        v["diffCount"].as_u64().unwrap() >= 1,
        "{}",
        describe(&args, &output)
    );
    // 게이트 신호: 차이 발견은 exit 3 (#2707 계약).
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}",
        describe(&args, &output)
    );

    let _ = std::fs::remove_file(&edited);
}

#[test]
fn ir_diff_identical_document_still_reports_no_difference() {
    // 무회귀 가드: 셀 재귀를 넣어도 같은 문서는 여전히 차이 0 이어야 한다.
    let src = sample(SAMPLE_FORM);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let s = src.to_str().unwrap();
    let args = ["ir-diff", s, s, "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("ir-diff JSON");
    assert_eq!(v["identical"], true, "{}", describe(&args, &output));
    assert_eq!(v["diffCount"], 0, "{}", describe(&args, &output));
}

#[test]
fn fill_fields_default_output_lands_next_to_input() {
    // [#3469] `-o` 를 생략하면 산출물은 **입력 파일 옆**에 생겨야 한다.
    // 종전에는 파일명만 써서 현재 작업 디렉터리에 떨어졌고, 임의 경로 문서를 다루는
    // 에이전트·MCP 클라이언트에게는 산출물이 엉뚱한 곳(저장소 루트 등)에 생겼다.
    let src = sample(SAMPLE_FORM);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    // 원본 폴더를 더럽히지 않도록 임시 폴더로 복사해 검증한다.
    let staged = temp_path("defaultout");
    std::fs::copy(&src, &staged).expect("샘플 복사 실패");

    let args = [
        "edit",
        "fill-fields",
        staged.to_str().unwrap(),
        "--data",
        r#"{"담당자명":"김가온"}"#,
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fill-fields JSON");
    let reported = v["output"].as_str().expect("output 경로");
    let reported_path = Path::new(reported);
    assert_eq!(
        reported_path.parent(),
        staged.parent(),
        "산출물은 입력 파일과 같은 폴더에 생겨야 합니다: {reported}"
    );
    assert!(
        reported_path.exists(),
        "보고된 경로에 파일이 실제로 있어야 합니다: {reported}"
    );

    let _ = std::fs::remove_file(&staged);
    let _ = std::fs::remove_file(reported_path);
}
