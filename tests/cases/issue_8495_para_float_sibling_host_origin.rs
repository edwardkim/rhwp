//! 관계형·고정형 — [#8495] 자리차지 표와 같은 문단에 있는 문단 기준 그림은 표가 확정한 host 줄 원점(첫 줄 위 − 문단 앞 간격)에서 세로 오프셋만큼 내려간다.
//! 주석 갱신: 2026-11-21 23:10 KST (새 파일)
//!
//! 입력: 구례군 공개 자치법규 별표(「[별표 2] 의회 의원 배지 제식」, HWP 5.0.0.6, 1쪽)
//! `samples/issue8495/council-badge-para-float-sibling-origin.hwp`.
//! 기준: 한컴 Print PDF `pdf/issue8495/council-badge-para-float-sibling-origin-2020.pdf`(1쪽).
//!
//! 문단 0.4(「 금색」, 문단 앞 간격 0)는 문단 기준 자리차지 표(3×2)와 문단 기준 그림 둘(위아래
//! 배치 세로 1946HU, 어울림 세로 1442HU)을 함께 가진다. 앞 문단에는 문단 뒤 간격 568HU 가 있다.
//! 수정 전에는 표가 문단 원점을 앞 문단의 문단 뒤 간격을 더하기 전 흐름 위치로 남겨, 그림 둘과
//! 치수선이 한/글보다 7.57px(568HU) 위에 그려졌다. 한/글의 문단 기준 원점은 첫 저장 줄 위에서
//! 그 문단의 문단 앞 간격을 뺀 자리다(문단 앞 간격 1200HU 인 다른 공개 문서의 Print PDF 에서
//! 그림 위 197.35px = 줄 위 165.92px − 16px + 오프셋 47.65px 로 확인).
//! - 고정형: 전체 쪽수 1(= PDF).
//! - 관계형: 그림 윗변 − 「금색」 줄 윗변 = 그림의 세로 오프셋(문단 앞 간격 0).
//! - 관계형(반례, 메모리 편집): 같은 저장 줄에 문단 앞 간격을 주면 그림 윗변 − 줄 윗변 =
//!   세로 오프셋 − 문단 앞 간격.
//!
//! 측정(2026-11-21, 이 파일과 같은 커밋의 코드, base = upstream/devel `d7b5286cbf`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 1쪽 / 1쪽, 1쪽 99.76(base 88.32), 전쪽 최저 99.76(1쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 1쪽 / 1쪽, 1쪽 99.94, 전쪽 최저 99.94(1쪽)
//! - 증거: `mydocs/pr/assets/issue_8495/`(native·wasm 전쪽 TSV, 1쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue8495/council-badge-para-float-sibling-origin.hwp";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 1;
/// 표와 그림 둘을 함께 가진 문단.
const HOST: usize = 4;

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn px(hu: i64) -> f64 {
    rhwp::renderer::hwpunit_to_px(hu as i32, rhwp::renderer::DEFAULT_DPI)
}

fn host_line_top(node: &RenderNode) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.para_index == Some(HOST) && line.line_index == Some(0))
    {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(host_line_top)
}

fn picture_top(node: &RenderNode, control: usize) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::Image(image)
        if image.para_index == Some(HOST) && image.control_index == Some(control))
    {
        return Some(node.bbox.y);
    }
    node.children
        .iter()
        .find_map(|child| picture_top(child, control))
}

/// 문단 기준 그림의 (control index, 세로 오프셋 px).
fn para_relative_pictures(core: &DocumentCore) -> Vec<(usize, f64)> {
    let host = &core.document().sections[0].paragraphs[HOST];
    assert!(
        host.controls
            .iter()
            .any(|control| matches!(control, Control::Table(_))),
        "host 문단은 자리차지 표를 가진다"
    );
    let pictures: Vec<_> = host
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| match control {
            Control::Picture(picture) => {
                Some((index, px(i64::from(picture.common.vertical_offset))))
            }
            _ => None,
        })
        .collect();
    assert_eq!(pictures.len(), 2, "host 문단의 문단 기준 그림 둘");
    pictures
}

/// 문단 앞 간격(px). HWP ParaShape 간격은 실제 HWPUNIT 의 두 배로 저장된다.
fn spacing_before(core: &DocumentCore) -> f64 {
    let doc = core.document();
    let shape = doc.sections[0].paragraphs[HOST].para_shape_id as usize;
    px(i64::from(doc.doc_info.para_shapes[shape].spacing_before) / 2)
}

fn assert_pictures_hang_from_host_origin(core: &DocumentCore, label: &str) {
    let tree = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let line = host_line_top(&tree.root).expect("host 문단 첫 줄");
    let before = spacing_before(core);
    for (control, offset) in para_relative_pictures(core) {
        let top = picture_top(&tree.root, control).expect("host 문단의 그림");
        assert!(
            (top - line - (offset - before)).abs() < 0.5,
            "{label}: 그림(ci={control}) 윗변 {top:.2}px 은 줄 위 {line:.2}px − 문단 앞 간격 \
             {before:.2}px + 세로 오프셋 {offset:.2}px 이어야 한다"
        );
    }
}

#[test]
fn sibling_pictures_hang_from_the_stored_host_line_origin() {
    let core = load();
    assert_eq!(core.page_count(), PDF_PAGES, "한/글 Print 와 같은 쪽수");
    assert_eq!(
        spacing_before(&core),
        0.0,
        "원본 host 문단의 문단 앞 간격은 0"
    );
    assert_pictures_hang_from_host_origin(&core, "원본");
}

#[test]
fn spacing_before_of_the_host_paragraph_lifts_the_shared_origin() {
    let mut core = load();
    let mut source = core.document().clone();
    let shape = source.sections[0].paragraphs[HOST].para_shape_id as usize;
    let mut spaced = source.doc_info.para_shapes[shape].clone();
    // 실제 1200HU(16px). 저장값은 두 배다.
    spaced.spacing_before = 2400;
    source.doc_info.para_shapes.push(spaced);
    source.sections[0].paragraphs[HOST].para_shape_id =
        (source.doc_info.para_shapes.len() - 1) as u16;
    core.set_document(source);
    assert_eq!(core.page_count(), PDF_PAGES, "편집 뒤에도 한 쪽");
    assert!((spacing_before(&core) - 16.0).abs() < 0.01);
    assert_pictures_hang_from_host_origin(&core, "문단 앞 간격 편집");
}
