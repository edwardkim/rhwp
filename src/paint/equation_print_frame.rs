//! 원본 HFT 수식에도 본문 글리프와 같은 물리 Print 프레임을 연결한다.

use std::collections::HashSet;

use super::{CacheHint, GroupKind, LayerAffineTransform, LayerNode, LayerNodeKind, PaintOp};
use crate::renderer::render_tree::{GroupNode, NodeId};

pub(super) fn publish(
    node: &mut LayerNode,
    sources: &HashSet<NodeId>,
    transform: LayerAffineTransform,
) {
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                publish(child, sources, transform);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            publish(child, sources, transform);
        }
        LayerNodeKind::Leaf { ops } => {
            // 채우기 뒤에는 PDF/DC의 반올림 상태만 달라진다. 원본 수식의
            // 자연 글줄 기준선은 그 상태와 무관하게 같은 소유자가 발행한다.
            for op in ops.iter_mut() {
                if let PaintOp::Equation { equation, .. } = op {
                    crate::renderer::equation::publish_original_hft_print_line(equation);
                }
            }
            if !node.source_node_id.is_some_and(|id| sources.contains(&id)) {
                return;
            }
            let children = std::mem::take(ops)
                .into_iter()
                .map(|op| {
                    let equation = matches!(&op, PaintOp::Equation { equation, .. }
                        if crate::renderer::equation::uses_original_hft_glyphs(&equation.layout_box));
                    let child = LayerNode::leaf(node.bounds, None, vec![op]);
                    if !equation {
                        return child;
                    }
                    // 앞에서 확정한 표시 글줄·원본 bbox·글자 크기는 변환에서 바꾸지
                    // 않는다. 공통 subtree transform으로 물리 종이에 한 번만 투영한다.
                    LayerNode::group(
                        node.bounds,
                        None,
                        vec![child],
                        CacheHint::None,
                        GroupKind::Group(GroupNode {
                            paint_transform: Some([
                                transform.a,
                                transform.b,
                                transform.c,
                                transform.d,
                                transform.e,
                                transform.f,
                            ]),
                            section_index: None,
                            para_index: None,
                            control_index: None,
                        }),
                    )
                })
                .collect();
            node.kind = LayerNodeKind::Group {
                children,
                cache_hint: CacheHint::None,
                group_kind: GroupKind::Generic,
            };
        }
    }
}
