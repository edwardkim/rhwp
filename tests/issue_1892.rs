//! Issue #1892 — 대법원 서식 hwp(HWP3) 계열 HWP5 라운드트립 렌더 위치 자기정합.
//!
//! 20k 라운드트립 위치 검사에서 대법원 행정예규 서식 계열이 페이지 내 초대형
//! 변위(최대 5449px)로 표면화했다. HWP3 파스 → export_hwp_with_adapter → 재파스
//! 경로의 3중 결함 체인 + 탭 확장 결함:
//!   1) 자식이 없는 '$con' SHAPE_COMPONENT(빈 묶음)를 파서가 사각형 폴백으로
//!      오분류 → 재파스 렌더 트리 Group→Rect 구조 분기,
//!   2) 직렬화기 rendering matrix 폴백이 offset_x/y 를 translation 으로 승격 —
//!      그룹 자식 위치의 단일 권위는 render_tx/ty 인데 메타데이터 offset 이
//!      행렬로 살아나 자식이 relative_pos 만큼 이동(최대 10485px),
//!   3) HWP3 build_common_obj_attr 가 크기 기준 비트(15-17/18-19)를 누락 —
//!      0(Paper 퍼센트)으로 저장되어 재파스 시 개체가 종이×(크기/10000)배 팽창,
//!   4) 직렬화기의 탭 "데이터 없음" 마커([0,...,0,0x0009])를 파서가 tab_extended
//!      로 실어 레이아웃이 ext[0]=0 을 탭 결과 위치로 해석 → 탭 무폭화.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{ShapeObject, SizeCriterion};
use rhwp::parser::parse_document;
use rhwp::renderer::layout::CellContext;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::serializer::serialize_document;

const SAMPLE_GROUP: &str = "samples/issue1892_hwp3_drawing_group_roundtrip.hwp";
const SAMPLE_TAB: &str = "samples/issue1892_hwp3_tab_roundtrip.hwp";

fn read_sample(rel: &str) -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn assert_roundtrip_render_self_consistent(rel: &str) {
    let data = read_sample(rel);
    let diff =
        roundtrip_geom(&data, Via::Hwp).unwrap_or_else(|e| panic!("roundtrip_geom({rel}): {e:?}"));

    assert_eq!(
        diff.page_count_a, diff.page_count_b,
        "{rel} 라운드트립 페이지 수 불일치: A={} B={}",
        diff.page_count_a, diff.page_count_b
    );
    assert!(
        diff.max_disp <= 1.0,
        "{rel} 라운드트립 렌더 변위 {:.2}px > 1.0px (HWP3 그리기/탭 왕복 회귀)",
        diff.max_disp
    );
    for pg in &diff.pages {
        assert!(
            !pg.structure_mismatch,
            "{rel} page {} 구조 불일치 (node {} vs {}) — 빈 묶음 Group→Rect 왕복 회귀",
            pg.page, pg.node_count_a, pg.node_count_b
        );
    }
}

/// 같은 독립 Print의 원본·저장본 전 2쪽을 양쪽 backend에서 확인한 계약.
/// 글꼴별 획 상자의 차이를 모든 노드의 1px 제한으로 고정하지 않는다.
#[test]
fn issue_1892_hwp3_group_roundtrip_preserves_owned_content_and_object_frames() {
    let mut original = DocumentCore::from_bytes(&read_sample(SAMPLE_GROUP)).expect("원본");
    let saved = original.export_hwp_with_adapter().expect("공개 HWP 저장");
    let reopened = DocumentCore::from_bytes(&saved).expect("저장본");
    assert_eq!(original.page_count(), 2, "독립 Print 전체 쪽수");
    assert_eq!(reopened.page_count(), original.page_count());
    let source_paper = &original.document().sections[0].section_def.page_def;
    let mut total_groups = 0;
    for page in 0..original.page_count() {
        let a = original.build_page_render_tree(page).expect("원본 쪽");
        let b = reopened.build_page_render_tree(page).expect("저장본 쪽");
        assert_eq!(
            owned_content(&a.root),
            owned_content(&b.root),
            "같은 쪽·문단·셀 내용의 순서·누락·중복: {page}"
        );
        assert_eq!(
            owned_tables(&a.root),
            owned_tables(&b.root),
            "인라인 표 소속·순서·누락·중복: {page}"
        );
        let mut left = Vec::new();
        let mut right = Vec::new();
        collect_groups(&a.root, &mut left);
        collect_groups(&b.root, &mut right);
        total_groups += left.len();
        assert_eq!(left.len(), right.len(), "묶음 도형 누락·중복: {page}");
        for (left, right) in left.into_iter().zip(right) {
            let RenderNodeType::Group(ga) = &left.node_type else {
                unreachable!()
            };
            let RenderNodeType::Group(gb) = &right.node_type else {
                unreachable!()
            };
            assert_eq!(
                (ga.section_index, ga.para_index, ga.control_index),
                (gb.section_index, gb.para_index, gb.control_index),
                "묶음 소유 보존"
            );
            assert_eq!(
                left.children
                    .iter()
                    .map(|n| std::mem::discriminant(&n.node_type))
                    .collect::<Vec<_>>(),
                right
                    .children
                    .iter()
                    .map(|n| std::mem::discriminant(&n.node_type))
                    .collect::<Vec<_>>(),
                "자식 도형 종류·순서 보존, 빈 묶음을 사각형으로 바꾸지 않는다"
            );
            // 소스 용지에 대한 개체 프레임 비율을 비교한다. 허용 오차는
            // 직렬화의 원본 정수 단위 1HU이며 관측 픽셀 위치를 고정하지 않는다.
            for (before, after, dimension, units) in [
                (
                    left.bbox.x - a.root.bbox.x,
                    right.bbox.x - b.root.bbox.x,
                    a.root.bbox.width,
                    source_paper.width,
                ),
                (
                    left.bbox.y - a.root.bbox.y,
                    right.bbox.y - b.root.bbox.y,
                    a.root.bbox.height,
                    source_paper.height,
                ),
                (
                    left.bbox.width,
                    right.bbox.width,
                    a.root.bbox.width,
                    source_paper.width,
                ),
                (
                    left.bbox.height,
                    right.bbox.height,
                    a.root.bbox.height,
                    source_paper.height,
                ),
            ] {
                assert!(dimension > 0.0 && units > 0);
                let difference = ((before - after) / dimension).abs();
                assert!(
                    difference.is_finite() && difference <= 1.0 / f64::from(units) + f64::EPSILON,
                    "원본 용지에 대한 개체 프레임 비율 보존: {difference}"
                );
            }
        }
    }
    assert!(total_groups > 0, "실제 출력에서 묶음 도형을 검사해야 한다");
}

#[derive(Debug, PartialEq)]
struct OwnedContent {
    section: Option<usize>,
    paragraph: Option<usize>,
    cell: Option<CellContext>,
    text: String,
}

fn owned_content(node: &RenderNode) -> Vec<OwnedContent> {
    fn walk(node: &RenderNode, out: &mut Vec<OwnedContent>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            // HWP3의 개체 슬롯 문자는 HWP 저장 시 실제 Table control로 보존된다.
            // 표시 문자를 비교하고 개체 소유는 owned_tables에서 별도로 검사한다.
            let text: String = run.text.chars().filter(|ch| *ch != '\u{fffc}').collect();
            if !text.is_empty() {
                if let Some(last) = out.last_mut().filter(|last| {
                    last.section == run.section_index
                        && last.paragraph == run.para_index
                        && last.cell == run.cell_context
                }) {
                    last.text.push_str(&text);
                } else {
                    out.push(OwnedContent {
                        section: run.section_index,
                        paragraph: run.para_index,
                        cell: run.cell_context.clone(),
                        text,
                    });
                }
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

#[derive(Debug, PartialEq)]
struct OwnedTable {
    section: Option<usize>,
    paragraph: Option<usize>,
    control: Option<usize>,
    cell: Option<CellContext>,
    rows: u16,
    cols: u16,
}

fn owned_tables(node: &RenderNode) -> Vec<OwnedTable> {
    fn walk(node: &RenderNode, out: &mut Vec<OwnedTable>) {
        if let RenderNodeType::Table(table) = &node.node_type {
            out.push(OwnedTable {
                section: table.section_index,
                paragraph: table.para_index,
                control: table.control_index,
                cell: table.cell_context.clone(),
                rows: table.row_count,
                cols: table.col_count,
            });
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

fn collect_groups<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Group(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect_groups(child, out);
    }
}

/// 대표 2952505 (문서건명부 서식): 탭 확장 "데이터 없음" 마커 축.
/// 수정 전 OVER 130px.
#[test]
fn issue_1892_hwp3_tab_roundtrip_render_is_self_consistent() {
    assert_roundtrip_render_self_consistent(SAMPLE_TAB);
}

/// 자식의 원본 상대 오프셋은 상위 묶음 원점에 누적된다. 중첩 묶음과
/// Absolute 크기 기준을 보존하며 메타데이터 오프셋을 행렬에 두 번 더하지 않는다.
#[test]
fn issue_1892_group_roundtrip_preserves_source_offsets_and_group_hierarchy() {
    let doc_a = parse_document(&read_sample(SAMPLE_GROUP)).expect("원본 파싱");
    let doc_b = parse_document(&serialize_document(&doc_a).expect("저장")).expect("재파싱");
    fn verify(
        node: &ShapeObject,
        parent: (f64, f64),
        depth: u16,
        nested: &mut usize,
        moved: &mut usize,
    ) {
        let attr = node.shape_attr();
        let origin = if depth == 0 {
            (0.0, 0.0)
        } else {
            (
                parent.0 + f64::from(attr.offset_x),
                parent.1 + f64::from(attr.offset_y),
            )
        };
        assert_eq!(
            (attr.render_tx, attr.render_ty),
            origin,
            "소스 상대 오프셋의 누적 원점"
        );
        assert_eq!(attr.group_level, depth, "소유 묶음 깊이");
        if depth > 0 && (attr.offset_x != 0 || attr.offset_y != 0) {
            *moved += 1;
        }
        if let ShapeObject::Group(group) = node {
            if depth > 0 {
                *nested += 1;
            }
            for child in &group.children {
                verify(child, origin, depth + 1, nested, moved);
            }
        }
    }
    fn groups(doc: &rhwp::model::document::Document) -> Vec<&ShapeObject> {
        fn shape_text<'a>(shape: &'a ShapeObject, out: &mut Vec<&'a ShapeObject>) {
            if let Some(drawing) = shape.drawing() {
                if let Some(text_box) = &drawing.text_box {
                    paragraphs(&text_box.paragraphs, out);
                }
                if let Some(caption) = &drawing.caption {
                    paragraphs(&caption.paragraphs, out);
                }
            }
            if let ShapeObject::Group(group) = shape {
                for child in &group.children {
                    shape_text(child, out);
                }
            }
        }
        fn paragraphs<'a>(
            source: &'a [rhwp::model::paragraph::Paragraph],
            out: &mut Vec<&'a ShapeObject>,
        ) {
            for para in source {
                for control in &para.controls {
                    match control {
                        Control::Shape(shape) => {
                            if matches!(**shape, ShapeObject::Group(_)) {
                                out.push(shape);
                            }
                            shape_text(shape, out);
                        }
                        Control::Table(table) => {
                            for cell in &table.cells {
                                paragraphs(&cell.paragraphs, out);
                            }
                            if let Some(caption) = &table.caption {
                                paragraphs(&caption.paragraphs, out);
                            }
                        }
                        Control::Header(header) => paragraphs(&header.paragraphs, out),
                        Control::Footer(footer) => paragraphs(&footer.paragraphs, out),
                        Control::Footnote(note) => paragraphs(&note.paragraphs, out),
                        Control::Endnote(note) => paragraphs(&note.paragraphs, out),
                        _ => {}
                    }
                }
            }
        }
        let mut out = Vec::new();
        for section in &doc.sections {
            paragraphs(&section.paragraphs, &mut out);
        }
        out
    }
    let original = groups(&doc_a);
    let saved = groups(&doc_b);
    assert!(!original.is_empty());
    assert_eq!(original.len(), saved.len(), "최상위 묶음 보존");
    let mut total_nested = 0;
    for (a, b) in original.into_iter().zip(saved) {
        let mut original_nested = 0;
        let mut saved_nested = 0;
        let mut original_moved = 0;
        let mut saved_moved = 0;
        verify(a, (0.0, 0.0), 0, &mut original_nested, &mut original_moved);
        verify(b, (0.0, 0.0), 0, &mut saved_nested, &mut saved_moved);
        assert_eq!(original_nested, saved_nested, "중첩 묶음 소유 보존");
        assert_eq!(original_moved, saved_moved, "비영 상대 오프셋 보존");
        total_nested += original_nested;
        assert_eq!(
            (a.common().width, a.common().height),
            (b.common().width, b.common().height),
            "원본 개체 크기 보존"
        );
        assert_eq!(
            (b.common().width_criterion, b.common().height_criterion),
            (SizeCriterion::Absolute, SizeCriterion::Absolute),
            "용지 퍼센트 크기로 바뀌지 않는다"
        );
    }
    assert!(total_nested > 0, "중첩 묶음이 있는 원본");
}

/// IR 핀: tab_extended 없는 탭이 직렬화→재파스 후에도 tab_extended 를 만들지
/// 않는다 (null 마커를 IR 로 실으면 레이아웃이 탭 무폭으로 해석).
#[test]
fn issue_1892_null_tab_extension_marker_not_parsed_into_ir() {
    let data = read_sample(SAMPLE_TAB);
    let doc_a = parse_document(&data).expect("parse A");
    let out = serialize_document(&doc_a).expect("serialize");
    let doc_b = parse_document(&out).expect("parse B");

    let count = |doc: &rhwp::model::document::Document| -> (usize, usize) {
        let mut tabs = 0usize;
        let mut exts = 0usize;
        for para in &doc.sections[0].paragraphs {
            tabs += para.text.matches('\t').count();
            exts += para.tab_extended.len();
        }
        (tabs, exts)
    };

    let (tabs_a, exts_a) = count(&doc_a);
    let (tabs_b, exts_b) = count(&doc_b);
    assert!(tabs_a > 0, "픽스처에 탭이 있어야 함");
    assert_eq!(tabs_a, tabs_b, "탭 문자 수 왕복 보존");
    assert_eq!(
        exts_a, exts_b,
        "tab_extended 수 왕복 보존 (null 마커가 IR 로 유입되는 회귀: A={exts_a} B={exts_b})"
    );
}
