//! [#4438] HWPX -> HWP 표 layout CTRL_DATA의 item id와 저장 snapshot 계약.
//!
//! 이 payload는 일반적인 item-id 할당 규칙이 아니라, 변환 경계에서만 재현하는
//! 고정된 104바이트 HWP5 저장 계약이다. 장기 수명의 호출자는 adapter를 원본 IR이
//! 아닌 snapshot에 적용해야 한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table, TablePageBreak};
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags;

const TABLE_LAYOUT_CTRL_DATA: [u8; 104] = [
    0x1b, 0x02, 0x01, 0x00, 0x00, 0x00, 0x42, 0x02, 0x00, 0x80, 0x42, 0x02, 0x0b, 0x00, 0x00, 0x00,
    0x00, 0x40, 0x04, 0x00, 0xf2, 0x0e, 0x00, 0x00, 0x01, 0x40, 0x04, 0x00, 0x18, 0x04, 0x00, 0x00,
    0x02, 0x40, 0x04, 0x00, 0xba, 0x6e, 0x00, 0x00, 0x03, 0x40, 0x04, 0x00, 0x1b, 0x21, 0x00, 0x00,
    0x04, 0x40, 0x04, 0x00, 0xc4, 0x02, 0x00, 0x00, 0x05, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x06, 0x40, 0x04, 0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x40, 0x04, 0x00, 0x09, 0x00, 0x00, 0x00,
    0x08, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x40, 0x04, 0x00, 0x88, 0xe8, 0x00, 0x00,
    0x0a, 0x40, 0x04, 0x00, 0xdc, 0x48, 0x01, 0x00,
];

const NON_SEQUENTIAL_ITEM_IDS: [u16; 11] = [
    0x401a, 0x4001, 0x4f20, 0x0201, 0x7777, 0x1337, 0x4010, 0x0007, 0x3555, 0x6abc, 0x400a,
];

fn top_level_table_ctrl_data(document: &Document) -> Vec<&[u8]> {
    document
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .filter_map(|(control_index, control)| {
                    matches!(control, Control::Table(_))
                        .then(|| paragraph.ctrl_data_records.get(control_index))
                        .flatten()
                        .and_then(Option::as_deref)
                })
        })
        .collect()
}

fn nested_cell_table_ctrl_data(document: &Document) -> Vec<&[u8]> {
    document
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .flat_map(|table| &table.cells)
        .flat_map(|cell| &cell.paragraphs)
        .flat_map(|paragraph| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .filter_map(|(control_index, control)| {
                    matches!(control, Control::Table(_))
                        .then(|| paragraph.ctrl_data_records.get(control_index))
                        .flatten()
                        .and_then(Option::as_deref)
                })
        })
        .collect()
}

fn top_level_adapter_policy_discriminators(
    document: &Document,
) -> Vec<(u16, u16, bool, TablePageBreak)> {
    document
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Table(table) => Some((
                table.row_count,
                table.col_count,
                table.repeat_header,
                table.page_break,
            )),
            _ => None,
        })
        .collect()
}

fn table_layout_payload_with_ids(item_ids: [u16; 11]) -> Vec<u8> {
    let mut payload = TABLE_LAYOUT_CTRL_DATA.to_vec();
    for (index, item_id) in item_ids.into_iter().enumerate() {
        let offset = 16 + index * 8;
        payload[offset..offset + 2].copy_from_slice(&item_id.to_le_bytes());
    }
    payload
}

fn replace_first_top_level_table_ctrl_data(core: &mut DocumentCore, payload: Vec<u8>) {
    for section in &mut core.document_mut().sections {
        for paragraph in &mut section.paragraphs {
            if let Some(control_index) = paragraph
                .controls
                .iter()
                .position(|control| matches!(control, Control::Table(_)))
            {
                paragraph.align_ctrl_data_records();
                paragraph.ctrl_data_records[control_index] = Some(payload);
                section.raw_stream = None;
                return;
            }
        }
    }
    panic!("synthetic document must contain a top-level table");
}

fn section0_records(hwp: &[u8]) -> Vec<Record> {
    let parsed = DocumentCore::from_bytes(hwp).expect("HWP header parse");
    let compressed = parsed.document().header.compressed;
    let distribution = parsed.document().header.distribution;
    let mut cfb = CfbReader::open(hwp).expect("HWP CFB open");
    let section = cfb
        .read_body_text_section(0, compressed, distribution)
        .expect("BodyText/Section0 read");
    Record::read_all(&section).expect("BodyText records")
}

fn first_table_ctrl_data_record(hwp: &[u8]) -> (Record, Record) {
    let records = section0_records(hwp);
    let header_index = records
        .iter()
        .position(|record| {
            record.tag_id == tags::HWPTAG_CTRL_HEADER
                && record.data.starts_with(&tags::CTRL_TABLE.to_le_bytes())
        })
        .expect("table CTRL_HEADER");
    (
        records[header_index].clone(),
        records
            .get(header_index + 1)
            .expect("record after table CTRL_HEADER")
            .clone(),
    )
}

fn synthetic_hwpx_source() -> DocumentCore {
    // 이 합성 표는 현재 adapter predicate를 격리하는 policy fixture다. 모든 3x2 표의
    // 보편적인 HWP5 의미를 주장하는 근거로 사용하지 않는다.
    let input = std::fs::read("samples/hwpx/blank_hwpx.hwpx").expect("HWPX fixture");
    let mut core = DocumentCore::from_bytes(&input).expect("HWPX parse");
    let paragraph = core.document_mut().sections[0]
        .paragraphs
        .first_mut()
        .expect("fixture body paragraph");
    paragraph.controls.push(Control::Table(Box::new(Table {
        row_count: 3,
        col_count: 2,
        page_break: TablePageBreak::RowBreak,
        repeat_header: true,
        ..Default::default()
    })));
    paragraph.align_ctrl_data_records();
    core
}

fn synthetic_nested_hwpx_source() -> DocumentCore {
    let input = std::fs::read("samples/hwpx/blank_hwpx.hwpx").expect("HWPX fixture");
    let mut core = DocumentCore::from_bytes(&input).expect("HWPX parse");
    let mut nested_paragraph = Paragraph::default();
    nested_paragraph
        .controls
        .push(Control::Table(Box::new(Table {
            row_count: 3,
            col_count: 2,
            page_break: TablePageBreak::RowBreak,
            repeat_header: true,
            ..Default::default()
        })));
    nested_paragraph.align_ctrl_data_records();

    let paragraph = core.document_mut().sections[0]
        .paragraphs
        .first_mut()
        .expect("fixture body paragraph");
    paragraph.controls.push(Control::Table(Box::new(Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            col_span: 1,
            row_span: 1,
            paragraphs: vec![nested_paragraph],
            ..Default::default()
        }],
        ..Default::default()
    })));
    paragraph.align_ctrl_data_records();
    core
}

#[test]
fn snapshot_export_emits_exact_table_ctrl_data_without_mutating_source_ir() {
    let core = synthetic_hwpx_source();

    let source_projection_before = core.export_hwpx_native().expect("source HWPX projection");
    let source_payloads_before = top_level_table_ctrl_data(core.document())
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>();
    let source_extra_streams_before = core.document().extra_streams.clone();

    let hwp = core
        .export_hwp_with_adapter_snapshot()
        .expect("snapshot HWP export");

    assert_eq!(
        top_level_table_ctrl_data(core.document())
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect::<Vec<_>>(),
        source_payloads_before,
        "adapter materialization must stay in the export snapshot"
    );
    assert_eq!(
        core.document().extra_streams,
        source_extra_streams_before,
        "the HWPX-origin marker must also stay in the export snapshot"
    );
    assert_eq!(
        core.export_hwpx_native()
            .expect("source HWPX projection after snapshot export"),
        source_projection_before,
        "snapshot HWP export must leave the complete serialized HWPX projection unchanged"
    );

    let reloaded = DocumentCore::from_bytes(&hwp).expect("exported HWP reload");
    let emitted = top_level_table_ctrl_data(reloaded.document());
    assert_eq!(emitted.len(), 1, "synthetic table CTRL_DATA count");
    for payload in emitted {
        assert_eq!(payload, TABLE_LAYOUT_CTRL_DATA);
    }
}

#[test]
fn nested_cell_owner_survives_snapshot_and_hwp_raw_regeneration() {
    let source = synthetic_nested_hwpx_source();
    let source_projection_before = source
        .export_hwpx_native()
        .expect("nested source HWPX projection");
    assert!(nested_cell_table_ctrl_data(source.document()).is_empty());

    let first_hwp = source
        .export_hwp_with_adapter_snapshot()
        .expect("nested snapshot HWP export");
    assert!(
        nested_cell_table_ctrl_data(source.document()).is_empty(),
        "nested adapter payload must stay out of the live HWPX source"
    );
    assert_eq!(
        source
            .export_hwpx_native()
            .expect("nested HWPX projection after snapshot export"),
        source_projection_before,
        "nested snapshot export changed the complete serialized HWPX source projection"
    );

    let mut parsed_hwp = DocumentCore::from_bytes(&first_hwp).expect("nested HWP reload");
    assert_eq!(
        nested_cell_table_ctrl_data(parsed_hwp.document()),
        vec![TABLE_LAYOUT_CTRL_DATA.as_slice()],
        "HWP parser must attach the exact payload to the nested table owner"
    );
    let first_placements = section0_records(&first_hwp)
        .windows(2)
        .filter(|records| {
            records[0].tag_id == tags::HWPTAG_CTRL_HEADER
                && records[0].data.starts_with(&tags::CTRL_TABLE.to_le_bytes())
                && records[1].tag_id == tags::HWPTAG_CTRL_DATA
                && records[1].data == TABLE_LAYOUT_CTRL_DATA
        })
        .map(|records| (records[0].level, records[1].level))
        .collect::<Vec<_>>();
    assert_eq!(first_placements.len(), 1, "nested exact payload placement");
    assert_eq!(first_placements[0].1, first_placements[0].0 + 1);

    for section in &mut parsed_hwp.document_mut().sections {
        section.raw_stream = None;
    }
    let nested_payloads_before = nested_cell_table_ctrl_data(parsed_hwp.document())
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>();
    let regenerated_hwp = parsed_hwp
        .export_hwp_native()
        .expect("nested HWP raw regeneration");
    assert_eq!(
        nested_cell_table_ctrl_data(parsed_hwp.document())
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect::<Vec<_>>(),
        nested_payloads_before,
        "nested HWP serialization changed the source owner payload"
    );
    let regenerated = DocumentCore::from_bytes(&regenerated_hwp).expect("regenerated HWP reload");
    assert_eq!(
        nested_cell_table_ctrl_data(regenerated.document()),
        vec![TABLE_LAYOUT_CTRL_DATA.as_slice()]
    );
    let regenerated_placements = section0_records(&regenerated_hwp)
        .windows(2)
        .filter(|records| {
            records[0].tag_id == tags::HWPTAG_CTRL_HEADER
                && records[0].data.starts_with(&tags::CTRL_TABLE.to_le_bytes())
                && records[1].tag_id == tags::HWPTAG_CTRL_DATA
                && records[1].data == TABLE_LAYOUT_CTRL_DATA
        })
        .map(|records| (records[0].level, records[1].level))
        .collect::<Vec<_>>();
    assert_eq!(regenerated_placements, first_placements);
}

/// HWP 전용 raw payload의 owner는 parser가 채운 `ctrl_data_records`이고, serializer는
/// item id를 해석하거나 연속 번호로 다시 쓰지 않는다. 첫 HWP는 비연속 ID를 가진 parser
/// 입력을 만들고, 두 번째 HWP가 실제 parser -> model -> serializer 보존 경계다.
#[test]
fn hwp_parser_model_serializer_preserves_non_sequential_opaque_item_ids() {
    let seed_hwp = synthetic_hwpx_source()
        .export_hwp_with_adapter_snapshot()
        .expect("seed HWP export");
    let opaque_payload = table_layout_payload_with_ids(NON_SEQUENTIAL_ITEM_IDS);

    let mut seed_model = DocumentCore::from_bytes(&seed_hwp).expect("seed HWP reload");
    replace_first_top_level_table_ctrl_data(&mut seed_model, opaque_payload.clone());
    let parser_input = seed_model
        .export_hwp_native()
        .expect("non-sequential parser input export");
    let (input_header, input_data) = first_table_ctrl_data_record(&parser_input);
    assert_eq!(input_data.tag_id, tags::HWPTAG_CTRL_DATA);
    assert_eq!(input_data.level, input_header.level + 1);
    assert_eq!(input_data.data, opaque_payload);

    let mut parsed = DocumentCore::from_bytes(&parser_input).expect("opaque HWP parse");
    assert_eq!(
        top_level_table_ctrl_data(parsed.document()),
        vec![opaque_payload.as_slice()],
        "HWP parser must retain the opaque payload in the table control slot"
    );
    for section in &mut parsed.document_mut().sections {
        section.raw_stream = None;
    }
    let source_payloads_before = top_level_table_ctrl_data(parsed.document())
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>();
    let source_streams_before = parsed.document().extra_streams.clone();

    let reserialized = parsed
        .export_hwp_native()
        .expect("parser-owned opaque HWP reserialize");
    assert_eq!(
        top_level_table_ctrl_data(parsed.document())
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect::<Vec<_>>(),
        source_payloads_before,
        "HWP serialization must not rewrite the source model payload"
    );
    assert_eq!(parsed.document().extra_streams, source_streams_before);

    let (output_header, output_data) = first_table_ctrl_data_record(&reserialized);
    assert_eq!(output_data.tag_id, tags::HWPTAG_CTRL_DATA);
    assert_eq!(output_data.level, output_header.level + 1);
    assert_eq!(output_data.data, opaque_payload);
    let reloaded = DocumentCore::from_bytes(&reserialized).expect("reserialized HWP reload");
    assert_eq!(
        top_level_table_ctrl_data(reloaded.document()),
        vec![opaque_payload.as_slice()]
    );
}

/// Multiplexed IR 경계의 불변식은 포맷별로 나눈다.
///
/// - HWP5 구간: opaque CTRL_DATA 104바이트가 exact해야 한다.
/// - HWPX/HWP5 공통 구간: 표의 row/column/repeat/page-break semantic이 같아야 한다.
/// - snapshot export: 어느 방향에서도 호출자의 live IR을 바꾸지 않아야 한다.
///
/// CFB와 ZIP은 서로 다른 container이므로 전체 출력 bytes 동일성은 계약이 아니다.
#[test]
fn adapter_policy_boundaries_preserve_format_specific_invariants() {
    let hwpx_source = synthetic_hwpx_source();
    let expected_discriminators = top_level_adapter_policy_discriminators(hwpx_source.document());
    let hwpx_projection_before = hwpx_source
        .export_hwpx_native()
        .expect("initial HWPX projection");

    // HWPX model -> adapter snapshot -> HWP serializer -> HWP parser/model.
    let hwp_bytes = hwpx_source
        .export_hwp_with_adapter_snapshot()
        .expect("first HWP snapshot export");
    assert_eq!(
        top_level_adapter_policy_discriminators(hwpx_source.document()),
        expected_discriminators,
        "HWP snapshot export changed the HWPX source model"
    );
    assert_eq!(
        hwpx_source
            .export_hwpx_native()
            .expect("HWPX projection after HWP snapshot export"),
        hwpx_projection_before,
        "HWP snapshot export changed the complete serialized source projection"
    );
    let hwp_model = DocumentCore::from_bytes(&hwp_bytes).expect("first HWP reload");
    assert_eq!(
        top_level_adapter_policy_discriminators(hwp_model.document()),
        expected_discriminators
    );
    assert_eq!(
        top_level_table_ctrl_data(hwp_model.document()),
        vec![TABLE_LAYOUT_CTRL_DATA.as_slice()]
    );

    // HWP parser/model -> HWPX serializer -> HWPX parser/model.
    let hwp_payloads_before = top_level_table_ctrl_data(hwp_model.document())
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>();
    let hwp_extra_streams_before = hwp_model.document().extra_streams.clone();
    let second_hwpx_bytes = hwp_model.export_hwpx_native().expect("HWPX export");
    assert_eq!(
        top_level_table_ctrl_data(hwp_model.document())
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect::<Vec<_>>(),
        hwp_payloads_before,
        "HWPX export changed the HWP source model"
    );
    assert_eq!(
        hwp_model.document().extra_streams,
        hwp_extra_streams_before,
        "HWPX export changed HWP source streams"
    );
    let second_hwpx_model =
        DocumentCore::from_bytes(&second_hwpx_bytes).expect("second HWPX reload");
    assert_eq!(
        top_level_adapter_policy_discriminators(second_hwpx_model.document()),
        expected_discriminators
    );

    // HWPX parser/model -> adapter snapshot -> HWP serializer/parser again.
    let second_hwpx_discriminators_before =
        top_level_adapter_policy_discriminators(second_hwpx_model.document());
    let second_hwpx_projection_before = second_hwpx_model
        .export_hwpx_native()
        .expect("second HWPX source projection");
    let second_hwpx_streams_before = second_hwpx_model.document().extra_streams.clone();
    let second_hwp_bytes = second_hwpx_model
        .export_hwp_with_adapter_snapshot()
        .expect("second HWP snapshot export");
    assert_eq!(
        top_level_adapter_policy_discriminators(second_hwpx_model.document()),
        second_hwpx_discriminators_before,
        "second HWP snapshot export changed the HWPX source model"
    );
    assert_eq!(
        second_hwpx_model.document().extra_streams,
        second_hwpx_streams_before,
        "second HWP snapshot export changed HWPX source streams"
    );
    assert_eq!(
        second_hwpx_model
            .export_hwpx_native()
            .expect("second HWPX projection after HWP export"),
        second_hwpx_projection_before,
        "second snapshot HWP export changed the complete HWPX source projection"
    );
    let second_hwp_model = DocumentCore::from_bytes(&second_hwp_bytes).expect("second HWP reload");
    assert_eq!(
        top_level_adapter_policy_discriminators(second_hwp_model.document()),
        expected_discriminators
    );
    assert_eq!(
        top_level_table_ctrl_data(second_hwp_model.document()),
        vec![TABLE_LAYOUT_CTRL_DATA.as_slice()]
    );
}
