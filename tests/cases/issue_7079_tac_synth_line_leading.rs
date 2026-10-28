//! [#7079] 저장 LINE_SEG 없는 TAC(글자처럼) 개체 문단의 합성 줄은 leading 을 개체 **뒤**에
//! 남긴다 — `#7062` 는 줄 높이만 고쳤고 이 leading 이 통째로 빠져 있었다.
//!
//! 저장 사다리가 값을 못박는다. `samples/issue7062/tac_object_host_line_height.hwp`
//! 2쪽 셀: `p[11] vpos=30525 lh=600 ls=0` → `p[13] vpos=61874` 이므로 그림 문단
//! `p[12]` 의 몫은 30749HU 인데 그림은 29997HU 다. 남는 **752HU(10.0px)** 는 그 문단의
//! 글자 20.0px · 줄간격 Percent 150% 에서 나온 `20.0 * 0.5` 다(개체 높이와 무관).
//!
//! 놓이는 자리는 한컴 engine 2020 출력을 **같은 96dpi 래스터로 겹쳐** 정한다(engine 2020,
//! job `94f14dc5-422a-462a-b4e6-810ef36ff98d`, 10쪽, `Hancom PDF 1.3.0.550`). 2쪽 잉크 기준:
//!
//! | 축 | 정본 | leading 없음(`#7062`) | 줄 뒤(이 수정) | 개체 위 |
//! | --- | ---: | ---: | ---: | ---: |
//! | 앞 본문줄 → 도해 잉크 | 84 | 86 | **86** | 96 |
//! | 도해 잉크 → `※` 상자 | 360 | 350 | **360** | 350 |
//! | 도해 잉크 → `※` 첫 줄 | 366 | 356 | **366** | 356 |
//!
//! 곧 leading 은 개체 뒤에 남고 잉크는 움직이지 않는다. 세 값이 함께 맞는 배치는 이것뿐이다.
//! (남는 균일 오프셋 −20px 은 이 칸 위쪽에서 이미 생기는 별도 축이다.)
//!
//! 반례: 저장 LINE_SEG 가 있는 문단, 비-TAC 개체, 그리고 글자모양·문단모양을 못 찾는
//! 문단은 leading 0 으로 종전 경로 그대로다(`renderer::mod` 단위 시험이 그 축을 맡는다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7062/tac_object_host_line_height.hwp";

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn page_nodes(core: &DocumentCore, page_index: u32) -> Vec<RenderNode> {
    let page = core
        .build_page_render_tree(page_index)
        .expect("render tree");
    let mut refs = Vec::new();
    walk(&page.root, &mut refs);
    refs.into_iter().cloned().collect()
}

fn diagram(nodes: &[RenderNode]) -> BoundingBox {
    nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Image(_) if (n.bbox.height - 400.0).abs() < 1.0 => Some(n.bbox),
            _ => None,
        })
        .expect("2쪽 400px 도해 그림")
}

/// 그림 뒤 `※` 표(1행 1열)의 상자.
fn following_table(nodes: &[RenderNode], image: &BoundingBox) -> BoundingBox {
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if n.bbox.y > image.y + 100.0 => Some(n.bbox),
            _ => None,
        })
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("그림 뒤 표")
}

#[test]
fn issue_7079_paragraph_advance_matches_stored_ladder() {
    let core = load();
    let nodes = page_nodes(&core, 1);
    let image = diagram(&nodes);
    let table = following_table(&nodes, &image);

    // 저장 사다리: 그림 문단의 몫 30749HU = 410.0px (그림 400.0 + leading 10.0).
    // leading 이 빠지면 400.0 이라 뒤 표가 10px 올라온다.
    let advance = table.y - image.y;
    assert!(
        (advance - 410.0).abs() < 1.0,
        "#7079: 그림 상단 → 뒤 표 상단은 사다리 410.0px 여야 한다 — leading 이 빠지면 400.0: {advance:.1}"
    );
}

#[test]
fn issue_7079_leading_stays_behind_the_object_ink() {
    let core = load();
    let nodes = page_nodes(&core, 1);
    let image = diagram(&nodes);

    // 잉크는 움직이지 않는다 — 앞 빈 문단(p[11], 저장 lh=600 = 8.0px) 바로 아래에서
    // 시작한다. leading 을 잉크 위로 올리면 이 간격이 10px 벌어지고, 한컴 출력과
    // 겹쳐 재면 앞 거리가 84 → 96px 로 어긋난다.
    let prev_line_bottom = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextLine(_)
                if n.bbox.y + n.bbox.height <= image.y + 0.01
                    && n.bbox.y > image.y - 40.0
                    && n.bbox.height < 20.0 =>
            {
                Some(n.bbox.y + n.bbox.height)
            }
            _ => None,
        })
        .fold(f64::MIN, f64::max);
    assert!(
        prev_line_bottom > f64::MIN,
        "#7079: 그림 앞 빈 문단 줄을 찾지 못했다"
    );
    assert!(
        (image.y - prev_line_bottom).abs() < 0.5,
        "#7079: 개체 잉크는 앞 줄 바로 아래여야 한다(leading 은 뒤에 남는다): {:.1}",
        image.y - prev_line_bottom
    );

    // 호스트 줄 상자도 잉크와 같은 자리에서 시작한다 — 줄과 개체가 갈리지 않는다.
    let host = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextLine(_) if (n.bbox.y - image.y).abs() < 0.5 => Some(n.bbox),
            _ => None,
        })
        .max_by(|a, b| a.height.total_cmp(&b.height))
        .expect("도해 그림의 호스트 줄");
    assert!(
        (host.height - image.height).abs() < 1.0,
        "#7079: 호스트 줄은 개체 높이를 담는다: {:.1}",
        host.height
    );
}

#[test]
fn issue_7079_page_count_unchanged() {
    // leading 은 문단 높이를 늘린다 — 이 문서의 쪽 경계는 움직이지 않아야 한다.
    assert_eq!(load().page_count(), 10, "#7079: 쪽수는 10 이어야 한다");
}
