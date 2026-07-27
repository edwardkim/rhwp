//! Issue #3413: 텍스트 표면이 수식(Equation) 내용을 조용히 누락하던 문제의 잔여 축.
//!
//! `search`·`export-text` 는 앞서 닫혔고(#3428 계열), `export-structure` 만 `para.text` 를
//! 그대로 써 수식이 통째로 빠져 있었다. 수학·과학 문서에서 발문과 선택지가 빈 값으로 나가고
//! **종료 코드는 0** 이라 파이프라인이 소실을 알 수 없다("죽지 않았지만 틀렸다" 계열).
//!
//! 재현 문서: `samples/exam_math.hwp` (20쪽 수능 수학, 선택지가 전부 수식 개체).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/exam_math.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn structure_json() -> String {
    let out = run(&["export-structure", sample().to_str().unwrap(), "--json"]);
    assert!(
        out.status.success(),
        "export-structure 실패: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 발문 안의 수식 script 가 구조 트리에 보존돼야 한다.
#[test]
fn structure_preserves_equation_script_in_headings() {
    let json = structure_json();
    for token in ["sqrt", "lim", "over"] {
        assert!(
            json.contains(token),
            "수식 script 토큰 {:?} 가 구조 출력에 없다 — 텍스트 표면이 수식을 버렸다",
            token
        );
    }
}

/// 선택지(①~⑤)는 값이 각각 수식 개체다. 값 없이 마커만 남으면 RAG·검색이 무의미해진다.
#[test]
fn structure_preserves_multiple_choice_values() {
    let json = structure_json();
    assert!(
        json.contains("① 1") && json.contains("⑤ 5"),
        "선택지 값이 비어 있다(마커만 남음) — got prefix: {}",
        &json[..json.len().min(400)]
    );
}
