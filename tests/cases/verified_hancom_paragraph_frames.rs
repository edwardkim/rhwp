//! 한컴 Print로 검증한 국어 HWP/HWPX의 문단 외곽·글줄·인라인 도형 소속.
//! 전쪽 Native/fresh WASM 90% 이상 및 14쪽 양쪽 98% 이상과 직접 판독을 확인했다.
//! 독립 좌표·입력/PDF SHA·TSV·대표 PNG: mydocs/working/task_m100_7642_stage1781.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const INPUTS: [&str; 2] = ["samples/exam_kor.hwp", "samples/hwpx/exam_kor.hwpx"];

fn page(input: &str) -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(input);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 20, "독립 Print의 전체 쪽수");
    core.build_page_render_tree(13).unwrap().root
}

fn collect<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        collect(c, out);
    }
}

fn contains_text(n: &RenderNode, text: &str) -> bool {
    matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text.contains(text))
        || n.children.iter().any(|c| contains_text(c, text))
}

fn passage_edges(root: &RenderNode) -> Vec<(&RenderNode, f64)> {
    let mut nodes = vec![];
    collect(root, &mut nodes);
    let columns: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|n| matches!(n.node_type, RenderNodeType::Column(_)))
        .collect();
    assert_eq!(columns.len(), 2, "두 본문 단의 소속 보존");
    columns
        .into_iter()
        .map(|column| {
            let edges: Vec<_> = column
                .children
                .iter()
                .filter_map(|n| {
                    let RenderNodeType::Line(l) = &n.node_type else {
                        return None;
                    };
                    ((l.x1 - l.x2).abs() < 1e-6
                        && (l.y2 - l.y1).abs() > column.bbox.height / 2.0
                        && l.x1 > column.bbox.x + column.bbox.width / 2.0)
                        .then_some(l.x1)
                })
                .collect();
            assert_eq!(edges.len(), 1, "각 지문의 오른쪽 외곽선 하나");
            (column, edges[0])
        })
        .collect()
}

fn header_shape(root: &RenderNode) -> (&RenderNode, &RenderNode) {
    let mut nodes = vec![];
    collect(root, &mut nodes);
    let cells: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::TableCell(_)) && contains_text(n, "홀수형")
        })
        .collect();
    assert_eq!(cells.len(), 1, "홀수형은 한 부모 셀에 한 번만 표시");
    let cell = cells[0];
    let shapes: Vec<_> = cell
        .children
        .iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::Rectangle(_)) && contains_text(n, "홀수형")
        })
        .collect();
    assert_eq!(shapes.len(), 1);
    let shape = shapes[0];
    let RenderNodeType::Rectangle(r) = &shape.node_type else {
        unreachable!()
    };
    let line=cell.children.iter().find(|n|matches!(&n.node_type,
        RenderNodeType::TextLine(l) if l.para_index==r.cell_para_index && l.line_index==Some(0)
    )).expect("도형을 소유한 셀 문단의 첫 글줄");
    (shape, line)
}

#[test]
fn verified_exam_passage_border_keeps_column_frame_around_quote_tables() {
    for input in INPUTS {
        let root = page(input);
        assert!(contains_text(&root, "얘들아"));
        assert!(contains_text(&root, "교훈"));
        for (column, right) in passage_edges(&root) {
            assert!(
                (right - column.bbox.x - column.bbox.width).abs() < 1e-6,
                "인용 표의 오프셋 추정으로 단 외곽을 넓히지 않는다: {input}"
            );
            let tables: Vec<_> = column
                .children
                .iter()
                .filter(|n| matches!(n.node_type, RenderNodeType::Table(_)))
                .collect();
            assert!(
                !tables.is_empty(),
                "인용 표가 사라져 검사가 통과하면 안 된다"
            );
            for table in tables {
                assert!(table.bbox.x >= column.bbox.x);
                assert!(table.bbox.x + table.bbox.width <= right);
            }
        }
    }
}

#[test]
fn verified_exam_inline_header_shape_stays_in_its_owner_line() {
    for input in INPUTS {
        let root = page(input);
        let (shape, line) = header_shape(&root);
        assert!(
            shape.bbox.y >= line.bbox.y,
            "도형이 이전 문단 원점으로 되돌아가면 안 된다: {input}"
        );
        assert!(
            shape.bbox.y + shape.bbox.height <= line.bbox.y + line.bbox.height,
            "독립 Print에서 확인한 도형의 자기 글줄 포함 관계: {input}"
        );
        assert!(shape.bbox.x >= line.bbox.x);
    }
}

#[test]
fn verified_exam_original_font_position_keeps_body_and_punctuation_baselines() {
    for input in INPUTS {
        let root = page(input);
        let mut nodes = vec![];
        collect(&root, &mut nodes);
        let line = nodes
            .iter()
            .find(|n| {
                matches!(&n.node_type,
                    RenderNodeType::TextLine(l) if l.section_index==Some(1)
                        && l.para_index==Some(30) && l.line_index==Some(0)
                )
            })
            .expect("학생 대화 첫 줄");
        let mut baselines = vec![];
        let mut body = false;
        let mut punctuation = false;
        for n in &line.children {
            let RenderNodeType::TextRun(r) = &n.node_type else {
                continue;
            };
            if !matches!(r.style.font_family.as_str(), "HY신명조" | "한양신명조")
                || r.text.trim().is_empty()
            {
                continue;
            }
            assert!(!r.style.superscript && !r.style.subscript);
            assert_eq!(r.style.font_position, Some(-10));
            body |= r.style.font_family == "HY신명조";
            punctuation |= r.style.font_family == "한양신명조";
            baselines.push(
                r.style
                    .script_draw_metrics(r.style.font_size, n.bbox.y + r.baseline)
                    .1,
            );
        }
        assert!(
            body && punctuation,
            "서로 다른 원본 face의 본문·구두점 보존"
        );
        let min = baselines.iter().copied().reduce(f64::min).unwrap();
        let max = baselines.iter().copied().reduce(f64::max).unwrap();
        assert!(
            (max - min).abs() < 1e-6,
            "독립 Print의 같은 줄 기준선을 글꼴 이름으로 갈라놓지 않는다"
        );
    }
}

// 14쪽은 Native 99.74686%, fresh WASM 99.72217%와 확대 판독을 완료했다.
// 96 dpi Print의 독립 외곽 좌표를 ±2 CSS px로 보호하고 위 관계 검사도 유지한다.
#[test]
fn verified_exam_completed_page14_print_header_and_frame_positions() {
    for input in INPUTS {
        let root = page(input);
        let (shape, _) = header_shape(&root);
        let close = |actual: f64, expected: f64| {
            assert!(
                (actual - expected).abs() <= 2.0,
                "한컴 Print 96 dpi의 독립 좌표 ±2 CSS px: {actual} vs {expected}: {input}"
            )
        };
        close(shape.bbox.x, 922.9906);
        close(shape.bbox.y, 148.7347);
        close(shape.bbox.width, 79.1134);
        close(shape.bbox.height, 39.6626);
        let edges = passage_edges(&root);
        close(edges[0].1, 539.5);
        close(edges[1].1, 1004.5);
    }
}
