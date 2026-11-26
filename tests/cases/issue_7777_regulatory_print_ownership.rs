//! 검사 유형: 관계형(본문·표·꼬리말 소속과 순서), 고정형(독립 Print 쪽수·꼬리말 기준선).
//! 주석 갱신: 2026-11-26 10:26:12 KST.
//! 측정: 2026-11-26, 한컴 2020 Print PDFium 96dpi, compat2022, 색상 경계 보정 전 2px raw.
//! 37787 원본 전 49쪽에서 Native 최저 99.21195%, fresh WASM 최저 99.14043%.
//! 14·27쪽은 Native/fresh WASM 각각 100.00%이며 독립 PDF 꼬리말 기준선은 두 쪽 1081.69784px.
//! 원본은 samples/issue5699/37787_regulatory_impact.hwp, 독립 PDF는
//! pdf/issue5699/37787_regulatory_impact-2020.pdf다.
//! 전쪽 TSV SHA-256: Native 66cf7ddc746b6dab6d407e2ad92b377a62c9bbda248a5073f7dac9f22443262f,
//! fresh WASM 3c3aa9e42d00900705b2b11b13a36d34c6637af831afa60013f1620f2ddf0679.
//! 증거는 mydocs/working/task_m100_7777_stage1.md의 #7777 둘째 후보 측정과
//! output/pr-review/issues7777-7787/merged-cbebdbeeca/7777-v2-candidate/{native,wasm,native-review,wasm-review}에 있다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

fn document() -> DocumentCore {
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue5699/37787_regulatory_impact.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    core
}

fn nodes<'a>(root: &'a RenderNode, result: &mut Vec<&'a RenderNode>) {
    result.push(root);
    for child in &root.children {
        nodes(child, result);
    }
}

fn paint_ops<'a>(node: &'a LayerNode, result: &mut Vec<&'a PaintOp>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                paint_ops(child, result);
            }
        }
        LayerNodeKind::ClipPath { child, .. } | LayerNodeKind::ClipRect { child, .. } => {
            paint_ops(child, result)
        }
        LayerNodeKind::Leaf { ops } => result.extend(ops),
    }
}

fn outer_tables(layout: &[&RenderNode], paragraph: usize) -> Vec<(Option<usize>, BoundingBox)> {
    layout
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Table(table)
                if table.para_index == Some(paragraph) && table.cell_context.is_none() =>
            {
                Some((table.control_index, node.bbox))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn regulatory_impact_blank_print_pages_keep_one_implicit_footer() {
    let core = document();
    assert_eq!(core.page_count(), 49, "한컴 Print 49쪽");
    for number in [14, 27] {
        let print = core
            .build_page_layer_tree_with_profile(number - 1, RenderProfile::Print)
            .unwrap();
        let mut ops = Vec::new();
        paint_ops(&print.root, &mut ops);
        let nonempty_text: Vec<_> = ops
            .into_iter()
            .filter_map(|op| match op {
                PaintOp::TextRun { bbox, run, .. } if !run.display_or_text().trim().is_empty() => {
                    Some((bbox.y + run.baseline, run.display_or_text().trim()))
                }
                _ => None,
            })
            .collect();
        let [(baseline, text)] = nonempty_text.as_slice() else {
            panic!("{number}쪽에서 꼬리말 쪽번호 하나를 기대함: {nonempty_text:?}");
        };
        assert_eq!(*text, format!("- {number} -"));
        assert!(
            (baseline - 1081.69784).abs() <= 1.0,
            "{number}쪽 독립 Print 꼬리말 기준선 1081.69784±1px, 실제 {baseline}"
        );
    }
}

#[test]
fn regulatory_impact_keeps_headings_and_coanchored_tables_on_their_source_pages() {
    let core = document();
    assert_eq!(core.page_count(), 49);
    let headings = [(155, 15), (276, 28), (397, 40)];
    let mut heading_pages = [Vec::new(), Vec::new(), Vec::new()];
    for page in 1..=49 {
        let tree = core.build_page_render_tree(page - 1).unwrap();
        let mut layout = Vec::new();
        nodes(&tree.root, &mut layout);
        for node in &layout {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                for (index, (paragraph, _)) in headings.iter().enumerate() {
                    if run.para_index == Some(*paragraph)
                        && run.display_or_text().trim() == "< 규제의 개요 >"
                    {
                        heading_pages[index].push(page);
                    }
                }
            }
        }
        if page == 6 {
            let lines: Vec<_> = layout
                .iter()
                .filter_map(|node| match &node.node_type {
                    RenderNodeType::TextLine(line) if line.para_index == Some(62) => {
                        Some(node.bbox)
                    }
                    _ => None,
                })
                .collect();
            let tables = outer_tables(&layout, 63);
            let ([line], [(_, table)]) = (lines.as_slice(), tables.as_slice()) else {
                panic!("6쪽의 안내 글줄과 바로 뒤 표가 각각 하나 있어야 함");
            };
            assert!(
                line.y + line.height <= table.y,
                "안내 글줄이 표 위에 놓인다"
            );
        }
        if page == 15 {
            let tables = outer_tables(&layout, 158);
            assert!(matches!(tables.as_slice(), [(Some(0), _)]));
        }
        for (first_page, paragraph) in [(23, 242), (36, 363)] {
            if page != first_page && page != first_page + 1 {
                continue;
            }
            let tables = outer_tables(&layout, paragraph);
            if page == first_page {
                assert_eq!(tables.len(), 1);
                assert_eq!(tables[0].0, Some(0));
            } else {
                let [(first_id, first), (second_id, second)] = tables.as_slice() else {
                    panic!("{page}쪽에서 표 두 조각을 기대함: {tables:?}");
                };
                assert_eq!((*first_id, *second_id), (Some(0), Some(1)));
                assert!(
                    first.y + first.height <= second.y,
                    "{page}쪽 표 순서와 비겹침"
                );
            }
        }
    }
    for ((paragraph, page), actual) in headings.into_iter().zip(heading_pages) {
        assert_eq!(
            actual,
            [page],
            "제목 문단 {paragraph}의 쪽 소속과 중복 방지"
        );
    }
}
