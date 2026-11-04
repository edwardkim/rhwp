//! 계산 빈 호스트·표·뒤 문단은 각자의 줄과 물리 상자를 보존한다.
//! 같은 입력의 Native/fresh 전쪽100%와 독립 Print는 아래 asset에 보존한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use serde_json::Value;
use std::path::Path;

const ASSETS: &str = "mydocs/pr/assets/computed_empty_host_line_20261020";
const EXPECTED: &str =
    include_str!("../../mydocs/pr/assets/computed_empty_host_line_20261020/expected.json");

struct Placed {
    name: String,
    expected: Value,
    table: BoundingBox,
    prefix: BoundingBox,
    successor: BoundingBox,
    outer_bottom: f64,
    successor_font: f64,
}

fn normalized(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn cases(mode: Option<&str>, count: usize) -> Vec<Placed> {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let mut result = Vec::new();
    for (name, oracle) in expected.as_object().unwrap() {
        if mode.is_some_and(|mode| oracle["mode"] != mode) {
            continue;
        }
        let bytes = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(ASSETS)
                .join("inputs")
                .join(oracle["input"].as_str().unwrap()),
        )
        .unwrap();
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(core.page_count(), 1, "{name}: 독립 Print의 전체 쪽수");
        let source = &core.document().sections[0].paragraphs;
        let owners = source
            .iter()
            .enumerate()
            .flat_map(|(pi, para)| {
                para.controls
                    .iter()
                    .enumerate()
                    .filter_map(move |(ci, control)| {
                        if let Control::Table(table) = control {
                            Some((pi, ci, table))
                        } else {
                            None
                        }
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(owners.len(), 1, "{name}: 원문 표 소유");
        let (pi, ci, original) = owners[0];
        assert!(source[pi].text.is_empty());
        let source_text = source
            .iter()
            .map(|p| normalized(&p.text))
            .collect::<String>();
        assert_eq!(source_text, oracle["source_text"].as_str().unwrap());
        let tree = core.build_page_render_tree(0).unwrap();
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        let tables = nodes
            .iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                if table.section_index == Some(0) && table.para_index == Some(pi)
                    && table.control_index == Some(ci) && table.cell_context.is_none())
            })
            .collect::<Vec<_>>();
        assert_eq!(tables.len(), 1, "{name}: 표 누락·중복·불필요한 분할");
        let RenderNodeType::Table(table) = &tables[0].node_type else {
            unreachable!()
        };
        assert_eq!(table.row_count, original.row_count);
        assert_eq!(table.col_count, original.col_count);
        let text = nodes
            .iter()
            .filter_map(|node| {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    (!run.text.is_empty()).then_some((normalized(&run.text), node.bbox, &run.style))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            text.iter().map(|t| t.0.as_str()).collect::<String>(),
            source_text,
            "{name}: 본문 내용 순서·누락·중복"
        );
        assert_eq!(text.len(), 2, "{name}: 앞·뒤 문단의 실제 글줄");
        assert_eq!(text[0].0, "앞");
        assert!(text[1].2.font_size > 0.0, "{name}: 원문 본문 글자 크기");
        assert!(
            text[1].1.y >= text[0].1.y + text[0].1.height,
            "{name}: 앞 문단과 뒤 문단의 비겹침"
        );
        result.push(Placed {
            name: name.clone(),
            expected: oracle.clone(),
            table: tables[0].bbox,
            prefix: text[0].1,
            successor: text[1].1,
            outer_bottom: rhwp::renderer::hwpunit_to_px(
                i32::from(original.outer_margin_bottom),
                96.0,
            ),
            successor_font: text[1].2.font_size,
        });
    }
    assert_eq!(result.len(), count, "독립 Print로 검증한 입력의 수");
    result
}

#[test]
fn short_empty_host_keeps_the_successor_line_in_the_gap_above_the_float() {
    for p in cases(Some("gap"), 2) {
        assert!(
            p.successor.y + p.successor.height <= p.table.y,
            "{}: 들어가는 글줄을 표 아래로 밀지 않는다",
            p.name
        );
    }
}

#[test]
fn tall_empty_host_preserves_its_own_line_box_beyond_the_float() {
    for p in cases(Some("tall"), 4) {
        assert!(
            p.successor.y > p.table.y + p.table.height + p.outer_bottom + p.successor_font,
            "{}: 큰 빈 줄을 표 상자 높이로 축소하지 않는다",
            p.name
        );
    }
}

#[test]
fn ordinary_host_moves_an_overlapping_successor_line_below_the_float_once() {
    for p in cases(Some("clear"), 6) {
        let gap = p.successor.y - p.table.y - p.table.height;
        assert!(
            gap >= 0.0 && gap <= p.outer_bottom + p.successor_font / 4.0,
            "{}: 실제 표 아래 여백의 점유·본문 비겹침",
            p.name
        );
    }
}

#[test]
fn successor_before_spacing_inside_the_float_reservation_is_not_added_again() {
    for p in cases(Some("spacing"), 2) {
        let gap = p.successor.y - p.table.y - p.table.height;
        assert!(
            gap >= 0.0 && gap <= p.outer_bottom + p.successor_font / 4.0,
            "{}: 앞 간격을 표 하단 뒤에 중복 계상하지 않는다",
            p.name
        );
    }
}

#[test]
fn completed_pages_keep_independent_print_grid_and_successor_coordinates() {
    // 양쪽 raw100%·96DPI 종이 원점. PDF의 vector 선 중심과 텍스트 잉크 bbox다.
    // font box와 잉크의 차이는 허용하며, 관계 검사는 위에서 별도로 유지한다.
    for p in cases(None, 14) {
        for (actual, field) in [
            (p.table.x, "table_left"),
            (p.table.x + p.table.width, "table_right"),
            (p.table.y, "table_top"),
            (p.table.y + p.table.height, "table_bottom"),
            (p.successor.y, "successor_top"),
        ] {
            assert!(
                (actual - p.expected[field].as_f64().unwrap()).abs() <= 2.0,
                "{}: 독립 Print의 {field}, actual={actual}",
                p.name
            );
        }
        assert!(
            (p.prefix.x - p.successor.x).abs() <= rhwp::renderer::hwpunit_to_px(1, 96.0),
            "{}: 앞뒤 본문의 같은 왼쪽 여백",
            p.name
        );
    }
}
