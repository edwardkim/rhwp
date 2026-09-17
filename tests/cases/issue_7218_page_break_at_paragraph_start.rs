//! [#7218] 문단 시작(offset 0)의 쪽 나눔은 문단을 가르지 않는다.
//!
//! 종전 `edit insert-page-break --offset 0` 은 offset 과 관계없이 문단을 갈라, 앞쪽에 원 문단
//! 모양(개요 수준 포함)을 물려받은 **빈 문단**을 남겼다. 개요 제목 앞이면 한글이 그 빈 문단에도
//! 번호를 붙여 번호가 비고 뒤 번호가 밀린다. 수정 후에는 대상 문단의 `column_type` 만 `Page` 로
//! 바꾸고 문단 수·텍스트·문단 모양을 그대로 둔다. 봉투는 `paragraphDelta`·`pageBreakParagraph` 로
//! 후속 좌표 편집이 어긋나지 않게 알린다.
//!
//! 범위: CLI `edit insert-page-break` 와 이를 호출하는 MCP `hwp_insert_page_break`.
//! Studio Ctrl+Enter·웹한글컨트롤 `BreakPage` 는 한컴 COM Oracle 근거가 없어 종전대로 둔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::model::paragraph::ColumnBreakType;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin()).args(args).output().expect("rhwp")
}

fn temp(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-7218-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// 이슈 재현 입력: 개요 제목 둘과 본문 둘.
fn scaffold_outline_doc() -> PathBuf {
    let json = temp("spec", "json");
    std::fs::write(
        &json,
        r#"{"version":"1","title":"repro","blocks":[
 {"type":"heading","level":1,"text":"First"},
 {"type":"paragraph","text":"body 1"},
 {"type":"heading","level":1,"text":"Second"},
 {"type":"paragraph","text":"body 2"}
]}"#,
    )
    .unwrap();
    let out = temp("doc", "hwpx");
    let output = run(&[
        "scaffold",
        json.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "scaffold: {output:?}");
    let _ = std::fs::remove_file(&json);
    out
}

struct Para {
    text: String,
    para_shape_id: u16,
    page_break: bool,
}

fn paragraphs(path: &Path) -> Vec<Para> {
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    doc.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| Para {
            text: p.text.clone(),
            para_shape_id: p.para_shape_id,
            page_break: p.column_type == ColumnBreakType::Page,
        })
        .collect()
}

fn index_of(paras: &[Para], text: &str) -> usize {
    paras
        .iter()
        .position(|p| p.text == text)
        .unwrap_or_else(|| panic!("`{text}` 문단이 없다"))
}

fn page_break(src: &Path, para: usize, offset: usize, tag: &str) -> (PathBuf, serde_json::Value) {
    let out = temp(tag, "hwpx");
    let para_arg = para.to_string();
    let offset_arg = offset.to_string();
    let output = run(&[
        "edit",
        "insert-page-break",
        src.to_str().unwrap(),
        "--section",
        "0",
        "--para",
        para_arg.as_str(),
        "--offset",
        offset_arg.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "insert-page-break: {output:?}"
    );
    let envelope = serde_json::from_slice(&output.stdout).expect("json envelope");
    (out, envelope)
}

#[test]
fn issue_7218_offset_zero_before_outline_heading_does_not_leave_empty_paragraph() {
    let src = scaffold_outline_doc();
    let before = paragraphs(&src);
    let target = index_of(&before, "Second");

    let (out, envelope) = page_break(&src, target, 0, "zero");
    let after = paragraphs(&out);

    assert_eq!(
        after.len(),
        before.len(),
        "문단 수가 바뀌었다 — 빈 문단이 생겼다"
    );
    assert!(
        after.iter().all(|p| !p.text.is_empty()),
        "빈 문단이 생겼다: {:?}",
        after.iter().map(|p| p.text.as_str()).collect::<Vec<_>>()
    );
    let second = &after[target];
    assert_eq!(second.text, "Second");
    assert!(second.page_break, "`Second` 문단에 쪽 나눔이 걸려야 한다");
    assert_eq!(
        second.para_shape_id, before[target].para_shape_id,
        "문단 모양(개요 수준)이 그대로여야 한다"
    );
    assert_eq!(
        after.iter().filter(|p| p.page_break).count(),
        1,
        "쪽 나눔은 대상 문단 하나에만 걸린다"
    );
    assert_eq!(envelope["paragraphDelta"], 0);
    assert_eq!(envelope["pageBreakParagraph"], target);

    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn issue_7218_repeated_offset_zero_does_not_accumulate() {
    let src = scaffold_outline_doc();
    let target = index_of(&paragraphs(&src), "Second");
    let (once, _) = page_break(&src, target, 0, "once");
    let (twice, envelope) = page_break(&once, target, 0, "twice");

    let first = paragraphs(&once);
    let second = paragraphs(&twice);
    assert_eq!(second.len(), first.len(), "반복 호출로 문단이 늘었다");
    assert!(second[target].page_break);
    assert_eq!(second.iter().filter(|p| p.page_break).count(), 1);
    assert_eq!(envelope["paragraphDelta"], 0);

    for path in [&src, &once, &twice] {
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn issue_7218_mid_paragraph_offset_still_splits() {
    let src = scaffold_outline_doc();
    let before = paragraphs(&src);
    let target = index_of(&before, "Second");

    let (out, envelope) = page_break(&src, target, 3, "mid");
    let after = paragraphs(&out);

    assert_eq!(after.len(), before.len() + 1, "중간 오프셋은 문단을 가른다");
    assert_eq!(after[target].text, "Sec");
    assert!(!after[target].page_break);
    assert_eq!(after[target + 1].text, "ond");
    assert!(after[target + 1].page_break);
    assert_eq!(envelope["paragraphDelta"], 1);
    assert_eq!(envelope["pageBreakParagraph"], target + 1);

    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}
