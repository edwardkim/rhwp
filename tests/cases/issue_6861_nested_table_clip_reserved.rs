//! [#6861] 셀 안 중첩 표의 오른쪽 테두리를 **조상 셀 clip** 이 지운다.
//!
//! `3194097` 1쪽은 바깥 표(1열 2행) 안에 6열 중첩 표가 둘 든 서식이다. 중첩 표가 부모
//! 셀보다 **30.15px 넓게** 저장돼 있고, 한/글은 그대로 넘겨 그린다. rhwp 는 조상 셀
//! clip(우단 720.00)으로 잘라 오른쪽 세로 테두리를 통째로, 가로 괘선의 꼬리 30.15px 를
//! 지웠다.
//!
//! ⭐ **`#5587` 과 정반대 요구다.** 그 이슈는 "부모 셀보다 넓게 저장된 중첩 표는 한글도
//! 부모 경계에서 자른다"로 정리했고, Hancom PDF 로 잠겨 있다. 갈림은 **저장 줄 폭**이
//! 준다 — 호스트 문단의 `LINE_SEG.segment_width` 가 중첩 표의 선언 폭을 품으면 한글이
//! 자리를 잡아 준 것이다.
//!
//! ```text
//!   3194097   sw 50,440 >= 중첩 50,170   → 자리를 잡아 줬다 → 넘겨 그린다
//!   #5587     sw 34,160 <  중첩 35,144   → 안 잡아 줬다     → 부모 경계에서 자른다
//! ```
//!
//! ⚠ 상한은 **용지**다. 사다리가 자리를 잡아 줬어도 용지 밖까지 열지는 않는다 —
//! 상한 없이 켜면 `1480000-201200206` 이 용지 밖 2 → 9 로 악화한다. 본문 우단으로
//! 잡으면 반대로 이 축이 통째로 닫힌다(이 문서의 본문 우단이 720.0 이다).
//!
//! ```text
//!             조상 셀 clip 우단     720 를 넘는 괘선
//!   수정 전        720.00              13개가 지워진다
//!   수정 후        750.91              전부 보인다
//!   정본(2020)   같은 자리에 그린다 (표1 오른쪽 749.7 vs rhwp 749.4)
//! ```
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6861/3194097-performance-rating-roster.hwp";
/// `#5587` 음성 대조 — 사다리가 자리를 안 잡아 준 과폭 중첩 표.
const NEGATIVE: &str = "samples/basic/issue1994_behindtext_table_20200830.hwp";

fn open(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}")))
        .unwrap_or_else(|e| panic!("open {sample}: {e}"))
}

/// clip 이 걸린 셀과 그 **직속 자식 표** 중, 표가 셀보다 넓은 쌍.
fn over_wide_hosts<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a RenderNode)>) {
    if let RenderNodeType::TableCell(meta) = &node.node_type {
        if meta.clip {
            for child in &node.children {
                if matches!(child.node_type, RenderNodeType::Table(_))
                    && child.bbox.width > node.bbox.width + 0.5
                {
                    out.push((node, child));
                }
            }
        }
    }
    for child in &node.children {
        over_wide_hosts(child, out);
    }
}

/// clip 이 걸린 셀과 그 **직속 자식 표** 전부.
///
/// ⚠ `over_wide_hosts` 로는 이 축을 잠글 수 없다 — 수정이 host clip 을 넓히므로
/// **수정 후에는 "표가 셀보다 넓은" 쌍이 사라진다.** 전제가 통과해 버려 시험이 공허해진다.
fn clipped_hosts_with_nested_table<'a>(
    node: &'a RenderNode,
    out: &mut Vec<(&'a RenderNode, &'a RenderNode)>,
) {
    if let RenderNodeType::TableCell(meta) = &node.node_type {
        if meta.clip {
            for child in &node.children {
                if matches!(child.node_type, RenderNodeType::Table(_)) {
                    out.push((node, child));
                }
            }
        }
    }
    for child in &node.children {
        clipped_hosts_with_nested_table(child, out);
    }
}

/// 그 셀 안에서 clip 우단을 넘는 세로 괘선을 센다.
fn vertical_borders_beyond(node: &RenderNode, limit: f64) -> usize {
    let mut count = 0;
    if let RenderNodeType::Line(line) = &node.node_type {
        if (line.x1 - line.x2).abs() < 0.01 && (line.y1 - line.y2).abs() > 1.0 && line.x1 > limit {
            count += 1;
        }
    }
    for child in &node.children {
        count += vertical_borders_beyond(child, limit);
    }
    count
}

/// 이 문서 중첩 표의 오른쪽 테두리 x (선언 폭 50,170HU 기준). 문서 상수다.
const NESTED_RIGHT_PX: f64 = 749.4;

#[test]
fn issue_6861_ladder_reserved_nested_table_border_is_not_clipped() {
    let core = open(SAMPLE);
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut hosts = Vec::new();
    clipped_hosts_with_nested_table(&tree.root, &mut hosts);

    // 전제 — 오른쪽 끝이 749.4 인 중첩 표가 clip 걸린 셀 안에 있다. 이 값은 문서 상수라
    // 수정 전후로 변하지 않는다(수정이 바꾸는 것은 **host clip** 이다).
    let mut checked = 0usize;
    for (host, nested) in hosts {
        let nested_right = nested.bbox.x + nested.bbox.width;
        if (nested_right - NESTED_RIGHT_PX).abs() > 1.0 {
            continue;
        }
        checked += 1;
        let host_right = host.bbox.x + host.bbox.width;
        assert!(
            host_right + 0.5 >= nested_right,
            "조상 셀 clip 이 중첩 표의 오른쪽 테두리를 품어야 한다 — clip 우단              {host_right:.2} < 표 우단 {nested_right:.2} (결함 시 720.00 이라 세로              테두리 통째로, 가로 괘선 꼬리 30.15px 가 지워졌다)"
        );
        assert_eq!(
            vertical_borders_beyond(nested, host_right),
            0,
            "clip 밖에 남는 세로 괘선이 없어야 한다"
        );
    }
    assert!(
        checked >= 1,
        "오른쪽 끝 {NESTED_RIGHT_PX} 인 중첩 표를 하나 이상 찾아야 한다"
    );
}

#[test]
fn issue_6861_negative_unreserved_over_wide_nested_table_keeps_the_clip() {
    // `#5587` 문서 — 저장 줄 폭(34,160)이 중첩 표(35,144)를 못 품는다. 한컴 정답지도
    // 부모 셀 오른쪽(512.2px) 너머에 아무것도 그리지 않는다. 이 수정으로 움직이면 안 된다.
    let core = open(NEGATIVE);
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut hosts = Vec::new();
    over_wide_hosts(&tree.root, &mut hosts);
    assert_eq!(hosts.len(), 1, "그 문서 1쪽의 과폭 host 는 하나다");
    let host_right = hosts[0].0.bbox.x + hosts[0].0.bbox.width;
    assert!(
        (host_right - 512.2).abs() <= 1.5,
        "사다리가 자리를 안 잡아 준 과폭 중첩 표는 부모 clip 을 그대로 써야 한다 \
         (한컴 정답지 경계 512.2px, 실측 {host_right:.2})"
    );
}
