//! [#6175 실측] 어울림(Square) 개체와 **같은 y 에서 시작하는** 문단의 배제 밴드.
//!
//! 156518601(모바일 운전면허증 보도자료) 1쪽 — `경찰청(청장 후보자 윤희근)은 …`
//! 문단(pi=8)의 저장 사다리는 네 줄 전부 `horzsize=29138`(=388.5px)이다. 이는
//! 묶음 그림(`horzOffset=29138`)이 만든 밴드의 왼쪽 레인 폭이다.
//!
//! 수정 전 rhwp 는 이 문단만 전폭(48188HU=642.5px)으로 다시 흘려 세 줄로 만들었고,
//! 그 세 줄의 471.6~718.1 구간이 그림에 덮여 사라졌다. 원인은 두 곳이다.
//!
//! 1. 저장 줄 전부가 **같은 폭**으로 좁아진 문단은 "폭이 섞였다"는 국소 증거가
//!    없어 프레임이 관할을 주장하고 전폭으로 재래핑했다.
//! 2. 밴드 arming 이 호스트 문단의 저장 사다리에만 기대는데, 이 문서의 호스트
//!    문단(pi=7)의 줄은 전폭이다 — 개체가 그 줄보다 아래에서 시작한다.
//!
//! 한글 2020 오라클(`pdf/pr6017/156518601_p1_square_host-2020.pdf`)의 같은 문단은
//! 네 줄이고 오른쪽 끝이 347.8pt(=463.7px)다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5809/156518601_p1_square_host.hwpx";

/// 묶음 그림의 왼쪽 모서리(px) — 본문 좌단 75.6 + horzOffset 29138HU(388.5px).
const BAND_LEFT_PX: f64 = 471.6;

fn line_text(node: &RenderNode) -> String {
    node.children
        .iter()
        .filter_map(|c| match &c.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

/// 첫 줄이 `prefix` 로 시작하는 문단의 모든 줄을 문서 순서대로 모은다.
fn collect_lines(node: &RenderNode, out: &mut Vec<(f64, f64, f64, String)>) {
    if let RenderNodeType::TextLine(_) = &node.node_type {
        out.push((
            node.bbox.x,
            node.bbox.y,
            node.bbox.width,
            line_text(node).trim().to_string(),
        ));
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

#[test]
fn issue_6175_uniformly_narrowed_ladder_keeps_square_band() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #6175 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #6175 fixture");
    let page = core.build_page_render_tree(0).expect("render p1");

    let mut lines = Vec::new();
    collect_lines(&page.root, &mut lines);

    let start = lines
        .iter()
        .position(|(_, _, _, text)| text.starts_with("경찰청(청장"))
        .expect("`경찰청(청장…` 문단 첫 줄");
    let end = lines
        .iter()
        .position(|(_, _, _, text)| text.starts_with("일제히 발급한다고"))
        .expect("`일제히 발급한다고 밝혔다.` 줄 — 수정 전엔 3줄로 뭉쳐 존재하지 않았다");
    assert_eq!(
        end - start + 1,
        4,
        "한글 오라클과 저장 사다리 모두 네 줄이다: {:?}",
        &lines[start..=end]
    );

    for (x, y, width, text) in &lines[start..=end] {
        let right = x + width;
        assert!(
            right <= BAND_LEFT_PX + 1.0,
            "줄 우단 {right:.1} 이 밴드 좌단 {BAND_LEFT_PX} 을 넘는다 \
             (y={y:.1}, 수정 전 718.1) — 그림에 가려진다: {text}"
        );
    }
}
