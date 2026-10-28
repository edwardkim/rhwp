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

/// 메타데이터 상대 오프셋과 실제 그리기 행렬은 별도 계약이다.
/// 독립 한컴 재저장/Print와 대조한 구조 묶음 identity·자식 affine·물리 끝점을 보존한다.
#[test]
fn issue_1892_group_roundtrip_preserves_source_offsets_and_group_hierarchy() {
    let doc_a = parse_document(&read_sample(SAMPLE_GROUP)).expect("원본 파싱");
    let doc_b = parse_document(&serialize_document(&doc_a).expect("저장")).expect("재파싱");
    fn verify(a: &ShapeObject, b: &ShapeObject, depth: u16, nested: &mut usize, moved: &mut usize) {
        let left = a.shape_attr();
        let right = b.shape_attr();
        assert_eq!(
            std::mem::discriminant(a),
            std::mem::discriminant(b),
            "도형 종류 보존"
        );
        assert_eq!(
            (left.offset_x, left.offset_y),
            (right.offset_x, right.offset_y),
            "원본 상대 위치 메타데이터 보존"
        );
        assert_eq!(
            (left.group_level, right.group_level),
            (depth, depth),
            "소유 묶음 깊이"
        );
        assert_eq!(
            (
                left.original_width,
                left.original_height,
                left.current_width,
                left.current_height
            ),
            (
                right.original_width,
                right.original_height,
                right.current_width,
                right.current_height
            ),
            "자식 원본·현재 크기 보존"
        );
        if let Some(drawing) = a.drawing() {
            let saved_drawing = b.drawing().expect("그리기 속성 보존");
            assert_eq!(
                drawing.fill, saved_drawing.fill,
                "채우기 의미를 HWP5 왕복에서 바꾸지 않는다"
            );
            if let Some(text) = &drawing.text_box {
                assert_eq!(
                    text.vertical_align,
                    rhwp::model::table::VerticalAlign::Center,
                    "표69 bit4와 독립 한컴 재저장의 가운데 정렬"
                );
                assert_eq!(
                    saved_drawing
                        .text_box
                        .as_ref()
                        .expect("글상자")
                        .vertical_align,
                    text.vertical_align
                );
                if let Some(solid) = drawing.fill.solid {
                    assert_eq!(
                        solid.pattern_type, -1,
                        "무늬 없는 면을 저장 뒤 수평 hatch로 바꾸지 않는다"
                    );
                }
            }
        }
        let affine = |s: &ShapeObject| {
            let x = s.shape_attr();
            [
                x.render_sx,
                x.render_b,
                x.render_tx,
                x.render_c,
                x.render_sy,
                x.render_ty,
            ]
        };
        let before = affine(a);
        let after = affine(b);
        assert!(before.iter().chain(after.iter()).all(|x| x.is_finite()));
        for (x, y) in before.into_iter().zip(after) {
            assert!(
                (x - y).abs() <= f64::EPSILON,
                "전체 source affine 왕복 보존: {x} vs {y}"
            );
        }
        if depth > 0 && (left.offset_x != 0 || left.offset_y != 0) {
            *moved += 1;
        }
        match (a, b) {
            (ShapeObject::Group(ga), ShapeObject::Group(gb)) => {
                // 자식 absolute_pos가 이미 소유 묶음 이동을 포함한다.
                // 구조 묶음의 relative_pos를 paint에 다시 더하지 않는다.
                assert_eq!(
                    before,
                    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                    "구조 묶음에는 두 번째 이동을 만들지 않는다"
                );
                assert_eq!(
                    ga.children.len(),
                    gb.children.len(),
                    "자식 개수·순서·빈 묶음 보존"
                );
                if depth > 0 {
                    *nested += 1;
                }
                for (x, y) in ga.children.iter().zip(&gb.children) {
                    verify(x, y, depth + 1, nested, moved);
                }
            }
            (ShapeObject::Line(line), _)
                if left.render_b == 0.0
                    && left.render_c == 0.0
                    && left.render_sx >= 0.0
                    && left.render_sy >= 0.0 =>
            {
                // 표74 object_size/shape_info의 실제 끝점은 독립 한컴 재저장과도
                // 원본 한 단위(4HU) 이내였다. 과거 평행사변형의 크기로 재확대하지 않는다.
                for (span, scale) in [
                    (f64::from(line.end.x - line.start.x), left.render_sx),
                    (f64::from(line.end.y - line.start.y), left.render_sy),
                ] {
                    if scale > 0.0 {
                        assert!(
                            (span * scale - span).abs() <= f64::EPSILON,
                            "직선의 물리 끝점 길이를 한 번만 적용"
                        );
                    }
                }
            }
            _ => {}
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
        let mut original_moved = 0;
        verify(a, b, 0, &mut original_nested, &mut original_moved);
        assert!(original_moved > 0, "비영 상대 위치를 가진 실제 원본");
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
    // 전2쪽 N/W와 p1>=98%·직접 판독을 확인한 동일 원본의 핵심 치수선.
    // PDF의 채워진 화살촉 끝점(96.668,671.146)/(96.668,127.301)pt를
    // 841pt 용지의 상단 기준96dpi CSS px로 환산했다. shaft 끝과 혼동하지 않는다.
    fn painted(node: &RenderNode, visible: &mut Vec<(f64, f64, f64, f64)>, invisible: &mut usize) {
        match &node.node_type {
            RenderNodeType::Line(line)
                if (line.section_index, line.para_index, line.control_index)
                    == (Some(0), Some(2), Some(1)) =>
            {
                assert_eq!(
                    line.style.color >> 24,
                    0,
                    "선 없음 sentinel이 검정 선으로 부활하지 않는다"
                );
                visible.push((line.x1, line.y1, line.x2, line.y2));
            }
            RenderNodeType::Path(path)
                if (path.section_index, path.para_index, path.control_index)
                    == (Some(0), Some(2), Some(1)) =>
            {
                assert!(
                    path.style.stroke_color.is_none()
                        && path.style.stroke_width == 0.0
                        && path.line_style.is_none(),
                    "비표시 선·화살촉은 소유 경로만 보존"
                );
                *invisible += 1;
            }
            _ => {}
        }
        for child in &node.children {
            painted(child, visible, invisible);
        }
    }
    let Control::Shape(source) = &doc_a.sections[0].paragraphs[2].controls[1] else {
        panic!("소속 묶음")
    };
    let ShapeObject::Group(outer) = source.as_ref() else {
        panic!("원본 묶음")
    };
    let ShapeObject::Group(inner) = &outer.children[0] else {
        panic!("중첩 묶음")
    };
    assert_eq!(inner.children.len(), 2);
    assert_eq!(
        inner.children[0]
            .drawing()
            .expect("원본 선")
            .border_line
            .attr
            & 0x3f,
        0,
        "원본 비표시 선"
    );
    assert_ne!(
        inner.children[1]
            .drawing()
            .expect("원본 선")
            .border_line
            .attr
            & 0x3f,
        0,
        "원본 가시 선"
    );
    let mut original = DocumentCore::from_bytes(&read_sample(SAMPLE_GROUP)).expect("원본");
    let saved = original.export_hwp_with_adapter().expect("공개 저장");
    for core in [original, DocumentCore::from_bytes(&saved).expect("재열기")] {
        let tree = core.build_page_render_tree(0).expect("치수선 소속 쪽");
        let mut lines = Vec::new();
        let mut invisible = 0;
        painted(&tree.root, &mut lines, &mut invisible);
        assert_eq!(lines.len(), 1, "같은 묶음의 가시 치수선 누락·중복 방지");
        assert_eq!(invisible, 1, "선 없음 개체의 소유는 삭제하지 않는다");
        let (x1, y1, x2, y2) = lines[0];
        assert!(y1 < y2, "윗 화살촉·아랫 화살촉의 순서");
        for (actual, expected) in [
            (x1, 96.668 * 4.0 / 3.0),
            (y1, (841.0 - 671.146) * 4.0 / 3.0),
            (x2, 96.668 * 4.0 / 3.0),
            (y2, (841.0 - 127.301) * 4.0 / 3.0),
        ] {
            assert!(
                (actual - expected).abs() <= 2.0,
                "독립 Print 화살촉 끝점 ±2CSSpx: {actual} vs {expected}"
            );
        }
    }
}

// 공개 저장 경로로 만든 동일 편집본을 독립 Hancom2020 Print와 전2쪽 N/W 대조한다.
// source control points는 원본 로컬 크기의 분수로 만들며 관측 px를 복사하지 않는다.
fn curved_visibility_variant(visible: bool) -> Vec<u8> {
    use rhwp::model::shape::{ConnectorControlPoint, ConnectorData, LinkLineType};
    let mut core = DocumentCore::from_bytes(&read_sample(SAMPLE_GROUP)).expect("원본");
    let doc = core.document_mut();
    let Control::Shape(shape) = &mut doc.sections[0].paragraphs[2].controls[1] else {
        panic!("원본 묶음")
    };
    let ShapeObject::Group(group) = shape.as_mut() else {
        panic!("원본 묶음")
    };
    let ShapeObject::Group(group) = &mut group.children[0] else {
        panic!("중첩 묶음")
    };
    let ShapeObject::Line(line) = &mut group.children[0] else {
        panic!("원본 비표시 선")
    };
    assert_eq!(line.drawing.border_line.attr & 63, 0);
    let w = line.drawing.shape_attr.original_width as i32;
    let h = line.drawing.shape_attr.original_height as i32;
    line.connector = Some(ConnectorData {
        link_type: LinkLineType::ArcBoth,
        control_points: vec![
            ConnectorControlPoint {
                x: w / 3,
                y: h / 3,
                point_type: 2,
            },
            ConnectorControlPoint {
                x: w * 2 / 3,
                y: h * 2 / 3,
                point_type: 2,
            },
        ],
        ..Default::default()
    });
    if visible {
        line.drawing.border_line.attr |= 1;
        line.drawing.border_line.color = 0;
    }
    core.export_hwp_with_adapter()
        .expect("동일 편집의 공개 HWP 저장")
}

#[test]
fn issue_1892_invisible_connector_keeps_its_curve_and_selection_endpoints() {
    use rhwp::renderer::{render_tree::PathNode, PathCommand};
    fn paths<'a>(node: &'a RenderNode, out: &mut Vec<&'a PathNode>) {
        if let RenderNodeType::Path(path) = &node.node_type {
            if (path.section_index, path.para_index, path.control_index)
                == (Some(0), Some(2), Some(1))
            {
                out.push(path);
            }
        }
        for child in &node.children {
            paths(child, out);
        }
    }
    let mut geometry = None;
    for visible in [false, true] {
        let mut original =
            DocumentCore::from_bytes(&curved_visibility_variant(visible)).expect("편집본");
        assert_eq!(original.page_count(), 2, "동일 편집 독립 Print 전체 쪽수");
        let saved = original.export_hwp_with_adapter().expect("다시 저장");
        let reopened = DocumentCore::from_bytes(&saved).expect("다시 열기");
        assert_eq!(reopened.page_count(), original.page_count());
        for page in 0..original.page_count() {
            let a = original.build_page_render_tree(page).expect("편집 쪽");
            let b = reopened.build_page_render_tree(page).expect("저장 쪽");
            assert_eq!(
                owned_content(&a.root),
                owned_content(&b.root),
                "편집 뒤 쪽·문단·셀 내용 소속/순서·누락·중복"
            );
            assert_eq!(
                owned_tables(&a.root),
                owned_tables(&b.root),
                "편집 뒤 표 소유"
            );
        }
        for core in [original, reopened] {
            let tree = core.build_page_render_tree(0).expect("연결선 소속 쪽");
            let mut owned = Vec::new();
            paths(&tree.root, &mut owned);
            assert_eq!(owned.len(), 1, "같은 source 소속의 연결선 누락·중복");
            let path = owned[0];
            assert!(
                path.connector_endpoints.is_some(),
                "비표시에도 선택 끝점을 보존"
            );
            let commands: Vec<_> = path
                .commands
                .iter()
                .map(|command| match command {
                    PathCommand::MoveTo(x, y) => (0, vec![*x, *y]),
                    PathCommand::CurveTo(a, b, c, d, x, y) => (1, vec![*a, *b, *c, *d, *x, *y]),
                    _ => panic!("원본 cubic 연결선을 직선이나 다른 경로로 바꾸지 않는다"),
                })
                .collect();
            assert_eq!(
                commands.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(),
                vec![0, 1],
                "원본 두 제어점의 cubic 경로"
            );
            let actual = (commands, path.connector_endpoints);
            if let Some(expected) = &geometry {
                assert_eq!(
                    &actual, expected,
                    "가시성/재저장이 경로·선택 끝점을 바꾸지 않는다"
                );
            } else {
                geometry = Some(actual);
            }
            if visible {
                assert_eq!(path.style.stroke_color, Some(0));
                assert!(
                    path.style.stroke_width > 0.0 && path.line_style.is_some(),
                    "가시 대조군의 획과 저장된 화살촉 유지"
                );
                let style = path.line_style.as_ref().expect("저장된 끝 화살촉");
                assert_eq!(
                    style.start_arrow,
                    rhwp::renderer::ArrowStyle::None,
                    "ARC_BOTH가 원본에 없는 시작 화살촉을 만들지 않는다"
                );
                assert_eq!(
                    style.end_arrow,
                    rhwp::renderer::ArrowStyle::OpenArrow,
                    "명시된 SPEAR를 일반 삼각형으로 바꾸지 않는다"
                );
                assert_eq!(style.end_arrow_size, 0, "원본의 작은 화살촉 크기");
            } else {
                assert!(
                    path.style.stroke_color.is_none()
                        && path.style.stroke_width == 0.0
                        && path.style.fill_color.is_none()
                        && path.line_style.is_none(),
                    "선 없음은 획·채움·화살촉을 그리지 않는다"
                );
            }
        }
    }
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
