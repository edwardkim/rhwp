//! 셀 경계에서 복사한 표·그림만 지우고, 선택 밖 글·서식·개체를 보존한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Field};
use rhwp::model::document::Document;
use rhwp::model::paragraph::{MarkpenMark, Paragraph, RangeTag};
use rhwp::model::shape::Caption;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
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

fn assert_inline_run_axis(
    doc: &HwpDocument,
    paragraph: &Paragraph,
    path: Option<&Path>,
    pi: usize,
) {
    fn collect<'a>(
        node: &'a RenderNode,
        path: Option<&Path>,
        pi: usize,
        runs: &mut Vec<&'a RenderNode>,
    ) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            let area_matches = match (path, &run.cell_context) {
                (None, None) => run.para_index == Some(pi),
                (Some(path), Some(context)) => {
                    context.parent_para_index == 1
                        && context.path.len() == path.len()
                        && context.path.iter().zip(path).enumerate().all(
                            |(i, (actual, expected))| {
                                actual.control_index == expected.0
                                    && actual.cell_index == expected.1
                                    && actual.cell_para_index
                                        == if i + 1 == path.len() { pi } else { expected.2 }
                            },
                        )
                }
                _ => false,
            };
            if run.section_index == Some(0)
                && run.char_start.is_some()
                && area_matches
                && (!run.text.is_empty() || run.is_para_end)
            {
                runs.push(node);
            }
        }
        for child in &node.children {
            collect(child, path, pi, runs);
        }
    }
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, path, pi, &mut runs);
    assert!(!runs.is_empty());
    let slots = paragraph.control_text_positions();
    let mut scalar = 0;
    let mut emitted = String::new();
    for node in &runs {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        while paragraph.text.chars().nth(scalar) == Some('\n') {
            emitted.push('\n');
            scalar += 1;
        }
        // 원문 글자 자리와 개체 자리로 기대 논리 좌표를 독립 계산한다.
        let logical = scalar + slots.iter().filter(|&&at| at <= scalar).count();
        assert_eq!(
            run.char_start,
            Some(logical),
            "text={:?}, path={path:?}, pi={pi}",
            run.text
        );
        emitted.push_str(&run.text);
        scalar += run.text.chars().count();
    }
    assert_eq!(emitted, paragraph.text);
    let logical_end = scalar + slots.len();
    let caret: serde_json::Value = serde_json::from_str(&if let Some(path) = path {
        let mut entries: Vec<_> = path.iter().map(|&(ci, cell, para)| serde_json::json!({"controlIndex":ci,"cellIndex":cell,"cellParaIndex":para})).collect();
        entries.last_mut().unwrap()["cellParaIndex"] = pi.into();
        doc.get_cursor_rect_by_path(0, 1, &serde_json::to_string(&entries).unwrap(), logical_end as u32).unwrap()
    } else {
        doc.get_cursor_rect_native(0, pi, logical_end).unwrap()
    }).unwrap();
    if path.is_some() {
        assert_eq!(
            caret["cellOverflowed"], false,
            "겹친 칸을 제외한 좌표 fixture: {caret}"
        );
    }
    let last = runs.last().unwrap();
    let RenderNodeType::TextRun(last_run) = &last.node_type else {
        unreachable!()
    };
    let layout: serde_json::Value =
        serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    let last_layout = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|run| {
            run["secIdx"] == 0
                && run["paraIdx"] == pi
                && run["charStart"] == last_run.char_start.unwrap()
                && run["text"] == last_run.text
                && (run["x"].as_f64().unwrap() - last.bbox.x).abs() < 0.06
                && (run["y"].as_f64().unwrap() - last.bbox.y).abs() < 0.06
        })
        .unwrap();
    // 반올림된 bbox 폭 대신 캐럿·선택이 공유하는 실제 글자 경계를 대조한다.
    let right = last_layout["x"].as_f64().unwrap()
        + last_layout["charX"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .as_f64()
            .unwrap();
    assert!(
        (caret["x"].as_f64().unwrap() - right).abs() <= 0.1 + f64::EPSILON * right.abs(),
        "문단 끝 캐럿 {caret} != 실제 마지막 글 끝 {right}"
    );
}

fn inline_axis_fixture(flat: bool, tables: bool) -> (HwpDocument, Path) {
    let (core, path) = paste_fixture(flat, tables, true, 0);
    let mut document = core.document().clone();
    let mut paragraphs = &mut document.sections[0].paragraphs;
    let mut pi = 1;
    for &(ci, cell, cell_para) in &path {
        let Control::Table(table) = &mut paragraphs[pi].controls[ci] else {
            unreachable!()
        };
        // 이 회귀는 overflow clamp를 섞지 않고 중첩 셀의 논리 좌표만 검증한다.
        let width = table.cells[0].width + table.cells[1].width;
        table.cells[0].width = width * 9 / 10;
        table.cells[1].width = width - table.cells[0].width;
        paragraphs = &mut table.cells[cell].paragraphs;
        pi = cell_para;
    }
    for paragraph in cell_mut(&mut document, &path) {
        for control in &mut paragraph.controls {
            if let Control::Table(table) = control {
                // 합성 표에 대입한 TAC 속성과 옛 template의 raw 헤더를 섞지 않는다.
                table.raw_ctrl_data.clear();
                table.raw_ctrl_seal = None;
                table.common.ctrl_id = rhwp::parser::tags::CTRL_TABLE;
            }
        }
    }
    let mut doc = HwpDocument::create_empty();
    doc.set_document(document);
    (doc, path)
}

#[test]
fn styled_inline_objects_keep_first_and_last_cell_paragraph_logical_offsets() {
    for flat in [true, false] {
        for tables in [false, true] {
            for format in 0..3 {
                let (mut doc, path) = inline_axis_fixture(flat, tables);
                if format != 0 {
                    let bytes = if format == 1 {
                        doc.export_hwp_native().unwrap()
                    } else {
                        doc.export_hwpx_native().unwrap()
                    };
                    doc = HwpDocument::from_bytes(&bytes).unwrap();
                }
                assert!(cell(doc.document(), &path)[0].controls.iter().all(
                    |control| match control {
                        Control::Table(table) => table.common.treat_as_char,
                        Control::Picture(picture) => picture.common.treat_as_char,
                        _ => false,
                    }
                ));
                let before = serde_json::to_value(&doc.document().sections[0].paragraphs).unwrap();
                for pi in [0, 2] {
                    assert_inline_run_axis(&doc, &cell(doc.document(), &path)[pi], Some(&path), pi);
                }
                assert_eq!(
                    serde_json::to_value(&doc.document().sections[0].paragraphs).unwrap(),
                    before
                );
            }
        }
    }
}

#[test]
fn styled_inline_objects_keep_body_run_logical_offsets() {
    for tables in [false, true] {
        for format in 0..3 {
            let (mut doc, path) = inline_axis_fixture(true, tables);
            let mut document = doc.document().clone();
            let paragraphs = cell(&document, &path).to_vec();
            document.sections[0].paragraphs.splice(1..2, paragraphs);
            doc.set_document(document);
            if format != 0 {
                let bytes = if format == 1 {
                    doc.export_hwp_native().unwrap()
                } else {
                    doc.export_hwpx_native().unwrap()
                };
                doc = HwpDocument::from_bytes(&bytes).unwrap();
            }
            let before = serde_json::to_value(&doc.document().sections[0].paragraphs).unwrap();
            for pi in [1, 3] {
                assert_inline_run_axis(&doc, &doc.document().sections[0].paragraphs[pi], None, pi);
            }
            assert_eq!(
                serde_json::to_value(&doc.document().sections[0].paragraphs).unwrap(),
                before
            );
        }
    }
}

#[test]
fn styled_inline_objects_keep_logical_offsets_across_line_breaks() {
    for flat in [true, false] {
        for format in 0..3 {
            let (mut doc, path) = inline_axis_fixture(flat, false);
            if flat {
                doc.insert_text_in_cell_native(0, 1, 0, 0, 0, 2, "\n")
                    .unwrap();
            } else {
                doc.insert_text_in_cell_by_path(0, 1, &path, 2, "\n")
                    .unwrap();
            }
            if format != 0 {
                let bytes = if format == 1 {
                    doc.export_hwp_native().unwrap()
                } else {
                    doc.export_hwpx_native().unwrap()
                };
                doc = HwpDocument::from_bytes(&bytes).unwrap();
            }
            assert_inline_run_axis(&doc, &cell(doc.document(), &path)[0], Some(&path), 0);
        }
    }
}

#[test]
fn styled_inline_objects_keep_prior_slots_on_continuation_pages() {
    let tail = (0..90)
        .map(|i| format!("{i:03}가나다\n"))
        .collect::<String>();
    let text = format!("가🦦나다라\n{tail}");
    for format in 0..3 {
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().unwrap();
        doc.insert_text_native(0, 0, 0, "머리").unwrap();
        doc.split_paragraph_native(0, 0, 2, None).unwrap();
        doc.insert_text_native(0, 1, 0, &text).unwrap();
        for at in [1, 3] {
            let result: serde_json::Value = serde_json::from_str(
                &doc.insert_picture_native(
                    0,
                    1,
                    at,
                    &[],
                    include_bytes!("../../assets/logo/logo-16.png"),
                    1200,
                    1400,
                    16,
                    16,
                    "png",
                    "앞쪽 TAC",
                    None,
                    None,
                )
                .unwrap(),
            )
            .unwrap();
            doc.set_picture_properties_native(
                0,
                1,
                result["controlIdx"].as_u64().unwrap() as usize,
                r#"{"treatAsChar":true}"#,
            )
            .unwrap();
        }
        doc.apply_char_format_native(0, 1, 0, 2, r#"{"bold":true}"#)
            .unwrap();
        if format != 0 {
            let bytes = if format == 1 {
                doc.export_hwp_native().unwrap()
            } else {
                doc.export_hwpx_native().unwrap()
            };
            doc = HwpDocument::from_bytes(&bytes).unwrap();
        }
        assert!(doc.page_count() > 1);
        let layout: serde_json::Value =
            serde_json::from_str(&doc.get_page_text_layout_native(1).unwrap()).unwrap();
        let run = layout["runs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|run| {
                run["secIdx"] == 0
                    && run["paraIdx"] == 1
                    && !run["text"].as_str().unwrap().is_empty()
            })
            .unwrap();
        let token = run["text"].as_str().unwrap();
        assert_eq!(token.chars().count(), 3, "고유한 세 자리 줄 번호: {run}");
        assert!(token.chars().all(|c| c.is_ascii_digit()));
        // 실제 다음 쪽에 그린 고유한 글을 원문에서 찾아 앞쪽 두 개체 칸만 더한다.
        let scalar = text[..text.find(token).unwrap()].chars().count();
        assert_eq!(
            run["charStart"],
            scalar + 2,
            "format={format}, continuation={run}"
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
