//! 관계형: Native 문단 블록 가져오기의 리소스 소유·참조·원자성과 반복 저장 계약.
//! 합성 입력의 구조를 검사하며 한컴 PDF의 시각 일치율이나 실물 px 좌표를 고정하지 않는다.
use rhwp::{
    document_core::{DocumentCore, ImportParagraphBlockLimits, ImportParagraphBlockRequest},
    model::{
        bin_data::{BinData, BinDataBytes, BinDataContent, BinDataResolver, BinDataType},
        control::Control,
        document::DocInfo,
        image::Picture,
        paragraph::{CharShapeRef, Paragraph},
        style::{CharShape, Font, ParaShape, Style, TabDef},
    },
};
use std::sync::Arc;

fn core(texts: &[&str]) -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().doc_info = DocInfo {
        font_faces: vec![
            vec![Font {
                name: "바탕".into(),
                ..Default::default()
            }];
            7
        ],
        char_shapes: vec![CharShape::default()],
        para_shapes: vec![ParaShape::default()],
        styles: vec![Style::default()],
        tab_defs: vec![TabDef::default()],
        ..Default::default()
    };
    c.document_mut().sections[0].paragraphs = texts
        .iter()
        .map(|text| Paragraph {
            text: (*text).into(),
            char_count: text.encode_utf16().count() as u32 + 1,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            ..Default::default()
        })
        .collect();
    c
}
fn request() -> ImportParagraphBlockRequest {
    ImportParagraphBlockRequest {
        source_section: 0,
        source_start: 1,
        source_end: 2,
        target_section: 0,
        insert_before: 1,
        count: 2,
        limits: ImportParagraphBlockLimits::default(),
    }
}
fn image(c: &mut DocumentCore, storage: u16, bytes: &[u8]) {
    c.document_mut().doc_info.bin_data_list.push(BinData {
        storage_id: storage,
        data_type: BinDataType::Embedding,
        extension: Some("png".into()),
        ..Default::default()
    });
    c.document_mut().bin_data_content.push(BinDataContent {
        id: storage,
        extension: "png".into(),
        data: bytes.to_vec().into(),
    });
    let mut pic = Picture::default();
    pic.common.width = 1000;
    pic.common.height = 1000;
    pic.image_attr.bin_data_id = 1;
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Picture(Box::new(pic)));
}
fn unchanged(c: &DocumentCore) -> (String, String) {
    (format!("{:?}", c.document()), c.serialize_event_log())
}

#[test]
fn imports_only_reachable_styles_and_preserves_target_boundaries_and_source() {
    let mut source = core(&["source before", "template", "source after"]);
    source.document_mut().doc_info.font_faces[0][0].name = "돋움".into();
    source.document_mut().doc_info.char_shapes[0].base_size = 1700;
    source.document_mut().doc_info.char_shapes.push(CharShape {
        font_ids: [400; 7],
        ..Default::default()
    });
    let source_before = unchanged(&source);
    let mut target = core(&["before", "after"]);
    let before_font = serde_json::to_value(&target.document().doc_info.font_faces).unwrap();
    let before_page = format!("{:?}", target.document().sections[0].section_def);
    let result = target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    assert_eq!(result.inserted, 1..3);
    assert_eq!(result.copies.len(), 2);
    assert_eq!(
        target.document().sections[0]
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>(),
        ["before", "template", "template", "after"]
    );
    assert_eq!(
        format!("{:?}", target.document().sections[0].section_def),
        before_page
    );
    let info = &target.document().doc_info;
    assert_eq!(
        serde_json::to_value(&info.font_faces[0][..1]).unwrap(),
        before_font[0]
    );
    assert_eq!(
        info.char_shapes.len(),
        2,
        "unreachable invalid char shape must not be imported"
    );
    let copied = &target.document().sections[0].paragraphs[1];
    let shape = &info.char_shapes[copied.char_shapes[0].char_shape_id as usize];
    assert_eq!(shape.base_size, 1700);
    assert_eq!(info.font_faces[0][shape.font_ids[0] as usize].name, "돋움");
    assert_eq!(unchanged(&source), source_before);
    let before = format!("{:?}", target.document().doc_info);
    let again = target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    assert_eq!(again.resources.added, 0);
    assert_eq!(format!("{:?}", target.document().doc_info), before);
}

#[test]
fn cyclic_styles_are_linked_after_reservation_and_reused_on_later_calls() {
    let mut source = core(&["unused", "template"]);
    source.document_mut().doc_info.styles = vec![
        Style {
            local_name: "A".into(),
            next_style_id: 1,
            ..Default::default()
        },
        Style {
            local_name: "B".into(),
            next_style_id: 0,
            ..Default::default()
        },
    ];
    let mut target = core(&["before", "after"]);
    target.document_mut().doc_info.styles[0].local_name = "keep".into();
    let existing = serde_json::to_value(&target.document().doc_info.styles[0]).unwrap();
    target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    let styles = &target.document().doc_info.styles;
    assert_eq!(styles.len(), 3);
    assert_eq!(styles[1].next_style_id, 2);
    assert_eq!(styles[2].next_style_id, 1);
    assert_eq!(serde_json::to_value(&styles[0]).unwrap(), existing);
    let second = target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    assert_eq!(second.resources.added, 0);
    assert_eq!(target.document().doc_info.styles.len(), 3);
}

#[test]
fn full_preview_matches_execution_and_has_no_mutation() {
    let mut source = core(&["unused", "template"]);
    source.document_mut().doc_info.styles[0].local_name = "foreign".into();
    let mut target = core(&["before", "after"]);
    let before = unchanged(&target);
    let preview = target
        .preview_paragraph_block_import_native(source.document(), &request())
        .unwrap();
    assert_eq!(unchanged(&target), before);
    assert_eq!(
        preview,
        target
            .import_paragraph_block_native(source.document(), &request())
            .unwrap()
    );
}

#[test]
fn style_capacity_and_metadata_failures_leave_no_partial_resources() {
    let mut source = core(&["unused", "template"]);
    source.document_mut().doc_info.styles[0].local_name = "new".into();
    let mut target = core(&["before", "after"]);
    target.document_mut().doc_info.styles = (0..256)
        .map(|i| Style {
            local_name: format!("existing-{i}"),
            ..Default::default()
        })
        .collect();
    let before = unchanged(&target);
    assert!(target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap_err()
        .to_string()
        .contains("style ID space"));
    assert_eq!(unchanged(&target), before);
    let mut req = request();
    req.limits.max_resource_metadata_bytes = 1;
    assert!(target
        .import_paragraph_block_native(source.document(), &req)
        .is_err());
    assert_eq!(unchanged(&target), before);
}

#[test]
fn image_ordinal_is_not_confused_with_source_storage_id_and_copies_have_unique_ids() {
    let mut source = core(&["unused", "template"]);
    image(&mut source, 17, &[1, 2, 3, 4]);
    let mut target = core(&["before", "after"]);
    image(&mut target, 1, &[9, 8, 7, 6]);
    let source_before = unchanged(&source);
    let result = target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    assert_eq!(result.resources.binaries_added, 1);
    assert_eq!(target.document().bin_data_content.len(), 2);
    assert_eq!(
        target.document().bin_data_content[1].data.load(),
        [1, 2, 3, 4]
    );
    let pictures: Vec<_> = target.document().sections[0].paragraphs[1..3]
        .iter()
        .map(|p| {
            let Control::Picture(pic) = &p.controls[0] else {
                panic!("picture")
            };
            pic
        })
        .collect();
    assert_eq!(pictures[0].image_attr.bin_data_id, 2);
    assert_eq!(pictures[1].image_attr.bin_data_id, 2);
    assert_ne!(pictures[0].instance_id, pictures[1].instance_id);
    assert_eq!(unchanged(&source), source_before);
    let again = target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    assert_eq!(again.resources.binaries_added, 0);
    assert_eq!(again.resources.binaries_reused, 1);
    assert_eq!(target.document().bin_data_content.len(), 2);
}

#[test]
fn binary_limit_counts_source_and_target_comparison_reads() {
    let mut source = core(&["unused", "template"]);
    image(&mut source, 7, &[1, 2, 3, 4]);
    let mut target = core(&["before", "after"]);
    image(&mut target, 1, &[1, 2, 3, 4]);
    let before = unchanged(&target);
    let mut req = request();
    req.limits.max_binary_bytes = 7;
    assert!(target
        .import_paragraph_block_native(source.document(), &req)
        .is_err());
    assert_eq!(unchanged(&target), before);
    req.limits.max_binary_bytes = 8;
    let result = target
        .import_paragraph_block_native(source.document(), &req)
        .unwrap();
    assert_eq!(result.resources.binary_bytes_read, 8);
    assert_eq!(result.resources.binaries_added, 0);
}

#[test]
fn unbounded_resolver_external_link_and_missing_metadata_are_rejected_without_mutation() {
    #[derive(Debug)]
    struct Unbounded;
    impl BinDataResolver for Unbounded {
        fn resolve(&self, _: &str) -> Vec<u8> {
            panic!("unbounded load forbidden")
        }
        fn resolved_len(&self, _: &str) -> usize {
            panic!("unbounded length forbidden")
        }
    }
    let mut source = core(&["unused", "template"]);
    image(&mut source, 17, &[1]);
    let mut target = core(&["before", "after"]);
    let before = unchanged(&target);
    source.document_mut().bin_data_content[0].data = BinDataBytes::Lazy {
        resolver: Arc::new(Unbounded),
        key: "forbidden".into(),
    };
    assert!(target
        .import_paragraph_block_native(source.document(), &request())
        .is_err());
    assert_eq!(unchanged(&target), before);
    source.document_mut().doc_info.bin_data_list[0].data_type = BinDataType::Link;
    assert!(target
        .import_paragraph_block_native(source.document(), &request())
        .is_err());
    assert_eq!(unchanged(&target), before);
    source.document_mut().doc_info.bin_data_list.clear();
    assert!(target
        .import_paragraph_block_native(source.document(), &request())
        .is_err());
    assert_eq!(unchanged(&target), before);
}

#[test]
fn zero_count_does_not_prepare_resources_or_emit_events() {
    let mut source = core(&["unused", "template"]);
    source.document_mut().doc_info.styles.clear();
    let mut target = core(&["before", "after"]);
    let before = unchanged(&target);
    let mut req = request();
    req.count = 0;
    assert_eq!(
        target
            .import_paragraph_block_native(source.document(), &req)
            .unwrap()
            .resources
            .added,
        0
    );
    assert_eq!(unchanged(&target), before);
}

#[test]
fn embedded_font_and_image_references_share_bytes_but_keep_reference_meaning() {
    let mut source = core(&["unused", "template"]);
    image(&mut source, 17, &[1, 2, 3, 4]);
    let font = &mut source.document_mut().doc_info.font_faces[0][0];
    font.is_embedded = true;
    font.resolved_bin_data_id = Some(17);
    font.bin_item_id_ref = "source-font-manifest-name".into();
    let mut target = core(&["before", "after"]);
    let result = target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    assert_eq!(result.resources.binaries_added, 1);
    assert_eq!(
        result.resources.binary_bytes_read, 4,
        "same source bytes read once"
    );
    let paragraph = &target.document().sections[0].paragraphs[1];
    let info = &target.document().doc_info;
    let char_shape = &info.char_shapes[paragraph.char_shapes[0].char_shape_id as usize];
    let font = &info.font_faces[0][char_shape.font_ids[0] as usize];
    assert_eq!(font.resolved_bin_data_id, Some(1));
    let font_id = char_shape.font_ids[0] as usize;
    target.document_mut().doc_info.font_faces[0][font_id].bin_item_id_ref =
        "different-equivalent-name".into();
    let again = target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap();
    assert_eq!(again.resources.added, 0);
    assert_eq!(again.resources.binaries_added, 0);
}

#[test]
fn sparse_target_storage_collision_and_opaque_numbering_are_not_silently_repaired() {
    let mut source = core(&["unused", "template"]);
    image(&mut source, 17, &[1, 2, 3]);
    let mut target = core(&["before", "after"]);
    image(&mut target, 2, &[9, 8, 7]);
    let before = unchanged(&target);
    assert!(target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap_err()
        .to_string()
        .contains("ordinal collides"));
    assert_eq!(unchanged(&target), before);
    source.document_mut().sections[0].paragraphs[1]
        .controls
        .clear();
    source.document_mut().doc_info.para_shapes[0].head_type = rhwp::model::style::HeadType::Number;
    source.document_mut().doc_info.para_shapes[0].numbering_id = 1;
    source
        .document_mut()
        .doc_info
        .numberings
        .push(rhwp::model::style::Numbering {
            // 읽은 1수준 값(글자 모양 0)과 다른 원본 참조는 고쳐 옮기지 않는다.
            raw_para_heads: Some("<hh:paraHead level=\"1\" charPrIDRef=\"7\"/>".into()),
            ..Default::default()
        });
    assert!(target
        .import_paragraph_block_native(source.document(), &request())
        .unwrap_err()
        .to_string()
        .contains("charPrIDRef differs"));
    assert_eq!(unchanged(&target), before);
}

/// HWPX에서 연 번호·글머리표 정의는 원본 `hh:paraHead` 구간을 그대로 다시 쓴다.
/// 다른 문서로 옮기면 그 안의 글자 모양 참조도 대상 문서의 ID로 바뀌어야 한다.
#[test]
fn hwpx_numbering_and_bullet_import_with_target_char_shape_references() {
    check_hwpx_list_import_char_refs("canonical");
}

#[test]
fn hwpx_numbering_and_bullet_import_preserves_single_quoted_char_references() {
    check_hwpx_list_import_char_refs("single_quotes");
}

#[test]
fn hwpx_numbering_and_bullet_import_preserves_spaced_char_reference_attributes() {
    check_hwpx_list_import_char_refs("attribute_whitespace");
}

// 관계형: 공개 가져오기 API와 HWP/HWPX 반복 저장에서 같은 글자 모양을 가리키는지
// 검사한다. 합성 입력의 ID 충돌·XML 표기만 달리하며 실물 배치의 px 기대값은 만들지 않는다.
fn check_hwpx_list_import_char_refs(xml_spelling: &str) {
    use rhwp::model::style::HeadType;
    use rhwp::wasm_api::HwpDocument;

    fn blank(texts: &[&str]) -> HwpDocument {
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().unwrap();
        for (para, text) in texts.iter().enumerate() {
            if para > 0 {
                let end = texts[para - 1].chars().count();
                doc.split_paragraph_native(0, para - 1, end, None).unwrap();
            }
            doc.insert_text_native(0, para, 0, text).unwrap();
        }
        doc
    }
    fn reopen(doc: &DocumentCore, hwpx: bool) -> DocumentCore {
        let bytes = if hwpx {
            doc.export_hwpx_native().unwrap()
        } else {
            doc.export_hwp_native().unwrap()
        };
        DocumentCore::from_bytes(&bytes).unwrap()
    }
    /// 원본 구간에서 `level` 수준(글머리표는 첫 태그)의 `charPrIDRef` 값.
    fn raw_char_ref(raw: &str, level: Option<usize>) -> u32 {
        let mut reader = quick_xml::Reader::from_str(raw);
        loop {
            match reader.read_event().unwrap() {
                quick_xml::events::Event::Start(tag) | quick_xml::events::Event::Empty(tag)
                    if tag.local_name().as_ref() == "paraHead" =>
                {
                    let attributes: Vec<_> = tag.attributes().map(Result::unwrap).collect();
                    let at = attributes
                        .iter()
                        .find(|a| a.key.as_ref() == "level")
                        .map(|a| {
                            a.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                .unwrap()
                                .parse::<usize>()
                                .unwrap()
                        });
                    if level.is_none() || at == level {
                        return attributes
                            .iter()
                            .find(|a| a.key.as_ref() == "charPrIDRef")
                            .unwrap()
                            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                            .unwrap()
                            .parse()
                            .unwrap();
                    }
                }
                quick_xml::events::Event::Eof => panic!("paraHead 참조가 없다"),
                _ => {}
            }
        }
    }

    let mut list = blank(&["head", "numb", "bull", "tail"]);
    let number =
        list.create_numbering(r#"{"levelFormats":["^1."],"numberFormats":[0],"startNumber":1}"#);
    list.set_numbering_char_shape_native(number, 0, Some(r#"{"bold":true,"fontSize":1500}"#))
        .unwrap();
    list.apply_para_format_native(
        0,
        1,
        &format!(r#"{{"headType":"Number","paraLevel":0,"numberingId":{number}}}"#),
    )
    .unwrap();
    let bullet = list
        .create_bullet_native(
            r#"{"char":"●","charShape":{"italic":true,"fontSize":1200}}"#,
            None,
        )
        .unwrap();
    list.apply_para_format_native(
        0,
        2,
        &format!(r#"{{"headType":"Bullet","numberingId":{bullet}}}"#),
    )
    .unwrap();
    let mut source = reopen(&list, true);
    let rewrite = |raw: &str| match xml_spelling {
        "canonical" => raw.to_owned(),
        "single_quotes" => raw.replace('"', "'"),
        "attribute_whitespace" => raw.replace(" charPrIDRef=", "\tcharPrIDRef = "),
        _ => unreachable!(),
    };
    for numbering in &mut source.document_mut().doc_info.numberings {
        if let Some(raw) = numbering.raw_para_heads.as_mut() {
            *raw = rewrite(raw);
        }
    }
    for bullet in &mut source.document_mut().doc_info.bullets {
        if let Some(raw) = bullet.raw_para_head.as_mut() {
            *raw = rewrite(raw);
        }
    }
    // 표기가 다른 실제 HWPX 바이트를 파싱한 뒤 가져온다. 원시 모델만 바꾼 검사가 아니다.
    let source = reopen(&source, true);
    let info = &source.document().doc_info;
    let source_number = &info.numberings[usize::from(number) - 1];
    let source_bullet = &info.bullets[usize::from(bullet) - 1];
    assert!(source_number.raw_para_heads.is_some() && source_bullet.raw_para_head.is_some());

    // 대상에 글자 모양을 먼저 쌓아 원본과 같은 ID가 다른 모양을 가리키게 한다.
    let mut target = blank(&["first", "last"]);
    for (index, size) in [700, 800, 900, 1100].into_iter().enumerate() {
        target
            .apply_char_format_native(0, 0, index, index + 1, &format!(r#"{{"fontSize":{size}}}"#))
            .unwrap();
    }
    let result = target
        .import_paragraph_block_native(
            source.document(),
            &ImportParagraphBlockRequest {
                source_start: 1,
                source_end: 3,
                count: 1,
                ..request()
            },
        )
        .unwrap();
    assert_eq!(result.inserted, 1..3);

    let check = |doc: &DocumentCore| {
        let info = &doc.document().doc_info;
        let paras = &doc.document().sections[0].paragraphs;
        let number_shape = &info.para_shapes[usize::from(paras[1].para_shape_id)];
        let bullet_shape = &info.para_shapes[usize::from(paras[2].para_shape_id)];
        assert_eq!(number_shape.head_type, HeadType::Number);
        assert_eq!(bullet_shape.head_type, HeadType::Bullet);
        let number = &info.numberings[usize::from(number_shape.numbering_id) - 1];
        let bullet = &info.bullets[usize::from(bullet_shape.numbering_id) - 1];
        let number_char = &info.char_shapes[number.heads[0].char_shape_id as usize];
        let bullet_char = &info.char_shapes[bullet.char_shape_id as usize];
        assert!(number_char.bold && number_char.base_size == 1500);
        assert!(bullet_char.italic && bullet_char.base_size == 1200);
        if let Some(raw) = &number.raw_para_heads {
            assert_eq!(raw_char_ref(raw, Some(1)), number.heads[0].char_shape_id);
        }
        if let Some(raw) = &bullet.raw_para_head {
            assert_eq!(raw_char_ref(raw, None), bullet.char_shape_id);
        }
    };
    check(&target);
    let imported = &target.document().doc_info;
    let paras = &target.document().sections[0].paragraphs;
    let number_id = imported.para_shapes[usize::from(paras[1].para_shape_id)].numbering_id;
    assert_ne!(
        imported.numberings[usize::from(number_id) - 1].heads[0].char_shape_id,
        source_number.heads[0].char_shape_id,
        "the fixture must move the reference to a different target ID"
    );
    for hwpx in [true, false] {
        let saved = reopen(&target, hwpx);
        check(&saved);
        check(&reopen(&saved, hwpx));
    }
}

#[test]
fn real_labnote_block_imports_into_another_document_and_reopens_in_both_formats() {
    let source_bytes = std::fs::read("samples/rnote/labnote-001.hwp").unwrap();
    let source = DocumentCore::from_bytes(&source_bytes).unwrap();
    let originals: Vec<_> = source.document().sections[0].paragraphs[12]
        .controls
        .iter()
        .filter_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(originals.len(), 3, "the real source is a three-table block");
    let before = format!("{:?}", source.document());
    let mut target = DocumentCore::new_empty();
    target.create_blank_document_native().unwrap();
    let req = ImportParagraphBlockRequest {
        source_start: 12,
        source_end: 13,
        ..request()
    };
    let result = target
        .import_paragraph_block_native(source.document(), &req)
        .unwrap();
    assert_eq!(result.inserted, 1..3);
    for bytes in [
        target.export_hwp_native().unwrap(),
        target.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let tables: Vec<_> = reopened.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter_map(|c| {
                if let Control::Table(t) = c {
                    Some(t)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(tables.len(), originals.len() * req.count);
        let ids: std::collections::BTreeSet<_> =
            tables.iter().map(|t| t.common.instance_id).collect();
        assert_eq!(ids.len(), tables.len());
        for (i, table) in tables.iter().enumerate() {
            let original = originals[i % originals.len()];
            assert_eq!(table.row_count, original.row_count);
            assert_eq!(table.col_count, original.col_count);
            let text = |t: &rhwp::model::table::Table| {
                t.cells
                    .iter()
                    .map(|cell| {
                        cell.paragraphs
                            .iter()
                            .map(|p| p.text.clone())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(text(table), text(original));
        }
    }
    assert_eq!(format!("{:?}", source.document()), before);
}

#[test]
#[ignore = "manual artifact export; set RHWP_3587_IMPORT_OUTPUT to a local output directory"]
fn materialize_labnote_foreign_import() {
    materialize_labnote_import(false);
}

#[test]
#[ignore = "manual same-page artifact export; set RHWP_3587_IMPORT_OUTPUT"]
fn materialize_labnote_foreign_import_matching_page() {
    materialize_labnote_import(true);
}

fn materialize_labnote_import(match_source_page: bool) {
    let output = std::path::PathBuf::from(
        std::env::var("RHWP_3587_IMPORT_OUTPUT").expect("explicit output directory"),
    );
    std::fs::create_dir_all(&output).unwrap();
    let hwp = std::fs::read("samples/rnote/labnote-001.hwp").unwrap();
    let original = DocumentCore::from_bytes(&hwp).unwrap();
    for (kind, bytes) in [
        ("hwp-source", hwp),
        (
            "derived-hwpx-source",
            original.export_hwpx_native().unwrap(),
        ),
    ] {
        let source = DocumentCore::from_bytes(&bytes).unwrap();
        let mut target = DocumentCore::new_empty();
        target.create_blank_document_native().unwrap();
        if match_source_page {
            // Prepare the independent destination before import, not a repair of its output.
            // Copy page configuration only; source content and DocInfo are not the destination.
            let pd = &source.document().sections[0].section_def.page_def;
            let binding = match pd.binding {
                rhwp::model::page::BindingMethod::SingleSided => 0,
                rhwp::model::page::BindingMethod::DuplexSided => 1,
                rhwp::model::page::BindingMethod::TopFlip => 2,
            };
            let props = serde_json::json!({
                "width": pd.width, "height": pd.height,
                "marginLeft": pd.margin_left, "marginRight": pd.margin_right,
                "marginTop": pd.margin_top, "marginBottom": pd.margin_bottom,
                "marginHeader": pd.margin_header, "marginFooter": pd.margin_footer,
                "marginGutter": pd.margin_gutter, "landscape": pd.landscape,
                "binding": binding,
            });
            target.set_page_def_native(0, &props.to_string()).unwrap();
            assert_eq!(
                serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap(),
                serde_json::to_value(pd).unwrap(),
                "the source and independent destination must start with the same page settings"
            );
            std::fs::write(
                output.join(format!("{kind}-blank-target.hwp")),
                target.export_hwp_native().unwrap(),
            )
            .unwrap();
            std::fs::write(
                output.join(format!("{kind}-blank-target.hwpx")),
                target.export_hwpx_native().unwrap(),
            )
            .unwrap();
        }
        let page_before =
            serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap();
        let req = ImportParagraphBlockRequest {
            source_start: 12,
            source_end: 13,
            ..request()
        };
        let result = target
            .import_paragraph_block_native(source.document(), &req)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&target.document().sections[0].section_def.page_def).unwrap(),
            page_before,
            "import must preserve the destination page settings"
        );
        std::fs::write(
            output.join(format!("{kind}.hwp")),
            target.export_hwp_native().unwrap(),
        )
        .unwrap();
        std::fs::write(
            output.join(format!("{kind}.hwpx")),
            target.export_hwpx_native().unwrap(),
        )
        .unwrap();
        std::fs::write(
            output.join(format!("{kind}.json")),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
    }
}
