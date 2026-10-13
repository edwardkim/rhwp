//! [#6929] 문단 기준 자리차지 표의 `vertOffset` 을 push-down 이 통째로 덮어쓰지 않는다.
//!
//! # 무엇이 깨져 있었나
//!
//! `#347` 의 push-down(`raw_y.max(y_start)`)은 자리차지(TopAndBottom) 표를 **앞선 표·텍스트
//! 아래로** 민다. 그런데 앵커 문단이 단 최상단에 있고 그 문단이 잉크를 하나도 안 내면,
//! `y_start` 가 앵커보다 아래인 것은 **앵커 문단 자신의 줄 예약**뿐이다. 그 아래로 밀면
//! 문단 기준 `vertOffset` 이 통째로 무시된다.
//!
//! ```text
//!   본문 상단 94.5 · vertOffset 433 HU = 5.77px
//!   raw_y   = 94.5 + 5.77 = 100.3      ← 선언대로
//!   y_start = 118.5 (= 94.5 + 24.0, 앵커 문단의 줄 예약)
//!   pushed  = max(100.3, 118.5) = 118.5   ← +18.2px, 표 아래끝이 제목을 침범
//! ```
//!
//! # 좁힘 — 공동 앵커 형제가 있으면 그대로 둔다
//!
//! 한 문단에 자리차지 표가 여러 개 달리면(co-anchored) 그 쌓임은 `y_start` 가 만든다.
//! `#1639`(`issue1639_empty_host_negative_offset_float.hwpx`)가 그 순서를 잠그고 있고,
//! 완화를 넓히면 형제들이 같은 자리로 모여 겹친다. 그래서 **단에 아직 아무것도 안 놓였을
//! 때**(`col_node.children.is_empty()`)만 앵커를 바닥으로 쓴다. 보이는 host 제목이 있는
//! `#1549` 형상은 앵커가 단 최상단이어도 제목 줄이 먼저 놓이므로 조건에서 빠진다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{TextWrap, VertAlign, VertRelTo};
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6929/148776468_search_ad_terms_press_release.hwp";

const DPI: f64 = 96.0;
const TITLE: &str = "인터넷포털 검색광고서비스";

fn document(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("재현체 {sample}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("로드 {sample}: {e}"))
}

fn page0(sample: &str) -> RenderNode {
    let core = document(sample);
    core.build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("렌더 {sample}: {e}"))
        .root
}

fn body_column(root: &RenderNode) -> &RenderNode {
    let body = root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문");
    body.children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .expect("본문 단")
}

fn body_head_table(column: &RenderNode) -> &RenderNode {
    column
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Table(_)))
        .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .expect("본문 머리 표")
}

fn source_table<'a>(core: &'a DocumentCore, node: &RenderNode) -> &'a Table {
    let RenderNodeType::Table(owner) = &node.node_type else {
        panic!("표 노드");
    };
    let para = &core.document().sections[owner.section_index.expect("표 구역")].paragraphs
        [owner.para_index.expect("표 문단")];
    let Control::Table(table) = &para.controls[owner.control_index.expect("표 컨트롤")] else {
        panic!("원본 표 컨트롤");
    };
    table
}

fn hu(value: i32) -> f64 {
    f64::from(value) * DPI / 7200.0
}

fn title_line(node: &RenderNode) -> Option<&RenderNode> {
    if !node.visible || node.editor_only {
        return None;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && line_text(node).contains(TITLE) {
        return Some(node);
    }
    node.children.iter().find_map(title_line)
}

fn line_text(node: &RenderNode) -> String {
    let mut s = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        s.push_str(run.display_or_text());
    }
    for c in &node.children {
        s.push_str(&line_text(c));
    }
    s
}

/// 본문 최상단 자리차지 표 — 쪽 맨 위의 머리 표.
fn head_table(node: &RenderNode, out: &mut Option<(f64, f64)>) {
    if !node.visible || node.editor_only {
        return;
    }
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        let top = node.bbox.y;
        if out.is_none_or(|(cur, _)| top < cur) {
            *out = Some((top, top + node.bbox.height));
        }
        return;
    }
    for c in &node.children {
        head_table(c, out);
    }
}

#[test]
fn para_relative_float_table_starts_at_its_declared_offset() {
    let core = document(SAMPLE);
    let root = core.build_page_render_tree(0).expect("1쪽 렌더").root;
    let column = body_column(&root);
    let table = body_head_table(column);
    let source = source_table(&core, table);
    assert!(!source.common.treat_as_char);
    assert_eq!(source.common.text_wrap, TextWrap::TopAndBottom);
    assert_eq!(source.common.vert_rel_to, VertRelTo::Para);
    assert_eq!(source.common.vert_align, VertAlign::Top);
    assert_eq!(source.outer_margin_top, 0);
    let offset = hu(source.common.vertical_offset as i32);
    assert!(
        (table.bbox.y - column.bbox.y - offset).abs() <= 1.0,
        "머리 표는 본문 단의 시작에서 원본 vertOffset만큼 떨어져야 합니다: \
         표 {}, 단 {}, 원본 오프셋 {offset}",
        table.bbox.y,
        column.bbox.y
    );
}

#[test]
fn the_head_table_does_not_reach_into_the_title() {
    let core = document(SAMPLE);
    let root = core.build_page_render_tree(0).expect("1쪽 렌더").root;
    let column = body_column(&root);
    let table = body_head_table(column);
    let title = title_line(column).expect("제목 문단");
    let RenderNodeType::TextLine(owner) = &title.node_type else {
        unreachable!();
    };
    let para = &core.document().sections[owner.section_index.expect("제목 구역")].paragraphs
        [owner.para_index.expect("제목 문단")];
    assert_eq!(
        para.line_segs.len(),
        1,
        "원본 제목은 저장 한 줄이어야 합니다"
    );
    let source_title_offset = hu(para.line_segs[0].vertical_pos);
    // 같은 원문의 독립 Print와 일치하는 저장 사다리 관계다. 절대 제목 y를
    // 고정하지 않고 단 원점과 제목 줄의 소유·저장 위치를 함께 검증한다.
    assert!(
        (title.bbox.y - column.bbox.y - source_title_offset).abs() <= 1.0,
        "제목 줄은 단 원점에서 원본 저장 사다리 위치에 있어야 합니다: \
         제목 {}, 단 {}, 저장 거리 {source_title_offset}",
        title.bbox.y,
        column.bbox.y
    );
    let bottom = table.bbox.y + table.bbox.height;
    assert!(
        bottom <= title.bbox.y + 0.5,
        "표 아래끝이 제목을 침범합니다 — 표 바닥 {bottom} · 제목 상단 {}",
        title.bbox.y
    );
}

/// 반례 — 보이는 host 제목이 있으면 표는 제목 **아래**로 간다(`#1549` 계약).
///
/// 이 완화가 넓어지면 제목과 표가 같은 자리에서 시작해 겹친다.
#[test]
fn visible_host_title_still_pushes_its_float_table_down() {
    let root = page0("samples/issue1549_multipositive_float_tables.hwpx");
    let mut title = None;
    title_top_of(&root, "MULTI POSITIVE TITLE", &mut title);
    let title = title.expect("host 제목을 못 찾았습니다");
    let mut table = None;
    head_table(&root, &mut table);
    let (top, _) = table.expect("자리차지 표를 못 찾았습니다");
    assert!(
        top > title,
        "보이는 host 제목이 있으면 표가 그 아래여야 합니다 — 제목 {title:.1} · 표 {top:.1}"
    );
}

fn title_top_of(node: &RenderNode, needle: &str, out: &mut Option<f64>) {
    if !node.visible || node.editor_only {
        return;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && line_text(node).contains(needle) {
        if out.is_none_or(|cur| node.bbox.y < cur) {
            *out = Some(node.bbox.y);
        }
        return;
    }
    for c in &node.children {
        title_top_of(c, needle, out);
    }
}
