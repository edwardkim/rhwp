//! #7728: 계속 표의 열 프레임과 독립 Print의 셀 괘선 끝점을 보호한다.
//! 주석 갱신: 2026-11-06T14:16:32+09:00.
//! 검사 분류: `continuation_column_frames_keep_opening_span_and_row_edges`는 관계형이다.
//! 검사 분류: `independent_print_page10_keeps_all_grid_edges`는 고정형(독립 PDF ±2 CSSpx)이다.
//! 원본 70833 HWP와 독립 한컴2020 Print PDF, compat2022,96DPI/PDFium/2px raw.
//! 코드04f03cfa 전18쪽 Native 최저94.62153%, fresh WASM 최저94.27517%, 미달·누락0.
//! 관계형 대상6쪽 N/W94.62153/94.82126%,7쪽94.79966/94.27517%: 절대 좌표를 고정하지 않는다.
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
