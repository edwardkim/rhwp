//! #8160: 공용 corpus에서 분리한 원본 스타일 단사성·차이·동등성 경계.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, document::Document, paragraph::Paragraph};
use std::path::{Path, PathBuf};

/// 표 셀 안쪽까지 포함해 모든 문단을 모은다.
fn collect_paragraphs<'a>(paragraphs: &'a [Paragraph], out: &mut Vec<&'a Paragraph>) {
    for para in paragraphs {
        out.push(para);
        for ctrl in &para.controls {
            if let Control::Table(table) = ctrl {
                for cell in &table.cells {
                    collect_paragraphs(&cell.paragraphs, out);
                }
            }
        }
    }
}

fn all_paragraphs(doc: &Document) -> Vec<&Paragraph> {
    let mut out = Vec::new();
    for section in &doc.sections {
        collect_paragraphs(&section.paragraphs, &mut out);
    }
    out
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn parse_hwp3(rel: &str) -> DocumentCore {
    let raw = std::fs::read(repo_path(rel)).expect("표본 읽기");
    DocumentCore::from_bytes(&raw).expect("HWP3 파싱")
}

/// 문단이 실제로 참조하는 글자 모양 id 를 모은다(대표 + 런별).
fn used_char_shape_ids(doc: &Document) -> Vec<u32> {
    let mut ids = Vec::new();
    for para in all_paragraphs(doc) {
        for cs in &para.char_shapes {
            ids.push(cs.char_shape_id);
        }
    }
    ids
}

/// `ids` 가 가리키는 값의 가짓수와 id 자체의 가짓수를 센다.
///
/// 값끼리는 `PartialEq` 로만 비교한다(`Hash` 를 요구하지 않는다). 표본의 고유 스타일은
/// 수백 개 규모라 선형 비교로 충분하다.
fn distinct_counts<T: PartialEq>(ids: &[u32], pool: &[T]) -> (usize, usize) {
    let mut seen_ids: Vec<u32> = Vec::new();
    let mut seen_values: Vec<&T> = Vec::new();
    for &id in ids {
        let Some(value) = pool.get(id as usize) else {
            panic!("스타일 id {id} 가 풀 범위({})를 벗어난다", pool.len());
        };
        if !seen_ids.contains(&id) {
            seen_ids.push(id);
        }
        if !seen_values.iter().any(|v| **v == *value) {
            seen_values.push(value);
        }
    }
    (seen_ids.len(), seen_values.len())
}
#[test]
fn issue_8160_original_style_pool_preserves_unique_values_and_distinct_styles() {
    let core = parse_hwp3("samples/hwp3-sample16.hwp");
    let doc = core.document();
    let char_ids = used_char_shape_ids(doc);
    let para_ids: Vec<u32> = all_paragraphs(doc)
        .iter()
        .map(|p| p.para_shape_id as u32)
        .collect();
    assert!(!char_ids.is_empty() && !para_ids.is_empty());
    let (ids, values) = distinct_counts(&char_ids, &doc.doc_info.char_shapes);
    assert_eq!(ids, values, "글자 모양 id와 고유 값의 단사성");
    assert!(values > 1, "서로 다른 모양을 합치지 않는다");
    let (ids, values) = distinct_counts(&para_ids, &doc.doc_info.para_shapes);
    assert_eq!(ids, values, "문단 모양 id와 고유 값의 단사성");
    assert!(doc
        .doc_info
        .char_shapes
        .iter()
        .all(|s| s.raw_data.is_none()));
    assert!(doc
        .doc_info
        .para_shapes
        .iter()
        .all(|s| s.raw_data.is_none() && !s.hwpx_plain_para_margin));
}
