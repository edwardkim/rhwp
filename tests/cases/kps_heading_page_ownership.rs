//! KPS 원본의 상세 요구사항 제목과 실제 표의 쪽 소속·내용 순서.
//! 독립 한컴 2022 Print의 장비 구성·기능 요구사항 제목은 해당 표와 같은 쪽이다.
//! 현재 Native/fresh WASM 전77쪽 최저90.85010%, 하락/미달/누락0과 직접 판독 후 등록했다.
//! 증거: mydocs/pr/archives/semanticist21_20261008_integrated_review.md의 KPS 리베이스 보정 기록.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;
use std::{
    path::PathBuf,
    process::Command,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

const SAMPLE: &str = "samples/kps-ai.hwp";

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}
fn source_paragraph(text: &str) -> usize {
    let core = DocumentCore::from_bytes(
        &std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
    )
    .unwrap();
    let found: Vec<_> = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.controls.is_empty() && compact(&p.text).contains(&compact(text)))
        .map(|(pi, _)| pi)
        .collect();
    assert_eq!(found.len(), 1, "원본 제목의 유일한 소유 문단");
    found[0]
}
struct OutputDir(PathBuf);
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let binary = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = OutputDir(
            std::env::temp_dir().join(format!("rhwp-kps-ownership-{}-{nonce}", std::process::id())),
        );
        std::fs::create_dir(&dir.0).unwrap();
        let output = Command::new(binary)
            .arg("export-render-tree")
            .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
            .args(["--compat", "2022", "-o"])
            .arg(&dir.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let mut files: Vec<_> = std::fs::read_dir(&dir.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("render_tree_") && n.ends_with(".json"))
            })
            .collect();
        files.sort();
        assert!(!files.is_empty(), "최종 출력 쪽을 실제로 검사한다");
        files
            .iter()
            .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
            .collect()
    })
}
fn rendered_text(node: &Value) -> String {
    let mut text = node["text"].as_str().unwrap_or_default().to_string();
    for child in node["children"].as_array().into_iter().flatten() {
        text.push_str(&rendered_text(child));
    }
    text
}
fn owner(pi: usize, kind: &str) -> (usize, usize) {
    let expected_type = match kind {
        "fullParagraph" => "TextLine",
        "table" => "Table",
        _ => panic!("지원하지 않는 소유 항목"),
    };
    let found: Vec<_> = pages()
        .iter()
        .enumerate()
        .flat_map(|(page, p)| {
            p["children"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|n| n["type"] == "Body")
                .flat_map(move |body| {
                    body["children"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .flat_map(move |column| {
                            column["children"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .enumerate()
                                .filter(move |(_, item)| {
                                    item["pi"] == pi && item["type"] == expected_type
                                })
                                .map(move |(order, item)| (page, order, item))
                        })
                })
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "제목/표는 최종 출력의 한 쪽에 한 번만 표시한다"
    );
    if kind == "fullParagraph" {
        let core = DocumentCore::from_bytes(
            &std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
        )
        .unwrap();
        assert_eq!(
            compact(&rendered_text(found[0].2)),
            compact(&core.document().sections[0].paragraphs[pi].text),
            "제목의 원본 내용이 빠지거나 중복되지 않는다"
        );
    }
    (found[0].0, found[0].1)
}
#[test]
fn equipment_heading_and_requirement_table_keep_their_source_page() {
    let chapter = source_paragraph("2. 상세 요구사항");
    let heading = source_paragraph("가. 시스템 장비구성 요구사항");
    let (chapter_page, chapter_order) = owner(chapter, "fullParagraph");
    let (heading_page, heading_order) = owner(heading, "fullParagraph");
    let (table_page, table_order) = owner(heading + 1, "table");
    let (catalogue_page, _) = owner(chapter - 1, "table");
    assert_eq!(
        chapter_page, heading_page,
        "장 제목과 장비 구성 제목은 같은 쪽"
    );
    assert_eq!(
        heading_page, table_page,
        "장비 구성 제목만 앞쪽 끝에 남지 않는다"
    );
    assert!(chapter_order < heading_order && heading_order < table_order);
    assert_eq!(
        chapter_page,
        catalogue_page + 1,
        "앞 요구사항 목록과 새 상세 장의 쪽 경계"
    );
}
#[test]
fn function_heading_stays_with_its_table_after_equipment_requirements() {
    let equipment = source_paragraph("가. 시스템 장비구성 요구사항");
    let function = source_paragraph("나. 기능 요구사항");
    let (equipment_page, _) = owner(equipment + 1, "table");
    let (heading_page, heading_order) = owner(function, "fullParagraph");
    let (table_page, table_order) = owner(function + 1, "table");
    assert_eq!(
        heading_page, table_page,
        "기능 요구사항 제목과 해당 표는 같은 쪽"
    );
    assert!(heading_order < table_order, "본문 순서 보존");
    assert_eq!(
        heading_page,
        equipment_page + 1,
        "장비 표 뒤의 독립 저장 쪽 경계"
    );
}
