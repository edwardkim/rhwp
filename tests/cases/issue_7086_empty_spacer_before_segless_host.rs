//! [#7086] 저장 LINE_SEG 없는 표 host 문단 **앞**의 빈 문단이 0 높이로 접혀 사라진다.
//!
//! `samples/issue7062/tac_object_host_line_height.hwp` 2쪽 바깥 칸의 저장 사다리다.
//!
//! ```text
//! p[16] ps_id=106 [table(tac)]  ls[0] vpos=0    lh=2982 ls=752   ← 제목 상자(39.8px)
//! p[17] ps_id=107 (빈 문단)      ls[0] vpos=3734 lh=600  ls=284   ← 8.0px + 3.8px
//! p[18] ps_id=71  [table(tac)]  저장 LINE_SEG 없음                ← 3×3 표
//! ```
//!
//! `p[16]` 의 슬롯 끝(0+2982+752 = 3734)이 `p[17].vpos` 와 정확히 맞는데, 빈 문단 collapse
//! 규칙이 **다음 문단이 control 을 host 하면** 그 vpos 를 증거로 못 쓴다는 이유로 이 줄을
//! 0 으로 접었다. `p[18]` 은 저장 seg 가 아예 없어 애초에 비교할 vpos 가 없다 — 그래서
//! 앞 문단의 슬롯이 이 vpos 에 닿는지로 판정하도록 좁혔다.
//!
//! 접히면 그 아래 쪽 전체가 11.8px 위로 올라간다. 한컴 engine 2020 출력(job
//! `94f14dc5-422a-462a-b4e6-810ef36ff98d`, `pdf/tac_object_host_line_height-2020.pdf`)을
//! 96dpi 래스터로 겹쳐 재면 수정 전 −21px 이던 잉크 차가 수정 후 **−9px 로 균일**해진다
//! (남는 −9 는 칸 첫 내용 상단 여백 축이며 이 수정 밖이다).
//!
//! 반례: 앞 문단의 슬롯이 이 문단 vpos 에 닿지 않는 빈 문단(장식용 겹침 스페이서)과 다음
//! 문단이 저장 seg 를 가진 경우는 종전 collapse 계약 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

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

#[test]
fn issue_7086_empty_paragraph_between_two_table_hosts_keeps_its_line() {
    let core = load();
    let nodes = page_nodes(&core, 1);

    // 제목 상자는 47.2..87.0 그대로다.
    let title_box = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if (n.bbox.height - 39.8).abs() < 0.5 => Some(n.bbox),
            _ => None,
        })
        .expect("제목 상자(39.8px)");
    assert!(
        (title_box.y - 47.2).abs() < 0.5,
        "#7086: 제목 상자 위치는 불변이어야 한다: {:.1}",
        title_box.y
    );

    // p[17] 의 빈 줄 — 저장 lh=600 = 8.0px. 접히면 이 줄이 아예 없다.
    let spacer = nodes.iter().find_map(|n| match &n.node_type {
        RenderNodeType::TextLine(_)
            if (n.bbox.y - 97.0).abs() < 0.5 && (n.bbox.height - 8.0).abs() < 0.5 =>
        {
            Some(n.bbox)
        }
        _ => None,
    });
    assert!(
        spacer.is_some(),
        "#7086: 제목 상자와 3×3 표 사이의 빈 문단 줄(y=97.0 h=8.0)이 있어야 한다"
    );

    // 3×3 표는 그 빈 문단의 몫(600+284HU = 11.8px)만큼 내려간다 — 접히면 97.0 이다.
    let grid = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if (n.bbox.height - 915.1).abs() < 1.0 => Some(n.bbox),
            _ => None,
        })
        .expect("3×3 표");
    assert!(
        (grid.y - 108.8).abs() < 0.5,
        "#7086: 3×3 표는 97.0 + 11.8 = 108.8 에서 시작해야 한다 — 접히면 97.0: {:.1}",
        grid.y
    );
}

#[test]
fn issue_7086_downstream_content_shifts_by_the_same_slot() {
    let core = load();
    let nodes = page_nodes(&core, 1);
    let image = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Image(_) if (n.bbox.height - 400.0).abs() < 1.0 => Some(n.bbox),
            _ => None,
        })
        .expect("도해 그림");
    // 548.1 + 11.8. 그 아래 상대 배치(#7062·#7079)는 그대로다.
    assert!(
        (image.y - 559.9).abs() < 0.5,
        "#7086: 도해 그림도 같은 11.8px 만큼 내려간다: {:.1}",
        image.y
    );
    let table = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table { .. } if n.bbox.y > image.y + 100.0 => Some(n.bbox),
            _ => None,
        })
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("그림 뒤 표");
    assert!(
        (table.y - image.y - 410.0).abs() < 1.0,
        "#7086: 그림 → 뒤 표 간격(410.0 = 사다리)은 이 수정으로 변하지 않는다: {:.1}",
        table.y - image.y
    );
}

#[test]
fn issue_7086_page_count_unchanged() {
    // 빈 줄 하나가 살아나도 이 문서의 쪽 경계는 움직이지 않는다.
    assert_eq!(load().page_count(), 10, "#7086: 쪽수는 10 이어야 한다");
}
