//! 관계형 검사. 글자가 없는 펼침·포갬 상태의 너비와 캐럿 소유를 검사한다.
//! 주석 갱신 2026-11-21T15:21:50+09:00. 직전 파일 변경 b6922ffed647ec34f0789da0dc329ef9e11cee9c 2026-11-21T02:24:03+09:00.
//! 2026-11-21 한컴 2020 독립 인쇄본과 96 DPI에서 색상 경계 보정 전 2px 실루엣을 측정했다.
//! 아래는 쪽 순서대로 Native Skia와 fresh WASM Canvas2D의 raw 일치율(%)이다.
//! `overlap-false.hwpx` 1쪽. Native [100.00000], WASM [100.00000].
//! `overlap-true.hwpx` 1쪽. Native [100.00000], WASM [100.00000].
//! 이 범위의 전쪽 최저는 Native 100.00000%, WASM 100.00000%다.
//! 입력·PDF 해시와 쪽별 근거는 `mydocs/pr/assets/semanticist21-20261121/empty-overlap/pages.tsv`에 있다.
//!
//! 한/글 12.30 Mac에서 `앞[빈 겹침]뒤끝`(HWP·HWPX)을 열면 포갠 겹침은 폭 없이 캐럿 한 칸만 두고,
//! 펼친 겹침은 전각 한 칸을 비운다. 절대 배치 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// `앞뒤끝`의 `뒤` 앞에 글자 없는 테두리 없는 겹침을 둔다. overlap이 참이면 포갠 겹침이다.
fn fixture(overlap: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞뒤끝").unwrap();
    core.insert_char_overlap_native(
        0,
        0,
        1,
        &format!(r#"{{"text":"가","shape":0,"overlap":{overlap}}}"#),
    )
    .unwrap();
    let mut doc = core.document().clone();
    for control in &mut doc.sections[0].paragraphs[0].controls {
        if let Control::CharOverlap(co) = control {
            co.chars.clear();
            co.char_shape_ids.truncate(1);
        }
    }
    core.set_document(doc);
    core
}

/// 첫 문단 글자 run의 (글, 왼쪽 x)와 겹침 run의 폭.
fn runs(node: &RenderNode, text: &mut Vec<(String, f64)>, overlap: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(0) {
            if run.char_overlap.is_some() {
                overlap.push(node.bbox.width);
            } else if !run.text.is_empty() {
                text.push((run.text.clone(), node.bbox.x));
            }
        }
    }
    for child in &node.children {
        runs(child, text, overlap);
    }
}

fn layout(core: &DocumentCore) -> (f64, f64) {
    let tree = core.build_page_render_tree(0).unwrap();
    let (mut text, mut overlap) = (Vec::new(), Vec::new());
    runs(&tree.root, &mut text, &mut overlap);
    assert_eq!(overlap.len(), 1, "{text:?}");
    let x = |needle: &str| {
        text.iter()
            .find(|(t, _)| t.starts_with(needle))
            .map(|(_, x)| *x)
            .unwrap_or_else(|| panic!("{needle}: {text:?}"))
    };
    (overlap[0], x("뒤") - x("앞"))
}

#[test]
fn empty_borderless_overlap_has_no_width() {
    let core = fixture(true);
    let (width, gap) = layout(&core);
    assert_eq!(width, 0.0);
    // `뒤`는 `앞` 한 글자 폭만큼만 뒤에 선다.
    let (_, spread_gap) = layout(&fixture(false));
    assert!(spread_gap > gap + 5.0, "{spread_gap} / {gap}");
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let saved = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(layout(&saved).0, 0.0);
    }
}

#[test]
fn empty_borderless_spread_overlap_keeps_a_full_width_cell() {
    let core = fixture(false);
    let (width, _) = layout(&core);
    let size = 10.0 * 96.0 / 72.0;
    assert!((width - size).abs() < 1.0, "{width}");
}
