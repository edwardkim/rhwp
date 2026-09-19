//! 저장 위치가 없는 셀의 세로 정렬과 본문 표 뒤 흐름의 독립 합성 계약.
//!
//! 빈 문서에서 생성한 fixture이며 사용자 양식의 내용은 포함하지 않는다.
//! Top 배치에서 확인한 점유 영역으로 Center/Bottom의 이동량을 정한다.
//! HWP5 계보와 순수 HWPX, 유효한 저장 앵커와 0으로 초기화된 앵커를 구분한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

fn nodes(name: &str) -> Vec<RenderNode> {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/stored-nested-content-flow")
        .join(name);
    nodes_from_file(&file)
}

fn nodes_from_file(file: &Path) -> Vec<RenderNode> {
    let core = DocumentCore::from_bytes(&std::fs::read(file).expect("read fixture"))
        .expect("parse fixture");
    nodes_from_core(&core)
}

fn nodes_from_core(core: &DocumentCore) -> Vec<RenderNode> {
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).expect("render page");
    fn collect(node: &RenderNode, output: &mut Vec<RenderNode>) {
        output.push(node.clone());
        for child in &node.children {
            collect(child, output);
        }
    }
    let mut output = Vec::new();
    collect(&tree.root, &mut output);
    output
}

// 시각 비교에는 실제 한컴 저장본을 사용한다. 부실 캐시 복구는 같은 문서의
// 저장 줄만 메모리에서 손상시켜 별도로 검사하며, 손상본의 한컴 PDF를 정답으로 삼지 않는다.
fn damaged_width_nodes(align: &str) -> Vec<RenderNode> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "samples/stored-nested-content-flow/width-{align}.hwp"
    ));
    let source = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut document = source.document().clone();
    let outer = document.sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|p| p.controls.iter_mut())
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .expect("outer table");
    let paragraphs = &mut outer.cells[0].paragraphs;
    assert_eq!(
        paragraphs[0].line_segs.len(),
        5,
        "손상 전 정상 저장 5줄 확인"
    );
    paragraphs[0].line_segs.truncate(1);
    for p in paragraphs.iter_mut() {
        for seg in &mut p.line_segs {
            seg.vertical_pos = 0;
        }
    }
    let mut core = DocumentCore::new_empty();
    core.set_document(document);
    nodes_from_core(&core)
}

fn synthetic_empty_nodes(align: &str) -> Vec<RenderNode> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "samples/stored-nested-content-flow/cell-{align}-collapsed.hwpx"
    ));
    let source = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut document = source.document().clone();
    let outer = document.sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|p| p.controls.iter_mut())
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .expect("outer table");
    let paragraph = &mut outer.cells[0].paragraphs[0];
    assert_eq!(paragraph.text.trim(), "Start");
    paragraph.text.clear();
    paragraph.char_offsets.clear();
    paragraph.char_count = 1;
    let mut core = DocumentCore::new_empty();
    core.set_document(document);
    nodes_from_core(&core)
}

#[test]
fn empty_leading_paragraph_keeps_its_line_space_in_nested_table_alignment() {
    let top = synthetic_empty_nodes("top");
    let outer = table(&top, 320.0);
    let nested = table(&top, 160.0);
    // 한컴 PDF의 LEFT/offset=0 앵커는 x=48.32px다. 가운데 배치(128px) 금지.
    assert!((nested.x - 48.32).abs() < 1.0);
    // The source line reserves (1000 + 200) HWPUNIT at 96 DPI.
    // An empty glyph run does not erase this explicit line box.
    assert!((nested.y - outer.y - 16.0).abs() < 0.6);
    let slack = outer.y + outer.height - nested.y - nested.height;
    for (align, fraction) in [("center", 0.5), ("bottom", 1.0)] {
        let aligned = synthetic_empty_nodes(align);
        let advance = table(&aligned, 160.0).y - nested.y;
        assert!(
            (advance - slack * fraction).abs() < 0.6,
            "empty/{align}: advance {advance}, expected {}",
            slack * fraction
        );
    }
}

#[test]
fn saved_empty_leading_paragraph_preserves_pdf_table_and_border_positions() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pr7200_empty_leading_paragraph");
    // 최종 HWPX 자체에서 출력한 Hancom PDF의 벡터 상단과 End 글리프 상단(pt).
    // 원본 합성 캐시의 16px 불변식과 별개인 유효 저장 앵커 경로다.
    for (align, table_top_pt, text_top_pt) in [
        ("top", 112.418, 127.428),
        ("center", 143.971, 158.981),
        ("bottom", 175.525, 190.535),
    ] {
        let page = nodes_from_file(&root.join(format!("empty-first-{align}.hwpx")));
        let nested = table(&page, 160.0);
        assert!(
            (nested.y - table_top_pt * 4.0 / 3.0).abs() < 0.6,
            "{align}: saved empty host anchor y={}, PDF={}",
            nested.y,
            table_top_pt * 4.0 / 3.0
        );
        let end = text(&page, "End");
        assert!(
            (end.y - text_top_pt * 4.0 / 3.0).abs() < 1.0,
            "{align}: End y={}, PDF={}",
            end.y,
            text_top_pt * 4.0 / 3.0
        );
        assert!(
            page.iter().any(
                |node| matches!(node.node_type, RenderNodeType::Rectangle(_))
                    && (node.bbox.width - 160.0).abs() < 0.5
                    && (node.bbox.y - text_top_pt * 4.0 / 3.0).abs() < 1.0
                    && (node.bbox.height - 22.55).abs() < 0.5
            ),
            "{align}: End 문단의 마지막 줄간격까지 포함한 테두리"
        );
    }
}

fn text(nodes: &[RenderNode], expected: &str) -> BoundingBox {
    nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if run.text == expected => Some(node.bbox),
            _ => None,
        })
        .expect("expected text")
}

fn table(nodes: &[RenderNode], width: f64) -> BoundingBox {
    let matches: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(node.node_type, RenderNodeType::Table { .. })
                && (node.bbox.width - width).abs() < 0.5
        })
        .collect();
    assert_eq!(matches.len(), 1, "unique table width {width}");
    matches[0].bbox
}

#[test]
fn nested_cell_alignment_uses_the_sequential_anchor_when_stored_positions_reset() {
    for prefix in ["", "pure-"] {
        for stored in ["collapsed", "intact"] {
            let top = nodes(&format!("{prefix}cell-top-{stored}.hwpx"));
            let outer = table(&top, 320.0);
            let nested = table(&top, 160.0);
            let slack = (outer.y + outer.height - nested.y - nested.height).max(0.0);
            let top_y = text(&top, "Start").y;
            for (align, fraction) in [("center", 0.5), ("bottom", 1.0)] {
                let aligned = nodes(&format!("{prefix}cell-{align}-{stored}.hwpx"));
                let advance = text(&aligned, "Start").y - top_y;
                assert!(
                    (advance - slack * fraction).abs() < 0.6,
                    "{prefix}{stored}/{align}: advance {advance}, expected {}",
                    slack * fraction
                );
            }
        }
    }
}

#[test]
fn preceding_negative_spacing_is_not_used_as_the_inline_table_host_spacing() {
    for prefix in ["", "pure-"] {
        for spacing in [-600, 0] {
            let page = nodes(&format!("{prefix}body-{spacing}.hwpx"));
            let outer = table(&page, 320.0);
            let footer = text(&page, "Footer");
            assert!(
                footer.y >= outer.y + outer.height - 0.5,
                "{prefix}{spacing}: footer {} precedes table bottom {}",
                footer.y,
                outer.y + outer.height
            );
        }
    }
}

#[test]
fn nested_alignment_uses_rewrapped_text_and_actual_float_flow() {
    for kind in ["overlay", "square", "flow", "width"] {
        let extension = if kind == "width" { "hwp" } else { "hwpx" };
        let top = if kind == "width" {
            damaged_width_nodes("top")
        } else {
            nodes(&format!("{kind}-top.{extension}"))
        };
        let outer = table(&top, 320.0);
        if kind == "overlay" {
            let behind = table(&top, 160.0);
            let following = table(&top, 10000.0 / 75.0);
            // 한컴 PDF: 객체 높이 대신 빈 호스트 줄(1000+200HU)만 전진한다.
            assert!((following.y - behind.y - 16.0).abs() < 0.6);
            assert!((following.x - behind.x - behind.width).abs() < 0.6);
            assert!(following.x + following.width <= outer.x + outer.width + 0.6);
        }
        let nested_bottom = top
            .iter()
            .filter(|node| {
                matches!(node.node_type, RenderNodeType::Table { .. }) && node.bbox.width < 319.5
            })
            .map(|node| node.bbox.y + node.bbox.height)
            .reduce(f64::max)
            .expect("nested tables");
        let first_y = |page: &[RenderNode]| {
            page.iter()
                .find_map(|node| match &node.node_type {
                    RenderNodeType::TextRun(run)
                        if run
                            .text
                            .starts_with(if kind == "width" { "Wide" } else { "Start" }) =>
                    {
                        Some(node.bbox.y)
                    }
                    _ => None,
                })
                .expect("first text")
        };
        let top_y = first_y(&top);
        if kind == "width" {
            assert!(
                top.iter().any(|node| {
                    matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains("text"))
                        && node.bbox.y > top_y + 1.0
                }),
                "stored text must actually wrap onto a later line"
            );
        }
        let slack = (outer.y + outer.height - nested_bottom).max(0.0);
        for (align, fraction) in [("center", 0.5), ("bottom", 1.0)] {
            let aligned = if kind == "width" {
                damaged_width_nodes(align)
            } else {
                nodes(&format!("{kind}-{align}.{extension}"))
            };
            let actual = first_y(&aligned) - top_y;
            assert!(
                (actual - slack * fraction).abs() < 0.6,
                "{kind}/{align}: advance {actual}, expected {}",
                slack * fraction
            );
        }
    }
}

#[test]
fn hancom_recomposed_lines_and_nested_table_match_pdf_positions() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/stored-nested-content-flow");
    // 같은 HWP를 Hancom 12.0.0.4605로 출력한 PDF 텍스트 상단(96 DPI).
    // 단일 저장 줄로 손상한 경계는 damaged_width_nodes에서 별도로 검사한다.
    for (align, expected) in [
        (
            "top",
            [127.718, 150.094, 172.629, 195.164, 217.540, 259.734],
        ),
        (
            "center",
            [244.550, 267.086, 289.621, 311.997, 334.532, 376.566],
        ),
        (
            "bottom",
            [361.542, 384.078, 406.453, 428.989, 451.524, 493.558],
        ),
    ] {
        let rendered = nodes_from_file(&fixture.join(format!("width-{align}.hwp")));
        let text_nodes: Vec<_> = rendered.iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
        }).collect();
        assert_eq!(text_nodes.len(), expected.len(), "{align}: 5줄과 End 보존");
        for (node, pdf_y) in text_nodes.iter().zip(expected) {
            assert!((node.bbox.x - 48.774).abs() < 1.0, "{align}: PDF 가로 원점");
            assert!(
                (node.bbox.y - pdf_y).abs() < 1.0,
                "{align}: y={} PDF={pdf_y}",
                node.bbox.y
            );
        }
        assert!((table(&rendered, 160.0).x - 48.32).abs() < 1.0);
        // PDF의 End 문단 테두리는 마지막 줄간격까지 포함한 약 22.5px다.
        // 테두리 연결이 꺼진 셀 문단을 본문/부모 셀 테두리와 병합하지 않는다.
        assert!(
            rendered.iter().any(|node| {
                matches!(node.node_type, RenderNodeType::Rectangle(_))
                    && (node.bbox.width - 160.0).abs() < 0.5
                    && (node.bbox.y - expected[5]).abs() < 1.0
                    && (node.bbox.height - 22.5).abs() < 0.5
            }),
            "{align}: 셀의 마지막 문단 테두리 누락/축소"
        );
        assert!(
            rendered.iter().any(|node| {
                matches!(node.node_type, RenderNodeType::Rectangle(_))
                    && (node.bbox.width - 384.0).abs() < 0.5
                    && node.bbox.height > 400.0
            }),
            "{align}: TAC 표 호스트의 본문 문단 테두리 누락"
        );
    }
}

#[test]
fn standalone_table_character_border_preserves_pdf_decoration_margins() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pr7200_hancom_recomposed");
    // Hancom PDF: character-border bottom minus physical table bottom, at 96 DPI.
    // Includes both sides of the 2.5 mm minimum and margins above that minimum.
    for (name, expected_tail) in [
        ("width-top.hwp", 18.699),
        ("margin-min-700.hwp", 18.716),
        ("margin-min-800.hwp", 19.996),
        ("margin-both-1000.hwp", 13.597),
        ("margin-both-2000.hwp", 26.875),
    ] {
        let page = if name == "width-top.hwp" {
            nodes(name)
        } else {
            nodes_from_file(&root.join(name))
        };
        let outer = page
            .iter()
            .find(|n| {
                matches!(n.node_type, RenderNodeType::Table(_))
                    && (n.bbox.width - 320.0).abs() < 0.5
            })
            .expect("outer table");
        let bottom = outer.bbox.y + outer.bbox.height;
        assert!(
            outer
                .children
                .iter()
                .any(|n| matches!(n.node_type, RenderNodeType::Line(_))
                    && (n.bbox.width - 320.0).abs() < 0.5
                    && matches!(&n.node_type, RenderNodeType::Line(line) if (line.y2-line.y1).abs() < 0.01)
                    && (n.bbox.y - bottom - expected_tail).abs() < 0.6),
            "{name}: missing/incorrect object character border tail {expected_tail}, table={:?}, lines={:?}", outer.bbox, outer.children.iter().filter(|n| matches!(n.node_type, RenderNodeType::Line(_))).map(|n| n.bbox).collect::<Vec<_>>()
        );
    }
    let page = nodes_from_file(&root.join("host-char-border-off.hwpx"));
    let outer = page
        .iter()
        .find(|n| {
            matches!(n.node_type, RenderNodeType::Table(_)) && (n.bbox.width - 320.0).abs() < 0.5
        })
        .expect("outer table");
    assert!(
        !outer
            .children
            .iter()
            .any(|n| matches!(n.node_type, RenderNodeType::Line(_))
                && (n.bbox.width - 320.0).abs() < 0.5
                && n.bbox.y > outer.bbox.y + outer.bbox.height + 1.0),
        "turning off character border must retain table/paragraph borders only"
    );
}
