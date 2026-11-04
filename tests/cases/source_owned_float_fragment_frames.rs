//! 같은 원본·독립 Print의 Native/fresh 전쪽 검증 후 표와 글줄의 실제 소유를 보호한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

const ASSETS: &str = "mydocs/pr/assets/float_fragment_frames_20261020";
const EXPECTED: &str =
    include_str!("../../mydocs/pr/assets/float_fragment_frames_20261020/expected.json");
const SOURCE_UNIT: f64 = 96.0 / 7200.0;

struct TableView {
    bounds: BoundingBox,
    rows: BTreeSet<u16>,
    cells: Vec<BoundingBox>,
}
struct PageView {
    table: Option<TableView>,
    text: String,
    runs: Vec<(String, BoundingBox)>,
}
struct Case {
    name: String,
    oracle: Value,
    row_count: u16,
    outer_bottom: f64,
    pages: Vec<PageView>,
}
fn normalize(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn walk<'a>(node: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    result.push(node);
    for child in &node.children {
        walk(child, result);
    }
}
fn cases(mode: Option<&str>) -> Vec<Case> {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let mut result = Vec::new();
    for (name, oracle) in expected.as_object().unwrap() {
        let selected = match mode {
            Some("normal") => oracle["source_rows"] == 2,
            Some("genuine") => name == "hancom-saved-public-rowbreak-normalized-2020",
            Some(mode) => oracle["mode"] == mode,
            None => true,
        };
        if !selected {
            continue;
        }
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(ASSETS)
            .join("inputs")
            .join(oracle["format"].as_str().unwrap())
            .join(oracle["input"].as_str().unwrap());
        let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
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
        let (owner, control, original) = owners[0];
        assert_eq!(
            format!("{:?}", original.page_break),
            oracle["policy"].as_str().unwrap()
        );
        assert_eq!(
            u64::from(original.row_count),
            oracle["source_rows"].as_u64().unwrap()
        );
        assert_eq!(
            core.page_count() as usize,
            oracle["pages"].as_array().unwrap().len(),
            "{name}: 독립 Print 전체 쪽수"
        );
        if name == "hancom-saved-public-rowbreak-normalized-2020" {
            assert_eq!(source[owner].line_segs.len(), 1);
            assert_eq!(source[owner].line_segs[0].segment_width, 0);
            assert_eq!(source[owner].line_segs[0].tag & 0x8000_0000, 0);
            let cell = &original.cells[0];
            assert!(cell.paragraphs[0].text.is_empty());
            assert!(
                cell.paragraphs[0].line_segs[0].line_height > cell.height as i32,
                "잉크 없는 실제 저장 글줄도 최소 셀 높이보다 큰 공간을 점유한다"
            );
        }
        let source_text = source
            .iter()
            .map(|p| normalize(&p.text))
            .collect::<String>();
        let mut pages = Vec::new();
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut nodes = Vec::new();
            walk(&tree.root, &mut nodes);
            let tables = nodes
                .iter()
                .filter(|node| {
                    matches!(&node.node_type, RenderNodeType::Table(table)
                    if table.section_index == Some(0) && table.para_index == Some(owner)
                    && table.control_index == Some(control) && table.cell_context.is_none())
                })
                .collect::<Vec<_>>();
            assert!(tables.len() <= 1, "{name}: 같은 쪽의 표 중복");
            let table = tables.first().map(|node| {
                let RenderNodeType::Table(data) = &node.node_type else {
                    unreachable!()
                };
                assert_eq!(data.col_count, original.col_count);
                let cells = node
                    .children
                    .iter()
                    .filter_map(|child| {
                        if let RenderNodeType::TableCell(cell) = &child.node_type {
                            Some((cell.row, child.bbox))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                assert!(!cells.is_empty(), "{name}: 표를 빈 상자로 대체하지 않는다");
                TableView {
                    bounds: node.bbox,
                    rows: cells.iter().map(|c| c.0).collect(),
                    cells: cells.into_iter().map(|c| c.1).collect(),
                }
            });
            let runs = nodes
                .iter()
                .filter_map(|node| {
                    if let RenderNodeType::TextRun(run) = &node.node_type {
                        let text = normalize(&run.text);
                        (!text.is_empty()).then_some((text, node.bbox))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let text = runs.iter().map(|r| r.0.as_str()).collect::<String>();
            assert_eq!(
                text,
                normalize(oracle["pages"][page as usize]["text"].as_str().unwrap()),
                "{name}: 쪽별 본문 소유·순서·누락·중복"
            );
            assert_eq!(
                table.is_some(),
                oracle["pages"][page as usize]["table"].as_bool().unwrap(),
                "{name}: 독립 Print의 표 쪽 소유"
            );
            pages.push(PageView { table, text, runs });
        }
        assert_eq!(
            pages.iter().map(|p| p.text.as_str()).collect::<String>(),
            source_text,
            "{name}: 전체 원문 보존"
        );
        result.push(Case {
            name: name.clone(),
            oracle: oracle.clone(),
            row_count: original.row_count,
            outer_bottom: rhwp::renderer::hwpunit_to_px(
                i32::from(original.outer_margin_bottom),
                96.0,
            ),
            pages,
        });
    }
    assert!(!result.is_empty());
    result
}
fn assert_owned_cells(case: &Case) {
    let mut consumed = BTreeSet::new();
    for page in &case.pages {
        if let Some(table) = &page.table {
            assert!(table.rows.is_disjoint(&consumed), "{}: 행 중복", case.name);
            consumed.extend(table.rows.iter().copied());
            for cell in &table.cells {
                assert!(
                    cell.x + SOURCE_UNIT >= table.bounds.x
                        && cell.y + SOURCE_UNIT >= table.bounds.y
                        && cell.x + cell.width <= table.bounds.x + table.bounds.width + SOURCE_UNIT
                        && cell.y + cell.height
                            <= table.bounds.y + table.bounds.height + SOURCE_UNIT,
                    "{}: 셀의 실제 점유를 표 상자 안에 보존",
                    case.name
                );
            }
        }
    }
    assert_eq!(
        consumed,
        (0..case.row_count).collect(),
        "{}: 원본 행 누락",
        case.name
    );
}
#[test]
fn unrestricted_no_split_keeps_all_rows_on_one_owned_page() {
    let cases = cases(Some("unrestricted"));
    assert_eq!(cases.len(), 7);
    for case in cases {
        assert_owned_cells(&case);
        assert_eq!(
            case.pages.iter().filter(|p| p.table.is_some()).count(),
            1,
            "{}: 나누지 않는 표",
            case.name
        );
    }
}
#[test]
fn restricted_no_split_keeps_host_and_successor_before_the_deferred_whole_table() {
    let cases = cases(Some("restricted"));
    assert_eq!(cases.len(), 8);
    for case in cases {
        assert_owned_cells(&case);
        assert_eq!(case.pages.iter().filter(|p| p.table.is_some()).count(), 1);
        if case.pages.len() > 1 {
            assert!(case.pages[0].table.is_none());
            assert!(case.pages[0].text.contains("다음문단"));
            assert_eq!(
                case.pages[1].table.as_ref().unwrap().rows.len(),
                usize::from(case.row_count)
            );
            assert!(
                case.pages[1].text.is_empty(),
                "{}: 이월 표 쪽에 호스트·뒤 본문을 중복하지 않는다",
                case.name
            );
        }
    }
}
#[test]
fn row_and_cell_break_preserve_the_independent_print_row_partition() {
    let cases = cases(Some("split"));
    assert_eq!(cases.len(), 21);
    for case in cases {
        assert_owned_cells(&case);
        let mut start = 0u16;
        for (page, expected) in case
            .pages
            .iter()
            .zip(case.oracle["pages"].as_array().unwrap())
        {
            let count = expected["grid_rows"].as_u64().unwrap() as u16;
            let table = page.table.as_ref().unwrap();
            assert_eq!(
                table.rows,
                (start..start + count).collect(),
                "{}: PDF 조각의 행 소유",
                case.name
            );
            start += count;
        }
        assert_eq!(start, case.row_count);
    }
}
#[test]
fn genuine_saved_empty_cell_fragment_keeps_successor_spacing_inside_its_source_frame() {
    let case = cases(Some("genuine"))
        .into_iter()
        .find(|c| c.name == "hancom-saved-public-rowbreak-normalized-2020")
        .unwrap();
    let page = case.pages.last().unwrap();
    let table = page.table.as_ref().unwrap();
    let line = page.runs.iter().find(|r| r.0 == "다음문단").unwrap();
    let gap = line.1.y - (table.bounds.y + table.bounds.height);
    assert!(
        gap >= -SOURCE_UNIT && gap <= case.outer_bottom + SOURCE_UNIT,
        "뒤 문단 앞 간격을 실제 마지막 조각 뒤에서 중복 소비하지 않는다: {gap}"
    );
}
#[test]
fn fitting_normal_controls_preserve_table_cells_and_following_text() {
    let normal = cases(Some("normal"));
    assert_eq!(normal.len(), 8);
    for case in normal {
        assert_eq!(case.pages.len(), 1);
        assert_owned_cells(&case);
        let page = &case.pages[0];
        let table = page.table.as_ref().unwrap();
        let next = page.runs.iter().find(|r| r.0 == "다음문단").unwrap();
        assert!(
            next.1.y + SOURCE_UNIT >= table.bounds.y + table.bounds.height,
            "{}: 정상 뒤 본문 비겹침",
            case.name
        );
    }
}
#[test]
fn completed_pages_keep_independent_print_grid_and_body_origins_within_two_pixels() {
    let cases = cases(None);
    assert_eq!(cases.len(), 36);
    for case in cases {
        for (page, expected) in case
            .pages
            .iter()
            .zip(case.oracle["pages"].as_array().unwrap())
        {
            if let Some(table) = &page.table {
                for (actual, field) in [
                    (table.bounds.x, "grid_left"),
                    (table.bounds.y, "first_grid_y"),
                    (table.bounds.x + table.bounds.width, "grid_right"),
                ] {
                    assert!(
                        (actual - expected[field].as_f64().unwrap()).abs() <= 2.0,
                        "{}: 독립 Print {field}",
                        case.name
                    );
                }
                if case.oracle["mode"] == "split" {
                    assert!(
                        (table.bounds.y + table.bounds.height
                            - expected["last_grid_y"].as_f64().unwrap())
                        .abs()
                            <= 2.0,
                        "{}: 독립 Print의 마지막 행 경계",
                        case.name
                    );
                }
            }
            for span in expected["spans"].as_array().unwrap() {
                let wanted = normalize(span["text"].as_str().unwrap());
                let (text, bounds) = page
                    .runs
                    .iter()
                    .find(|r| r.0 == wanted)
                    .expect("독립 Print 본문");
                assert_eq!(text, &wanted);
                for (actual, index) in [(bounds.x, 0), (bounds.y, 1)] {
                    assert!(
                        (actual - span["bbox"][index].as_f64().unwrap()).abs() <= 2.0,
                        "{}: 독립 Print 본문 {wanted}",
                        case.name
                    );
                }
            }
        }
    }
}
