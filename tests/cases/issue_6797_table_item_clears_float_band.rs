//! [Issue #6797] **표 항목**이 앞 문단의 자리차지 밴드를 통과해 두 표가
//! **633.1 × 113.4px** 겹쳤다.
//!
//! 렌더의 배제 밴드 소비 블록은 `item_is_paragraph` 전용이라, 빈 host 에 표만 달린
//! 항목(`PageItem::Table`)은 앞 문단 float 표의 밴드를 그대로 지나간다.
//!
//! `156160455` 7쪽 실측 — 저장 사다리는 겹치지 않는다.
//!
//! ```text
//!   문단 0.70  글자 host       ls[0] vpos=3252  ls[1] vpos=5204
//!              표 3x4  vert=문단(4764 = +63.5px)  633.1 x 113.4px
//!   문단 0.71  빈 문단         ls[0] vpos=16306  ( = 217.4px )
//!              표 1x2  vert=문단(0)               635.0 x 165.7px
//!
//!   사다리(단 기준)   pi=70 표 106.9..220.3      pi=71 앵커 217.4  ← 표 바닥
//!   rhwp(수정 전)     pi=70 표 181.5..294.9      pi=71 표   174.8  ← -118.2px
//! ```
//!
//! ⚠⚠ **시작점만 보면 안 된다** — `pi=71` 의 표는 밴드 **위**(174.8)에서 시작해
//! 밴드(181.5..294.9)를 **가로지른다**. `starts_in_zone` 만으로는 안 잡힌다.
//! `#6764` 와 같은 교훈이라 자기 높이로 밴드를 넘는지 함께 본다.
//!
//! 결과: 겹침 1 → **0**, `pi=71` 표가 `296.8` (앞 표 바닥 294.9 바로 아래)로 이동.
//! 쪽수 11 유지.
//!
//! ⚠ 이 문서는 `hancom-office-2010` 저장본이라 한/글 2024 오라클과 드리프트가 있다.
//! **판정은 저장 사다리**가 준다 — 겹침 자체가 버전과 무관한 결함이다.
//! 남는 `overflow` 8건은 이 축과 무관하다(수정 전후 동일).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6797/156160455-social-pig-farm-income.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6797 정식 HWP fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

/// 단 바로 아래의 표 상자들(중첩 표는 제외).
fn top_level_tables(column: &RenderNode) -> Vec<(f64, f64)> {
    column
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::Table { .. }))
        .map(|c| (c.bbox.y, c.bbox.y + c.bbox.height))
        .collect()
}

/// 7쪽의 표들이 세로로 겹치면 안 된다.
///
/// 수정 전: `pi=70` 표 `181.5..294.9` 와 `pi=71` 표 `174.8..340.5` 가 113.4px 겹쳤다.
#[test]
fn table_item_clears_the_previous_float_band() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 11, "쪽수 핀 — 본 수정은 조판 불변");

    let tree = core.build_page_render_tree(6).expect("7쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");
    let column = body
        .children
        .iter()
        .find(|c| matches!(c.node_type, RenderNodeType::Column(_)))
        .expect("Column 노드");

    let mut boxes = top_level_tables(column);
    assert!(
        boxes.len() >= 2,
        "7쪽에 표가 둘 이상 있어야 한다: {boxes:?}"
    );
    boxes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut worst = 0.0f64;
    for pair in boxes.windows(2) {
        let overlap = pair[0].1 - pair[1].0;
        worst = worst.max(overlap);
    }

    assert!(
        worst <= 0.5,
        "7쪽의 표들이 세로로 겹치면 안 된다 — #6797 회귀          \
         (겹침 {worst:.1}px, 표 {boxes:?}; 수정 전 113.4px)"
    );
}
