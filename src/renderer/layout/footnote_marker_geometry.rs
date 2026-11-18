//! 발행한 각주 번호의 실제 문자 경계로 뒤 개체의 가로 원점을 정한다.

use crate::renderer::render_tree::{RenderNode, RenderNodeType};

pub(super) fn number_right(node: &RenderNode, marker: &str) -> Option<f64> {
    if marker.is_empty() {
        return None;
    }
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => {
            let display = run.display_or_text();
            display.find(marker).and_then(|offset| {
                // 번호 앞의 서식 공백은 허용하지만 본문 안에 우연히 등장한
                // 같은 숫자를 별도의 각주 번호로 취급하지 않는다.
                if !display[..offset].chars().all(char::is_whitespace) {
                    return None;
                }
                let end = display[..offset + marker.len()].chars().count();
                // paint와 같은 확정 문자 경계를 소비한다. 비례 글꼴의 숫자·
                // 괄호·공백 폭을 문자열 길이의 비율로 추정하지 않는다.
                run.replay_positions_for(display)
                    .get(end)
                    .copied()
                    .map(|advance| node.bbox.x + advance)
                    .filter(|right| right.is_finite())
            })
        }
        _ => None,
    };
    node.children
        .iter()
        .filter_map(|child| number_right(child, marker))
        .chain(own)
        .reduce(f64::max)
}
