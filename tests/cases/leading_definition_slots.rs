//! 관계형·고정형 — 선행 정의 삽입의 슬롯 소유와 HWP 저장 재열기의 본문 보존을 검사한다.
//! 주석 갱신: 2026-11-07T10:55:27+09:00; 직전 파일 변경: 2026-11-07T09:52:50+09:00
//! (`4e5a8d1565543daa999c776668d491fcc0beade5`).
//! 합성 입력 세 검사는 제어 스트림의 슬롯·필드 소유·저장 읽기 전용 계약을 검사하므로 시각 측정은 해당하지 않는다.
//! 실물 입력: `mydocs/pr/assets/leading-definition-slots/input/report-implicit-column.hwpx`.
//! 독립 기준: `pdf/leading-definition-slots/report-implicit-column-hancom-print.pdf` (Mac 한컴 Print, 1쪽).
//! 2026-11-07 협업자 측정: raw 실루엣(2px 이웃, 96dpi, pypdfium2), 원본·HWP 저장 재열기
//! 각각 Native/fresh WASM 전 1쪽 100.00% / 100.00%; 양쪽 전쪽 최저도 각각 100.00%다.
//! 근거: `mydocs/pr/assets/pr_8226_integration_20261107/repro-visual.tsv`,
//! `mydocs/pr/assets/pr_8226_integration_20261107/repro-provenance.json`, `mydocs/pr/archives/pr_8226_review.md`.
//! 고정형은 독립 Print의 전체 쪽수 1뿐이며, 실제 배치는 내용·순서·소유·포함 관계로 검사하고 절대 px 좌표를 고정하지 않는다.

use rhwp::document_core::converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Field, FieldType};
use rhwp::model::paragraph::{CharShapeRef, FieldRange, LineSeg, Paragraph, RangeTag, TitleMark};
use rhwp::model::table::{Cell, Table};

fn document(para: Paragraph) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.document_mut().sections[0].raw_stream = None;
    core.document_mut().sections[0].paragraphs = vec![para];
    core
}

fn table() -> Control {
    Control::Table(Box::new(Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1],
        cells: vec![Cell {
            paragraphs: vec![Paragraph {
                text: "첫 표".into(),
                char_count: 4,
                ..Paragraph::new_empty()
            }],
            ..Cell::new_empty(0, 0, 1000, 1000, 0)
        }],
        ..Default::default()
    }))
}

fn assert_table_position(para: &Paragraph, position: usize) {
    let index = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();
    assert_eq!(para.control_text_positions()[index], position, "{para:?}");
    let Control::Table(table) = &para.controls[index] else {
        unreachable!()
    };
    assert_eq!(table.cells[0].paragraphs[0].text, "첫 표");
    for (control, position) in para.controls.iter().zip(para.control_text_positions()) {
        if matches!(control, Control::Header(_) | Control::Footer(_)) {
            assert_eq!(position, 0);
        }
    }
}

#[test]
fn hwp_save_keeps_table_between_text_after_leading_headers() {
    for hidden in 0..=2 {
        for existing_defs in 0..=2 {
            let mut core = document(Paragraph::new_empty());
            let section_def = core.document().sections[0].section_def.clone();
            let mut controls = Vec::new();
            if existing_defs == 1 {
                controls.push(Control::ColumnDef(Default::default()));
            }
            if existing_defs == 2 {
                controls.push(Control::SectionDef(Box::new(section_def)));
            }
            if hidden > 0 {
                controls.push(Control::Header(Default::default()));
            }
            if hidden > 1 {
                controls.push(Control::Footer(Default::default()));
            }
            let leading = controls.len() as u32 * 8;
            controls.push(table());
            core.document_mut().sections[0].paragraphs[0] = Paragraph {
                text: "앞😊\u{fffc}뒤".into(),
                char_offsets: vec![leading, leading + 1, leading + 3, leading + 11],
                char_count: leading + 13,
                controls,
                ..Paragraph::new_empty()
            };
            let before = serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap();
            assert_table_position(&core.document().sections[0].paragraphs[0], 2);
            let saved = core.export_hwp_native().unwrap();
            assert_eq!(
                serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
                before
            );
            let reopened = DocumentCore::from_bytes(&saved).unwrap();
            let para = &reopened.document().sections[0].paragraphs[0];
            assert_eq!(para.text, "앞😊뒤");
            assert_table_position(para, 2);
            let again = DocumentCore::from_bytes(&reopened.export_hwp_native().unwrap()).unwrap();
            assert_table_position(&again.document().sections[0].paragraphs[0], 2);
        }
    }
}

#[test]
fn adapter_keeps_owned_slots_metadata_and_unused_reservation() {
    for controls in [vec![], vec![Control::Header(Default::default())]] {
        let occupied = !controls.is_empty();
        let mut core = document(Paragraph {
            text: "가나".into(),
            char_offsets: vec![8, 9],
            char_count: 11,
            ctrl_data_records: if occupied {
                vec![Some(vec![1, 2, 3])]
            } else {
                vec![]
            },
            controls,
            char_shapes: vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                },
                CharShapeRef {
                    start_pos: 9,
                    char_shape_id: 0,
                },
            ],
            line_segs: vec![
                LineSeg {
                    text_start: 0,
                    ..Default::default()
                },
                LineSeg {
                    text_start: 9,
                    ..Default::default()
                },
            ],
            range_tags: vec![RangeTag {
                start: 8,
                end: 10,
                tag: 0,
            }],
            ..Paragraph::new_empty()
        });
        convert_hwpx_to_hwp_ir(core.document_mut());
        let para = &core.document().sections[0].paragraphs[0];
        let shift = if occupied { 8 } else { 0 };
        assert_eq!(para.char_offsets, [8 + shift, 9 + shift]);
        assert_eq!(para.char_shapes[0].start_pos, 0);
        assert_eq!(para.char_shapes[1].start_pos, 9 + shift);
        assert_eq!(para.line_segs[0].text_start, 0);
        assert_eq!(para.line_segs[1].text_start, 9 + shift);
        assert_eq!(
            (para.range_tags[0].start, para.range_tags[0].end),
            (8 + shift, 10 + shift)
        );
        if occupied {
            assert_eq!(para.ctrl_data_records[1], Some(vec![1, 2, 3]));
        }
        let before = serde_json::to_value(para).unwrap();
        convert_hwpx_to_hwp_ir(core.document_mut());
        assert_eq!(
            serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
            before
        );
    }
}

#[test]
fn definition_insertion_preserves_empty_field_and_title_slots() {
    let mut core = document(Paragraph {
        text: "앞\u{fffc}뒤".into(),
        char_offsets: vec![24, 25, 33],
        char_count: 35,
        controls: vec![
            Control::Field(Field {
                field_type: FieldType::ClickHere,
                ctrl_id: rhwp::parser::tags::FIELD_CLICKHERE,
                field_id: 42,
                ..Default::default()
            }),
            table(),
        ],
        field_ranges: vec![FieldRange {
            control_idx: 0,
            ..Default::default()
        }],
        title_marks: vec![TitleMark {
            char_idx: 0,
            ignore: false,
        }],
        ..Paragraph::new_empty()
    });
    convert_hwpx_to_hwp_ir(core.document_mut());
    let para = &core.document().sections[0].paragraphs[0];
    assert_eq!(para.field_ranges[0].control_idx, 1);
    assert_eq!(para.char_offsets, [32, 33, 41]);
    let saved = core.export_hwp_native().unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    let para = &reopened.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "앞뒤");
    assert_table_position(para, 1);
    assert_eq!(para.title_marks.len(), 1);
    assert_eq!(para.field_ranges.len(), 1);
    let range = &para.field_ranges[0];
    assert_eq!((range.start_char_idx, range.end_char_idx), (0, 0));
    assert!(
        matches!(&para.controls[range.control_idx], Control::Field(field) if field.field_id == 42)
    );
}

/// 공개 재현 문서와 독립 한컴 Print PDF의 Native/fresh WASM 전쪽 대조를
/// 2026-11-07 제품 코드 24bb81d03에서 완료했다(각 1쪽, 원본·HWP 저장본 모두 100%).
/// 숫자 좌표 대신 본문 글자 상자의 수용, 표 전후 순서와 내용 보존을 검사한다.
#[test]
fn reopened_report_keeps_complete_text_boxes_around_inline_table() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn descendants<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
        nodes.push(node);
        for child in &node.children {
            descendants(child, nodes);
        }
    }

    fn check(core: &DocumentCore) {
        assert_eq!(core.page_count(), 1);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut nodes = Vec::new();
        descendants(&tree.root, &mut nodes);
        let body: Vec<_> = nodes
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(0) && run.cell_context.is_none() =>
                {
                    Some((*node, run))
                }
                _ => None,
            })
            .collect();
        let text: String = body.iter().map(|(_, run)| run.text.as_str()).collect();
        assert_eq!(
            text,
            "담당 업무 내역은 다음과 같습니다. 검토 후 승인해 주세요."
        );
        for (node, run) in &body {
            assert!(
                node.bbox.height + 1e-6 >= run.style.font_size,
                "저장 줄이 없는 본문도 글자 상자를 온전히 담는다: {:?}",
                node.bbox
            );
            assert!(run.baseline > 0.0 && run.baseline <= node.bbox.height);
        }
        let tables: Vec<_> = nodes
            .iter()
            .filter(|node| matches!(node.node_type, RenderNodeType::Table(_)))
            .collect();
        assert_eq!(tables.len(), 1);
        let table = tables[0];
        let prefix = body
            .iter()
            .find(|(_, run)| run.text.starts_with("담당 업무"))
            .unwrap()
            .0;
        let suffix = body
            .iter()
            .find(|(_, run)| run.text.starts_with("검토 후"))
            .unwrap()
            .0;
        assert!(prefix.bbox.x + prefix.bbox.width <= table.bbox.x);
        assert!(table.bbox.x + table.bbox.width <= suffix.bbox.x);
        assert!(prefix.bbox.y < table.bbox.y + table.bbox.height);
        assert!(suffix.bbox.y < table.bbox.y + table.bbox.height);
        for expected in ["검토용 업무 보고서", "업무", "담당", "자료 검토", "김 담당"]
        {
            let count = nodes.iter().filter(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == expected)).count();
            assert_eq!(count, 1, "머리말·셀 내용은 한 번만 보존한다: {expected}");
        }
    }

    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/leading-definition-slots/input/report-implicit-column.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    check(&core);
    let before = serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap();
    let bytes = core.export_hwp_with_adapter().unwrap();
    assert_eq!(
        serde_json::to_value(&core.document().sections[0].paragraphs[0]).unwrap(),
        before
    );
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    assert!(reopened.document().sections[0].paragraphs[0]
        .line_segs
        .is_empty());
    check(&reopened);
    let twice = DocumentCore::from_bytes(&reopened.export_hwp_with_adapter().unwrap()).unwrap();
    check(&twice);
}
