//! [Issue #4680] HWP3 파서가 스타일을 문단·런·표 셀마다 **새로** 등록해
//! `DocInfo` 가 수십 배로 부푼다.
//!
//! HWP3 은 문단마다 대표 글자 모양을, 런마다 개별 글자 모양을, 표 셀마다 테두리를
//! **값으로** 들고 있다. 파서가 조회 없이 그대로 밀어 넣어 같은 모양이 수천 벌 쌓였다 —
//! 264쪽 문서 실측에서 `CHAR_SHAPE` 13,902개(고유 166) · `PARA_SHAPE` 2,784개(고유 691) ·
//! `BORDER_FILL` 638개(고유 7), `DocInfo` 비압축 1,291,178B. 같은 문서를 한/글이 저장하면
//! 62,940B 다.
//!
//! 잠금 계약은 **단사성**이다 — 쓰이는 id 의 가짓수와 그 id 가 가리키는 값의 가짓수가
//! 같아야 한다. 중복이 있으면 id 가 값보다 많아지고(수정 전), 과도하게 합치면 값이
//! id 보다 많아진다(반대 방향 결함). 한쪽만 막으면 다른 쪽으로 틀릴 수 있어 둘 다 잠근다.
//!
//! 저장 왕복으로는 이 계약을 잠글 수 없다 — `HWP3→HWP5` 저장이 문단 모양을 정규화해
//! `samples/hwp3-sample.hwp` 문단 0 의 해석된 문단 모양이 바뀐다. 이 정리와 **무관하게**
//! 수정 전 기준선에서도 똑같이 바뀌는 것을 확인했으므로 별건이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
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

/// HWP3 표본 — 본문만 있는 것, 표가 있는 것, 쪽수가 큰 것을 섞는다.
const SAMPLES: [&str; 4] = [
    "samples/hwp3-sample.hwp",
    "samples/hwp3-sample11.hwp",
    "samples/hwp3-sample16.hwp",
    "samples/hwp3-empty-cell.hwp",
];

#[test]
fn hwp3_char_shape_ids_map_one_to_one_onto_values() {
    for rel in SAMPLES {
        let core = parse_hwp3(rel);
        let doc = core.document();
        let ids = used_char_shape_ids(doc);
        assert!(!ids.is_empty(), "{rel}: 글자 모양 참조가 하나도 없다");
        let (id_count, value_count) = distinct_counts(&ids, &doc.doc_info.char_shapes);
        assert_eq!(
            id_count, value_count,
            "{rel}: 쓰이는 글자 모양 id {id_count}개가 값 {value_count}개를 가리킨다 — \
             같은 모양이 여러 id 로 중복 등록됐다"
        );
    }
}

#[test]
fn hwp3_para_shape_ids_map_one_to_one_onto_values() {
    for rel in SAMPLES {
        let core = parse_hwp3(rel);
        let doc = core.document();
        let ids: Vec<u32> = all_paragraphs(doc)
            .iter()
            .map(|p| p.para_shape_id as u32)
            .collect();
        assert!(!ids.is_empty(), "{rel}: 문단이 하나도 없다");
        let (id_count, value_count) = distinct_counts(&ids, &doc.doc_info.para_shapes);
        assert_eq!(
            id_count, value_count,
            "{rel}: 쓰이는 문단 모양 id {id_count}개가 값 {value_count}개를 가리킨다"
        );
    }
}

#[test]
fn hwp3_cell_border_fill_ids_map_one_to_one_onto_values() {
    // 표가 있는 표본만 의미가 있다.
    let core = parse_hwp3("samples/hwp3-empty-cell.hwp");
    let doc = core.document();
    let mut ids = Vec::new();
    for section in &doc.sections {
        let mut paras = Vec::new();
        collect_paragraphs(&section.paragraphs, &mut paras);
        for para in paras {
            for ctrl in &para.controls {
                if let Control::Table(table) = ctrl {
                    for cell in &table.cells {
                        // 저장되는 id 는 1-based 다.
                        assert!(cell.border_fill_id >= 1, "셀 테두리 id 가 0 이다");
                        ids.push(cell.border_fill_id as u32 - 1);
                    }
                }
            }
        }
    }
    assert!(!ids.is_empty(), "표 셀을 하나도 못 찾았다");
    let (id_count, value_count) = distinct_counts(&ids, &doc.doc_info.border_fills);
    assert_eq!(
        id_count, value_count,
        "셀 테두리 id {id_count}개가 값 {value_count}개를 가리킨다"
    );
}

#[test]
fn distinct_styles_are_not_merged_away() {
    // 반례 — 정리가 과해서 서로 다른 모양까지 하나로 합치면 안 된다.
    // 이 표본들은 본문에 서로 다른 글자 모양이 실제로 섞여 있다.
    for rel in ["samples/hwp3-sample11.hwp", "samples/hwp3-sample16.hwp"] {
        let core = parse_hwp3(rel);
        let doc = core.document();
        let ids = used_char_shape_ids(doc);
        let (_, value_count) = distinct_counts(&ids, &doc.doc_info.char_shapes);
        assert!(
            value_count > 1,
            "{rel}: 글자 모양이 {value_count}가지뿐이다 — 서로 다른 모양까지 합쳐졌다"
        );
    }
}

#[test]
fn equality_used_for_pooling_is_total_on_hwp3_styles() {
    // 정리는 "같다고 판정된" 값의 자리를 재사용한다. 그래서 유일한 위험은 동등성 비교가
    // **의미 있는 필드를 빼먹는** 경우다 — `CharShape` 은 `raw_data` 를, `ParaShape` 은
    // `raw_data` 와 `hwpx_plain_para_margin` 을 비교에서 뺀다(다른 포맷 왕복 보존용).
    // HWP3 파스에서 이 필드들이 항상 기본값이어야 그 비교가 전동치이고 정리가 안전하다.
    //
    // (저장 왕복으로는 이 계약을 못 잠근다 — HWP3→HWP5 저장이 문단 모양을 정규화해
    // 정리 여부와 무관하게 값이 바뀐다. 수정 전 기준선에서도 똑같이 바뀌는 것을 확인했다.)
    for rel in SAMPLES {
        let core = parse_hwp3(rel);
        let doc = core.document();
        for (i, cs) in doc.doc_info.char_shapes.iter().enumerate() {
            assert!(
                cs.raw_data.is_none(),
                "{rel}: 글자 모양 {i} 이 raw_data 를 들고 있다 — 동등성 비교가 이 필드를 \
                 빼므로 내용이 다른 모양이 합쳐질 수 있다"
            );
        }
        for (i, ps) in doc.doc_info.para_shapes.iter().enumerate() {
            assert!(
                ps.raw_data.is_none(),
                "{rel}: 문단 모양 {i} 이 raw_data 를 들고 있다"
            );
            assert!(
                !ps.hwpx_plain_para_margin,
                "{rel}: 문단 모양 {i} 이 hwpx_plain_para_margin 을 켜고 있다 — \
                 동등성 비교가 이 필드를 뺀다"
            );
        }
    }
}
