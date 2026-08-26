//! [Issue #5747] 매니페스트 순서와 이름 번호가 어긋난 HWPX 에서 그림이 통째로 뒤바뀐다.
//!
//! `canonicalize_bin_item_refs`(#3460)가 항목마다 전체 XML 을 **순차 문자열 치환**해,
//! 산출 이름공간(`image1…`)이 아직 처리하지 않은 입력 이름과 겹치면 앞 회차가 바꾼
//! 참조를 뒤 회차가 또 바꿨다. 156532835 실측: 참조 19개 중 12개가 다른 BinData 로
//! 착지(코퍼스 HWPX 2,283건 중 34건). 수정: id→위치+1 맵 기반 단일 패스.
//!
//! 픽스처 `samples/issue5747/mismatched_manifest_refs.hwpx` — 매니페스트 선언 순서가
//! `image3, image1, image2` 로 어긋나고 본문 그림이 `image3` 을 참조한다. 각 BinData
//! 는 PNG IEND 뒤에 `MARK{n}` 꼬리로 구별된다.
//!
//! - 정답: `image3` 은 매니페스트 1번째 → 정규화 `image1` → 적재 id=1 =
//!   `BinData/image3.png`(`MARK3`).
//! - 종전 결함: 치환 연쇄(image3→image1→image2→image3)로 `image2.png`(`MARK2`)가 나왔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5747/mismatched_manifest_refs.hwpx";

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_5747_mismatched_manifest_ref_resolves_to_declared_item() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    let images: Vec<&[u8]> = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Image(img) => img.data.as_deref(),
            _ => None,
        })
        .collect();
    assert!(!images.is_empty(), "그림 노드가 있어야 한다");
    for data in images {
        assert!(
            data.ends_with(b"MARK3"),
            "image3 참조는 매니페스트 1번째 항목(BinData/image3.png, MARK3)으로 해석돼야 한다 \
             — 치환 연쇄 결함이면 MARK2 가 나온다. 실제 꼬리: {:?}",
            &data[data.len().saturating_sub(5)..]
        );
    }
}
