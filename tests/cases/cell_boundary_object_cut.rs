//! 셀 경계에서 복사한 표·그림만 지우고, 선택 밖 글·서식·개체를 보존한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Field};
use rhwp::model::document::Document;
use rhwp::model::paragraph::{MarkpenMark, Paragraph, RangeTag};
use rhwp::model::shape::Caption;
use rhwp::wasm_api::HwpDocument;

type Path = Vec<(usize, usize, usize)>;

fn fixture(depth: usize, tables: bool) -> (DocumentCore, Path) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    for (pi, text) in ["머리", "가🦦나다라", "중간", "앞뒤끝말", ""]
        .iter()
        .enumerate()
    {
        core.insert_text_native(0, pi, 0, text).unwrap();
        if pi < 4 {
            core.split_paragraph_native(0, pi, text.chars().count(), None)
                .unwrap();
        }
    }
    for (pi, at, name) in [
        (1, 1, "첫 문단 밖"),
        (1, 3, "첫 문단 안"),
        (3, 1, "끝 문단 안"),
        (3, 3, "끝 문단 밖"),
    ] {
        let result: serde_json::Value = serde_json::from_str(
            &core
                .insert_picture_native(
                    0,
                    pi,
                    at,
                    &[],
                    include_bytes!("../../assets/logo/logo-16.png"),
                    1200,
                    1400,
                    16,
                    16,
                    "png",
                    name,
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
        core.set_picture_properties_native(
            0,
            pi,
            result["controlIdx"].as_u64().unwrap() as usize,
            r#"{"treatAsChar":true,"transparency":25}"#,
        )
        .unwrap();
    }
    core.apply_char_format_native(0, 1, 0, 2, r#"{"bold":true}"#)
        .unwrap();
    core.apply_char_format_native(0, 3, 2, 4, r#"{"italic":true}"#)
        .unwrap();
    core.create_table_native(0, 4, 0, 1, 2).unwrap();
    let mut document = core.document().clone();
    let template = document.sections[0].paragraphs[4]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some((**table).clone())
            } else {
                None
            }
        })
        .unwrap();
    let mut paragraphs = document.sections[0].paragraphs[1..=3].to_vec();
    let plain = document.sections[0].paragraphs[2].clone();
    let host = document.sections[0].paragraphs[4].clone();
    if tables {
        for paragraph in &mut paragraphs {
            for control in &mut paragraph.controls {
                if let Control::Picture(picture) = control {
                    let mut table = template.clone();
                    table.common = picture.common.clone();
                    *control = Control::Table(Box::new(table));
                }
            }
        }
    }
    // 중간 단계의 문단 1에만 자식 표를 두어 최내측 경로 해석도 검증한다.
    let mut path = vec![(0, 0, 1); depth];
    path.last_mut().unwrap().2 = 0;
    for _ in 0..depth {
        let mut table = template.clone();
        table.cells[0].paragraphs = paragraphs;
        table.cells[1].paragraphs = vec![plain.clone()];
        let mut owner = host.clone();
        owner.controls = vec![Control::Table(Box::new(table))];
        paragraphs = vec![plain.clone(), owner];
    }
    let outer = paragraphs.pop().unwrap();
    document.sections[0].paragraphs = vec![
        document.sections[0].paragraphs[0].clone(),
        outer,
        Paragraph::default(),
    ];
    document.sections[0].raw_stream = None;
    core.set_document(document);
    (core, path)
}

fn cell<'a>(document: &'a Document, path: &[(usize, usize, usize)]) -> &'a [Paragraph] {
    let mut paragraphs = &document.sections[0].paragraphs;
    let mut pi = 1;
    for &(ci, cei, cpi) in path {
        let Control::Table(table) = &paragraphs[pi].controls[ci] else {
            panic!("표 경로")
        };
        paragraphs = &table.cells[cei].paragraphs;
        pi = cpi;
    }
    paragraphs
}

fn cell_mut<'a>(document: &'a mut Document, path: &Path) -> &'a mut Vec<Paragraph> {
    let mut paragraphs = &mut document.sections[0].paragraphs;
    let mut pi = 1;
    for &(ci, cei, cpi) in path {
        let Control::Table(table) = &mut paragraphs[pi].controls[ci] else {
            panic!("표 경로")
        };
        paragraphs = &mut table.cells[cei].paragraphs;
        pi = cpi;
    }
    paragraphs
}

fn siblings(document: &Document, path: &Path) -> Vec<Vec<Paragraph>> {
    let mut result = Vec::new();
    let mut paragraphs = &document.sections[0].paragraphs;
    let mut pi = 1;
    for &(ci, cei, cpi) in path {
        let Control::Table(table) = &paragraphs[pi].controls[ci] else {
            panic!("표 경로")
        };
        result.push(table.cells[1].paragraphs.clone());
        paragraphs = &table.cells[cei].paragraphs;
        pi = cpi;
    }
    result
}

fn images(document: &Document) -> Vec<(u16, String, Vec<u8>)> {
    document
        .bin_data_content
        .iter()
        .map(|b| (b.id, b.extension.clone(), b.data.load()))
        .collect()
}

fn names(paragraphs: &[Paragraph]) -> Vec<String> {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|control| match control {
            Control::Picture(p) => Some(p.common.description.clone()),
            Control::Table(t) => Some(t.common.description.clone()),
            _ => None,
        })
        .collect()
}

fn style_at(paragraph: &Paragraph, at: usize) -> u32 {
    let utf16 = paragraph.char_offsets[at];
    paragraph
        .char_shapes
        .iter()
        .rfind(|s| s.start_pos <= utf16)
        .unwrap()
        .char_shape_id
}

fn delete(
    core: &mut DocumentCore,
    path: &Path,
    flat: bool,
    start: (usize, usize),
    end: (usize, usize),
) -> Result<String, rhwp::error::HwpError> {
    if flat {
        core.delete_range_native(0, start.0, start.1, end.0, end.1, Some((1, 0, 0)))
    } else {
        core.delete_range_in_cell_by_path(0, 1, path, start.0, start.1, end.0, end.1)
    }
}

fn copy(core: &mut DocumentCore, path: &Path, flat: bool) {
    if flat {
        core.copy_selection_in_cell_native(0, 1, 0, 0, 0, 2, 2, 2)
            .unwrap();
    } else {
        core.copy_selection_in_cell_by_path_native(0, 1, path, 0, 2, 2, 2)
            .unwrap();
    }
}

fn cut_boundary(flat: bool, depth: usize, tables: bool) {
    let (mut core, path) = fixture(depth, tables);
    let before_document = core.document().clone();
    let before = cell(core.document(), &path).to_vec();
    let outside = [before[0].controls[0].clone(), before[2].controls[1].clone()];
    let sibling = siblings(core.document(), &path);
    let image_bytes = images(core.document());
    copy(&mut core, &path, flat);
    // 삭제하지 않은 원문에서 복사만 한 결과와 새 ID·원시 속성까지 독립 대조한다.
    let mut copied = DocumentCore::new_empty();
    copied.set_document(before_document.clone());
    copy(&mut copied, &path, flat);
    copied.paste_internal_native(0, 2, 0).unwrap();
    let expected_inside: Vec<_> = copied.document().sections[0].paragraphs[2..]
        .iter()
        .flat_map(|p| &p.controls)
        .collect();
    delete(&mut core, &path, flat, (0, 2), (2, 2)).unwrap();
    let after = cell(core.document(), &path);
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].text, "가🦦끝말");
    assert_eq!(
        names(after),
        ["첫 문단 밖", "끝 문단 밖"],
        "depth={depth}, flat={flat}, tables={tables}"
    );
    assert_eq!(
        serde_json::to_value(&after[0].controls).unwrap(),
        serde_json::to_value(&outside).unwrap()
    );
    assert_eq!(after[0].control_text_positions(), [1, 3]);
    // 기대 트리는 원문의 개체를 그대로 둔 채 문단 소속만 최소로 교체한다.
    // HWPX는 트리 순서로 문단 ID를 재부여하므로 같은 소속의 저장본을 비교한다.
    let mut expected_document = before_document;
    let mut expected_survivor = after[0].clone();
    expected_survivor.controls = outside.to_vec();
    *cell_mut(&mut expected_document, &path) = vec![expected_survivor];
    expected_document.sections[0].paragraphs.truncate(2);
    expected_document.sections[0]
        .paragraphs
        .extend_from_slice(&copied.document().sections[0].paragraphs[2..]);
    let mut expected_stored = DocumentCore::new_empty();
    expected_stored.set_document(expected_document);
    let saved_expected = [
        expected_stored.export_hwp_native().unwrap(),
        expected_stored.export_hwpx_native().unwrap(),
    ]
    .map(|bytes| HwpDocument::from_bytes(&bytes).unwrap());
    for at in 0..4 {
        let expected = if at < 2 {
            style_at(&before[0], at)
        } else {
            style_at(&before[2], at)
        };
        assert_eq!(style_at(&after[0], at), expected, "선택 밖 글자 서식 {at}");
    }
    assert_eq!(
        serde_json::to_value(siblings(core.document(), &path)).unwrap(),
        serde_json::to_value(sibling).unwrap()
    );
    core.paste_internal_native(0, 2, 0).unwrap();
    assert_eq!(images(core.document()), image_bytes);
    assert_eq!(
        names(&core.document().sections[0].paragraphs[2..]),
        ["첫 문단 안", "끝 문단 안"]
    );
    let pasted: Vec<_> = core.document().sections[0].paragraphs[2..]
        .iter()
        .flat_map(|p| &p.controls)
        .collect();
    assert_eq!(
        serde_json::to_value(pasted).unwrap(),
        serde_json::to_value(expected_inside).unwrap()
    );
    for (format, bytes) in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .into_iter()
    .enumerate()
    {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap();
        assert_eq!(images(reopened.document()), image_bytes);
        assert_eq!(
            serde_json::to_value(&cell(reopened.document(), &path)[0].controls).unwrap(),
            serde_json::to_value(&cell(saved_expected[format].document(), &path)[0].controls)
                .unwrap()
        );
        assert_eq!(
            serde_json::to_value(
                reopened.document().sections[0].paragraphs[2..]
                    .iter()
                    .flat_map(|p| &p.controls)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            serde_json::to_value(
                saved_expected[format].document().sections[0].paragraphs[2..]
                    .iter()
                    .flat_map(|p| &p.controls)
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );
        assert_eq!(cell(reopened.document(), &path)[0].text, "가🦦끝말");
        assert_eq!(
            cell(reopened.document(), &path)[0].control_text_positions(),
            [1, 3]
        );
    }
}

#[test]
fn flat_cell_cut_removes_boundary_pictures_and_tables() {
    for tables in [false, true] {
        cut_boundary(true, 1, tables);
    }
}

#[test]
fn path_cell_cut_removes_boundary_pictures_and_tables_at_each_depth() {
    for depth in 1..=3 {
        for tables in [false, true] {
            cut_boundary(false, depth, tables);
        }
    }
}

#[test]
fn same_paragraph_cell_cut_removes_only_selected_object() {
    for flat in [true, false] {
        for tables in [false, true] {
            for tail in [false, true] {
                let (mut core, path) = fixture(1, tables);
                if tail {
                    let mut document = core.document().clone();
                    let paragraph = &mut cell_mut(&mut document, &path)[0];
                    paragraph.delete_text_at(3, 2);
                    match &mut paragraph.controls[1] {
                        Control::Picture(picture) => picture.common.treat_as_char = false,
                        Control::Table(table) => table.common.treat_as_char = false,
                        _ => unreachable!(),
                    }
                    assert_eq!(paragraph.control_text_positions(), [1, 3]);
                    core.set_document(document);
                }
                let outside = cell(core.document(), &path)[0].controls[0].clone();
                delete(
                    &mut core,
                    &path,
                    flat,
                    (0, 2),
                    (0, if tail { 3 } else { 4 }),
                )
                .unwrap();
                let after = cell(core.document(), &path);
                assert_eq!(after[0].text, if tail { "가🦦" } else { "가🦦라" });
                assert_eq!(names(&after[..1]), ["첫 문단 밖"]);
                assert_eq!(
                    serde_json::to_value(&after[0].controls).unwrap(),
                    serde_json::to_value([outside]).unwrap()
                );
                assert_eq!(names(&after[2..]), ["끝 문단 안", "끝 문단 밖"]);
            }
        }
    }
}

#[test]
fn invalid_cell_ranges_keep_document_and_clipboard() {
    for flat in [true, false] {
        for (start, end) in [
            ((2, 0), (0, 0)),
            ((0, 4), (0, 2)),
            ((9, 0), (9, 1)),
            ((0, 0), (9, 1)),
            ((0, 99), (2, 1)),
            ((0, 1), (2, 99)),
        ] {
            let (mut core, path) = fixture(1, false);
            let bytes = core.export_hwp_native().unwrap();
            core.set_document(HwpDocument::from_bytes(&bytes).unwrap().document().clone());
            assert!(core.document().sections[0].raw_stream.is_some());
            core.copy_selection_native(0, 0, 0, 0, 1).unwrap();
            let before = format!("{:?}", core.document());
            let clipboard = core.get_clipboard_text_native();
            assert!(
                delete(&mut core, &path, flat, start, end).is_err(),
                "flat={flat}, {start:?}..{end:?}"
            );
            assert_eq!(format!("{:?}", core.document()), before);
            assert_eq!(core.get_clipboard_text_native(), clipboard);
        }
    }
}

#[test]
fn cell_boundary_with_field_is_rejected_before_mutation() {
    for flat in [true, false] {
        for coanchored in [false, true] {
            let (mut core, path) = fixture(1, false);
            let mut document = core.document().clone();
            let paragraphs = cell_mut(&mut document, &path);
            if coanchored {
                paragraphs[0].char_offsets = vec![0, 17, 19, 20, 21];
                assert_eq!(paragraphs[0].control_text_positions(), [1, 1]);
            } else {
                paragraphs[2]
                    .controls
                    .push(Control::Field(Field::default()));
            }
            core.set_document(document);
            let before = format!("{:?}", core.document());
            let events = core.serialize_event_log();
            assert!(delete(&mut core, &path, flat, (0, 0), (2, 2)).is_err());
            assert_eq!(format!("{:?}", core.document()), before);
            assert_eq!(core.serialize_event_log(), events);
        }
    }
}

#[test]
fn empty_cell_ranges_keep_tail_objects_raw_stream_and_events() {
    for flat in [true, false] {
        for tables in [false, true] {
            let (mut core, path) = fixture(if flat { 1 } else { 3 }, tables);
            let mut document = core.document().clone();
            let paragraphs = cell_mut(&mut document, &path);
            paragraphs[0].delete_text_at(3, 2);
            match &mut paragraphs[0].controls[1] {
                Control::Picture(picture) => picture.common.treat_as_char = false,
                Control::Table(table) => table.common.treat_as_char = false,
                _ => unreachable!(),
            }
            assert_eq!(paragraphs[0].control_text_positions(), [1, 3]);
            core.set_document(document);
            let bytes = core.export_hwp_native().unwrap();
            let reopened = HwpDocument::from_bytes(&bytes).unwrap();
            core.set_document(reopened.document().clone());
            assert!(core.document().sections[0].raw_stream.is_some());
            core.copy_selection_native(0, 0, 0, 0, 1).unwrap();
            let before = format!("{:?}", core.document());
            let events = core.serialize_event_log();
            let clipboard = core.get_clipboard_text_native();
            for offset in [0, 1, 3] {
                delete(&mut core, &path, flat, (0, offset), (0, offset)).unwrap();
                assert_eq!(format!("{:?}", core.document()), before);
                assert_eq!(core.serialize_event_log(), events);
                assert_eq!(core.get_clipboard_text_native(), clipboard);
            }
        }
    }
}

fn reject_boundary_metadata(
    flat: bool,
    tables: bool,
    end_para: usize,
    tagged: usize,
    tag: Option<u32>,
) {
    let (mut core, path) = fixture(if flat { 1 } else { 3 }, tables);
    let mut document = core.document().clone();
    let paragraph = &mut cell_mut(&mut document, &path)[tagged];
    let end = paragraph.char_offsets[3];
    if let Some(tag) = tag {
        paragraph.range_tags.push(RangeTag { start: 0, end, tag });
    } else {
        paragraph.markpen_marks = vec![
            MarkpenMark {
                char_idx: 0,
                color: Some("#00ff00".into()),
                utf16_pos: Some(0),
            },
            MarkpenMark {
                char_idx: 3,
                color: None,
                utf16_pos: Some(end),
            },
        ];
    }
    core.set_document(document);
    // HWPX는 범위 태그 없이 형광펜 표지만 읽으므로 두 보호 조건을 독립 검증한다.
    let bytes = if tag.is_some() {
        core.export_hwp_native().unwrap()
    } else {
        core.export_hwpx_native().unwrap()
    };
    core.set_document(HwpDocument::from_bytes(&bytes).unwrap().document().clone());
    let paragraph = &cell(core.document(), &path)[tagged];
    if tag.is_some() {
        assert_eq!(paragraph.range_tags.len(), 1);
        assert!(core.document().sections[0].raw_stream.is_some());
    } else {
        assert!(paragraph.range_tags.is_empty());
        assert_eq!(paragraph.markpen_marks.len(), 2);
    }
    core.copy_selection_native(0, 0, 0, 0, 1).unwrap();
    let before = format!("{:?}", core.document());
    let stored = core.export_hwp_native().unwrap();
    let events = core.serialize_event_log();
    let clipboard = core.get_clipboard_text_native();
    let end_offset = if end_para == 0 { 4 } else { 2 };
    assert!(
        delete(&mut core, &path, flat, (0, 2), (end_para, end_offset)).is_err(),
        "flat={flat}, tables={tables}, end={end_para}, tagged={tagged}, tag={tag:?}"
    );
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.export_hwp_native().unwrap(), stored);
    assert_eq!(core.serialize_event_log(), events);
    assert_eq!(core.get_clipboard_text_native(), clipboard);
}

#[test]
fn cell_boundary_range_tags_are_rejected_before_mutation() {
    for flat in [true, false] {
        for tables in [false, true] {
            for (end_para, tagged) in [(0, 0), (2, 0), (2, 2)] {
                for tag in [0x0100_0003, 0x0300_0003, 0x0200_ff00] {
                    reject_boundary_metadata(flat, tables, end_para, tagged, Some(tag));
                }
            }
        }
    }
}

#[test]
fn cell_boundary_markpen_marks_are_rejected_before_mutation() {
    for flat in [true, false] {
        for tables in [false, true] {
            for (end_para, tagged) in [(0, 0), (2, 0), (2, 2)] {
                reject_boundary_metadata(flat, tables, end_para, tagged, None);
            }
        }
    }
}

#[test]
fn metadata_without_boundary_objects_keeps_existing_deletion_paths() {
    for flat in [true, false] {
        let (mut core, path) = fixture(1, false);
        let mut document = core.document().clone();
        let paragraph = &mut cell_mut(&mut document, &path)[0];
        let end = paragraph.char_offsets[3];
        paragraph.range_tags = vec![RangeTag {
            start: 0,
            end,
            tag: 0x0300_0003,
        }];
        core.set_document(document);
        // 첫 그림 앞의 순수 글자 삭제는 기존 태그 축소 경로를 유지한다.
        delete(&mut core, &path, flat, (0, 0), (0, 1)).unwrap();
        let paragraph = &cell(core.document(), &path)[0];
        assert_eq!(paragraph.text, "🦦나다라");
        assert_eq!(
            names(&cell(core.document(), &path)[..1]),
            ["첫 문단 밖", "첫 문단 안"]
        );
        assert_eq!(
            serde_json::to_value(&paragraph.range_tags).unwrap(),
            serde_json::json!([{ "start": 0, "end": end - 1, "tag": 0x0300_0003u32 }])
        );

        let (mut core, path) = fixture(1, false);
        let mut document = core.document().clone();
        cell_mut(&mut document, &path)[1].range_tags = vec![RangeTag {
            start: 0,
            end: 2,
            tag: 0x0300_0003,
        }];
        core.set_document(document);
        // 통째로 선택한 사이 문단의 태그는 첫·끝 경계 보호와 구별한다.
        delete(&mut core, &path, flat, (0, 2), (2, 2)).unwrap();
        assert_eq!(cell(core.document(), &path)[0].text, "가🦦끝말");
        assert!(cell(core.document(), &path)[0].range_tags.is_empty());
    }
}

fn paste_cell(core: &mut DocumentCore, path: &Path, flat: bool, offset: usize) {
    let result = if flat {
        core.paste_internal_in_cell_native(0, 1, 0, 0, 0, offset)
    } else {
        core.paste_internal_in_cell_by_path_native(0, 1, path, offset)
    }
    .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["ok"], true);
}

fn paste_fixture(flat: bool, tables: bool, inline: bool, format: usize) -> (DocumentCore, Path) {
    let (mut core, path) = fixture(if flat { 1 } else { 3 }, tables);
    if !inline {
        let mut document = core.document().clone();
        for paragraph in cell_mut(&mut document, &path) {
            for control in &mut paragraph.controls {
                match control {
                    Control::Picture(picture) => picture.common.treat_as_char = false,
                    Control::Table(table) => table.common.treat_as_char = false,
                    _ => {}
                }
            }
        }
        core.set_document(document);
    }
    if format != 0 {
        let bytes = if format == 1 {
            core.export_hwp_native().unwrap()
        } else {
            core.export_hwpx_native().unwrap()
        };
        core.set_document(HwpDocument::from_bytes(&bytes).unwrap().document().clone());
    }
    (core, path)
}

fn assert_cell_text_and_style(actual: &[Paragraph], expected: &[Paragraph]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.text, expected.text);
        assert_eq!(actual.char_offsets, expected.char_offsets);
        assert_eq!(
            actual.control_text_positions(),
            expected.control_text_positions()
        );
        for at in 0..expected.text.chars().count() {
            assert_eq!(style_at(actual, at), style_at(expected, at));
        }
        assert_eq!(
            serde_json::to_value(&actual.controls).unwrap(),
            serde_json::to_value(&expected.controls).unwrap()
        );
    }
}

#[test]
fn cell_cut_paste_restores_text_offsets_after_inline_and_floating_objects() {
    for flat in [true, false] {
        for tables in [false, true] {
            for inline in [true, false] {
                for format in 0..3 {
                    let (mut core, path) = paste_fixture(flat, tables, inline, format);
                    let before_document = core.document().clone();
                    let before = cell(core.document(), &path).to_vec();
                    let sibling = siblings(core.document(), &path);
                    let image_bytes = images(core.document());
                    copy(&mut core, &path, flat);
                    let mut oracle = DocumentCore::new_empty();
                    oracle.set_document(before_document.clone());
                    copy(&mut oracle, &path, flat);
                    delete(&mut core, &path, flat, (0, 2), (2, 2)).unwrap();
                    assert_eq!(cell(core.document(), &path)[0].text, "가🦦끝말");
                    // 같은 ID 집합의 빈 본문에 복사만 붙여 새 ID와 전체 개체 원문을 독립 대조한다.
                    oracle.set_document(core.document().clone());
                    oracle.paste_internal_native(0, 2, 0).unwrap();
                    let inside: Vec<_> = oracle.document().sections[0].paragraphs[2..]
                        .iter()
                        .flat_map(|p| &p.controls)
                        .cloned()
                        .collect();
                    assert_eq!(inside.len(), 2);
                    let mut expected_document = before_document;
                    let expected = cell_mut(&mut expected_document, &path);
                    expected[0].controls[1] = inside[0].clone();
                    expected[2].controls[0] = inside[1].clone();
                    expected_document.sections[0].raw_stream = None;
                    let mut baseline = DocumentCore::new_empty();
                    baseline.set_document(expected_document);
                    paste_cell(&mut core, &path, flat, 2);
                    assert_cell_text_and_style(
                        cell(core.document(), &path),
                        cell(baseline.document(), &path),
                    );
                    assert_eq!(
                        serde_json::to_value(siblings(core.document(), &path)).unwrap(),
                        serde_json::to_value(&sibling).unwrap()
                    );
                    assert_eq!(images(core.document()), image_bytes);
                    for (actual, expected) in [
                        (
                            core.export_hwp_native().unwrap(),
                            baseline.export_hwp_native().unwrap(),
                        ),
                        (
                            core.export_hwpx_native().unwrap(),
                            baseline.export_hwpx_native().unwrap(),
                        ),
                    ] {
                        let actual = HwpDocument::from_bytes(&actual).unwrap();
                        let expected = HwpDocument::from_bytes(&expected).unwrap();
                        assert_cell_text_and_style(
                            cell(actual.document(), &path),
                            cell(expected.document(), &path),
                        );
                        assert_eq!(images(actual.document()), images(expected.document()));
                        assert_eq!(
                            serde_json::to_value(siblings(actual.document(), &path)).unwrap(),
                            serde_json::to_value(siblings(expected.document(), &path)).unwrap()
                        );
                    }
                    assert_eq!(
                        before.iter().map(|p| &p.text).collect::<Vec<_>>(),
                        cell(core.document(), &path)
                            .iter()
                            .map(|p| &p.text)
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}

#[test]
fn cell_single_control_paste_keeps_same_anchor_objects_before_the_scalar_offset() {
    for flat in [true, false] {
        for format in 0..3 {
            let (mut core, path) = paste_fixture(flat, false, true, 0);
            let mut document = core.document().clone();
            cell_mut(&mut document, &path)[0].char_offsets = vec![0, 17, 19, 20, 21];
            core.set_document(document);
            if format != 0 {
                let bytes = if format == 1 {
                    core.export_hwp_native().unwrap()
                } else {
                    core.export_hwpx_native().unwrap()
                };
                core.set_document(HwpDocument::from_bytes(&bytes).unwrap().document().clone());
            }
            let before = cell(core.document(), &path).to_vec();
            assert_eq!(before[0].control_text_positions(), [1, 1]);
            if flat {
                core.copy_selection_in_cell_native(0, 1, 0, 0, 2, 0, 2, 2)
                    .unwrap();
            } else {
                core.copy_selection_in_cell_by_path_native(0, 1, &path, 2, 0, 2, 2)
                    .unwrap();
            }
            paste_cell(&mut core, &path, flat, 2);
            let after = cell(core.document(), &path);
            assert_eq!(after[0].text, "가🦦앞뒤나다라");
            assert_eq!(after[0].control_text_positions(), [1, 1, 3]);
            assert_eq!(
                serde_json::to_value(&after[0].controls[..2]).unwrap(),
                serde_json::to_value(&before[0].controls).unwrap()
            );
            for at in 0..before[0].text.chars().count() {
                assert_eq!(
                    style_at(&after[0], if at < 2 { at } else { at + 2 }),
                    style_at(&before[0], at)
                );
            }
            assert_eq!(
                serde_json::to_value(&after[1..]).unwrap(),
                serde_json::to_value(&before[1..]).unwrap()
            );
            for bytes in [
                core.export_hwp_native().unwrap(),
                core.export_hwpx_native().unwrap(),
            ] {
                let reopened = HwpDocument::from_bytes(&bytes).unwrap();
                assert_eq!(cell(reopened.document(), &path)[0].text, "가🦦앞뒤나다라");
                assert_eq!(
                    cell(reopened.document(), &path)[0].control_text_positions(),
                    [1, 1, 3]
                );
            }
        }
    }
}

#[test]
fn cell_text_paste_keeps_scalar_fast_path_and_multiline_split_after_inline_objects() {
    for flat in [true, false] {
        for format in 0..3 {
            for multiline in [false, true] {
                let (mut core, path) = paste_fixture(flat, false, true, format);
                let before = cell(core.document(), &path).to_vec();
                if multiline {
                    core.insert_text_native(0, 2, 0, "새").unwrap();
                    core.split_paragraph_native(0, 2, 1, None).unwrap();
                    core.insert_text_native(0, 3, 0, "둘").unwrap();
                    core.copy_selection_native(0, 2, 0, 3, 1).unwrap();
                } else {
                    core.copy_selection_native(0, 0, 0, 0, 2).unwrap();
                }
                paste_cell(&mut core, &path, flat, 2);
                let after = cell(core.document(), &path);
                let expected = if multiline {
                    vec!["가🦦새", "둘나다라", "중간", "앞뒤끝말"]
                } else {
                    vec!["가🦦머리나다라", "중간", "앞뒤끝말"]
                };
                assert_eq!(
                    after.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
                    expected
                );
                for at in 0..2 {
                    assert_eq!(style_at(&after[0], at), style_at(&before[0], at));
                }
                let (suffix, base) = if multiline {
                    (&after[1], 1)
                } else {
                    (&after[0], 4)
                };
                for at in 2..before[0].text.chars().count() {
                    assert_eq!(style_at(suffix, base + at - 2), style_at(&before[0], at));
                }
                assert_eq!(
                    serde_json::to_value(&after[after.len() - 2..]).unwrap(),
                    serde_json::to_value(&before[1..]).unwrap()
                );
                for bytes in [
                    core.export_hwp_native().unwrap(),
                    core.export_hwpx_native().unwrap(),
                ] {
                    let reopened = HwpDocument::from_bytes(&bytes).unwrap();
                    assert_eq!(
                        cell(reopened.document(), &path)
                            .iter()
                            .map(|p| p.text.as_str())
                            .collect::<Vec<_>>(),
                        expected
                    );
                }
            }
        }
    }
}

#[test]
fn flat_table_caption_text_delete_keeps_legacy_address() {
    let (mut core, path) = fixture(1, false);
    let caption = cell(core.document(), &path)[1].clone();
    let mut document = core.document().clone();
    let Control::Table(table) = &mut document.sections[0].paragraphs[1].controls[0] else {
        unreachable!()
    };
    table.caption = Some(Caption {
        paragraphs: vec![caption],
        ..Caption::default()
    });
    core.set_document(document);
    core.delete_range_native(0, 0, 0, 0, 1, Some((1, 0, 65534)))
        .unwrap();
    let Control::Table(table) = &core.document().sections[0].paragraphs[1].controls[0] else {
        unreachable!()
    };
    assert_eq!(table.caption.as_ref().unwrap().paragraphs[0].text, "간");
}
