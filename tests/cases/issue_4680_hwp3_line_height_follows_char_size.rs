//! #4680: HWP3 채권·채무 서식의 원본 글자 크기와 HWPX 글줄 소유를 보존한다.
//!
//! 원본의 위치·구분 정보가 없는 LineInfo는 저장 줄 구성의 근거가 아니므로
//! HWPX에 합성 lineseg가 반드시 있다고 가정하지 않는다. 같은 원본·독립 Print PDF의
//! 현재 Native/fresh WASM 전 1쪽은 98.94647%이고, 제목·표·하단 서명 내용을 직접 확인했다.
//! 증적: mydocs/pr/assets/issue7445/completed-20261009/current-base-qualification/.
//! 이전 캐시 수치 대신 원본 글자모양의 em 크기와 실제 글줄 안의 포함 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::RenderNodeType;
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue4680/20117321_gijang_handover_bond_ledger_form.hwp";

/// (원본 글자 크기, 내보낸 글자 크기, 실제 글줄 높이)를 내용 소유가 있는 run에서 모은다.
fn exported_line_metrics() -> Vec<(f64, f64, f64)> {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("HWP3 정식 원본");
    let source = rhwp::parse_document(&bytes).expect("원본 글자모양");
    let hwpx = HwpDocument::from_bytes(&bytes)
        .expect("HWP3 파싱")
        .export_hwpx()
        .expect("HWPX 내보내기");
    let round = DocumentCore::from_bytes(&hwpx).expect("HWPX 다시 열기");
    assert_eq!(round.page_count(), 1, "독립 Print PDF의 단일 서식 쪽");
    let tree = round.build_page_render_tree(0).expect("서식 전체 글줄");
    let mut stack = vec![&tree.root];
    let mut metrics = Vec::new();
    let mut text = String::new();
    while let Some(node) = stack.pop() {
        stack.extend(node.children.iter().rev());
        if let RenderNodeType::TextLine(line) = &node.node_type {
            for child in &node.children {
                let RenderNodeType::TextRun(run) = &child.node_type else {
                    continue;
                };
                if run.text.trim().is_empty() {
                    continue;
                }
                text.push_str(run.display_or_text());
                let id = run.char_shape_id.expect("글자모양 소속") as usize;
                let original = source.doc_info.char_shapes.get(id).expect("원본 글자모양");
                let exported = round
                    .document()
                    .doc_info
                    .char_shapes
                    .get(id)
                    .expect("내보낸 글자모양");
                assert_eq!(
                    exported.base_size, original.base_size,
                    "저장 줄 상자 높이로 원본 글자모양을 바꾸지 않는다"
                );
                metrics.push((
                    hwpunit_to_px(original.base_size, DEFAULT_DPI),
                    run.style.font_size,
                    line.line_height,
                ));
                assert!(
                    child.bbox.y >= node.bbox.y
                        && child.bbox.y + child.bbox.height
                            <= node.bbox.y + node.bbox.height + line.line_height * 1e-9,
                    "실제 글자 상자는 자신이 소속된 글줄 안에 있다"
                );
            }
        }
    }
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    for label in [
        "채권",
        "채무조서",
        "실과별",
        "건명",
        "금액",
        "채무자",
        "확인자",
        "성명",
    ] {
        assert!(compact.contains(label), "서식 내용 누락: {label}");
    }
    metrics
}

#[test]
fn exported_text_keeps_the_source_font_size_and_line_owner() {
    let metrics = exported_line_metrics();
    assert!(!metrics.is_empty(), "서식의 실제 글자 run을 검사한다");
    for (source_em, exported_em, _) in metrics {
        assert!(
            (exported_em - source_em).abs() <= source_em * 1e-9,
            "글자 크기는 원본 em이며 저장 줄 상자를 글자 크기로 쓰지 않는다"
        );
    }
}

#[test]
fn the_exported_line_contains_its_source_glyph_box() {
    let metrics = exported_line_metrics();
    assert!(
        !metrics.is_empty(),
        "빈 지표 집합을 반례 통과로 취급하지 않는다"
    );
    for (source_em, _, line_height) in metrics {
        assert!(
            line_height + source_em * 1e-9 >= source_em,
            "원본 글자보다 작은 글줄을 만들어 글자를 겹치지 않는다"
        );
    }
}
