//! #7552 — 쪽 나누기 문단으로 시작하는 쪽의 저장 원점과, 그 원점을 바로잡으면 드러나는
//! 표 host·표 조각 경계를 함께 고정한다.
//!
//! 쪽 첫 문단의 저장 vpos 가 그 문단의 앞 간격과 같으면 쪽 원점은 0 이다(앞 간격 보존).
//! 종전에는 쪽 나누기 문단을 이 증거에서 빼, 조판 커서만 앞 간격만큼 위로 당겨졌다.
//! 그 커서로 정한 표 기준점 때문에 위 캡션이 앞 문단 마지막 줄에 겹쳤다
//! (`1480000-201900042` 76쪽 `<표 4-1>`, 정본 표 윗변 861.2px).
//!
//! 기대값은 화면 좌표가 아니라 독립 근거에서 온다 — 앞뒤 줄과 캡션·표의 순서, 그리고
//! 문서 자신의 저장 LineSeg 사다리 거리. 위치·모양은 한컴 PDF Visual Sweep 으로 확인했다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const DPI: f64 = 96.0;

fn open(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn hu(value: i64) -> f64 {
    value as f64 * DPI / 7200.0
}

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 본문(`Body`) 단의 직계 노드. 머리말·꼬리말 단은 제외한다.
fn column_nodes(root: &RenderNode) -> Vec<&RenderNode> {
    fn walk<'a>(node: &'a RenderNode, in_body: bool, out: &mut Vec<&'a RenderNode>) {
        let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
        if in_body && matches!(node.node_type, RenderNodeType::Column(_)) {
            out.extend(node.children.iter());
            return;
        }
        for child in &node.children {
            walk(child, in_body, out);
        }
    }
    let mut out = Vec::new();
    walk(root, false, &mut out);
    out
}

fn table_node<'a>(nodes: &[&'a RenderNode], section: usize, para: usize) -> Option<&'a RenderNode> {
    nodes.iter().copied().find(|n| {
        matches!(&n.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(section) && t.para_index == Some(para) && t.cell_context.is_none())
    })
}

/// 본문 글줄(캡션 제외) 중 해당 문단의 줄들.
fn body_lines<'a>(nodes: &[&'a RenderNode], section: usize, para: usize) -> Vec<&'a RenderNode> {
    nodes
        .iter()
        .copied()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(line)
                if line.caption_owner.is_none()
                    && line.section_index == Some(section)
                    && line.para_index == Some(para))
        })
        .collect()
}

/// 표를 담은 쪽 — 렌더 트리에서 표 노드를 찾아 고른다.
fn page_with_table(core: &DocumentCore, section: usize, para: usize) -> u32 {
    (0..core.page_count() as u32)
        .find(|page| {
            let tree = core.build_page_render_tree(*page).expect("render tree");
            table_node(&column_nodes(&tree.root), section, para).is_some()
        })
        .unwrap_or_else(|| panic!("구역 {section} pi={para} 표를 담은 쪽이 없다"))
}

/// 결함 검출 — 쪽 나누기 문단으로 시작하는 쪽에서 위 캡션 표가 앞 문단을 덮지 않는다.
///
/// 수정 전: 캡션 780.6 < 앞 줄 하단 806.9(겹침 5.68px 텍스트 겹침 2건), 표 808.0.
/// 한컴 2020 정본: 앞 줄 791.7..807.7, 캡션 831.7, 표 윗변 861.2.
#[test]
fn caption_table_on_page_break_page_starts_below_previous_paragraph() {
    const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
    const SECTION: usize = 4;
    const PREV: usize = 117;
    const HOST: usize = 118;
    let core = open(SAMPLE);
    let page = page_with_table(&core, SECTION, HOST);
    let tree = core.build_page_render_tree(page).expect("render tree");
    let nodes = column_nodes(&tree.root);

    let prev_bottom = body_lines(&nodes, SECTION, PREV)
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        prev_bottom.is_finite(),
        "앞 문단 pi={PREV} 이 같은 쪽에 없다"
    );
    let captions: Vec<&RenderNode> = nodes
        .iter()
        .copied()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(line)
                if line.caption_owner.is_some_and(|owner| owner.sec_idx == SECTION && owner.para_idx == HOST))
        })
        .collect();
    assert!(!captions.is_empty(), "표 pi={HOST} 의 위 캡션이 없다");
    let caption_top = captions
        .iter()
        .map(|n| n.bbox.y)
        .fold(f64::INFINITY, f64::min);
    let caption_bottom = captions
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);
    let table = table_node(&nodes, SECTION, HOST).expect("표");

    assert!(
        caption_top >= prev_bottom - 0.5,
        "캡션 윗변 {caption_top:.2} 이 앞 문단 마지막 줄 하단 {prev_bottom:.2} 위에 있다"
    );
    assert!(
        table.bbox.y >= caption_bottom - 0.5,
        "표 윗변 {:.2} 이 캡션 하단 {caption_bottom:.2} 위에 있다",
        table.bbox.y
    );
}

/// 대조군 — 쪽 나누기 문단이 쪽을 열어도 저장 사다리가 앞 간격을 증언하면 원점은 0 이다.
/// 첫 문단 줄과 다음 문단 줄의 거리가 저장 vpos 차이와 같다(76쪽 pi=107 → pi=108).
#[test]
fn page_break_first_paragraph_keeps_stored_ladder_distance() {
    const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
    const SECTION: usize = 4;
    let core = open(SAMPLE);
    let page = page_with_table(&core, SECTION, 118);
    let tree = core.build_page_render_tree(page).expect("render tree");
    let nodes = column_nodes(&tree.root);
    let first = body_lines(&nodes, SECTION, 107);
    let second = body_lines(&nodes, SECTION, 108);
    let (Some(first), Some(second)) = (first.first(), second.first()) else {
        panic!("pi=107·108 첫 줄이 같은 쪽에 없다");
    };
    let vpos = |n: &RenderNode| match &n.node_type {
        RenderNodeType::TextLine(line) => line.vpos.expect("저장 vpos"),
        _ => unreachable!(),
    };
    let expected = hu(i64::from(vpos(second)) - i64::from(vpos(first)));
    let actual = second.bbox.y - first.bbox.y;
    assert!(
        (actual - expected).abs() <= 0.5,
        "pi=107→108 줄 거리 {actual:.2} 가 저장 사다리 {expected:.2} 와 다르다"
    );
}

/// 대조군 — 글자 없는 자리차지 표 host(세로 오프셋 0)의 표 괘선은 앞 문단 저장 줄 끝 +
/// 바깥 위 여백이다. 쪽 원점을 바로잡으면 저장 스냅이 host 앞 간격을 미리 뺀 목표를 수용하는데,
/// 이 host 는 그 간격을 다시 더하는 글줄이 없다(issue1853 8쪽 pi=67, 정본 괘선 600.3 =
/// 문단 상단 599.1 + 141HU).
#[test]
fn empty_table_host_opens_at_previous_stored_line_end() {
    const SAMPLE: &str = "samples/issue1853_caption_precedes_body_split.hwpx";
    const SECTION: usize = 0;
    const PREV: usize = 66;
    const HOST: usize = 67;
    let core = open(SAMPLE);
    let page = page_with_table(&core, SECTION, HOST);
    let tree = core.build_page_render_tree(page).expect("render tree");
    let nodes = column_nodes(&tree.root);
    let prev = *body_lines(&nodes, SECTION, PREV)
        .last()
        .expect("앞 문단 줄");
    let table = table_node(&nodes, SECTION, HOST).expect("표");

    let paragraphs = &core.document().sections[SECTION].paragraphs;
    let seg = paragraphs[PREV].line_segs.last().expect("앞 문단 저장 줄");
    let Some(rhwp::model::control::Control::Table(host_table)) = paragraphs[HOST].controls.first()
    else {
        panic!("pi={HOST} 표 컨트롤");
    };
    let expected = hu(i64::from(seg.line_height)
        + i64::from(seg.line_spacing)
        + i64::from(host_table.outer_margin_top));
    let actual = table.bbox.y - prev.bbox.y;
    assert!(
        (actual - expected).abs() <= 0.5,
        "표 윗변 - 앞 줄 윗변 = {actual:.2}, 저장 줄 끝 거리 {expected:.2}"
    );
}

/// 대조군(HWP5) — 같은 형상에서 표 뒤 다음 문단 줄은 저장 사다리 거리에 선다. host 의
/// 앞·뒤 간격은 공간을 차지하지 않는다(hwpctl 49쪽 pi=1171 → pi=1173, 앞 끝 5460 + 상자
/// 11448 = 다음 줄 16908).
#[test]
fn hwp5_empty_table_host_flow_follows_stored_ladder() {
    const SAMPLE: &str = "samples/hwpctl_API_v2.4.hwp";
    const SECTION: usize = 0;
    let core = open(SAMPLE);
    let page = page_with_table(&core, SECTION, 1172);
    let tree = core.build_page_render_tree(page).expect("render tree");
    let nodes = column_nodes(&tree.root);
    let before = *body_lines(&nodes, SECTION, 1171)
        .last()
        .expect("pi=1171 마지막 줄");
    let after = *body_lines(&nodes, SECTION, 1173)
        .first()
        .expect("pi=1173 첫 줄");
    let vpos = |n: &RenderNode| match &n.node_type {
        RenderNodeType::TextLine(line) => line.vpos.expect("저장 vpos"),
        _ => unreachable!(),
    };
    let expected = hu(i64::from(vpos(after)) - i64::from(vpos(before)));
    let actual = after.bbox.y - before.bbox.y;
    assert!(
        (actual - expected).abs() <= 1.0,
        "표 뒤 줄 거리 {actual:.2} 가 저장 사다리 {expected:.2} 와 다르다"
    );
}

/// 표 첫 조각을 담은 쪽의 글자(공백 제거).
fn first_fragment_page_text(core: &DocumentCore, section: usize, para: usize) -> String {
    let page = page_with_table(core, section, para);
    flat(&core.extract_page_text_native(page).expect("쪽 글자"))
}

/// 대조군 — 칸 문단 안 저장 되감김(`vpos > 0 → 0`)이 증언한 마지막 줄은 첫 조각 쪽에 남는다.
/// 그 줄의 뒤 줄간격은 다음 쪽의 것이다. 원점 수정으로 앞 내용이 한컴 위치로 내려가도
/// 줄을 잃지 않아야 한다(issue1853 13쪽 2×2 표 pi=105 — 칸이 둘째 행이라 선언 상자에
/// 앞 행이 들어간다).
#[test]
fn stored_cell_reset_line_stays_on_first_fragment_page() {
    const SAMPLE: &str = "samples/issue1853_caption_precedes_body_split.hwpx";
    let core = open(SAMPLE);
    let text = first_fragment_page_text(&core, 0, 105);
    assert!(
        text.contains("1.주요정책을수립하거나정치적"),
        "저장 되감김 앞 줄이 표 첫 조각 쪽에 없다"
    );
}

/// 대조군 — 같은 행의 형제 칸이 되감김으로 첫 조각 상자를 증명하면, 칸 여백이 0 이라 상자가
/// 선언 하단 안에서 끝나는 칸도 같은 줄까지 첫 조각에 담는다(issue1853 37쪽 pi=311
/// 오른쪽 칸 4번째 줄).
#[test]
fn row_sibling_reset_keeps_padless_cell_line_on_first_fragment_page() {
    const SAMPLE: &str = "samples/issue1853_caption_precedes_body_split.hwpx";
    let core = open(SAMPLE);
    let text = first_fragment_page_text(&core, 0, 311);
    assert!(
        text.contains("호의사항을포함한데이터통합관리플랫"),
        "오른쪽 칸 되감김 앞 줄이 표 첫 조각 쪽에 없다"
    );
}
