//! [#5592] IR 에서 수정한 개체 공통 배치 속성의 HWP5 저장 보존 계약.
//!
//! CTRL_HEADER attr 는 파스 시점 raw(u32)가 `common.attr` 에 캐시되는데, 종전
//! 직렬화기는 attr != 0 이면 그 캐시를 통째로 우선해 IR enum 필드(text_wrap·
//! vert_rel_to·treat_as_char 등)의 수정이 HWP5 저장에서 전량 유실됐다(HWPX
//! 저장은 보존 — #5592 비대칭). #4495 봉인이 "직접 수정 → IR 합성" 을 약속해도
//! 합성기가 캐시를 다시 우선해 약속이 깨졌다. 수정 후 계약: 알려진 의미 비트는
//! IR 이 정본, 미지 비트만 raw 보존.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::model::control::Control;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::wasm_api::HwpDocument;

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn load_sample() -> HwpDocument {
    let bytes = std::fs::read(repo_path("samples/hwp_table_test-m.hwp")).expect("샘플 읽기");
    HwpDocument::from_bytes(&bytes).expect("샘플 파싱")
}

fn first_table_common(doc: &HwpDocument) -> &rhwp::model::shape::CommonObjAttr {
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .find_map(|p| {
            p.controls.iter().find_map(|c| match c {
                Control::Table(t) => Some(&t.common),
                _ => None,
            })
        })
        .expect("표")
}

#[test]
fn mutated_placement_attrs_survive_hwp5_roundtrip() {
    let mut doc = load_sample();
    {
        let para = doc
            .document_mut()
            .sections
            .iter_mut()
            .flat_map(|s| s.paragraphs.iter_mut())
            .find(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
            .expect("표 문단");
        let table = para
            .controls
            .iter_mut()
            .find_map(|c| match c {
                Control::Table(t) => Some(t),
                _ => None,
            })
            .expect("표");
        table.common.treat_as_char = false;
        table.common.text_wrap = TextWrap::Square;
        table.common.vert_rel_to = VertRelTo::Para;
        table.common.vertical_offset = 1904;
    }

    let bytes = doc.export_hwp().expect("HWP5 저장");
    let rt = HwpDocument::from_bytes(&bytes).expect("재파싱");
    let common = first_table_common(&rt);
    assert!(
        !common.treat_as_char
            && matches!(common.text_wrap, TextWrap::Square)
            && matches!(common.vert_rel_to, VertRelTo::Para)
            && common.vertical_offset == 1904,
        "HWP5 저장이 IR 배치 수정을 유실했다: tac={} wrap={:?} vrel={:?} voff={}",
        common.treat_as_char,
        common.text_wrap,
        common.vert_rel_to,
        common.vertical_offset
    );
}

#[test]
fn unmodified_roundtrip_keeps_original_placement_attrs() {
    // 무수정 왕복 — 병합이 원본 의미를 바꾸지 않는다(raw ↔ pack 항등 확인).
    let mut doc = load_sample();
    let before = first_table_common(&doc).clone();
    let _ = &mut doc;
    let bytes = doc.export_hwp().expect("HWP5 저장");
    let rt = HwpDocument::from_bytes(&bytes).expect("재파싱");
    let after = first_table_common(&rt);
    assert_eq!(before.attr, after.attr, "무수정 attr 이 변형됐다");
    assert_eq!(before.treat_as_char, after.treat_as_char);
    assert_eq!(before.text_wrap, after.text_wrap);
    assert_eq!(before.vert_rel_to, after.vert_rel_to);
    assert_eq!(before.vertical_offset, after.vertical_offset);
}
