//! [Issue #6619] `hp:cellzone` 의 **테두리**를 렌더러가 한 번도 방출하지 않아, 오직
//! zone 만 참조하는 선이 통째로 사라지던 결함의 가드.
//!
//! zone 은 셀 고유 `borderFillIDRef` 위에 얹는 **영역 덮어쓰기**다. 종전 렌더러는
//! zone 에 대해 배경(`render_cell_background`)과 대각선만 그리고 네 변을 방출하지
//! 않았다. 이후 셀 테두리 패스는 각 셀의 고유 `border_fill_id` 만 쓰므로 zone 덮어쓰기가
//! 통째로 사라진다.
//!
//! 실측 — `156745900` 2쪽 `일 러 두 기` 틀 (px @96dpi, 한/글 2020 `SaveAs PDF` 대조):
//!
//! ```text
//! rhwp lines 6 / oracle lines 13   — 오라클에만 있는 선 5, rhwp 에만 있는 선 0
//!   H y= 102.4  x= 76.8.. 290.6  #BBBBBB 0.4mm   제목 상자 왼쪽 가로선
//!   V x=  77.4  y=128.2..1041.7  #BBBBBB 0.4mm   본문 왼쪽 세로선
//!   V x= 719.6  y=128.2..1041.7  #BBBBBB 0.4mm   본문 오른쪽 세로선
//!   H y=1040.9  x= 76.8.. 720.4  #BBBBBB 0.4mm   본문 아래 가로선
//! ```
//!
//! `#BBBBBB` 는 그 문서 904개 borderFill 중 zone 이 참조하는 둘에만 있고 `hp:tc` 는
//! 어디서도 참조하지 않는다. 31쪽 통계표에서는 zone(L/R SOLID)이 무시돼 셀 고유의
//! **점선**이 그대로 남았다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{BorderFill, BorderLine, BorderLineType, CharShape, ParaShape};
use rhwp::model::table::{Cell, Table, TableZone};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::StrokeDash;
use rhwp::serializer::hwpx::serialize_hwpx;

const COL: u32 = 4000;
const ROW: u32 = 3000;

fn line(line_type: BorderLineType, color: u32) -> BorderLine {
    BorderLine {
        line_type,
        width: 1,
        color,
    }
}

fn none_fill() -> BorderFill {
    let none = line(BorderLineType::None, 0);
    BorderFill {
        borders: [none, none, none, none],
        ..Default::default()
    }
}

/// 왼쪽·오른쪽·아래만 SOLID, 위는 NONE — 문서의 zone 38 과 같은 모양.
fn zone_fill() -> BorderFill {
    let solid = line(BorderLineType::Solid, 0x00BB_BBBB);
    BorderFill {
        borders: [solid, solid, line(BorderLineType::None, 0), solid],
        ..Default::default()
    }
}

/// 네 변 모두 점선 — 31쪽 통계표의 셀 고유 테두리에 해당한다.
fn dotted_fill() -> BorderFill {
    let dotted = line(BorderLineType::Dot, 0);
    BorderFill {
        borders: [dotted, dotted, dotted, dotted],
        ..Default::default()
    }
}

fn cell(col: u16, row: u16, bf: u16) -> Cell {
    Cell {
        col,
        row,
        col_span: 1,
        row_span: 1,
        width: COL,
        height: ROW,
        border_fill_id: bf,
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    }
}

/// 2×2 표. 모든 칸은 `cell_bf`, 표 자신은 테두리 없음(1), zone 하나가 표 전체를 덮는다.
fn zoned_document(cell_bf: u16, border_fills: Vec<BorderFill>) -> Document {
    let mut table = Table {
        row_count: 2,
        col_count: 2,
        border_fill_id: 1,
        cells: vec![
            cell(0, 0, cell_bf),
            cell(1, 0, cell_bf),
            cell(0, 1, cell_bf),
            cell(1, 1, cell_bf),
        ],
        zones: vec![TableZone {
            start_col: 0,
            start_row: 0,
            end_col: 1,
            end_row: 1,
            border_fill_id: 2,
        }],
        ..Default::default()
    };
    table.common.width = COL * 2;
    table.common.height = ROW * 2;
    table.rebuild_grid();

    let mut owner = Paragraph::default();
    owner.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59528,
        height: 84188,
        ..Default::default()
    };
    section.paragraphs.push(owner);

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.doc_info.char_shapes = vec![CharShape::default()];
    doc.doc_info.border_fills = border_fills;
    doc.doc_properties.section_count = 1;
    doc.sections.push(section);
    doc
}

/// `(x1, y1, x2, y2, color, dash)`
type Seg = (f64, f64, f64, f64, u32, StrokeDash);

fn collect_lines(node: &RenderNode, out: &mut Vec<Seg>) {
    if let RenderNodeType::Line(l) = &node.node_type {
        out.push((l.x1, l.y1, l.x2, l.y2, l.style.color, l.style.dash));
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

fn find_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node);
    }
    node.children.iter().find_map(find_table)
}

fn table_lines(doc: Document) -> (Vec<Seg>, (f64, f64, f64, f64)) {
    let bytes = serialize_hwpx(&doc).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("reload");
    let page = core.build_page_render_tree(0).expect("render tree");
    let table = find_table(&page.root).expect("zoned table");
    let mut lines = Vec::new();
    collect_lines(table, &mut lines);
    let b = table.bbox;
    (lines, (b.x, b.y, b.x + b.width, b.y + b.height))
}

/// 축 ① — 칸이 하나도 안 그리는 변을 zone 이 그린다.
///
/// 칸 전부 NONE + 표 NONE 이면 zone 의 L/R/B 세 변만 나와야 한다. 종전에는 **선이
/// 하나도** 나오지 않았다 — 문서의 틀 5선 소실이 정확히 이 형태다.
#[test]
fn cellzone_border_draws_sides_no_cell_owns() {
    let (lines, (left, top, right, bottom)) =
        table_lines(zoned_document(1, vec![none_fill(), zone_fill()]));

    let has_left = lines.iter().any(|&(x1, y1, x2, y2, _, _)| {
        (x1 - left).abs() <= 2.0
            && (x2 - left).abs() <= 2.0
            && (y1.min(y2) - top).abs() <= 4.0
            && y1.max(y2) >= bottom - 4.0
    });
    let has_right = lines.iter().any(|&(x1, y1, x2, y2, _, _)| {
        (x1 - right).abs() <= 2.0
            && (x2 - right).abs() <= 2.0
            && (y1.min(y2) - top).abs() <= 4.0
            && y1.max(y2) >= bottom - 4.0
    });
    let has_bottom = lines.iter().any(|&(x1, y1, x2, y2, _, _)| {
        (y1 - bottom).abs() <= 2.0
            && (y2 - bottom).abs() <= 2.0
            && (x1.min(x2) - left).abs() <= 4.0
            && x1.max(x2) >= right - 4.0
    });
    // 위 변은 zone 이 NONE 이므로 나오면 안 된다 — 덮어쓰기 범위가 넓어지는 것을 막는다.
    let has_top = lines
        .iter()
        .any(|&(_, y1, _, y2, _, _)| (y1 - top).abs() <= 2.0 && (y2 - top).abs() <= 2.0);

    assert!(
        has_left && has_right && has_bottom,
        "cellzone 의 L/R/B 세 변이 나와야 한다 — #6619 회귀. \
         left={has_left} right={has_right} bottom={has_bottom} lines={lines:?}"
    );
    assert!(
        !has_top,
        "cellzone 의 NONE 인 위 변까지 그리면 안 된다. lines={lines:?}"
    );
}

/// 축 ② — zone 은 칸 고유 테두리를 **이긴다**.
///
/// 칸이 네 변 점선인데 zone 이 L/R/B SOLID 면 그 세 변은 SOLID 여야 한다. 종전에는
/// zone 이 무시돼 점선이 그대로 남았다 (31쪽 `stroke-dasharray="2 2"`).
#[test]
fn cellzone_border_overrides_cell_own_border() {
    let (lines, (left, _top, right, bottom)) = table_lines(zoned_document(
        3,
        vec![none_fill(), zone_fill(), dotted_fill()],
    ));

    // 아래 변을 지목한다 — zone 색(#BBBBBB) 실선이어야 하고, 칸 고유의 검정 점선이
    // 남아 있으면 안 된다.
    let bottom_seg = lines.iter().find(|&&(x1, y1, x2, y2, _, _)| {
        (y1 - bottom).abs() <= 2.0
            && (y2 - bottom).abs() <= 2.0
            && (x1.min(x2) - left).abs() <= 4.0
            && x1.max(x2) >= right - 4.0
    });

    let Some(&(_, _, _, _, color, dash)) = bottom_seg else {
        panic!("zone 아래 변이 아예 없다 — #6619 회귀. lines={lines:?}");
    };
    assert_eq!(
        color, 0x00BB_BBBB,
        "아래 변이 zone 색이어야 한다(칸 점선이 이기고 있다) — #6619 회귀. lines={lines:?}"
    );
    assert!(
        !matches!(dash, StrokeDash::Dot | StrokeDash::Dash),
        "아래 변이 실선이어야 한다(31쪽 stroke-dasharray 회귀) — got {dash:?}, lines={lines:?}"
    );
}
