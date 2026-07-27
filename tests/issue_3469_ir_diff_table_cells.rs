//! Issue #3469: `ir-diff` 가 표 셀 안의 텍스트 변경을 감지하지 못해 `--verify` 게이트가
//! 표 손상을 통과시키던 결함의 계약 회귀 테스트.
//!
//! `diff_table()` 은 표 수준 속성(cell_spacing·text_wrap·page_break 등)만 비교하고 셀 문단을
//! 들여다보지 않았다. 한국 문서는 제목·기관명·연락처가 대개 표 셀 안에 있어, 변환이 표
//! 안의 모든 텍스트를 손상시켜도 `identical:true` · exit 0 으로 통과했다. #1807 이 글상자에
//! 대해 닫은 구멍과 같은 계열이다.
//!
//! 재현 문서: `samples/20250130-hongbo.hwp` (실물 보도자료 서식, 누름틀 12개가 전부 표 셀 안).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/20250130-hongbo.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-issue3469-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

/// 표 셀 안 누름틀 값을 채운 문서는 원본과 **다르게** 판정돼야 한다(exit 3).
#[test]
fn ir_diff_detects_table_cell_text_change() {
    let before = temp_path("before", "hwp");
    let after = temp_path("after", "hwp");
    let data = temp_path("data", "json");
    std::fs::copy(sample(SAMPLE), &before).expect("샘플 복사");
    std::fs::write(
        &data,
        r#"{"기관명":"가상광역시 상수도사업본부","담당자명":"홍길동"}"#,
    )
    .expect("data.json 쓰기");

    let fill = run(&[
        "edit",
        "fill-fields",
        before.to_str().unwrap(),
        "--data",
        &format!("@{}", data.to_str().unwrap()),
        "-o",
        after.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        fill.status.success(),
        "fill-fields 실패: {}",
        String::from_utf8_lossy(&fill.stderr)
    );

    let diff = run(&[
        "ir-diff",
        before.to_str().unwrap(),
        after.to_str().unwrap(),
        "--json",
    ]);
    let stdout = String::from_utf8_lossy(&diff.stdout).into_owned();
    for path in [&before, &after, &data] {
        let _ = std::fs::remove_file(path);
    }
    assert!(
        stdout.contains("\"identical\":false"),
        "표 셀 텍스트 변경이 검출되어야 함, got={}",
        stdout
    );
    assert_eq!(
        diff.status.code(),
        Some(3),
        "IR 차이는 exit 3 으로 신호해야 함, stdout={}",
        stdout
    );
}

/// 같은 문서끼리는 여전히 동일 판정(무회귀) — 셀 재귀가 거짓 차이를 만들지 않아야 한다.
#[test]
fn ir_diff_reports_identical_for_same_document() {
    let path = sample(SAMPLE);
    let path = path.to_str().unwrap();
    let diff = run(&["ir-diff", path, path, "--json"]);
    let stdout = String::from_utf8_lossy(&diff.stdout);
    assert!(
        stdout.contains("\"identical\":true"),
        "동일 문서는 identical:true 여야 함, got={}",
        stdout
    );
    assert_eq!(
        diff.status.code(),
        Some(0),
        "동일 문서는 exit 0, {}",
        stdout
    );
}
