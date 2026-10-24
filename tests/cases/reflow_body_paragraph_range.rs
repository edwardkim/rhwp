//! 본문 범위 재조판의 계약. 텍스트 편집처럼 문단 vpos를 선형으로 잇지 않고,
//! 기존 단 설정 명령과 같은 새 줄을 발행하며 범위 밖 저장 줄은 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::page::{ColumnDef, PageDef};
use rhwp::model::paragraph::ColumnBreakType;
use rhwp::renderer::page_layout::PageLayoutInfo;
use serde_json::Value;

const PARAGRAPHS: usize = 80;

fn source() -> Vec<u8> {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.begin_batch_native().unwrap();
    for paragraph in 0..PARAGRAPHS {
        if paragraph > 0 {
            core.insert_paragraph_native(0, paragraph).unwrap();
        }
        core.insert_text_native(
            0,
            paragraph,
            0,
            &format!("다단 설정 표시 문단 {}", paragraph + 1),
        )
        .unwrap();
    }
    core.end_batch_native().unwrap();
    assert_eq!(core.page_count(), 2);
    core.export_hwpx_native().unwrap()
}

fn first_definition(core: &mut DocumentCore) -> &mut ColumnDef {
    core.document_mut().sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|paragraph| &mut paragraph.controls)
        .find_map(|control| match control {
            Control::ColumnDef(definition) => Some(definition),
            _ => None,
        })
        .unwrap()
}

fn equal_columns(core: &mut DocumentCore) {
    let definition = first_definition(core);
    definition.column_count = 2;
    definition.same_width = true;
    definition.spacing = 2268;
    definition.raw_attr = 0;
}

fn layout(core: &DocumentCore) -> Vec<Value> {
    (0..core.page_count())
        .map(|page| serde_json::from_str(&core.get_page_text_layout_native(page).unwrap()).unwrap())
        .collect()
}

fn caret(core: &DocumentCore, paragraph: usize) -> Value {
    serde_json::from_str(&core.get_cursor_rect_native(0, paragraph, 0).unwrap()).unwrap()
}

fn assert_body_bounds(core: &DocumentCore) {
    let section = &core.document().sections[0];
    let definition = section
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::ColumnDef(definition) => Some(definition),
            _ => None,
        })
        .unwrap();
    let page = PageLayoutInfo::from_page_def_default(&section.section_def.page_def, definition);
    let bottom = page.body_area.y + page.body_area.height;
    let layouts = layout(core);
    for (paragraph, para) in section.paragraphs.iter().enumerate() {
        assert_eq!(para.text, format!("다단 설정 표시 문단 {}", paragraph + 1));
        let rect = caret(core, paragraph);
        assert!(rect["y"].as_f64().unwrap() >= page.body_area.y - 0.1);
        assert!(
            rect["y"].as_f64().unwrap() + rect["height"].as_f64().unwrap() <= bottom + 0.1,
            "문단 {paragraph} 캐럿이 본문 아래로 나갔다: {rect}"
        );
        let runs: Vec<_> = layouts
            .iter()
            .flat_map(|page| page["runs"].as_array().unwrap())
            .filter(|run| run["paraIdx"] == paragraph)
            .collect();
        assert!(!runs.is_empty(), "문단 {paragraph} 렌더 누락");
        for run in runs {
            assert!(run["y"].as_f64().unwrap() >= page.body_area.y - 0.1);
            assert!(
                run["y"].as_f64().unwrap() + run["h"].as_f64().unwrap() <= bottom + 0.1,
                "문단 {paragraph} 글자가 본문 아래로 나갔다: {run}"
            );
        }
    }
}

#[test]
fn equal_columns_match_canonical_reflow_and_refresh_cached_render_and_carets() {
    let bytes = source();
    let mut expected = DocumentCore::from_bytes(&bytes).unwrap();
    expected.set_column_def_native(0, 2, 0, true, 2268).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let before = layout(&core);
    let stored_caret = format!("{:?}", core.document().doc_properties);
    let events = core.serialize_event_log();
    equal_columns(&mut core);
    core.reflow_body_paragraph_range_native(0, 0..PARAGRAPHS)
        .unwrap();
    assert_ne!(
        layout(&core),
        before,
        "옛 쪽 트리 캐시를 재사용하면 안 된다"
    );
    assert_eq!(layout(&core), layout(&expected));
    assert_eq!(core.page_count(), 1);
    for paragraph in 0..PARAGRAPHS {
        assert_eq!(caret(&core, paragraph), caret(&expected, paragraph));
    }
    assert_eq!(
        format!("{:?}", core.document().doc_properties),
        stored_caret
    );
    assert_eq!(core.serialize_event_log(), events);
    assert_body_bounds(&core);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.page_count(), 1);
        assert_body_bounds(&reopened);
        assert_eq!(caret(&reopened, 50)["x"], caret(&core, 50)["x"]);
    }
}

#[test]
fn separator_refresh_preserves_stored_lines_text_coordinates_and_invalid_section_state() {
    let mut original = DocumentCore::from_bytes(&source()).unwrap();
    original.set_column_def_native(0, 2, 0, true, 2268).unwrap();
    let mut core = DocumentCore::from_bytes(&original.export_hwpx_native().unwrap()).unwrap();
    let lines: Vec<_> = core.document().sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.line_segs.clone())
        .collect();
    let before = layout(&core);
    let carets: Vec<_> = (0..PARAGRAPHS)
        .map(|paragraph| caret(&core, paragraph))
        .collect();
    let svg = core.render_page_svg_native(0).unwrap();
    let events = core.serialize_event_log();
    let properties = format!("{:?}", core.document().doc_properties);
    let model = format!("{:?}", core.document());
    assert!(core.refresh_section_native(1).is_err());
    assert_eq!(format!("{:?}", core.document()), model);
    assert_eq!(layout(&core), before);
    assert_eq!(core.render_page_svg_native(0).unwrap(), svg);
    assert_eq!(core.serialize_event_log(), events);

    let definition = first_definition(&mut core);
    definition.separator_type = 1;
    definition.separator_width = 5;
    definition.separator_color = 0x0000ff;
    core.refresh_section_native(0).unwrap();
    assert_eq!(layout(&core), before, "구분선은 글자 좌표를 바꾸지 않는다");
    assert_eq!(core.page_count(), before.len() as u32);
    for (paragraph, (expected_lines, expected_caret)) in lines.iter().zip(carets).enumerate() {
        assert_eq!(
            core.document().sections[0].paragraphs[paragraph].line_segs,
            *expected_lines
        );
        assert_eq!(caret(&core, paragraph), expected_caret);
    }
    let refreshed_svg = core.render_page_svg_native(0).unwrap();
    assert_ne!(refreshed_svg, svg, "구분선이 옛 렌더 캐시를 갱신해야 한다");
    assert!(refreshed_svg.contains("stroke=\"#ff0000\""));
    assert_eq!(core.serialize_event_log(), events);
    assert_eq!(format!("{:?}", core.document().doc_properties), properties);
}

#[test]
fn unequal_columns_use_the_paragraphs_current_column_and_preserve_outside_lines() {
    let mut core = DocumentCore::from_bytes(&source()).unwrap();
    core.set_column_def_native(0, 2, 0, true, 2268).unwrap();
    let outside: Vec<_> = core.document().sections[0].paragraphs[..40]
        .iter()
        .map(|paragraph| paragraph.line_segs.clone())
        .collect();
    let definition = first_definition(&mut core);
    definition.same_width = false;
    definition.proportional_widths = false;
    definition.widths = vec![15000, 25252];
    definition.gaps = vec![2268];
    core.reflow_body_paragraph_range_native(0, 40..PARAGRAPHS)
        .unwrap();
    let paragraphs = &core.document().sections[0].paragraphs;
    assert_eq!(paragraphs[40].line_segs[0].segment_width, 15000);
    assert_eq!(paragraphs[50].line_segs[0].segment_width, 25252);
    for (paragraph, expected) in paragraphs[..40].iter().zip(outside) {
        assert_eq!(paragraph.line_segs, expected, "범위 밖 저장 줄 보존");
    }
    assert_body_bounds(&core);
    assert!(caret(&core, 50)["x"].as_f64().unwrap() > caret(&core, 40)["x"].as_f64().unwrap());
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let definition = first_definition(&mut reopened);
        assert!(definition.proportional_widths);
        assert_eq!(definition.widths, [11560, 19460]);
        assert_eq!(definition.gaps, [1748, 0]);
        assert_eq!(reopened.page_count(), core.page_count());
        for paragraph in [40, 50, 79] {
            for axis in ["x", "y"] {
                let before = caret(&core, paragraph)[axis].as_f64().unwrap();
                let after = caret(&reopened, paragraph)[axis].as_f64().unwrap();
                assert!(
                    (before - after).abs() < 0.02,
                    "{paragraph} {axis}: {before} → {after}"
                );
            }
        }
    }
    assert!(!first_definition(&mut core).proportional_widths);
    assert_eq!(first_definition(&mut core).widths, [15000, 25252]);
}

fn absolute_columns() -> ColumnDef {
    ColumnDef {
        column_count: 2,
        widths: vec![15000, 25252],
        gaps: vec![2268],
        ..Default::default()
    }
}

fn serialized(core: &DocumentCore) -> [Vec<u8>; 2] {
    [
        rhwp::serializer::serialize_hwp(core.document()).unwrap(),
        rhwp::serializer::serialize_hwpx(core.document()).unwrap(),
    ]
}

fn set_page_definition(core: &mut DocumentCore, page: PageDef) {
    let section = &mut core.document_mut().sections[0];
    section.section_def.page_def = page.clone();
    for control in &mut section.paragraphs[0].controls {
        if let Control::SectionDef(definition) = control {
            definition.page_def = page.clone();
        }
    }
}

#[test]
fn partial_body_and_unrepresentable_absolute_columns_keep_their_wire_values() {
    let input = source();
    for definition in [
        ColumnDef {
            widths: vec![200, 300],
            gaps: vec![40],
            ..absolute_columns()
        },
        ColumnDef {
            widths: vec![15000, 25252, 1],
            ..absolute_columns()
        },
        ColumnDef {
            gaps: vec![2268, 0, 1],
            ..absolute_columns()
        },
        ColumnDef {
            column_count: 4,
            widths: vec![1, 32767, 32767, 32767],
            gaps: vec![],
            ..absolute_columns()
        },
        ColumnDef {
            widths: vec![0, 25252],
            ..absolute_columns()
        },
        ColumnDef {
            gaps: vec![-1],
            ..absolute_columns()
        },
    ] {
        let mut core = DocumentCore::from_bytes(&input).unwrap();
        if definition.widths != [200, 300] {
            let total: i32 = definition
                .widths
                .iter()
                .chain(&definition.gaps)
                .map(|&value| i32::from(value))
                .sum();
            let paper_width = (total + 17008) as u32;
            let mut page = core.document().sections[0].section_def.page_def.clone();
            page.width = paper_width;
            set_page_definition(&mut core, page);
        }
        *first_definition(&mut core) = definition;
        let absolute = serialized(&core);
        first_definition(&mut core).proportional_widths = true;
        assert_eq!(
            serialized(&core),
            absolute,
            "지원하지 않는 절대 크기는 재해석하지 않는다"
        );
    }
}

#[test]
fn invalid_page_definitions_do_not_turn_render_fallbacks_into_stored_proportions() {
    let input = source();
    for (page, widths) in [
        (
            PageDef {
                height: 84188,
                ..Default::default()
            },
            vec![29764, 29764],
        ),
        (
            PageDef {
                width: 59528,
                ..Default::default()
            },
            vec![29764, 29764],
        ),
        (
            PageDef {
                width: 60000,
                height: 84188,
                margin_left: 30000,
                margin_right: 30000,
                ..Default::default()
            },
            vec![27000, 27000],
        ),
    ] {
        let mut core = DocumentCore::from_bytes(&input).unwrap();
        set_page_definition(&mut core, page);
        *first_definition(&mut core) = ColumnDef {
            widths,
            gaps: vec![],
            ..absolute_columns()
        };
        let absolute = serialized(&core);
        first_definition(&mut core).proportional_widths = true;
        assert_eq!(serialized(&core), absolute);
    }
}

#[test]
fn later_page_definitions_supply_the_body_width_in_both_control_orders() {
    let input = source();
    for columns_first in [false, true] {
        for (widths, expected) in [
            (vec![15000, 25252], vec![15000, 25252]),
            (vec![20000, 27732], vec![13107, 18174]),
        ] {
            let mut core = DocumentCore::from_bytes(&input).unwrap();
            let mut next = core.document().sections[0].section_def.clone();
            next.page_def.width += 7480;
            let paragraph = &mut core.document_mut().sections[0].paragraphs[40];
            paragraph.column_type = ColumnBreakType::Section;
            paragraph.raw_break_type = 0x01;
            paragraph.controls = vec![
                Control::SectionDef(Box::new(next)),
                Control::ColumnDef(ColumnDef {
                    widths: widths.clone(),
                    ..absolute_columns()
                }),
            ];
            if columns_first {
                paragraph.controls.swap(0, 1);
            }
            paragraph.align_ctrl_data_records();
            for offset in &mut paragraph.char_offsets {
                *offset += 16;
            }
            for shape in paragraph
                .char_shapes
                .iter_mut()
                .filter(|shape| shape.start_pos > 0)
            {
                shape.start_pos += 16;
            }
            paragraph.char_count += 16;
            for bytes in serialized(&core) {
                let reopened = DocumentCore::from_bytes(&bytes).unwrap();
                let columns = reopened
                    .document()
                    .sections
                    .iter()
                    .flat_map(|section| &section.paragraphs)
                    .flat_map(|paragraph| &paragraph.controls)
                    .find_map(|control| match control {
                        Control::ColumnDef(definition) if !definition.widths.is_empty() => {
                            Some(definition)
                        }
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(columns.widths, expected, "단 정의 우선={columns_first}");
            }
            let live = core.document().sections[0].paragraphs[40]
                .controls
                .iter()
                .find_map(|control| match control {
                    Control::ColumnDef(definition) => Some(definition),
                    _ => None,
                })
                .unwrap();
            assert_eq!(live.widths, widths);
            assert!(!live.proportional_widths);
        }
    }
}

#[test]
fn a_definition_inside_the_range_supplies_its_own_width_and_batch_defers_pagination() {
    let mut core = DocumentCore::from_bytes(&source()).unwrap();
    let before = layout(&core);
    let outside = core.document().sections[0].paragraphs[39].line_segs.clone();
    // 실제 다단 나누기처럼 경계 표시와 앞의 8유닛 단 정의 슬롯을 함께 넣는다.
    let paragraph = &mut core.document_mut().sections[0].paragraphs[40];
    paragraph.column_type = ColumnBreakType::MultiColumn;
    paragraph.raw_break_type = 0x02;
    paragraph.align_ctrl_data_records();
    paragraph.controls.insert(
        0,
        Control::ColumnDef(ColumnDef {
            column_count: 3,
            same_width: true,
            spacing: 2268,
            ..Default::default()
        }),
    );
    paragraph.ctrl_data_records.insert(0, None);
    for offset in &mut paragraph.char_offsets {
        *offset += 8;
    }
    for shape in paragraph
        .char_shapes
        .iter_mut()
        .filter(|shape| shape.start_pos > 0)
    {
        shape.start_pos += 8;
    }
    paragraph.char_count += 8;
    core.begin_batch_native().unwrap();
    core.reflow_body_paragraph_range_native(0, 40..PARAGRAPHS)
        .unwrap();
    assert_eq!(core.page_count(), 2, "배치 중에는 쪽을 나누지 않는다");
    let paragraphs = &core.document().sections[0].paragraphs;
    assert_eq!(paragraphs[39].line_segs, outside);
    let width = paragraphs[40].line_segs[0].segment_width;
    assert!(
        width < 15000,
        "구역 첫 한 단 폭 대신 현재 세 단 폭: {width}"
    );
    assert!(paragraphs[40..]
        .iter()
        .all(|paragraph| paragraph.line_segs[0].segment_width == width));
    core.end_batch_native().unwrap();
    assert!(
        layout(&core) != before,
        "실제 다단 경계의 배치가 바뀌어야 한다"
    );
}

#[test]
fn empty_or_invalid_ranges_preserve_model_events_render_and_snapshots() {
    let mut core = DocumentCore::from_bytes(&source()).unwrap();
    let snapshot = core.save_snapshot_native();
    let model = format!("{:?}", core.document());
    let before = layout(&core);
    let events = core.serialize_event_log();
    core.reflow_body_paragraph_range_native(0, 0..0).unwrap();
    core.reflow_body_paragraph_range_native(0, PARAGRAPHS..PARAGRAPHS)
        .unwrap();
    for (section, start, end) in [(1, 0, 0), (0, 2, 1), (0, 0, 81), (0, 81, 81)] {
        assert!(core
            .reflow_body_paragraph_range_native(section, start..end)
            .is_err());
    }
    assert_eq!(format!("{:?}", core.document()), model);
    assert_eq!(layout(&core), before);
    assert_eq!(core.serialize_event_log(), events);
    core.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(format!("{:?}", core.document()), model);
    assert_eq!(layout(&core), before);

    let definition = first_definition(&mut core);
    definition.column_count = 8;
    definition.spacing = i16::MAX;
    let model = format!("{:?}", core.document());
    let before = layout(&core);
    let events = core.serialize_event_log();
    assert!(core
        .reflow_body_paragraph_range_native(0, 0..PARAGRAPHS)
        .is_err());
    assert_eq!(format!("{:?}", core.document()), model);
    assert_eq!(layout(&core), before);
    assert_eq!(core.serialize_event_log(), events);
}

#[test]
fn distributed_and_parallel_columns_preserve_canonical_reflow_and_saved_content() {
    use rhwp::model::page::ColumnType;

    let bytes = source();
    for (kind, column_type) in [(1, ColumnType::Distribute), (2, ColumnType::Parallel)] {
        let mut expected = DocumentCore::from_bytes(&bytes).unwrap();
        expected
            .set_column_def_native(0, 2, kind, true, 2268)
            .unwrap();
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();
        equal_columns(&mut core);
        first_definition(&mut core).column_type = column_type;
        core.reflow_body_paragraph_range_native(0, 0..PARAGRAPHS)
            .unwrap();
        assert_eq!(layout(&core), layout(&expected));
        assert_eq!(core.page_count(), expected.page_count());
        assert_body_bounds(&core);
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(first_definition(&mut reopened).column_type, column_type);
            assert_eq!(reopened.page_count(), core.page_count());
            assert_body_bounds(&reopened);
        }
    }
}
