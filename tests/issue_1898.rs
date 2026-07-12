//! Issue #1898 기전 1 — tac(글자처럼) 인라인 그림 문단의 렌더 줄 전진 과대.
//!
//! 36388711 p9 의 불릿(▷) 리스트는 각 문단이 2.5mm tac 그림을 갖는다. 종전에는
//! tac 그림의 Shape 페이지 항목이 vpos 기준점(page/lazy base)을 초기화해, 다음
//! 문단의 vpos_adjust 가 lazy_base 재산출에서 trailing-ls bridge 를 다시 적용
//! → 그림 문단마다 렌더 y 가 줄간격 1회분(+11.7px) 과대 전진했다
//! (layout 33.1px/줄 vs 렌더 44.8px/줄; 한컴 오라클 32.9px — layout 이 정답).
//! tac 개체는 호스트 LINE_SEG 에 통합되어 흐름에 독립 높이를 만들지 않으므로
//! 기준점을 초기화하지 않는다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str =
    "samples/hwpx/opengov/36388711_사회보장제도 신설 협의요청서(청년오피스)_260624.hwpx";

/// p9 (0-based 8) 의 pi=95/96/97 참고자료 리스트: TextLine 간 y 간격이 layout
/// (lh+ls+sb = 33.1px)과 일치해야 한다. 종전 44.8px (= +ls 이중 가산).
#[test]
fn issue_1898_tac_picture_line_advance_matches_layout() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let data = fs::read(Path::new(repo_root).join(SAMPLE)).unwrap_or_else(|e| panic!("read: {e}"));
    let core = DocumentCore::from_bytes(&data).expect("load");
    let tree = core.build_page_render_tree(8).expect("render p9");

    // pi=95..=97 의 TextLine y 수집 (본문 컬럼)
    fn collect(node: &RenderNode, out: &mut Vec<(usize, f64)>) {
        if let RenderNodeType::TextLine(ref tl) = node.node_type {
            if let Some(pi) = tl.para_index {
                if (95..=97).contains(&pi) {
                    out.push((pi, node.bbox.y));
                }
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let mut ys = Vec::new();
    collect(&tree.root, &mut ys);
    ys.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.partial_cmp(&b.1).unwrap()));

    let y95 = ys.iter().find(|(pi, _)| *pi == 95).map(|(_, y)| *y);
    let y96 = ys.iter().find(|(pi, _)| *pi == 96).map(|(_, y)| *y);
    let y97 = ys.iter().find(|(pi, _)| *pi == 97).map(|(_, y)| *y);
    let (Some(y95), Some(y96), Some(y97)) = (y95, y96, y97) else {
        panic!("pi=95/96/97 TextLine 미발견: {ys:?}");
    };

    // layout 정답 = lh(1100)+ls(880)+sb(500) = 2480 HU = 33.07px (한컴 32.9px).
    // 종전 결함 값 = 44.8px. 허용 오차 ±1.5px.
    for (label, delta) in [("95→96", y96 - y95), ("96→97", y97 - y96)] {
        assert!(
            (delta - 33.1).abs() <= 1.5,
            "{label} TextLine 간격 {delta:.1}px ≠ 33.1px (tac 그림 vpos 기준점 초기화 → \
             trailing-ls bridge 이중 가산 회귀: 종전 44.8px)"
        );
    }
}
