//! [#7418] 머리글/꼬리글 표의 낡은 표 선언 높이(`common.height`) 계약.
//!
//! 법제처 국가법령정보센터 HWPX 의 머리글(1×1)·꼬리글(1×3) 표는 칸 선언(`cellSz` 1982HU = 26.43px)이
//! 한 줄 내용을 이미 담는데, 표 개체 선언(`hp:sz` 56850HU = 758.0px)이 낡은 값으로 남아 있다.
//! 머리글/꼬리글 표는 측정 표 없이(`measured_table = None`) 그려지므로 `fit_row_heights_to_common_height`
//! 가 마지막 행을 표 선언까지 늘렸고, 꼬리글이 쪽 밖으로 밀렸다.
//!
//! 공개 API 로 만든 합성 문서에서 표 상자의 높이를 잠근다. 아래 넷이 **양쪽을 잠근다** —
//! 낡은 표 선언을 따르면 ①이, 격차가 작은 표까지 칸 선언으로 줄이면 ②가, 내용이 키운 행을 무시하면 ③이,
//! 떠 있는 표로 번지면 ④가 깨진다.
//!
//! ```text
//!   ① 글자처럼 취급(TAC) 표, 칸 선언 26.43px 이 내용을 담고 표 선언 758.0px (28.7배)  → 26.43px
//!   ② TAC 표, 칸 선언 351.4px 이 내용을 담고 표 선언 357.2px (1.02배)                  → 357.2px
//!   ③ TAC 표, 칸 선언 3.79px 이 최소값이라 내용(48px)이 행을 키운 표, 표 선언 200px    → 200px
//!   ④ 떠 있는(비-TAC) 표, ①과 같은 선언                                                 → 758.0px
//! ```
//!
//! ④는 이 검사가 근거를 확보한 범위(TAC 머리글/꼬리글)를 벗어난 표를 건드리지 않는다는 선이다.
//! 쪽 아래 정렬 고정 틀은 비-TAC 이고 표 선언이 바깥 높이를 정한다 —
//! `issue_1658_page_bottom_fixed_exclusion` 의 관악구 틀(선언 357.2px·칸 합 351.4px)이 그 사례다.
//! 범위를 넓히려면 근거와 함께 ④를 바꾼다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Section;
use rhwp::model::header_footer::{Footer, Header, HeaderFooterApply};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 1 HWPUNIT 이 96dpi 에서 차지하는 픽셀 수(7200 HWPUNIT = 1inch = 96px).
const PX_PER_HWPUNIT: f64 = 96.0 / 7200.0;

/// 합성 표를 담을 머리글/꼬리글 영역.
#[derive(Debug, Clone, Copy)]
enum HeaderFooterRegion {
    Header,
    Footer,
}

/// 한 칸짜리 표의 형상 — 칸 선언과 표 선언만 다르게 준다.
#[derive(Debug, Clone, Copy)]
struct SingleCellTableShape {
    treat_as_char: bool,
    cell_declared_height: u32,
    object_declared_height: u32,
    content_line_count: usize,
}

fn hwpunit_to_px(hwpunit: u32) -> f64 {
    f64::from(hwpunit) * PX_PER_HWPUNIT
}

fn content_paragraph(line_count: usize) -> Paragraph {
    let line_segs = (0..line_count)
        .map(|line_index| LineSeg {
            vertical_pos: i32::try_from(line_index).expect("줄 번호") * 1200,
            line_height: 1200,
            text_height: 1200,
            baseline_distance: 1020,
            segment_width: 40000,
            ..Default::default()
        })
        .collect();
    Paragraph {
        text: "가나다".to_owned(),
        char_count: 3,
        char_offsets: (0..=3).collect(),
        line_segs,
        ..Default::default()
    }
}

fn single_cell_table(shape: SingleCellTableShape) -> Table {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            col_span: 1,
            row_span: 1,
            width: 40000,
            height: shape.cell_declared_height,
            paragraphs: vec![content_paragraph(shape.content_line_count)],
            ..Default::default()
        }],
        ..Default::default()
    };
    table.common.width = 40000;
    table.common.height = shape.object_declared_height;
    table.common.treat_as_char = shape.treat_as_char;
    table.rebuild_grid();
    table
}

/// 표 하나를 머리글/꼬리글 문단에 넣은 한 쪽짜리 문서를 조판해 그 영역의 표 상자 높이를 돌려준다.
fn table_box_height_in_region(region: HeaderFooterRegion, shape: SingleCellTableShape) -> f64 {
    let region_paragraph = Paragraph {
        controls: vec![Control::Table(Box::new(single_cell_table(shape)))],
        ..Default::default()
    };
    let region_control = match region {
        HeaderFooterRegion::Header => Control::Header(Box::new(Header {
            apply_to: HeaderFooterApply::Both,
            paragraphs: vec![region_paragraph],
            ..Default::default()
        })),
        HeaderFooterRegion::Footer => Control::Footer(Box::new(Footer {
            apply_to: HeaderFooterApply::Both,
            paragraphs: vec![region_paragraph],
            ..Default::default()
        })),
    };
    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59529,
        height: 84189,
        margin_left: 8504,
        margin_right: 8504,
        margin_top: 5668,
        margin_bottom: 4252,
        margin_header: 4252,
        margin_footer: 4252,
        ..Default::default()
    };
    section.paragraphs.push(Paragraph {
        controls: vec![region_control],
        ..Default::default()
    });
    // 기본 글자 모양/글꼴을 갖춘 문서에서 출발해야 TextLine 이 생성된다.
    let mut core = DocumentCore::new_empty();
    let mut document = core.document().clone();
    document.doc_info.para_shapes = vec![ParaShape::default()];
    document.sections = vec![section];
    core.set_document(document);
    let page_root = core
        .build_page_render_tree(0)
        .expect("합성 문서 첫 쪽 렌더")
        .root;
    table_box_height_inside_region(&page_root, region)
}

fn table_box_height_inside_region(page_root: &RenderNode, region: HeaderFooterRegion) -> f64 {
    fn find_region_node(node: &RenderNode, region: HeaderFooterRegion) -> Option<&RenderNode> {
        let is_region = matches!(
            (&node.node_type, region),
            (RenderNodeType::Header, HeaderFooterRegion::Header)
                | (RenderNodeType::Footer, HeaderFooterRegion::Footer)
        );
        if is_region {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find_region_node(child, region))
    }
    fn find_table_box_height(node: &RenderNode) -> Option<f64> {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            return Some(node.bbox.height);
        }
        node.children.iter().find_map(find_table_box_height)
    }
    let region_node = find_region_node(page_root, region)
        .unwrap_or_else(|| panic!("{region:?} 영역 노드가 렌더 트리에 있어야 한다"));
    find_table_box_height(region_node)
        .unwrap_or_else(|| panic!("{region:?} 영역 안에 표 노드가 있어야 한다"))
}

fn assert_table_box_height(
    region: HeaderFooterRegion,
    shape: SingleCellTableShape,
    expected_px: f64,
    meaning: &str,
) {
    let actual_px = table_box_height_in_region(region, shape);
    println!(
        "[{region:?}] {meaning}: 표 상자 {actual_px:.2}px (기대 {expected_px:.2}px) — {shape:?}"
    );
    assert!(
        (actual_px - expected_px).abs() < 0.5,
        "[{region:?}] {meaning}: 표 상자 높이 {actual_px:.2}px, 기대 {expected_px:.2}px — {shape:?}"
    );
}

#[test]
fn inline_table_keeps_cell_declared_height_when_object_height_is_stale() {
    for region in [HeaderFooterRegion::Header, HeaderFooterRegion::Footer] {
        let shape = SingleCellTableShape {
            treat_as_char: true,
            cell_declared_height: 1982,
            object_declared_height: 56850,
            content_line_count: 1,
        };
        assert_table_box_height(
            region,
            shape,
            hwpunit_to_px(1982),
            "칸 선언이 내용을 담고 표 선언이 28.7배면 칸 선언 높이",
        );
    }
}

#[test]
fn inline_table_with_slightly_taller_object_height_follows_object_height() {
    for region in [HeaderFooterRegion::Header, HeaderFooterRegion::Footer] {
        let shape = SingleCellTableShape {
            treat_as_char: true,
            cell_declared_height: 26355,
            object_declared_height: 26793,
            content_line_count: 1,
        };
        assert_table_box_height(
            region,
            shape,
            hwpunit_to_px(26793),
            "표 선언이 칸 선언보다 2배 미만으로 크면 표 선언 높이",
        );
    }
}

#[test]
fn inline_table_grown_by_content_follows_object_height() {
    for region in [HeaderFooterRegion::Header, HeaderFooterRegion::Footer] {
        let shape = SingleCellTableShape {
            treat_as_char: true,
            cell_declared_height: 284,
            object_declared_height: 15000,
            content_line_count: 3,
        };
        assert_table_box_height(
            region,
            shape,
            hwpunit_to_px(15000),
            "내용이 최소값 칸 선언을 넘어 행을 키웠으면 표 선언 높이",
        );
    }
}

#[test]
fn floating_table_is_not_subject_to_the_stale_object_height_check() {
    for region in [HeaderFooterRegion::Header, HeaderFooterRegion::Footer] {
        let shape = SingleCellTableShape {
            treat_as_char: false,
            cell_declared_height: 1982,
            object_declared_height: 56850,
            content_line_count: 1,
        };
        assert_table_box_height(
            region,
            shape,
            hwpunit_to_px(56850),
            "떠 있는 표는 낡은 값 검사 대상이 아니라 표 선언 높이",
        );
    }
}
