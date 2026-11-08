//! #6535: 하단 쪽 앵커의 본문 끝은 활성 단의 저장 좌표 기준으로 복원한다.
//!
//! 원본과 독립 한컴 PDF는 같은 문서의 1쪽이다. 이전 검사 이관 시 본문은
//! 첫 쪽에 맞았지만 하단 틀만 둘째 쪽으로 밀렸다. 활성 lazy 기준을 누락하여
//! 저장 본문 끝을 원시 구역 좌표로 사용한 것이 원인이었다.
//! 전쪽 Native/fresh WASM 시각 검증 후 쪽 소속과 본문·서명·연락처 관계를 검사한다.
//! 내용의 누락·중복과 소속 하단 표는 maintainer_hwpx_overlay_table_extent의
//! 같은 원본 검사에서 확인하며 실측 픽셀 위치는 기대값으로 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue6535/36339092_low_slack_absorb_block.hwpx";
/// 한/글과 같은 쪽수. 회귀 시에는 블록이 2쪽으로 밀려 본문 없는 쪽이 생긴다.
const EXPECTED_PAGES: u32 = 1;

#[test]
fn low_slack_anchor_block_is_absorbed_into_the_same_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = DocumentCore::from_bytes(&bytes).expect("저슬랙 원본 문서 로드");

    let pages = document.page_count();
    assert_eq!(
        pages, EXPECTED_PAGES,
        "독립 PDF와 달리 하단 쪽 앵커가 별도 쪽으로 밀렸다 — 실측 {pages}쪽. \
         저장 본문 끝과 현재 단의 흐름을 같은 좌표 기준으로 판정해야 한다"
    );

    // 독립 PDF에서 본문, 시장 서명, 연락처는 같은 쪽에 이 순서로 표시된다.
    // 쪽수만 맞추고 본문이나 하단 틀을 누락하는 출력도 회귀로 판정한다.
    let tree = document.build_page_render_tree(0).expect("원본 첫 쪽");
    fn collect<'a>(node: &'a RenderNode, text: &str, hits: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.display_or_text().contains(text))
        {
            hits.push(node);
        }
        for child in &node.children {
            collect(child, text, hits);
        }
    }
    let find = |text: &str| {
        let mut hits = Vec::new();
        collect(&tree.root, text, &mut hits);
        assert_eq!(hits.len(), 1, "같은 쪽 내용의 누락·중복: {text}");
        hits[0]
    };
    let body = find("우리 시 기후환경 정책에");
    let signature = find("서 울 특 별 시 장");
    let phone = find("02-2133-9778");
    let fax = find("02-2133-1021");
    assert!(
        body.bbox.y + body.bbox.height <= signature.bbox.y,
        "본문 뒤에 시장 서명이 표시된다"
    );
    assert!(
        signature.bbox.y + signature.bbox.height <= phone.bbox.y,
        "시장 서명 뒤에 연락처가 표시된다"
    );
    assert!(
        phone.bbox.x + phone.bbox.width <= fax.bbox.x,
        "전화와 전송 번호가 같은 줄에서 순서대로 표시된다"
    );
    assert!(
        phone.bbox.y < fax.bbox.y + fax.bbox.height
            && fax.bbox.y < phone.bbox.y + phone.bbox.height,
        "전화와 전송 번호의 글줄이 분리되지 않는다"
    );
}
