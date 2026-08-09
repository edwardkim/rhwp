//! Issue #4252: 재귀 분할된 중첩 표가 합성 `(para=0, control=0)`을
//! `TextRun.cell_context`에 노출해 Studio 표 객체 선택이 실패했다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::renderer::layout::{CellContext, CellPathEntry};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const FIXTURE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";

fn control_kind(control: &Control) -> &'static str {
    match control {
        Control::Table(_) => "Table",
        Control::Shape(_) => "Shape",
        Control::Picture(_) => "Picture",
        Control::SectionDef(_) => "SectionDef",
        _ => "Other",
    }
}

fn fixture_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn resolve_table_path(
    document: &Document,
    section_index: usize,
    parent_para_index: usize,
    path: &[CellPathEntry],
) -> Result<(), String> {
    let section = document
        .sections
        .get(section_index)
        .ok_or_else(|| format!("section[{section_index}] 범위 초과"))?;
    let mut paragraph = section
        .paragraphs
        .get(parent_para_index)
        .ok_or_else(|| format!("paragraph[{parent_para_index}] 범위 초과"))?;

    for (depth, entry) in path.iter().enumerate() {
        let control = paragraph
            .controls
            .get(entry.control_index)
            .ok_or_else(|| format!("path[{depth}] controls[{}] 범위 초과", entry.control_index))?;
        if depth + 1 == path.len() {
            return if matches!(control, Control::Table(_)) {
                Ok(())
            } else {
                Err(format!(
                    "path[{depth}] controls[{}]가 표가 아님: {}",
                    entry.control_index,
                    control_kind(control)
                ))
            };
        }

        let Control::Table(table) = control else {
            return Err(format!(
                "path[{depth}] controls[{}]가 중간 표가 아님: {}",
                entry.control_index,
                control_kind(control)
            ));
        };
        let cell = table
            .cells
            .get(entry.cell_index)
            .ok_or_else(|| format!("path[{depth}] cells[{}] 범위 초과", entry.cell_index))?;
        paragraph = cell.paragraphs.get(entry.cell_para_index).ok_or_else(|| {
            format!(
                "path[{depth}] cell paragraphs[{}] 범위 초과",
                entry.cell_para_index
            )
        })?;
    }
    Err("빈 cell path".to_string())
}

fn visit_runs(node: &RenderNode, visit: &mut impl FnMut(&str, &CellContext)) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let Some(context) = &run.cell_context {
            visit(&run.text, context);
        }
    }
    for child in &node.children {
        visit_runs(child, visit);
    }
}

fn visit_runs_with_node(node: &RenderNode, visit: &mut impl FnMut(&RenderNode, &CellContext)) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let Some(context) = &run.cell_context {
            visit(node, context);
        }
    }
    for child in &node.children {
        visit_runs_with_node(child, visit);
    }
}

fn json_cell_path(hit: &Value) -> Vec<(usize, usize, usize)> {
    hit["cellPath"]
        .as_array()
        .expect("hit cellPath array")
        .iter()
        .map(|entry| {
            (
                entry["controlIndex"].as_u64().expect("controlIndex") as usize,
                entry["cellIndex"].as_u64().expect("cellIndex") as usize,
                entry["cellParaIndex"].as_u64().expect("cellParaIndex") as usize,
            )
        })
        .collect()
}

fn collect_hit_probe_points(node: &RenderNode, points: &mut Vec<(f64, f64)>) {
    if matches!(
        node.node_type,
        RenderNodeType::TextRun(_) | RenderNodeType::TableCell(_)
    ) && node.bbox.width > 1.0
        && node.bbox.height > 1.0
    {
        points.push((
            node.bbox.x + node.bbox.width / 2.0,
            node.bbox.y + node.bbox.height / 2.0,
        ));
    }
    for child in &node.children {
        collect_hit_probe_points(child, points);
    }
}

fn path_key(section_index: usize, context: &CellContext) -> String {
    format!(
        "sec={section_index},ppi={},path={:?}",
        context.parent_para_index,
        context
            .path
            .iter()
            .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
            .collect::<Vec<_>>()
    )
}

#[test]
fn issue_4252_all_nested_partial_table_paths_resolve_against_original_ir() {
    let core = DocumentCore::from_bytes(&fixture_bytes()).expect("parse #4252 fixture");
    assert_eq!(core.page_count(), 17, "#4069 17쪽 pagination 계약");

    let mut checked = BTreeSet::new();
    let mut failures = Vec::new();
    for page_index in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("render page {}: {error}", page_index + 1));
        visit_runs(&tree.root, &mut |_text, context| {
            if context.path.len() < 2 {
                return;
            }
            let key = path_key(0, context);
            if checked.insert(key.clone()) {
                if let Err(error) =
                    resolve_table_path(core.document(), 0, context.parent_para_index, &context.path)
                {
                    failures.push(format!("page={} {key}: {error}", page_index + 1));
                }
            }
        });
    }

    assert!(!checked.is_empty(), "중첩 표 cell context를 하나 이상 검사");
    assert!(
        failures.is_empty(),
        "원본 IR로 resolve되지 않는 중첩 표 경로 {}건:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn issue_4252_page5_child_table_keeps_full_path_and_bbox_lookup_succeeds() {
    let bytes = fixture_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("parse #4252 fixture");
    let page5 = core
        .build_page_render_tree(4)
        .expect("render physical page 5");
    let mut target = None;
    visit_runs(&page5.root, &mut |text, context| {
        if text == "구 분" && target.is_none() {
            target = Some(context.clone());
        }
    });
    let context = target.expect("physical page 5 `구 분` TextRun cell context");

    assert_eq!(
        context.parent_para_index, 7,
        "합성 parentPara=0이 아니라 원본 본문 paragraph[7]"
    );
    let path = context
        .path
        .iter()
        .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
        .collect::<Vec<_>>();
    assert_eq!(
        path,
        vec![(1, 0, 0), (2, 0, 12), (0, 0, 0)],
        "외부 표 → 래퍼 표 → 물리 5쪽 자식 표 실제 경로"
    );
    resolve_table_path(core.document(), 0, context.parent_para_index, &context.path)
        .expect("physical page 5 child table path resolves to Table");

    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse HwpDocument");
    let bboxes = document
        .get_table_cell_bboxes_by_path(
            0,
            7,
            r#"[{"controlIndex":1,"cellIndex":0,"cellParaIndex":0},{"controlIndex":2,"cellIndex":0,"cellParaIndex":12},{"controlIndex":0,"cellIndex":0,"cellParaIndex":0}]"#,
        )
        .expect("Studio path-based table bbox lookup");
    assert!(
        bboxes.matches("\"cellIdx\"").count() >= 55,
        "10×6 병합 표의 55개 셀 bbox 반환: {bboxes}"
    );

    let parent_cursor = document
        .get_cursor_rect_by_path(
            0,
            7,
            r#"[{"controlIndex":1,"cellIndex":0,"cellParaIndex":0},{"controlIndex":2,"cellIndex":0,"cellParaIndex":12}]"#,
            0,
        )
        .expect("Esc로 자식 표 선택을 해제할 때 부모의 table-only paragraph caret");
    assert!(
        parent_cursor.contains("\"pageIndex\""),
        "부모 table-only paragraph cursor rect: {parent_cursor}"
    );
}

#[test]
fn issue_4252_page2_unsplit_child_table_keeps_existing_path() {
    let core = DocumentCore::from_bytes(&fixture_bytes()).expect("parse #4252 fixture");
    let page2 = core
        .build_page_render_tree(1)
        .expect("render physical page 2");
    let mut found = false;
    visit_runs(&page2.root, &mut |_text, context| {
        let path = context
            .path
            .iter()
            .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
            .collect::<Vec<_>>();
        if path == [(1, 1, 0), (5, 0, 0)] {
            found = true;
        }
    });
    assert!(
        found,
        "비분할 물리 2쪽의 기존 outer controls[1] → child controls[5] 경로 유지"
    );
}

#[test]
fn issue_4252_hit_test_does_not_replace_valid_equal_depth_text_path() {
    let core = DocumentCore::from_bytes(&fixture_bytes()).expect("parse #4252 fixture");
    let mut target = None;
    for page_index in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("render page {}: {error}", page_index + 1));
        visit_runs_with_node(&tree.root, &mut |node, context| {
            if target.is_none()
                && context.path.len() >= 3
                && context.path[1].cell_para_index == 99
                && node.bbox.width > 0.0
                && node.bbox.height > 0.0
            {
                target = Some((page_index, node.bbox, context.clone()));
            }
        });
        if target.is_some() {
            break;
        }
    }
    let (page_index, bbox, context) = target.expect("wrapper cell paragraph[99] TextRun");
    let hit_json = core
        .hit_test_native(
            page_index,
            bbox.x + bbox.width / 2.0,
            bbox.y + bbox.height / 2.0,
        )
        .expect("hit-test paragraph[99] nested TextRun");
    let hit: Value = serde_json::from_str(&hit_json).expect("parse hit JSON");
    let expected = context
        .path
        .iter()
        .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
        .collect::<Vec<_>>();

    assert_eq!(
        hit["parentParaIndex"].as_u64(),
        Some(context.parent_para_index as u64),
        "hit-test outer paragraph"
    );
    assert_eq!(
        json_cell_path(&hit),
        expected,
        "동일 깊이 traversal context가 유효한 TextRun 경로를 덮어쓰면 안 됨"
    );

    let parent_path = context.path[..context.path.len() - 1]
        .iter()
        .map(|entry| {
            serde_json::json!({
                "controlIndex": entry.control_index,
                "cellIndex": entry.cell_index,
                "cellParaIndex": entry.cell_para_index,
            })
        })
        .collect::<Vec<_>>();
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&fixture_bytes())
        .expect("parse HwpDocument for paragraph[99] parent caret");
    document
        .get_cursor_rect_by_path(
            0,
            context.parent_para_index as u32,
            &Value::Array(parent_path).to_string(),
            0,
        )
        .expect("wrapper cell paragraph[99] table-only caret rect");
}

#[test]
fn issue_4252_nested_hit_test_paths_resolve_across_all_pages() {
    let core = DocumentCore::from_bytes(&fixture_bytes()).expect("parse #4252 fixture");
    let mut checked = BTreeSet::new();
    let mut failures = Vec::new();

    for page_index in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page_index)
            .unwrap_or_else(|error| panic!("render page {}: {error}", page_index + 1));
        let mut points = Vec::new();
        collect_hit_probe_points(&tree.root, &mut points);
        for (x, y) in points {
            let Ok(hit_json) = core.hit_test_native(page_index, x, y) else {
                continue;
            };
            let hit: Value = serde_json::from_str(&hit_json).expect("parse hit JSON");
            let Some(path_json) = hit["cellPath"].as_array() else {
                continue;
            };
            if path_json.len() < 2 {
                continue;
            }
            let Some(section_index) = hit["sectionIndex"].as_u64().map(|value| value as usize)
            else {
                continue;
            };
            let Some(parent_para_index) =
                hit["parentParaIndex"].as_u64().map(|value| value as usize)
            else {
                continue;
            };
            let path = path_json
                .iter()
                .map(|entry| CellPathEntry {
                    control_index: entry["controlIndex"].as_u64().expect("controlIndex") as usize,
                    cell_index: entry["cellIndex"].as_u64().expect("cellIndex") as usize,
                    cell_para_index: entry["cellParaIndex"].as_u64().expect("cellParaIndex")
                        as usize,
                    text_direction: entry["textDirection"].as_u64().unwrap_or(0) as u8,
                })
                .collect::<Vec<_>>();
            let key = format!(
                "page={},x={:.1},y={:.1},sec={},ppi={},path={:?}",
                page_index + 1,
                x,
                y,
                section_index,
                parent_para_index,
                json_cell_path(&hit)
            );
            if checked.insert(key.clone()) {
                if let Err(error) =
                    resolve_table_path(core.document(), section_index, parent_para_index, &path)
                {
                    failures.push(format!("{key}: {error}"));
                }
            }
        }
    }

    assert!(!checked.is_empty(), "중첩 hit-test 경로를 하나 이상 검사");
    assert!(
        failures.is_empty(),
        "원본 IR로 resolve되지 않는 중첩 hit-test 경로 {}건:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
