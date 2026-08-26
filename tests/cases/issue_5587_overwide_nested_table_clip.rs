//! Issue #5587: 부모 셀보다 넓게 저장된 중첩표가 부모 밖으로 삐져나오지 않는다.
//!
//! `samples/basic/issue1994_behindtext_table_20200830.hwp` p1 의 셀(34,161HU)
//! 안에는 35,144HU 로 저장된 TAC 중첩표가 있다. 정답지
//! `samples/issue1994/issue_1994.pdf`(Hancom PDF 1.3.0.550) 1쪽에는 그 셀의
//! 오른쪽 경계(384.15pt = 512.2px @96dpi) 너머로 그려진 stroke·clip·glyph 가
//! 하나도 없다 — 중첩표 선언 폭의 오른쪽 끝(396.8pt)에는 아무것도 없다.
//!
//! `extend_clipped_cell_horizontal_clip_to_nested_table_borders` 는 42065
//! (#2007) 처럼 **저장 폭이 부모 셀 이하**인 중첩표가 셀 왼쪽 패딩만큼 밀려
//! 오른쪽 테두리를 clip 밖에 두는 경우를 위해 host clip 을 넓힌다. 부모보다
//! 넓게 저장된 표는 그 예외가 아니므로 host viewport 를 그대로 둔다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/basic/issue1994_behindtext_table_20200830.hwp";

/// 직접 자식 표가 자기보다 넓은 clipped host cell 을 모두 모은다.
fn over_wide_nested_hosts<'a>(
    node: &'a RenderNode,
    found: &mut Vec<(&'a RenderNode, &'a RenderNode)>,
) {
    if let RenderNodeType::TableCell(meta) = &node.node_type {
        if meta.clip {
            for child in &node.children {
                if matches!(child.node_type, RenderNodeType::Table(_))
                    && child.bbox.width > node.bbox.width + 0.5
                {
                    found.push((node, child));
                }
            }
        }
    }
    for child in &node.children {
        over_wide_nested_hosts(child, found);
    }
}

#[test]
fn issue_5587_over_wide_nested_table_keeps_host_cell_clip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5587 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5587 fixture");

    let page = core.build_page_render_tree(0).expect("render p1");
    let mut hosts = Vec::new();
    over_wide_nested_hosts(&page.root, &mut hosts);
    assert_eq!(
        hosts.len(),
        1,
        "p1 에는 부모 셀보다 넓게 저장된 중첩표 host cell 이 하나여야 한다"
    );

    let (host, nested) = hosts[0];
    let host_right = host.bbox.x + host.bbox.width;
    let nested_right = nested.bbox.x + nested.bbox.width;

    // 전제: 중첩표는 실제로 부모 셀 오른쪽 너머까지 선언돼 있다.
    assert!(
        nested_right > host_right + 1.0,
        "전제 붕괴 — 중첩표가 부모 셀을 넘지 않는다: host_right={host_right:.2}, \
         nested_right={nested_right:.2}"
    );

    // 한컴 정답지의 오른쪽 paint 한계 512.2px(384.15pt). host clip 이 중첩표의
    // 선언 폭(529.1px)까지 넓어지면 그 밖까지 그려진다.
    assert!(
        (host_right - 512.2).abs() <= 1.5,
        "host cell clip 오른쪽이 한컴 정답지 경계(512.2px)를 벗어났다: {host_right:.2}"
    );
}
