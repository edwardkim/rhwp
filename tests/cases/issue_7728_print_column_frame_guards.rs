//! 관계형·고정형: #7728 계속 표의 열 프레임과 독립 Print의 셀 괘선 끝점을 보호한다.
//! 주석 갱신: 2026-11-06T17:52:38+09:00.
//! 검사 분류: `continuation_column_frames_keep_opening_span_and_row_edges`와
//! `continued_parent_border_contains_owned_nested_frames`,
//! `sibling_nested_source_frames_close_on_the_parent_row_boundary`는 관계형이다.
//! 검사 분류: `declared_partial_cell_centers_only_its_owned_lines`는 관계형이다.
//! 검사 분류: `independent_print_page6_keeps_the_enclosing_bottom_border`와
//! `independent_print_page10_keeps_all_grid_edges`는 고정형(독립 PDF ±2 CSSpx)이다.
//! 6쪽 고정형 근거: 코드9274604528 N99.97875% / fresh WASM99.97413%, 전18쪽 및 직접 판독.
//! 12쪽 정렬 관계는 실제 FAIL → PASS이며 최종 N99.83837% / W99.83778%다.
//! 검사 분류: `closed_tac_wrapper_paints_the_outer_top_reserved_by_its_source_frame`와
//! `atomic_child_continuation_keeps_its_source_outer_top_margin`는 관계형(원본 프레임/여백 계약)이다.
//! 최신 공통 보정: 전18쪽 N최저98.83905% / fresh WASM최저98.83565%, 누락0.
//! 최신6쪽 N99.58584% / W99.26284%,7쪽99.70863/99.71627%,9쪽98.83905/98.87584%.
//! 6쪽·10쪽만 독립 PDF의 ±2px 좌표를 고정하며 나머지는 소속·포함·정렬·원본 여백 관계다.
//! 최신 증적: mydocs/pr/assets/issue7728/phase10-all-pages-98-20261106/README.md.
//! 원본 70833 HWP와 독립 한컴2020 Print PDF, compat2022,96DPI/PDFium/2px raw.
//! 코드04f03cfa 전18쪽 Native 최저94.62153%, fresh WASM 최저94.27517%, 미달·누락0.
//! 포함 관계 검사는 HEAD429에서 실제 FAIL → 후보 PASS이며, 아래 phase6 수치는 보정 전 이력이다.
//! phase6 당시 관계형 대상6쪽 N/W94.62153/94.82126%,7쪽94.79966/94.27517%였다.
//! 고정형 대상10쪽 N/W99.57724/99.47384%: 독립 벡터 선 중심·직접 판독 후 ±2px를 쓴다.
//! 셀 간격·병합을 포함한 선언 폭을 그대로 paint 폭으로 간주하지 않고 시작·계속 조각의 같은 열 소유를 검사한다.
//! 증적: mydocs/pr/assets/issue7728/phase6-important-regressions-20261106/README.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("70833 원본 읽기"))
        .expect("70833 원본 파싱")
}

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn source_tables(paragraphs: &[Paragraph]) -> Vec<&Table> {
    let mut out: Vec<&Table> = Vec::new();
    for paragraph in paragraphs {
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                out.push(table);
                for cell in &table.cells {
                    out.extend(source_tables(&cell.paragraphs));
                }
            }
        }
    }
    out
}

// 관계형: 같은 원본의 병합 칸과 각 행은 분할 전후 같은 열 프레임을 공유한다.
#[test]
fn continuation_column_frames_keep_opening_span_and_row_edges() {
    let core = open();
    let opening = core.build_page_render_tree(5).expect("6쪽 시작 조각");
    let closing = core.build_page_render_tree(6).expect("7쪽 마지막 조각");
    let source = source_tables(&core.document().sections[0].paragraphs[15..16]);
    for shape in [(9, 4), (13, 4)] {
        let originals: Vec<_> = source
            .iter()
            .filter(|t| (t.row_count, t.col_count) == shape)
            .collect();
        assert_eq!(originals.len(), 1);
        let find_table = |root: &RenderNode| -> RenderNode {
            let found:Vec<_>=nodes(root).into_iter().filter(|n|matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==shape)).collect();
            assert_eq!(found.len(), 1);
            found[0].clone()
        };
        let first = find_table(&opening.root);
        let last = find_table(&closing.root);
        let first_span=first.children.iter().find(|n|matches!(&n.node_type,RenderNodeType::TableCell(c)if c.col==0&&c.col_span==shape.1)).expect("시작 쪽의 전체 열 병합 칸");
        let last_span=last.children.iter().find(|n|matches!(&n.node_type,RenderNodeType::TableCell(c)if c.col==0&&c.col_span==shape.1)).expect("마지막 쪽의 전체 열 병합 칸");
        let source_cell = |node: &RenderNode| {
            let RenderNodeType::TableCell(p) = &node.node_type else {
                unreachable!()
            };
            originals[0]
                .cells
                .iter()
                .find(|c| c.row == p.row && c.col == p.col)
                .unwrap()
        };
        assert_eq!(
            source_cell(first_span).width,
            source_cell(last_span).width,
            "두 칸의 원본 선언 열 프레임이 같음"
        );
        assert!(
            (first_span.bbox.width - last_span.bbox.width).abs() < 1e-6,
            "같은 원본 열 프레임의 시작·마지막 칸 폭 보존: {shape:?}, 앞{}, 뒤{}",
            first_span.bbox.width,
            last_span.bbox.width
        );
        assert!(
            ((first_span.bbox.x - first.bbox.x) - (last_span.bbox.x - last.bbox.x)).abs() < 1e-6,
            "표 내부 열 프레임의 상대 원점 보존"
        );
        let mut reference: Option<Vec<(f64, f64)>> = None;
        let mut checked = 0;
        for row in 0..shape.0 {
            let mut cells: Vec<_> = last
                .children
                .iter()
                .filter_map(|node| match &node.node_type {
                    RenderNodeType::TableCell(cell) if cell.row == row && cell.col_span == 1 => {
                        Some((cell.col, node))
                    }
                    _ => None,
                })
                .collect();
            cells.sort_by_key(|x| x.0);
            if cells.len() != usize::from(shape.1) {
                continue;
            }
            assert_eq!(
                cells.iter().map(|x| x.0).collect::<Vec<_>>(),
                (0..shape.1).collect::<Vec<_>>()
            );
            let edges: Vec<_> = cells
                .iter()
                .map(|(_, n)| {
                    (
                        n.bbox.x - last.bbox.x,
                        n.bbox.x + n.bbox.width - last.bbox.x,
                    )
                })
                .collect();
            assert!(
                (cells[0].1.bbox.x - last_span.bbox.x).abs() < 1e-6,
                "첫 열과 병합 칸 왼쪽 경계 일치"
            );
            assert!(
                (cells.last().unwrap().1.bbox.x + cells.last().unwrap().1.bbox.width
                    - last_span.bbox.x
                    - last_span.bbox.width)
                    .abs()
                    < 1e-6,
                "마지막 열과 병합 칸 오른쪽 경계 일치"
            );
            if let Some(ref prior) = reference {
                for (current, previous) in edges.iter().zip(prior) {
                    assert!(
                        (current.0 - previous.0).abs() < 1e-6
                            && (current.1 - previous.1).abs() < 1e-6,
                        "같은 열의 좌우 경계가 행마다 변하지 않음: {shape:?}, row{row}"
                    );
                }
            } else {
                reference = Some(edges);
            }
            checked += 1;
        }
        assert!(checked >= 2, "다른 두 행 이상에서 열별 프레임을 실제 확인");
    }
}

// 고정형: 양쪽 98% 이상인 10쪽의 독립 Print 벡터 괘선 끝점을 사용한다.
#[test]
fn independent_print_page10_keeps_all_grid_edges() {
    let core = open();
    let page = core.build_page_render_tree(9).expect("독립 Print 10쪽");
    let tables: Vec<_> = nodes(&page.root).into_iter().filter(|node| matches!(&node.node_type, RenderNodeType::Table(t) if t.row_count==3 && t.col_count==3)).collect();
    assert_eq!(tables.len(), 1);
    // PDF pt의 선 중심 × 96/72. 원본 원점만 고정한 기존 검사와 달리 각 셀의 끝점을 보호한다.
    let xs = [
        77.39866638183594,
        288.8053385416667,
        500.0506591796875,
        711.4573567708334,
    ];
    let ys = [
        439.8386637369792,
        464.4519856770833,
        490.6626790364583,
        526.7840169270834,
    ];
    let mut checked = 0;
    for node in &tables[0].children {
        let RenderNodeType::TableCell(cell) = &node.node_type else {
            continue;
        };
        let col = usize::from(cell.col);
        let row = usize::from(cell.row);
        for (label, actual, expected) in [
            ("왼쪽", node.bbox.x, xs[col]),
            (
                "오른쪽",
                node.bbox.x + node.bbox.width,
                xs[col + usize::from(cell.col_span)],
            ),
            ("위쪽", node.bbox.y, ys[row]),
            (
                "아래쪽",
                node.bbox.y + node.bbox.height,
                ys[row + usize::from(cell.row_span)],
            ),
        ] {
            assert!(
                (actual - expected).abs() <= 2.0,
                "독립 Print 셀 ({row},{col}) {label}: 실제 {actual}, 독립 {expected}, 허용2px"
            );
        }
        checked += 1;
    }
    assert_eq!(
        checked, 7,
        "독립 Print의 병합 머리와 두 본문 행 셀 모두 검사"
    );
}

// 관계형: 부모 행의 하단 괘선은 현재 컷에 속한 자식 표와 원본 안여백을 포함해야 한다.
#[test]
fn continued_parent_border_contains_owned_nested_frames() {
    let core = open();
    let page = core.build_page_render_tree(5).expect("6쪽 계속 조각");
    let outer: Vec<_> = nodes(&page.root)
        .into_iter()
        .filter(
            |n| matches!(&n.node_type,RenderNodeType::Table(t) if (t.row_count,t.col_count)==(2,2)),
        )
        .collect();
    assert_eq!(outer.len(), 1);
    fn bottom_border(table: &RenderNode) -> f64 {
        table
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::Line(l) if (l.y1 - l.y2).abs() < 1e-6 && l.x1 != l.x2 => Some(l.y1),
                _ => None,
            })
            .reduce(f64::max)
            .expect("실제 수평 괘선")
    }
    let original = source_tables(&core.document().sections[0].paragraphs[15..16]);
    let source = original
        .iter()
        .find(|t| (t.row_count, t.col_count) == (2, 2))
        .unwrap();
    let parent_bottom = bottom_border(outer[0]);
    let mut checked = 0;
    for cell in &outer[0].children {
        let RenderNodeType::TableCell(owner) = &cell.node_type else {
            continue;
        };
        if owner.row != 1 {
            continue;
        }
        let source_cell = source
            .cells
            .iter()
            .find(|c| c.row == owner.row && c.col == owner.col)
            .unwrap();
        let child=nodes(cell).into_iter().find(|n|matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==(7,1))).expect("현재 칸의 안쪽 표");
        let pad = source_cell.effective_padding(&source.padding).bottom as f64 * 96.0 / 7200.0;
        assert!(
            parent_bottom + 1e-6 >= bottom_border(child) + pad,
            "부모 하단 {parent_bottom}이 자식 하단 {}과 원본 안여백 {pad}를 포함해야 함",
            bottom_border(child)
        );
        checked += 1;
    }
    assert_eq!(checked, 2, "양쪽 열의 실제 괘선 확인");
}

// 관계형: 같은 부모 물리 행의 양쪽 자식 프레임은 공통 하단에서 닫힌다.
#[test]
fn sibling_nested_source_frames_close_on_the_parent_row_boundary() {
    let core = open();
    let page = core.build_page_render_tree(5).unwrap();
    let outer = nodes(&page.root)
        .into_iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==(2,2)))
        .unwrap();
    let mut bottoms = Vec::new();
    for cell in &outer.children {
        if !matches!(&cell.node_type,RenderNodeType::TableCell(c)if c.row==1) {
            continue;
        }
        let child=nodes(cell).into_iter().find(|n|matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==(7,1))).unwrap();
        let bottom = child
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::Line(l) if (l.y1 - l.y2).abs() < 1e-6 && l.x1 != l.x2 => Some(l.y1),
                _ => None,
            })
            .reduce(f64::max)
            .unwrap();
        bottoms.push(bottom);
    }
    assert_eq!(bottoms.len(), 2);
    assert!(
        (bottoms[0] - bottoms[1]).abs() < 1e-6,
        "같은 부모 물리 행에서 닫히는 두 자식의 하단은 같아야 함: {bottoms:?}"
    );
}

// 관계형: 선언된 첫 물리 조각 안에서 선택한 본문은 원본 Center 정렬을 지킨다.
#[test]
fn declared_partial_cell_centers_only_its_owned_lines() {
    let core = open();
    let page = core.build_page_render_tree(11).unwrap();
    let outer = nodes(&page.root)
        .into_iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==(7,4)))
        .unwrap();
    let cell = outer
        .children
        .iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==6&&c.col==3))
        .unwrap();
    let Control::Table(source) = &core.document().sections[0].paragraphs[83].controls[0] else {
        panic!("원본 표")
    };
    let original = source
        .cells
        .iter()
        .find(|c| c.row == 6 && c.col == 3)
        .unwrap();
    assert_eq!(
        original.vertical_align,
        rhwp::model::table::VerticalAlign::Center
    );
    let padding = original.effective_padding(&source.padding);
    let scale = 96.0 / 7200.0;
    let runs: Vec<_> = nodes(cell)
        .into_iter()
        .filter(|n| matches!(&n.node_type,RenderNodeType::TextRun(t)if!t.text.trim().is_empty()))
        .collect();
    assert!(runs.len() >= 2);
    let top = runs.iter().map(|n| n.bbox.y).reduce(f64::min).unwrap();
    let bottom = runs
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .reduce(f64::max)
        .unwrap();
    let above = top - cell.bbox.y - padding.top as f64 * scale;
    let below = cell.bbox.y + cell.bbox.height - padding.bottom as f64 * scale - bottom;
    assert!(
        (above - below).abs() <= 2.0 * scale,
        "선택된 글줄 상자의 위아래 공간이 같아야 함: 위 {above}, 아래 {below}"
    );
}

// 고정형: Native99.97875% / fresh WASM99.97413%와 직접 판독을 마친 6쪽의 독립 PDF ±2px.
#[test]
fn independent_print_page6_keeps_the_enclosing_bottom_border() {
    let core = open();
    let page = core.build_page_render_tree(5).unwrap();
    let outer = nodes(&page.root)
        .into_iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==(2,2)))
        .unwrap();
    let (left, right, bottom) = outer
        .children
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Line(line)
                if (line.y1 - line.y2).abs() < 1e-6 && line.x1 != line.x2 =>
            {
                Some((line.x1.min(line.x2), line.x1.max(line.x2), line.y1))
            }
            _ => None,
        })
        .max_by(|a, b| a.2.total_cmp(&b.2))
        .unwrap();
    // 독립 한컴2020 Print PDF 6쪽의 바깥 표 하단 선 중심(pt×96/72).
    for (actual, expected) in [
        (left, 77.39866638183594),
        (right, 717.3746744791666),
        (bottom, 1041.2613118489583),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 하단 괘선: 실제 {actual}, PDF {expected}, 허용2px"
        );
    }
}

// 관계형: 선언 상자가 자식 본체·양쪽 바깥여백·부모 안여백으로 닫힌 TAC 호스트다.
#[test]
fn closed_tac_wrapper_paints_the_outer_top_reserved_by_its_source_frame() {
    let core = open();
    let page = core.build_page_render_tree(8).unwrap();
    let wrapper = nodes(&page.root)
        .into_iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::Table(t)if t.para_index==Some(46)))
        .unwrap();
    let owner = wrapper
        .children
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TableCell(_)))
        .unwrap();
    let child = nodes(owner)
        .into_iter()
        .find(|n| matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==(6,3)))
        .unwrap();
    let Control::Table(source) = &core.document().sections[0].paragraphs[46].controls[0] else {
        panic!("원본 래퍼")
    };
    let cell = &source.cells[0];
    let child_source = source_tables(&cell.paragraphs)
        .into_iter()
        .find(|t| (t.row_count, t.col_count) == (6, 3))
        .unwrap();
    assert!(child_source.common.treat_as_char);
    let pad = cell.effective_padding(&source.padding);
    assert_eq!(
        source.common.height as i32,
        child_source.common.height as i32
            + child_source.outer_margin_top as i32
            + child_source.outer_margin_bottom as i32
            + pad.top as i32
            + pad.bottom as i32,
        "독립 원본 프레임 등식"
    );
    let expected =
        owner.bbox.y + (pad.top as f64 + child_source.outer_margin_top as f64) * 96.0 / 7200.0;
    assert!(
        (child.bbox.y - expected).abs() <= 96.0 / 7200.0,
        "예약한 위 바깥여백: 실제{}, 원본{expected}",
        child.bbox.y
    );
}

// 관계형: 줄 내부 컷이 없는 행 이어받기도 원본 바깥 위 여백을 한 번 소비한다.
#[test]
fn atomic_child_continuation_keeps_its_source_outer_top_margin() {
    let core = open();
    let source = source_tables(&core.document().sections[0].paragraphs[15..16]);
    for page in [5, 6] {
        let tree = core.build_page_render_tree(page).unwrap();
        for shape in [(9, 4), (13, 4)] {
            let child_source = source
                .iter()
                .find(|t| (t.row_count, t.col_count) == shape)
                .unwrap();
            let parent_source=source.iter().find(|t|(t.row_count,t.col_count)==(7,1)&&t.cells.iter().any(|c|c.paragraphs.iter().any(|p|p.controls.iter().any(|c|matches!(c,Control::Table(x)if x.common.instance_id==child_source.common.instance_id))))).unwrap();
            let parent=nodes(&tree.root).into_iter().find(|n|matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==(7,1))&&nodes(n).into_iter().any(|x|matches!(&x.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==shape))).unwrap();
            let owner = parent
                .children
                .iter()
                .find(|n| matches!(&n.node_type,RenderNodeType::TableCell(c)if c.row==6))
                .unwrap();
            let child=nodes(owner).into_iter().find(|n|matches!(&n.node_type,RenderNodeType::Table(t)if(t.row_count,t.col_count)==shape)).unwrap();
            let cell = parent_source.cells.iter().find(|c| c.row == 6).unwrap();
            let pad = cell.effective_padding(&parent_source.padding);
            let expected = owner.bbox.y
                + (pad.top as f64 + child_source.outer_margin_top as f64) * 96.0 / 7200.0;
            assert!(
                (child.bbox.y - expected).abs() <= 96.0 / 7200.0,
                "같은 원본 위 여백: page{}, shape{shape:?}, 실제{}, 원본{}",
                page + 1,
                child.bbox.y,
                expected
            );
        }
    }
}
