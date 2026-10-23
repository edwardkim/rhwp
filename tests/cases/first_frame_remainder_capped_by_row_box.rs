//! 첫 분할 행의 이어받기 물리 잔여는 선언 행 상자를 넘지 않는다.
//!
//! `stored_first_frame_row_remainder`(#6803 계열, 108edfbf8)는 원본 첫 개체 프레임에서 끊긴
//! 행의 남은 **물리** 높이를 다음 조각 시작 행에 넘긴다. 측정 행 높이가 선언 상자보다 큰
//! 행에서는 그 초과분이 내용(마지막 줄의 줄간격 포함)이 만든 높이라 빈 물리 띠가 아니다.
//! 그 초과분까지 넘기면 이어받기 상자가 남은 내용보다 한 줄간격만큼 길어진다.
//!
//! 독립 근거: `samples/80168_regulatory_analysis.hwp` 5쪽(`pdf/80168_regulatory_analysis-hwp-2024.pdf`
//! 5쪽, 한/글 2024)은 2행 표의 행 1 이어받기를 마지막 글줄 「군수등이 정하여 고시한다.」 바로
//! 아래에서 닫는다. 수정 전 devel 은 상자 바닥을 그 글줄 아래 한 줄간격 가까이 더 내렸다
//! (Visual Sweep 5쪽 100.0 → 83.55, 수정 후 100.0; 4·6쪽 99.60·99.91 로 무변화).
//!
//! 검사는 픽셀 위치가 아니라 관계다: 이어받기 칸 상자 바닥과 그 칸 마지막 글줄 바닥 사이의
//! 빈 띠가 그 칸의 아래 안 여백과 한 줄간격(1120HU)의 절반을 넘지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/80168_regulatory_analysis.hwp";
/// 0 기준 쪽 번호 — PDF 5쪽.
const PAGE: u32 = 4;
const LAST_LINE: &str = "고시한다";

fn walk<'a>(
    node: &'a RenderNode,
    path: &mut Vec<&'a RenderNode>,
    out: &mut Vec<Vec<&'a RenderNode>>,
) {
    path.push(node);
    out.push(path.clone());
    for child in &node.children {
        walk(child, path, out);
    }
    path.pop();
}

fn text_of(node: &RenderNode) -> String {
    let mut text = String::new();
    fn collect(node: &RenderNode, text: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            text.push_str(&run.text);
        }
        for child in &node.children {
            collect(child, text);
        }
    }
    collect(node, &mut text);
    text
}

fn bottom(bbox: &BoundingBox) -> f64 {
    bbox.y + bbox.height
}

#[test]
fn continued_first_frame_row_closes_at_its_remaining_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let tree = core.build_page_render_tree(PAGE).expect("render tree");
    let mut paths = Vec::new();
    walk(&tree.root, &mut Vec::new(), &mut paths);

    // 마지막 글줄과 그 글줄을 담은 칸.
    let line_path = paths
        .iter()
        .find(|path| {
            let node = path.last().unwrap();
            matches!(node.node_type, RenderNodeType::TextLine(_))
                && text_of(node).contains(LAST_LINE)
        })
        .expect("이어받기 칸의 마지막 글줄");
    let line = line_path.last().unwrap();
    let cell = line_path
        .iter()
        .rev()
        .find(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .expect("글줄을 담은 칸");

    let blank_band = bottom(&cell.bbox) - bottom(&line.bbox);
    // 아래 안 여백 141HU(1.88px) + 줄간격 1120HU(14.93px)의 절반.
    let allowance = 141.0 / 75.0 + 1120.0 / 75.0 / 2.0;
    assert!(
        blank_band <= allowance,
        "이어받기 칸이 마지막 글줄 아래로 {blank_band:.2}px 를 더 차지한다(허용 {allowance:.2}px)"
    );
    assert!(
        blank_band >= 0.0,
        "글줄이 칸 밖으로 나갔다: {blank_band:.2}px"
    );
}
