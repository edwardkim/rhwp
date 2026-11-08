//! 현재 글줄·표 조각·뒤 문단의 소유와 내용 보존을 독립 Print로 보호한다.
//! 전쪽 Native/fresh >=90%, 좌표 대상 쪽 양쪽 >=98%와 직접 판독 증거:
//! mydocs/pr/assets/computed_split_host_20261022/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

const ASSETS: &str = "mydocs/pr/assets/computed_split_host_20261022";
const EVIDENCE: &str =
    include_str!("../../mydocs/pr/assets/computed_split_host_20261022/expected.json");

#[derive(Clone)]
struct Text {
    page: usize,
    value: String,
    bounds: BoundingBox,
}

struct Observed {
    oracle: Value,
    host: Vec<Text>,
    following: Vec<Text>,
    tables: Vec<(usize, BoundingBox)>,
    first_cells: Vec<(usize, BoundingBox)>,
    column_tops: BTreeMap<usize, f64>,
    outer_top: f64,
}

fn compact(value: &str) -> String {
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

fn descendants(node: &RenderNode) -> Vec<&RenderNode> {
    let mut nodes = vec![node];
    for child in &node.children {
        nodes.extend(descendants(child));
    }
    nodes
}

impl Observed {
    fn find_host(&self, needle: &str) -> &Text {
        let found: Vec<_> = self
            .host
            .iter()
            .filter(|t| t.value.contains(needle))
            .collect();
        assert_eq!(found.len(), 1, "호스트의 누락·중복: {needle}");
        found[0]
    }
}

fn observe(name: &str) -> Observed {
    let expected: Value = serde_json::from_str(EVIDENCE).unwrap();
    let oracle = expected[name].clone();
    assert!(oracle.is_object(), "독립 Print 계약: {name}");
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(ASSETS)
            .join(oracle["input"].as_str().unwrap()),
    )
    .expect("검증한 원본 fixture");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let paragraphs = &core.document().sections[0].paragraphs;
    let hosts: Vec<_> = paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.text.contains("국민적 관심이 큰"))
        .collect();
    assert_eq!(hosts.len(), 1, "원본의 논리 호스트");
    let (pi, source) = hosts[0];
    let owners: Vec<_> = source
        .controls
        .iter()
        .enumerate()
        .filter_map(|(ci, c)| {
            if let Control::Table(table) = c {
                Some((ci, table))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(owners.len(), 1, "호스트 표 소유");
    let (ci, table) = owners[0];
    let expected_cells: BTreeMap<_, _> = table
        .cells
        .iter()
        .map(|cell| {
            (
                (cell.row, cell.col),
                compact(
                    &cell
                        .paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<String>(),
                ),
            )
        })
        .collect();
    let expected_host = compact(&source.text);
    let expected_following = oracle
        .get("successor_page")
        .map(|_| compact(&paragraphs[pi + 1].text));
    let mut out = Observed {
        oracle,
        host: Vec::new(),
        following: Vec::new(),
        tables: Vec::new(),
        first_cells: Vec::new(),
        column_tops: BTreeMap::new(),
        outer_top: rhwp::renderer::hwpunit_to_px(i32::from(table.outer_margin_top), 96.0),
    };
    assert_eq!(
        core.page_count() as usize,
        out.oracle["pages"].as_u64().unwrap() as usize,
        "{name}: 독립 Print의 전체 쪽수"
    );
    let mut actual_cells: BTreeMap<(u16, u16), String> = BTreeMap::new();
    for index in 0..core.page_count() {
        let page = (index + 1) as usize;
        let tree = core.build_page_render_tree(index).unwrap();
        let bodies: Vec<_> = descendants(&tree.root)
            .into_iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
            .collect();
        assert_eq!(bodies.len(), 1, "본문 소유");
        let nodes = descendants(bodies[0]);
        for node in &nodes {
            if matches!(node.node_type, RenderNodeType::Column(_)) {
                out.column_tops.insert(page, node.bbox.y);
            }
            if let RenderNodeType::Table(meta) = &node.node_type {
                if meta.para_index != Some(pi)
                    || meta.control_index != Some(ci)
                    || meta.cell_context.is_some()
                {
                    continue;
                }
                assert_eq!(
                    (meta.row_count, meta.col_count),
                    (table.row_count, table.col_count)
                );
                out.tables.push((page, node.bbox));
                for cell in &node.children {
                    let RenderNodeType::TableCell(meta) = &cell.node_type else {
                        continue;
                    };
                    actual_cells
                        .entry((meta.row, meta.col))
                        .or_default()
                        .push_str(&compact(&text(cell)));
                    if meta.row == 2 && meta.col == 0 {
                        out.first_cells.push((page, cell.bbox));
                    }
                    for text in descendants(cell) {
                        let RenderNodeType::TextRun(run) = &text.node_type else {
                            continue;
                        };
                        if compact(&run.text).is_empty() {
                            continue;
                        }
                        // 원본 셀 패딩과 실제 글줄의 포함 관계다. 화면 영역을 고정하지 않는다.
                        let tolerance = rhwp::renderer::hwpunit_to_px(1, 96.0);
                        assert!(
                            text.bbox.y + tolerance >= cell.bbox.y
                                && text.bbox.y + text.bbox.height
                                    <= cell.bbox.y + cell.bbox.height + tolerance,
                            "{name}: {page}쪽 셀({},{})의 글줄 포함",
                            meta.row,
                            meta.col
                        );
                    }
                }
            }
        }
        fn body_text(node: &RenderNode, pi: usize, page: usize, out: &mut Observed) {
            if matches!(node.node_type, RenderNodeType::Table(_)) {
                return;
            }
            if let RenderNodeType::TextRun(run) = &node.node_type {
                let placed = Text {
                    page,
                    value: run.text.clone(),
                    bounds: node.bbox,
                };
                if run.para_index == Some(pi) {
                    out.host.push(placed);
                } else if run.para_index == Some(pi + 1) {
                    out.following.push(placed);
                }
            }
            for child in &node.children {
                body_text(child, pi, page, out);
            }
        }
        body_text(bodies[0], pi, page, &mut out);
    }
    assert_eq!(
        actual_cells, expected_cells,
        "{name}: 셀 주소별 내용 보존·누락·중복"
    );
    assert_eq!(
        compact(
            &out.host
                .iter()
                .map(|t| t.value.as_str())
                .collect::<String>()
        ),
        expected_host,
        "{name}: 호스트 전체 내용 순서·누락·중복"
    );
    if let Some(value) = expected_following {
        assert_eq!(
            compact(
                &out.following
                    .iter()
                    .map(|t| t.value.as_str())
                    .collect::<String>()
            ),
            value,
            "{name}: 후속 문단 보존·누락·중복"
        );
    }
    let pages: Vec<_> = out.tables.iter().map(|(p, _)| *p).collect();
    let expected_pages: Vec<_> = out.oracle["table_pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    assert_eq!(pages, expected_pages, "{name}: 표 조각의 쪽 소속·중복");
    let prefix = out.find_host("국민적 관심");
    let tail = out.find_host("선대책 발표");
    assert_eq!(
        prefix.page,
        out.oracle["prefix_page"].as_u64().unwrap() as usize
    );
    assert_eq!(
        tail.page,
        out.oracle["tail_page"].as_u64().unwrap() as usize
    );
    for (page, frame) in &out.tables {
        if *page == prefix.page {
            assert!(
                prefix.bounds.y + prefix.bounds.height <= frame.y,
                "{name}: 앞 글줄과 표 조각의 비겹침"
            );
        }
        if *page == tail.page {
            assert!(
                frame.y + frame.height <= tail.bounds.y,
                "{name}: 표 조각 뒤 호스트 글줄의 비겹침"
            );
        }
    }
    out
}

#[test]
fn original_computed_host_preserves_both_text_ranges_around_its_table_once() {
    observe("original");
}

#[test]
fn computed_host_keeps_table_and_tail_together_when_the_group_fits_a_fresh_page() {
    for name in ["body-budget-160", "body-budget-122"] {
        let out = observe(name);
        assert_eq!(out.tables.len(), 1);
        assert_eq!(out.tables[0].0, out.find_host("선대책 발표").page);
    }
}

#[test]
fn computed_host_resumes_its_tail_once_after_the_last_row_fragment() {
    let out = observe("body-budget-85");
    assert_eq!(
        out.first_cells.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        [1, 2]
    );
    let following = out
        .following
        .iter()
        .find(|t| !compact(&t.value).is_empty())
        .unwrap();
    assert_eq!(
        following.page,
        out.oracle["successor_page"].as_u64().unwrap() as usize
    );
    assert!(
        following.page > out.find_host("선대책 발표").page,
        "마지막 조각과 호스트 뒤의 후속 본문 소유"
    );
}

#[test]
fn computed_host_preserves_cell_boundaries_and_repeated_fragment_margins() {
    let out = observe("cell-boundary");
    assert_eq!(out.first_cells.len(), 1, "나누지 않은 셀의 단일 소유");
    assert_eq!(out.first_cells[0].0, 2);
    let top = out.column_tops[&2];
    assert!(
        out.first_cells[0].1.y >= top + out.outer_top - rhwp::renderer::hwpunit_to_px(1, 96.0),
        "이어받은 셀 조각은 원본의 위 바깥 여백을 다시 연다"
    );
}

#[test]
fn unbroken_float_leaves_the_host_and_successor_on_the_original_page() {
    let out = observe("unbroken-table");
    let tail = out.find_host("선대책 발표");
    assert!(tail.page < out.tables[0].0);
    assert_eq!(
        out.following
            .iter()
            .filter(|t| !compact(&t.value).is_empty())
            .map(|t| t.page)
            .collect::<Vec<_>>(),
        [tail.page]
    );
    let following = out
        .following
        .iter()
        .find(|t| !compact(&t.value).is_empty())
        .unwrap();
    assert!(following.bounds.y >= tail.bounds.y + tail.bounds.height);
}

#[test]
fn completed_split_host_pages_keep_independent_print_border_coordinates() {
    for name in [
        "body-budget-160",
        "body-budget-122",
        "body-budget-85",
        "cell-boundary",
        "unbroken-table",
    ] {
        let out = observe(name);
        assert!(
            out.oracle["native_minimum"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()
                >= 98.0
        );
        assert!(
            out.oracle["wasm_minimum"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()
                >= 98.0
        );
        for expected in out.oracle["first_column_frames"].as_array().unwrap() {
            let page = expected["page"].as_u64().unwrap() as usize;
            let found: Vec<_> = out.first_cells.iter().filter(|(p, _)| *p == page).collect();
            assert_eq!(found.len(), 1);
            let bounds = found[0].1;
            for (actual, key) in [
                (bounds.x, "left"),
                (bounds.x + bounds.width, "right"),
                (bounds.y, "top"),
                (bounds.y + bounds.height, "bottom"),
            ] {
                assert!(
                    (actual - expected[key].as_f64().unwrap()).abs() <= 2.0,
                    "{name} {page}쪽 독립 Print {key}: {actual}"
                );
            }
        }
    }
}
