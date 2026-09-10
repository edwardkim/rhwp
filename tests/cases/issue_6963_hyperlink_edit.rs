//! #6963 웹 링크의 주소·범위·표시 문자열·저장 왕복 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::hyperlink::HyperlinkTarget;
use rhwp::document_core::queries::field_query::{FieldInfo, NestedEntry};
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FieldType, Parameter};
use rhwp::model::hyperlink::{command_uri, web_command};

fn blank(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    core
}

fn roundtrips(core: &DocumentCore) -> Vec<DocumentCore> {
    [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .iter()
    .map(|bytes| DocumentCore::from_bytes(bytes).unwrap())
    .collect()
}

#[test]
fn command_codec_preserves_unicode_query_fragment_and_escaped_delimiters() {
    let uri = "https://example.com/한글?q=가;나&x=%20#MN::문서:";
    let command = web_command(uri).unwrap();
    assert_eq!(command_uri(&command), uri);
    assert!(command.contains(r"https\://"));
    assert!(command.contains(r"가\;나"));
    assert!(command.contains(r"\#MN\:\:"));
    assert_eq!(command_uri(r"a\\b\;c;1;0;0;"), r"a\b;c");
    assert_eq!(
        command_uri("mailto:person@example.com;1;0;0;"),
        "mailto:person@example.com"
    );
    for invalid in [
        "",
        "javascript:alert(1)",
        "file:///tmp/a",
        "https://",
        "https:///a",
        "https://a\nb",
        " https://example.com",
        "https://user@example.com",
    ] {
        assert!(web_command(invalid).is_err(), "{invalid:?}");
    }
    assert!(web_command(&format!("https://example.com/{}", "가".repeat(65536))).is_err());
}

#[test]
fn selected_unicode_text_survives_insert_update_remove_and_both_formats() {
    let mut core = blank("앞😀한글\t링크 뒤");
    let original = core.document().sections[0].paragraphs[0].text.clone();
    let id = core
        .insert_hyperlink_native(
            &HyperlinkTarget::body(0, 0),
            1,
            7,
            "https://example.com/한글?a=1;2#위치",
        )
        .unwrap();
    for reopened in roundtrips(&core) {
        let links = reopened
            .hyperlinks_native(&HyperlinkTarget::body(0, 0))
            .unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!((links[0].start, links[0].end), (1, 7));
        assert_eq!(links[0].text, "😀한글\t링크");
        assert_eq!(links[0].uri, "https://example.com/한글?a=1;2#위치");
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, original);
    }
    core.update_hyperlink_native(
        &HyperlinkTarget::body(0, 0),
        id,
        "http://example.org/?q=새주소#b",
    )
    .unwrap();
    for mut reopened in roundtrips(&core) {
        assert_eq!(
            reopened
                .hyperlinks_native(&HyperlinkTarget::body(0, 0))
                .unwrap()[0]
                .uri,
            "http://example.org/?q=새주소#b"
        );
        reopened
            .remove_hyperlink_native(&HyperlinkTarget::body(0, 0), id)
            .unwrap();
        for removed in roundtrips(&reopened) {
            assert!(removed
                .hyperlinks_native(&HyperlinkTarget::body(0, 0))
                .unwrap()
                .is_empty());
            assert_eq!(removed.document().sections[0].paragraphs[0].text, original);
        }
    }
}

#[test]
fn adjacent_links_keep_ranges_when_first_middle_or_last_is_removed() {
    for remove in 0..3 {
        let mut core = blank("가나다라마바");
        let mut ids = Vec::new();
        for i in 0..3 {
            ids.push(
                core.insert_hyperlink_native(
                    &HyperlinkTarget::body(0, 0),
                    i * 2,
                    i * 2 + 2,
                    &format!("https://example.com/{i}"),
                )
                .unwrap(),
            );
        }
        for mut reopened in roundtrips(&core) {
            reopened
                .remove_hyperlink_native(&HyperlinkTarget::body(0, 0), ids[remove])
                .unwrap();
            for result in roundtrips(&reopened) {
                let links = result
                    .hyperlinks_native(&HyperlinkTarget::body(0, 0))
                    .unwrap();
                assert_eq!(links.len(), 2);
                for (link, i) in links.iter().zip((0..3).filter(|i| *i != remove)) {
                    assert_eq!((link.start, link.end), (i * 2, i * 2 + 2));
                    assert_eq!(link.uri, format!("https://example.com/{i}"));
                }
                assert_eq!(
                    result.document().sections[0].paragraphs[0].text,
                    "가나다라마바"
                );
            }
        }
    }
}

#[test]
fn failures_and_same_address_do_not_mutate_document_or_events() {
    let mut core = blank("테스트 문서");
    let id = core
        .insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 1, 3, "https://example.com")
        .unwrap();
    let before = format!("{:?}", core.document());
    let events = core.serialize_event_log();
    assert!(!core
        .update_hyperlink_native(&HyperlinkTarget::body(0, 0), id, "https://example.com")
        .unwrap());
    for (start, end) in [(0, 99), (1, 1), (3, 2), (0, 2)] {
        assert!(core
            .insert_hyperlink_native(
                &HyperlinkTarget::body(0, 0),
                start,
                end,
                "https://example.org"
            )
            .is_err());
    }
    assert!(core
        .update_hyperlink_native(&HyperlinkTarget::body(0, 0), id, "javascript:bad")
        .is_err());
    assert!(core
        .remove_hyperlink_native(&HyperlinkTarget::body(0, 0), id + 100)
        .is_err());
    assert!(core
        .insert_hyperlink_native(&HyperlinkTarget::body(999, 0), 0, 1, "https://example.org")
        .is_err());
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.serialize_event_log(), events);
}

#[test]
fn hancom_hwpx_edit_replaces_cached_command_and_path_without_losing_other_parameters() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/hwpx_sample2.hwpx"
    ))
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let info = core
        .collect_all_fields()
        .into_iter()
        .find(|info| {
            info.field.field_type == FieldType::Hyperlink
                && command_uri(&info.field.command)
                    == "https://apply.lh.or.kr/LH/index.html#MN::CLCC_MN_0010:"
        })
        .expect("한컴의 fragment 링크");
    let target = target_of(&info);
    let id = info.field.field_id;
    let parameters = info.field.parameters;
    let uri = "https://example.com/변경?q=a;b#문서::";
    core.update_hyperlink_native(&target, id, uri).unwrap();
    let reopened_hwp = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    let fields = reopened_hwp.collect_all_fields();
    assert_eq!(
        command_uri(
            &fields
                .iter()
                .find(|info| info.field.field_id == id)
                .unwrap()
                .field
                .command
        ),
        uri
    );
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    {
        let fields = reopened.collect_all_fields();
        let field = &fields
            .iter()
            .find(|info| info.field.field_id == id)
            .unwrap()
            .field;
        assert_eq!(command_uri(&field.command), uri);
        for item in &parameters.items {
            match item {
                Parameter::String {
                    name: Some(name), ..
                } if name == "Command" || name == "Path" => {}
                _ => assert!(
                    field.parameters.items.contains(item),
                    "사라진 파라미터: {item:?}"
                ),
            }
        }
        assert!(field.parameters.items.iter().any(|item| matches!(item, Parameter::String { name: Some(name), value, .. } if name == "Path" && value == uri)));
    }
}

#[test]
fn hancom_hwp_link_removal_preserves_display_text_and_neighboring_controls() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/basic/Textmail.hwp"
    ))
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let info = core
        .collect_all_fields()
        .into_iter()
        .find(|info| info.field.field_type == FieldType::Hyperlink)
        .expect("한컴의 링크");
    let target = target_of(&info);
    let text = all_text(core.document());
    let link = core
        .hyperlinks_native(&target)
        .unwrap()
        .into_iter()
        .find(|l| l.field_id == info.field.field_id)
        .unwrap();
    let id = info.field.field_id;
    core.remove_hyperlink_native(&target, id).unwrap();
    for reopened in roundtrips(&core) {
        assert!(reopened.hyperlinks_native(&target).unwrap().is_empty());
        assert_eq!(all_text(reopened.document()), text);
    }
    let replacement = core
        .insert_hyperlink_native(&target, link.start, link.end, "https://example.com/새링크")
        .unwrap();
    for reopened in roundtrips(&core) {
        let link = reopened
            .hyperlinks_native(&target)
            .unwrap()
            .into_iter()
            .find(|l| l.field_id == replacement)
            .unwrap();
        assert_eq!(link.uri, "https://example.com/새링크");
        assert_eq!(all_text(reopened.document()), text);
    }
}

#[test]
fn snapshot_restore_undoes_link_metadata_and_style_boundaries_stay_visible() {
    use rhwp::model::paragraph::CharShapeRef;
    let mut core = blank("앞😀링크뒤");
    let p = &mut core.document_mut().sections[0].paragraphs[0];
    p.char_shapes = vec![
        CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        },
        CharShapeRef {
            start_pos: p.char_offsets[2],
            char_shape_id: 1,
        },
    ];
    let styles = core.get_char_shape_runs_native(0, 0, 0, 5).unwrap();
    let before = core.save_snapshot_native();
    let id = core
        .insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 1, 4, "https://example.com")
        .unwrap();
    assert_eq!(core.get_char_shape_runs_native(0, 0, 0, 5).unwrap(), styles);
    let after = core.save_snapshot_native();
    core.restore_snapshot_native(before).unwrap();
    assert!(core
        .hyperlinks_native(&HyperlinkTarget::body(0, 0))
        .unwrap()
        .is_empty());
    core.restore_snapshot_native(after).unwrap();
    assert_eq!(
        core.hyperlinks_native(&HyperlinkTarget::body(0, 0))
            .unwrap()[0]
            .field_id,
        id
    );
    core.remove_hyperlink_native(&HyperlinkTarget::body(0, 0), id)
        .unwrap();
    assert_eq!(core.get_char_shape_runs_native(0, 0, 0, 5).unwrap(), styles);
}

#[test]
fn unsupported_object_paragraph_is_rejected_without_mutation() {
    let mut core = blank("본문");
    core.document_mut().sections[0].paragraphs[0]
        .controls
        .push(Control::Picture(Default::default()));
    let before = format!("{:?}", core.document());
    assert!(core
        .insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 0, 2, "https://example.com")
        .is_err());
    assert_eq!(format!("{:?}", core.document()), before);
}

#[test]
fn new_ids_include_fields_in_footnotes_and_reject_overflow() {
    use rhwp::model::control::Field;
    use rhwp::model::paragraph::Paragraph;
    let mut core = blank("본문");
    let mut note = rhwp::model::footnote::Footnote::default();
    note.paragraphs.push(Paragraph {
        controls: vec![Control::Field(Field {
            field_id: 5000,
            ..Default::default()
        })],
        ..Default::default()
    });
    core.document_mut().sections[0].paragraphs.push(Paragraph {
        controls: vec![Control::Footnote(Box::new(note))],
        ..Default::default()
    });
    let id = core
        .insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 0, 1, "https://example.com")
        .unwrap();
    assert!(id > 5000);
    let Control::Field(field) = core.document_mut().sections[0].paragraphs[0]
        .controls
        .last_mut()
        .unwrap()
    else {
        panic!()
    };
    field.field_id = u32::MAX;
    let before = format!("{:?}", core.document());
    assert!(core
        .insert_hyperlink_native(&HyperlinkTarget::body(0, 0), 1, 2, "https://example.com")
        .is_err());
    assert_eq!(format!("{:?}", core.document()), before);
}

fn target_of(info: &FieldInfo) -> HyperlinkTarget {
    HyperlinkTarget {
        section: info.location.section_index,
        para: info.location.para_index,
        cell_path: info
            .location
            .nested_path
            .iter()
            .map(|entry| match entry {
                NestedEntry::TableCell {
                    control_index,
                    cell_index,
                    para_index,
                } => (*control_index, *cell_index, *para_index),
                NestedEntry::TextBox {
                    control_index,
                    para_index,
                } => (*control_index, 0, *para_index),
            })
            .collect(),
    }
}

fn all_text(doc: &rhwp::model::document::Document) -> String {
    fn visit(p: &rhwp::model::paragraph::Paragraph, out: &mut String) {
        out.push_str(&p.text);
        for c in &p.controls {
            match c {
                Control::Table(t) => {
                    for cell in &t.cells {
                        for p in &cell.paragraphs {
                            visit(p, out);
                        }
                    }
                }
                Control::Shape(s) => {
                    if let Some(d) = s.drawing() {
                        if let Some(t) = &d.text_box {
                            for p in &t.paragraphs {
                                visit(p, out);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = String::new();
    for s in &doc.sections {
        for p in &s.paragraphs {
            visit(p, &mut out);
        }
    }
    out
}

#[test]
fn text_edits_before_and_inside_link_preserve_saved_range() {
    let target = HyperlinkTarget::body(0, 0);
    let mut core = blank("앞링크뒤");
    let id = core
        .insert_hyperlink_native(&target, 1, 3, "https://example.com")
        .unwrap();
    core.insert_text_native(0, 0, 0, "😀").unwrap();
    core.insert_text_native(0, 0, 3, "새").unwrap();
    for reopened in roundtrips(&core) {
        let link = &reopened.hyperlinks_native(&target).unwrap()[0];
        assert_eq!((link.start, link.end, link.text.as_str()), (2, 5, "링새크"));
    }
    core.delete_text_native(0, 0, 2, 3).unwrap();
    core.remove_hyperlink_native(&target, id).unwrap();
    for reopened in roundtrips(&core) {
        assert!(reopened.hyperlinks_native(&target).unwrap().is_empty());
        assert_eq!(reopened.document().sections[0].paragraphs[0].text, "😀앞뒤");
    }
}

#[test]
fn unknown_schemes_survive_neighbor_edit_and_bad_nested_paths_fail_atomically() {
    let target = HyperlinkTarget::body(0, 0);
    let mut core = blank("메일 웹주소");
    let mail_id = core
        .insert_hyperlink_native(&target, 0, 2, "https://example.com")
        .unwrap();
    let Control::Field(field) = core.document_mut().sections[0].paragraphs[0]
        .controls
        .last_mut()
        .unwrap()
    else {
        panic!()
    };
    field.command = "mailto:user@example.com;1;0;0;".into();
    let web_id = core
        .insert_hyperlink_native(&target, 3, 6, "https://example.org")
        .unwrap();
    assert!(core
        .update_hyperlink_native(&target, mail_id, "https://example.net")
        .is_err());
    core.update_hyperlink_native(&target, web_id, "https://example.org/new")
        .unwrap();
    let before = format!("{:?}", core.document());
    let bad = HyperlinkTarget {
        cell_path: vec![(999, 0, 0)],
        ..target.clone()
    };
    assert!(core.remove_hyperlink_native(&bad, web_id).is_err());
    assert_eq!(format!("{:?}", core.document()), before);
    for reopened in roundtrips(&core) {
        let links = reopened.hyperlinks_native(&target).unwrap();
        assert_eq!(links[0].uri, "mailto:user@example.com");
        assert_eq!(links[1].uri, "https://example.org/new");
    }
}

#[test]
fn replace_display_text_keeps_adjacent_fields_and_roundtrips() {
    let mut core = blank("앞가나다뒤");
    let target = HyperlinkTarget::body(0, 0);
    let ids: Vec<_> = (1..4)
        .map(|i| {
            core.insert_hyperlink_native(&target, i, i + 1, "https://example.com")
                .unwrap()
        })
        .collect();
    for text in ["새로운😀문자열", "짧", "다시 늘림"] {
        assert!(core
            .replace_hyperlink_text_native(&target, ids[1], text)
            .unwrap());
        let expected = core.hyperlinks_native(&target).unwrap();
        assert_eq!(expected[0].text, "가");
        assert_eq!(expected[1].text, text);
        assert_eq!(expected[2].text, "다");
        assert_eq!(expected[0].end, expected[1].start);
        assert_eq!(expected[1].end, expected[2].start);
        assert_eq!(expected.iter().map(|l| l.field_id).collect::<Vec<_>>(), ids);
        assert!(!core
            .replace_hyperlink_text_native(&target, ids[1], text)
            .unwrap());
        for reopened in roundtrips(&core) {
            assert_eq!(reopened.hyperlinks_native(&target).unwrap(), expected);
        }
    }
    let before = core.hyperlinks_native(&target).unwrap();
    for invalid in ["", " ", "두\n줄", "탭\t"] {
        assert!(core
            .replace_hyperlink_text_native(&target, ids[1], invalid)
            .is_err());
        assert_eq!(core.hyperlinks_native(&target).unwrap(), before);
    }
}

#[test]
fn typing_after_link_keeps_original_format_and_excludes_new_text() {
    let mut core = blank("가나다");
    let target = HyperlinkTarget::body(0, 0);
    let original_shape = core.document().sections[0].paragraphs[0].char_shape_id_at(0);
    core.insert_hyperlink_native(&target, 0, 3, "https://www.hancom.com")
        .unwrap();
    core.apply_char_format_native(
        0,
        0,
        0,
        3,
        r##"{"textColor":"#0000ff","underlineType":"Bottom","underlineColor":"#0000ff"}"##,
    )
    .unwrap();
    // 방문 색으로 바꾸어도 링크 밖의 복원 서식은 유지되어야 한다.
    core.apply_char_format_native(
        0,
        0,
        0,
        3,
        r##"{"textColor":"#800080","underlineColor":"#800080"}"##,
    )
    .unwrap();
    let reopened = roundtrips(&core);
    for mut candidate in std::iter::once(core).chain(reopened) {
        // 한글 조합 갱신과 같은 같은 위치의 delete/insert 경로.
        candidate
            .replace_body_text_local_native(0, 0, 3, 0, "ㄱ")
            .unwrap();
        candidate
            .replace_body_text_local_native(0, 0, 3, 1, "가")
            .unwrap();
        candidate.insert_text_native(0, 0, 4, "나😀").unwrap();
        let reopened = roundtrips(&candidate);
        for result in std::iter::once(candidate).chain(reopened) {
            let link = &result.hyperlinks_native(&target).unwrap()[0];
            assert_eq!((link.start, link.end, link.text.as_str()), (0, 3, "가나다"));
            let para = &result.document().sections[0].paragraphs[0];
            assert_eq!(para.text, "가나다가나😀");
            assert_eq!(para.char_shape_id_at(3), original_shape);
            assert_eq!(para.char_shape_id_at(5), original_shape);
        }
    }
}

#[test]
fn typing_inside_link_still_extends_its_range() {
    let mut core = blank("가나다");
    let target = HyperlinkTarget::body(0, 0);
    core.insert_hyperlink_native(&target, 0, 3, "https://www.hancom.com")
        .unwrap();
    core.insert_text_native(0, 0, 1, "😀").unwrap();
    let reopened = roundtrips(&core);
    for result in std::iter::once(core).chain(reopened) {
        let link = &result.hyperlinks_native(&target).unwrap()[0];
        assert_eq!(
            (link.start, link.end, link.text.as_str()),
            (0, 4, "가😀나다")
        );
    }
}
