//! 동일 원본 HWP/HWPX와 독립 한컴 Print의 중첩 표·미주 소유 계약.
//! 시각 선행 증거는 mydocs/pr/assets/issue_7833_nested_source/README.md에 연결한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType, TextRunNode};

fn originals() -> [(&'static str, &'static [u8]); 2] {
    [
        (
            "HWP",
            include_bytes!("../../mydocs/pr/assets/issue_7833_nested_source/inputs/original.hwp"),
        ),
        (
            "HWPX",
            include_bytes!("../../mydocs/pr/assets/issue_7833_nested_source/inputs/original.hwpx"),
        ),
    ]
}

struct Run<'a> {
    index: usize,
    node: &'a RenderNode,
    text: &'a TextRunNode,
    cell: Option<BoundingBox>,
}

fn collect<'a>(
    node: &'a RenderNode,
    cell: Option<BoundingBox>,
    runs: &mut Vec<Run<'a>>,
    markers: &mut Vec<(&'a RenderNode, Option<BoundingBox>)>,
) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node.bbox)
    } else {
        cell
    };
    if let RenderNodeType::TextRun(text) = &node.node_type {
        runs.push(Run {
            index: runs.len(),
            node,
            text,
            cell,
        });
    }
    if matches!(node.node_type, RenderNodeType::FootnoteMarker(_)) {
        markers.push((node, cell));
    }
    for child in &node.children {
        collect(child, cell, runs, markers);
    }
}

fn unique<'a>(runs: &'a [Run<'a>], text: &str) -> &'a Run<'a> {
    let found: Vec<_> = runs.iter().filter(|run| run.text.text == text).collect();
    assert_eq!(found.len(), 1, "문자 누락·중복: {text}");
    found[0]
}

#[test]
fn nested_inline_table_keeps_host_baseline_and_cell_ownership() {
    for (format, bytes) in originals() {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        assert_eq!(core.page_count(), 1, "독립 Print의 전쪽 수: {format}");
        let tree = core.build_page_render_tree(0).unwrap();
        let (mut runs, mut markers) = (Vec::new(), Vec::new());
        collect(&tree.root, None, &mut runs, &mut markers);
        let (front, back, body) = (
            unique(&runs, "앞"),
            unique(&runs, "뒤"),
            unique(&runs, "원문"),
        );
        let outer = front.cell.expect("호스트의 바깥 셀");
        let inner = body.cell.expect("본문의 안쪽 셀");
        assert!(outer.contains(&front.node.bbox));
        assert!(outer.contains(&back.node.bbox));
        assert!(outer.contains(&inner));
        assert!(inner.contains(&body.node.bbox));
        assert!(front.node.bbox.x < inner.x);
        assert!(inner.x + inner.width <= back.node.bbox.x);
        assert!(
            front.node.bbox.y + front.text.baseline > body.node.bbox.y + body.text.baseline,
            "표의 글자와 호스트 기준선 순서: {format}"
        );
        assert_eq!(
            front.node.bbox.y + front.text.baseline,
            back.node.bbox.y + back.text.baseline
        );
        assert_eq!(markers.len(), 1, "인라인 참조 번호 보존: {format}");
        let (marker, owner) = markers[0];
        assert!(owner.unwrap().contains(&marker.bbox));
        assert!(marker.bbox.x < body.node.bbox.x);
        let RenderNodeType::FootnoteMarker(reference) = &marker.node_type else {
            unreachable!()
        };
        assert_eq!(reference.number, 1);
        assert_eq!(reference.text, "1)");
        assert_eq!(reference.para_index, body.text.para_index.unwrap());
        let mut trailing: Vec<_> = runs.iter().filter(|run| run.text.text == "1) ").collect();
        assert_eq!(trailing.len(), 2, "원문과 생성 미주 접두사의 개별 보존");
        trailing.sort_by(|a, b| a.node.bbox.y.total_cmp(&b.node.bbox.y));
        assert!(trailing.iter().all(|run| run.cell.is_none()));
        assert!(trailing[0].node.bbox.y >= outer.y + outer.height);
        assert!(trailing[1].node.bbox.y >= trailing[0].node.bbox.y + trailing[0].node.bbox.height);
    }
}

#[test]
fn nested_endnote_trace_never_aliases_literal_or_body_source() {
    for (format, bytes) in originals() {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        let source = &core.document().sections[0].paragraphs[0];
        let table_index = source
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        let tree = core.build_page_render_tree(0).unwrap();
        let (mut runs, mut markers) = (Vec::new(), Vec::new());
        collect(&tree.root, None, &mut runs, &mut markers);
        // 생성 미주의 문단 주소는 본문 구역의 원문 주소가 아니다. 셀의 직접 TextRun이라는
        // 과거 가정 대신 실제 소스 문단 소유와 별도의 FootnoteMarker를 대조한다.
        let generated: Vec<_> =
            runs.iter()
                .filter(|run| {
                    run.text.text == "1) "
                        && run.cell.is_none()
                        && run.text.para_index.is_some_and(|index| {
                            index >= core.document().sections[0].paragraphs.len()
                        })
                })
                .collect();
        assert_eq!(generated.len(), 1, "생성 미주 접두사: {format}");
        let model = format!("{:?}", core.document());
        let svg = core.render_page_svg_native(0).unwrap();
        let hwp = core.export_hwp_native().unwrap();
        let hwpx = core.export_hwpx_native().unwrap();
        let trace: serde_json::Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":4096}"#)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(format!("{:?}", core.document()), model);
        assert_eq!(core.render_page_svg_native(0).unwrap(), svg);
        assert_eq!(core.export_hwp_native().unwrap(), hwp);
        assert_eq!(core.export_hwpx_native().unwrap(), hwpx);
        let records = trace["records"].as_array().unwrap();
        let generated_records: Vec<_> = records
            .iter()
            .filter(|record| record["source"]["runIndex"] == generated[0].index)
            .collect();
        assert_eq!(generated_records.len(), 3);
        for record in generated_records {
            assert_eq!(
                record["source"]["status"], "unavailable",
                "미주 표시의 본문 alias: {record}"
            );
            assert!(record["source"]["charOffset"].is_null());
        }
        for (path, text) in [
            (vec![], "1) "),
            (vec![table_index, 0, 1], "앞뒤"),
            (vec![table_index, 0, 1, 0, 0, 0], "원문"),
        ] {
            let actual: Vec<_> = records
                .iter()
                .filter(|r| {
                    r["source"]["status"] == "complete"
                        && r["source"]["nestedPath"] == serde_json::json!(path)
                })
                .map(|r| {
                    (
                        r["source"]["charOffset"].as_u64().unwrap(),
                        r["source"]["character"].as_str().unwrap().to_string(),
                    )
                })
                .collect();
            let expected: Vec<_> = text
                .chars()
                .enumerate()
                .map(|(i, ch)| (i as u64, ch.to_string()))
                .collect();
            assert_eq!(actual, expected, "{format} path={path:?}");
        }
        assert_eq!(records.len(), "원문앞뒤1) 1) ".chars().count());
    }
}
