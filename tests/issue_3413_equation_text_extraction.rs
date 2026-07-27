// [#3413] export-text 가 수식(Equation) 내용을 조용히 누락하던 결함의 회귀 계약.
// 실제 수능 수학 20쪽 문서(정답지 PDF 있음)에서 발견: 발문·선택지의 수식이 통째로
// 비었고(exit 0, 경고 없음), 파서/렌더는 정상이었다(`dump`로 script 확인됨).
use rhwp::document_core::DocumentCore;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

#[test]
fn export_text_includes_equation_script_not_empty_choices() {
    let bin = rhwp_bin();
    let sample = "samples/exam_math.hwp";
    let out = Command::new(&bin)
        .args(["export-text", "--json", sample])
        .output()
        .expect("export-text 실행 실패");
    assert!(out.status.success(), "exit code: {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON 파싱 실패");
    let pages = v["pages"].as_array().expect("pages 배열 없음");
    // p13(index 12)의 23번 문항: 수정 전에는 "23.\t\t의 값은?" 처럼 수식이 비었다.
    let p13 = pages[12]["text"].as_str().unwrap_or("");
    assert!(
        p13.contains("lim") || p13.contains("sin"),
        "p13 텍스트에 수식 스크립트(lim/sin)가 없음 — 수식 누락 회귀: {p13:?}"
    );
    // "23.\t\t의 값은?" 처럼 문항번호 뒤 탭 다음이 곧장 "의 값은"으로 이어지는(=사이의
    // 수식이 통째로 빈) 패턴이 없어야 한다.
    assert!(
        !p13.contains("\t\t의 값은") && !p13.contains("\t의 값은"),
        "발문에 수식이 빠진 빈 패턴이 여전히 존재함: {p13:?}"
    );
}

#[test]
fn markdown_includes_equation_scripts_from_table_cells() {
    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_math.hwp");
    let bytes = std::fs::read(&sample).expect("read samples/exam_math.hwp");
    let core = DocumentCore::from_bytes(&bytes).expect("parse samples/exam_math.hwp");
    let markdown = core
        .extract_page_markdown_native(12)
        .expect("extract page 13 markdown");

    assert!(
        markdown.contains('|'),
        "page 13 markdown should contain the answer-choice table"
    );
    assert!(
        markdown.contains("lim") || markdown.contains("sin"),
        "page 13 table markdown should preserve equation scripts: {markdown:?}"
    );
}
