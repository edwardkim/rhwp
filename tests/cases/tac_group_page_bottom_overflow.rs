//! Native HWP5의 저장 page-tail 소유권을 보존하는 페이지네이션 회귀.
//!
//! `atomic TAC top-fit`은 차트/그림처럼 분할할 수 없는 인라인 개체가 하단 여백에
//! 조금 걸쳐도 현재 쪽에 남을 수 있게 한다. 그러나 저장 줄이 `vpos=0`으로 새 쪽
//! 시작을 기록하고 앞 줄의 저장 하단과 실제 flow가 모두 본문 끝에 닿은 경우까지
//! 이 예외를 적용하면 묶음 제목이 이전 쪽 밖에 그려진다.
//!
//! 다행 `RowBreak` 표는 저장된 첫 조각과 object frame의 쪽 소유권도 보존해야 한다.
//! 단, source top과 현재 flow가 사실상 같은 표를 과거 anchor로 되감으면 일반 표가
//! 여러 쪽씩 압축되므로 실제 host-spacing drift와 page-tail 여유를 함께 요구한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{GroupShape, ShapeObject, TextWrap, VertRelTo};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table, TablePageBreak};

fn line(vertical_pos: i32, line_height: i32, line_spacing: i32) -> LineSeg {
    LineSeg {
        text_start: 0,
        vertical_pos,
        line_height,
        text_height: line_height,
        baseline_distance: line_height * 85 / 100,
        line_spacing,
        segment_width: 48_188,
        ..Default::default()
    }
}

fn document(title_vpos: i32) -> Document {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];

    // 본문 높이 70,018 HU 중 69,079 HU를 쓰는 저장 줄. 줄의 시작 vpos는 낮지만
    // `vpos + line_height`는 쪽 하단에 닿는다.
    let before = Paragraph {
        text: "앞".to_string(),
        char_count: 1,
        char_offsets: vec![0, 1],
        line_segs: vec![line(1_760, 67_219, 660)],
        ..Default::default()
    };

    let mut group = GroupShape::default();
    group.common.treat_as_char = true;
    group.common.text_wrap = TextWrap::InFrontOfText;
    group.common.width = 26_101;
    group.common.height = 1_825;
    group.common.margin.top = 567;

    let title = Paragraph {
        char_count: 9,
        line_segs: vec![line(title_vpos, 2_392, 660)],
        controls: vec![Control::Shape(Box::new(ShapeObject::Group(group)))],
        ..Default::default()
    };

    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59_528,
        height: 84_188,
        margin_left: 5_669,
        margin_right: 5_669,
        margin_top: 4_535,
        margin_bottom: 3_685,
        margin_header: 3_118,
        margin_footer: 2_834,
        ..Default::default()
    };
    section.paragraphs = vec![before, title];
    doc.sections.push(section);
    doc
}

fn page_has_item(core: &DocumentCore, page: u32, para_index: u64, kind: &str) -> bool {
    core.dump_page_items_json(Some(page))
        .as_array()
        .and_then(|pages| pages.first())
        .and_then(|page| page.get("columns"))
        .and_then(|columns| columns.as_array())
        .into_iter()
        .flatten()
        .filter_map(|column| column.get("items").and_then(|items| items.as_array()))
        .flatten()
        .any(|item| {
            item.get("paraIndex").and_then(|value| value.as_u64()) == Some(para_index)
                && item.get("kind").and_then(|value| value.as_str()) == Some(kind)
        })
}

fn rowbreak_table_paragraph(
    anchor_vpos: i32,
    common_height: u32,
    row_heights: &[u32],
) -> Paragraph {
    let mut table = Table {
        row_count: row_heights.len() as u16,
        col_count: 1,
        row_sizes: row_heights.iter().map(|&height| height as i16).collect(),
        cells: row_heights
            .iter()
            .enumerate()
            .map(|(row, &height)| Cell {
                col: 0,
                row: row as u16,
                col_span: 1,
                row_span: 1,
                width: 20_000,
                height,
                paragraphs: vec![Paragraph {
                    text: format!("행 {row}"),
                    char_count: 3,
                    line_segs: vec![line(0, 1_100, 0)],
                    ..Default::default()
                }],
                ..Default::default()
            })
            .collect(),
        page_break: TablePageBreak::RowBreak,
        ..Default::default()
    };
    table.common.width = 20_000;
    table.common.height = common_height;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.vert_rel_to = VertRelTo::Para;
    table.rebuild_grid();

    Paragraph {
        line_segs: vec![line(anchor_vpos, 1_100, 660)],
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    }
}

fn rowbreak_document(paragraphs: Vec<Paragraph>) -> DocumentCore {
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];

    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59_528,
        height: 84_188,
        margin_left: 5_669,
        margin_right: 5_669,
        margin_top: 4_535,
        margin_bottom: 3_685,
        margin_header: 3_118,
        margin_footer: 2_834,
        ..Default::default()
    };
    section.paragraphs = paragraphs;
    doc.sections.push(section);

    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    core
}

#[test]
fn overflowing_tac_group_starts_on_the_next_page() {
    let mut core = DocumentCore::new_empty();
    core.set_document(document(0));

    assert_eq!(
        core.page_count(),
        2,
        "본문 하단을 넘는 TAC 묶음 제목은 이전 쪽 여백에 스필하지 않아야 한다"
    );
}

#[test]
fn stored_same_page_atomic_group_keeps_the_existing_small_spill() {
    let mut core = DocumentCore::new_empty();
    core.set_document(document(68_200));

    assert_eq!(
        core.page_count(),
        1,
        "현재 쪽 하단을 가리키는 저장 줄에는 기존 atomic top-fit 예외를 유지해야 한다"
    );
}

#[test]
fn saved_split_anchor_keeps_the_atomic_first_row_on_its_source_page() {
    let core = rowbreak_document(vec![
        Paragraph {
            text: "앞".to_string(),
            char_count: 1,
            line_segs: vec![line(0, 67_424, 660)],
            ..Default::default()
        },
        rowbreak_table_paragraph(68_034, 21_948, &[1_948, 20_000]),
        Paragraph {
            text: "다음 쪽".to_string(),
            char_count: 4,
            line_segs: vec![line(22_202, 1_100, 660)],
            ..Default::default()
        },
    ]);

    assert_eq!(
        core.page_count(),
        2,
        "저장된 첫 행 조각 때문에 쪽이 늘면 안 된다"
    );
    assert!(
        page_has_item(&core, 0, 1, "partialTable"),
        "첫 행의 보이는 내용이 들어가고 다음 문단이 되감기면 표의 첫 조각은 저장 원본 쪽에 남아야 한다"
    );
}

#[test]
fn saved_rowbreak_object_frame_resynchronizes_small_host_spacing_drift() {
    let core = rowbreak_document(vec![
        Paragraph {
            text: "앞".to_string(),
            char_count: 1,
            line_segs: vec![line(0, 53_800, 660)],
            ..Default::default()
        },
        rowbreak_table_paragraph(54_224, 15_409, &[7_704, 7_705]),
        Paragraph {
            text: "다음 쪽".to_string(),
            char_count: 4,
            line_segs: vec![line(0, 1_100, 660)],
            ..Default::default()
        },
    ]);

    assert_eq!(
        core.page_count(),
        2,
        "저장 프레임 안에 드는 표를 작은 host-spacing 오차로 이월해 빈 쪽을 만들면 안 된다"
    );
    assert!(
        page_has_item(&core, 0, 1, "table"),
        "표 전체의 저장 하단이 본문 안에 있고 다음 문단이 되감기면 저장 top으로 재동기화해야 한다"
    );
}

#[test]
fn sub_pixel_host_spacing_drift_does_not_turn_an_oversized_table_into_a_saved_frame() {
    let core = rowbreak_document(vec![
        Paragraph {
            text: "앞".to_string(),
            char_count: 1,
            line_segs: vec![line(0, 53_800, 660)],
            ..Default::default()
        },
        // 저장 top은 현재 flow보다 60HU(0.8px) 앞서지만, 그 정도 부동소수점/반올림
        // 차이까지 saved-frame 재동기화로 취급하면 실측 전체 높이가 남은 본문을
        // 아주 조금 넘는 일반 표도 이전 쪽으로 되감긴다.
        rowbreak_table_paragraph(54_400, 15_565, &[7_782, 7_783]),
        Paragraph {
            text: "다음 쪽".to_string(),
            char_count: 4,
            line_segs: vec![line(0, 1_100, 660)],
            ..Default::default()
        },
    ]);

    assert_eq!(
        core.page_count(),
        3,
        "일반 이월·후속 저장 되감김이 만든 기존 쪽 경계를 유지해야 한다"
    );
    assert!(
        !page_has_item(&core, 0, 1, "table") && !page_has_item(&core, 0, 1, "partialTable"),
        "실측 표 높이가 남은 본문보다 크면 기존 이월 결정을 유지해야 한다"
    );
}
