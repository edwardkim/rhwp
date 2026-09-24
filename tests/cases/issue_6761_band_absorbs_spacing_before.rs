//! [#6761] 자리차지 표 밴드 뒤 문단의 앞 간격을 밴드 아래에 **한 번 더** 더한다.
//!
//! ## 무엇이 문제였나
//!
//! 글자 있는 host 문단이 비-TAC 자리차지 표(vert=문단, 양수 offset)를 달면, 다음 문단은
//! 표 밴드 아래에서 재개한다. 한컴은 그 문단의 **앞 간격을 밴드 안에서 소비**한다 —
//! 첫 줄 top 은 `max(흐름 + 앞 간격, 표 바깥여백 상자 하단)` 이다. rhwp 의 layout 은
//! 밴드 하단으로 민 뒤 문단 layout 이 앞 간격을 또 더해, 줄이 앞 간격만큼 아래에 그려졌다.
//!
//! ```text
//!   1480000-201900042 83쪽  <표> pi=152 (vert=문단 3045HU, 높이 14846HU, 바깥여백 141HU)
//!     host 문단 상단  = vpos 27845 - 앞 간격 1500           = 26345
//!     표 본체         = 26345 + 3045 + 141 .. + 14846        = 29531 .. 44377
//!     바깥여백 상자 하단 = 44377 + 141                         = 44518 = pi=153 저장 vpos
//!     rhwp 수정 전   pi=153 = 표 하단 + 1.9 + 앞 간격 20px     → 뒤 문단 전부 +20px,
//!                    마지막 표가 본문 바닥을 7.9px 넘는다
//! ```
//!
//! ## 기대값의 독립 근거
//!
//! - 저장 사다리: 위 계산(`rhwp dump -s 4 -p 152`, `-p 153`, `-p 154`)
//! - 한컴 정본 `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` 83쪽:
//!   `□ 일반인 대상 선호도 조사` y=751.1px = 저장 46410HU(+본문 상단 132.28px)
//! - 코퍼스 HWP5 6,582건에서 같은 형상 141곳 중 다음 문단 vpos 가 상자 하단과 같은 곳 90곳
//!   (모두 0HU 차), 하단+앞 간격인 곳 0곳
//!
//! ## 반례 — 흐름이 밴드 안에서 시작하고 앞 간격이 밴드 남은 높이보다 크면 흐름 + 앞 간격이 이긴다
//!
//! 흡수는 `max` 이지 "밴드 하단으로 당기기"가 아니다. 표를 흐름 위로 올리고 앞 간격을 230px 로 키운 변형에서는
//! 흐름 + 앞 간격이 밴드 하단보다 아래이므로 줄은 그 자리에 있어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
/// host 문단의 글자 — 이 문단이 표 pi=152 를 단다.
const HOST: &str = "환경부,환기원및연구진선호도조사";
/// 같은 문장이 앞쪽 요약에도 있으므로 다음 문단 글자와 함께 찾는다.
const NEXT_TEXT: &str = "일반인대상선호도조사";
const SECTION: usize = 4;
const HOST_PARA: usize = 152;
const EMPTY_PARA: usize = 153;
const NEXT_PARA: usize = 154;
/// 바깥여백 141HU.
const OUTER_MARGIN_PX: f64 = 141.0 / 75.0;
/// 저장 pi=154 vpos 46410 - 표 본체 하단 44377.
const NEXT_FROM_TABLE_PX: f64 = (46410.0 - 44377.0) / 75.0;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// host 표가 있는 쪽. 앞쪽 요약 구역에 같은 문장이 되풀이되고 렌더 노드의 문단 번호는
/// 구역 안 번호라, 글자와 함께 본문에 표 pi=152 가 실제로 있는 쪽을 고른다.
fn host_page(core: &DocumentCore) -> u32 {
    (0..core.page_count() as u32)
        .find(|page| {
            let text = flat(&core.extract_page_text_native(*page).unwrap_or_default());
            if !(text.contains(HOST) && text.contains(NEXT_TEXT)) {
                return false;
            }
            let tree = core.build_page_render_tree(*page).expect("render tree");
            column_nodes(&tree.root).iter().any(|n| {
                matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(HOST_PARA))
            })
        })
        .expect("host 표를 담은 쪽")
}

/// 본문(`Body`) 단의 직계 노드. 머리말도 단 노드를 가질 수 있어 본문 아래에서만 찾는다.
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

fn table_bottom(nodes: &[&RenderNode], para: usize) -> f64 {
    nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Table(t) if t.para_index == Some(para) => {
                Some(n.bbox.y + n.bbox.height)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            let seen: Vec<String> = nodes
                .iter()
                .map(|n| match &n.node_type {
                    RenderNodeType::Table(t) => format!("Table{:?}", t.para_index),
                    RenderNodeType::TextLine(l) => format!("Line{:?}", l.para_index),
                    _ => "기타".to_string(),
                })
                .collect();
            panic!("표 pi={para} 가 없다 — 본문 단 노드: {seen:?}")
        })
}

fn line_top(nodes: &[&RenderNode], para: usize) -> f64 {
    nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextLine(line) if line.para_index == Some(para) => Some(n.bbox.y),
            _ => None,
        })
        .unwrap_or_else(|| panic!("pi={para} 첫 줄이 없다"))
}

/// 밴드 뒤 문단의 첫 줄은 표 바깥여백 상자 하단에서 시작한다(앞 간격 흡수).
#[test]
fn next_paragraph_starts_at_table_outer_margin_bottom() {
    let core = open();
    let page = host_page(&core);
    let tree = core.build_page_render_tree(page).expect("render tree");
    let nodes = column_nodes(&tree.root);
    let bottom = table_bottom(&nodes, HOST_PARA);
    let empty_gap = line_top(&nodes, EMPTY_PARA) - bottom;
    let next_gap = line_top(&nodes, NEXT_PARA) - bottom;
    assert!(
        (empty_gap - OUTER_MARGIN_PX).abs() <= 0.6,
        "{}쪽 pi={EMPTY_PARA} 이 표 하단에서 {empty_gap:.1}px 떨어졌다 — 저장 vpos 대로 바깥여백 \
         {OUTER_MARGIN_PX:.1}px 여야 한다. 앞 간격(20px)을 밴드 아래에 또 더하면 21.9px 가 된다",
        page + 1
    );
    assert!(
        (next_gap - NEXT_FROM_TABLE_PX).abs() <= 1.0,
        "{}쪽 `□ 일반인 대상 선호도 조사` 가 표 하단에서 {next_gap:.1}px — 저장 {NEXT_FROM_TABLE_PX:.1}px \
         (정본 751.1px 와 같은 자리)여야 한다",
        page + 1
    );
}

/// 반례 — 흐름이 밴드 **안에서** 시작하고 앞 간격이 밴드 남은 높이보다 크면, 줄은
/// `흐름 + 앞 간격` 에 있어야 한다. 흡수는 `max` 이지 "밴드 하단 - 앞 간격으로 당기기"가 아니다.
///
/// 원본은 host 줄 끝(흐름)이 표 상단보다 위라 밴드 밀기 자체가 일어나지 않고, 저장 vpos 스냅이
/// 걸리면 `흐름 + 앞 간격 = 밴드 하단` 이 늘 성립해 경계를 가르지 못한다. 그래서 표 세로
/// offset 을 3045 → 1000HU 로 줄여 표 상단을 흐름 위로 올리고, 앞 간격을 230px 로 키운다.
///
/// 이 불변식은 흡수 목표의 `max` 와 밴드 밀기의 "앞으로만 민다" 검사(`jump_to > y_offset`)가
/// 함께 지킨다 — 흡수 목표를 무조건 쓰는 변이도 뒤 검사에 막혀 이 시험은 통과한다. 둘 중
/// 뒤 검사까지 무너뜨리는 변경을 잡는 가드다.
#[test]
fn spacing_before_larger_than_band_remainder_keeps_flow_position() {
    use rhwp::model::control::Control;
    const BIG_SPACING_BEFORE_PX: f64 = 230.0;
    let mut core = open();
    let mut doc = core.document().clone();
    let section = &mut doc.sections[SECTION];
    let Some(Control::Table(table)) = section.paragraphs[HOST_PARA].controls.get_mut(0) else {
        panic!("host 문단의 표");
    };
    assert_eq!(table.common.vertical_offset, 3045, "원본 표 세로 offset");
    table.common.vertical_offset = 1000;
    let shape_id = section.paragraphs[EMPTY_PARA].para_shape_id as usize;
    let mut shape = doc.doc_info.para_shapes[shape_id].clone();
    // ParaShape 의 앞 간격 단위는 HU 의 2배다(3000 → 1500HU = 20px).
    shape.spacing_before = (BIG_SPACING_BEFORE_PX * 75.0 * 2.0) as i32;
    doc.doc_info.para_shapes.push(shape);
    let new_id = (doc.doc_info.para_shapes.len() - 1) as u16;
    doc.sections[SECTION].paragraphs[EMPTY_PARA].para_shape_id = new_id;
    core.set_document(doc);

    let page = host_page(&core);
    let tree = core.build_page_render_tree(page).expect("render tree");
    let nodes = column_nodes(&tree.root);
    let (table_top, bottom) = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Table(t) if t.para_index == Some(HOST_PARA) => {
                Some((n.bbox.y, n.bbox.y + n.bbox.height))
            }
            _ => None,
        })
        .expect("host 표");
    let host_line_end = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextLine(line) if line.para_index == Some(HOST_PARA) => {
                Some(n.bbox.y + n.bbox.height)
            }
            _ => None,
        })
        .fold(f64::MIN, f64::max);
    let top = line_top(&nodes, EMPTY_PARA);
    assert!(
        table_top < host_line_end && host_line_end + BIG_SPACING_BEFORE_PX > bottom,
        "전제: 흐름({host_line_end:.1})이 밴드({table_top:.1}..{bottom:.1}) 안에서 시작하고 \
         흐름 + 앞 간격이 밴드 하단보다 아래여야 반례가 된다"
    );
    assert!(
        top >= host_line_end + BIG_SPACING_BEFORE_PX - 0.5,
        "앞 간격 {BIG_SPACING_BEFORE_PX}px 인 pi={EMPTY_PARA} 첫 줄이 {top:.1}px — 흐름({host_line_end:.1}) \
         + 앞 간격 아래여야 한다. 흡수가 줄을 밴드 하단({bottom:.1}) 쪽으로 당기면 안 된다"
    );
}
