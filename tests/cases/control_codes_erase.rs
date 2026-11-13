//! 한컴 조판 부호 지우기·찾아가기의 갈래별 자리(`control_codes_native`)와 지우기
//! (`erase_control_codes_native`).
//!
//! 필드 꼴은 한컴 12.30 Mac이 저장한 HWPX에서 그대로 옮겼다(누름틀·날짜·파일 경로·문서 요약·
//! 메모·메일 머지·교정 부호·제목 차례). 한컴 12.30은 이것들을 지우면 필드만 풀고 글을 남긴다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use std::io::{Cursor, Read, Write};

/// HWPX 로 저장한 본문 XML 을 고쳐 다시 연다.
fn rewrite_section(core: &DocumentCore, edit: impl Fn(&str) -> String) -> DocumentCore {
    let bytes = core.export_hwpx_native().unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        let name = entry.name().expect("HWPX ZIP 엔트리 이름").to_string();
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).unwrap();
            let edited = edit(&xml);
            assert_ne!(edited, xml, "본문 XML 을 고치지 못했다");
            data = edited.into_bytes();
        }
        output
            .start_file(
                name,
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        output.write_all(&data).unwrap();
    }
    DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap()
}

/// 줄마다 한 문단인 빈 문서.
fn lines(texts: &[&str]) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    for (i, text) in texts.iter().enumerate() {
        if i > 0 {
            let end = texts[i - 1].chars().count();
            doc.split_paragraph_native(0, i - 1, end, None).unwrap();
        }
        if !text.is_empty() {
            doc.insert_text_native(0, i, 0, text).unwrap();
        }
    }
    doc
}

fn field(id: u32, ty: &str, params: &str, content: &str) -> String {
    format!(
        r#"<hp:ctrl><hp:fieldBegin id="{id}" type="{ty}" name="" editable="0" dirty="1" zorder="-1" fieldid="{id}" metaTag=""><hp:parameters cnt="2" name=""><hp:integerParam name="Prop">8</hp:integerParam>{params}</hp:parameters></hp:fieldBegin></hp:ctrl><hp:t>{content}</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="{id}" fieldid="{id}"/></hp:ctrl>"#
    )
}

fn command(value: &str) -> String {
    format!(r#"<hp:stringParam name="Command">{value}</hp:stringParam>"#)
}

/// 한컴 12.30 저장본의 필드 꼴을 줄마다 하나씩 담은 문서.
fn hancom_fields() -> DocumentCore {
    let doc = lines(&["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9"]);
    rewrite_section(&doc, |xml| {
        let memo = r#"<hp:ctrl><hp:fieldBegin id="5" type="MEMO" name="" editable="1" dirty="1" zorder="1" fieldid="5" metaTag=""><hp:parameters cnt="3" name=""><hp:integerParam name="Prop">0</hp:integerParam><hp:stringParam name="Command">MEMO/65535/1/1/1/author/\;;</hp:stringParam><hp:integerParam name="Number">1</hp:integerParam></hp:parameters><hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0"><hp:p id="0" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>MEMOBODY</hp:t></hp:run></hp:p></hp:subList></hp:fieldBegin></hp:ctrl><hp:t>here</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="5" fieldid="5"/></hp:ctrl>"#;
        let parts = [
            (
                "F1",
                format!(
                    "<hp:t>F1 </hp:t>{}<hp:t> tail</hp:t>",
                    field(
                        1,
                        "CLICK_HERE",
                        r#"<hp:stringParam name="Command" xml:space="preserve">Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  </hp:stringParam>"#,
                        "fill",
                    )
                ),
            ),
            (
                "F2",
                format!(
                    "<hp:t>F2 </hp:t>{}",
                    field(2, "DATE", &command("\\:1년 2월 3일;0;"), "2026년 11월 13일")
                ),
            ),
            (
                "F3",
                format!(
                    "<hp:t>F3 </hp:t>{}",
                    field(3, "PATH", &command("$F"), "a.hwpx")
                ),
            ),
            (
                "F4",
                format!(
                    "<hp:t>F4 </hp:t>{}",
                    field(4, "SUMMERY", &command("$author"), "someone")
                ),
            ),
            ("F5", format!("<hp:t>F5 memo </hp:t>{memo}")),
            (
                "F6",
                format!(
                    "<hp:t>F6 mm </hp:t>{}",
                    field(6, "MAILMERGE", &command("이름"), "{{이름}}")
                ),
            ),
            (
                "F7",
                format!(
                    "<hp:t>F7 proof </hp:t>{}",
                    field(
                        7,
                        "PROOFREADING_MARKS_DELETE",
                        &command("$RevisionDelete;"),
                        "word"
                    )
                ),
            ),
            (
                "F8",
                format!(
                    "<hp:t>F8 good </hp:t>{}",
                    field(
                        8,
                        "PROOFREADING_MARKS_PRAISE",
                        &command("$RevisionPraise;"),
                        "word"
                    )
                ),
            ),
            (
                "F9",
                r#"<hp:t><hp:titleMark ignore="1"/>F9 toc</hp:t>"#.to_string(),
            ),
        ];
        let mut xml = xml.to_string();
        for (marker, replacement) in parts {
            let from = format!("<hp:t>{marker}</hp:t>");
            assert!(xml.contains(&from), "{marker} 문단이 없다");
            xml = xml.replacen(&from, &replacement, 1);
        }
        xml
    })
}

const FIELD_KINDS: [&str; 9] = [
    "누름틀",
    "현재 날짜",
    "파일 경로",
    "문서 요약",
    "메모",
    "메일 머지",
    "교정 부호:지움표",
    "교정 부호:칭찬표",
    "제목 차례",
];

const ERASED_TEXT: [&str; 9] = [
    "F1 fill tail",
    "F2 2026년 11월 13일",
    "F3 a.hwpx",
    "F4 someone",
    "F5 memo here",
    "F6 mm {{이름}}",
    "F7 proof word",
    "F8 good word",
    "F9 toc",
];

fn names(kinds: &[&str]) -> Vec<String> {
    kinds.iter().map(|k| k.to_string()).collect()
}

fn texts(core: &DocumentCore) -> Vec<String> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect()
}

fn has_fields_or_marks(core: &DocumentCore) -> bool {
    core.document().sections[0].paragraphs.iter().any(|p| {
        !p.title_marks.is_empty()
            || !p.field_ranges.is_empty()
            || p.controls.iter().any(|c| matches!(c, Control::Field(_)))
    })
}

#[test]
fn hancom_field_kinds_are_listed_in_document_order() {
    let core = hancom_fields();
    let codes = core.control_codes_native(&names(&FIELD_KINDS));
    let kinds: Vec<&str> = codes.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, FIELD_KINDS);
    let paras: Vec<usize> = codes.iter().map(|c| c.para).collect();
    assert_eq!(paras, (0..9).collect::<Vec<_>>());
    // 교정 부호 묶음 이름은 모든 교정 부호를 고른다.
    let marks = core.control_codes_native(&names(&["교정 부호"]));
    assert_eq!(marks.len(), 2);
}

#[test]
fn erasing_fields_keeps_their_text_like_hancom() {
    let mut core = hancom_fields();
    let count = core
        .erase_control_codes_native(&names(&FIELD_KINDS), None)
        .unwrap();
    assert_eq!(count, 9);
    assert_eq!(texts(&core), ERASED_TEXT);
    assert!(!has_fields_or_marks(&core));
    assert!(core.control_codes_native(&names(&FIELD_KINDS)).is_empty());
    // 두 형식으로 저장해 다시 열어도 글은 남고 필드는 없다.
    for bytes in [
        core.export_hwpx_native().unwrap(),
        core.export_hwp_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(texts(&reopened), ERASED_TEXT);
        assert!(!has_fields_or_marks(&reopened));
    }
}

#[test]
fn erasing_chosen_indices_leaves_the_rest() {
    let mut core = hancom_fields();
    let kinds = names(&FIELD_KINDS);
    assert_eq!(
        core.erase_control_codes_native(&kinds, Some(&[1usize, 3][..]))
            .unwrap(),
        2
    );
    let left: Vec<&str> = core
        .control_codes_native(&kinds)
        .iter()
        .map(|c| c.kind)
        .collect();
    assert_eq!(
        left,
        [
            "누름틀",
            "파일 경로",
            "메모",
            "메일 머지",
            "교정 부호:지움표",
            "교정 부호:칭찬표",
            "제목 차례"
        ]
    );
}

/// 본문·표 셀·머리말·각주에 탭을 하나씩 둔 문서.
fn tabs_everywhere() -> HwpDocument {
    let mut doc = lines(&["본문\t끝", ""]);
    let table: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 1, 0, 1, 1).unwrap()).unwrap();
    let table_para = table["paraIdx"].as_u64().unwrap() as usize;
    let table_ctrl = table["controlIdx"].as_u64().unwrap() as usize;
    doc.insert_text_in_cell_by_path(0, table_para, &[(table_ctrl, 0, 0)], 0, "칸\t")
        .unwrap();
    doc.create_header_footer_native(0, true, 0).unwrap();
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, "머리\t말")
        .unwrap();
    let note: serde_json::Value =
        serde_json::from_str(&doc.insert_footnote_native(0, 0, 2).unwrap()).unwrap();
    let note_ctrl = note["controlIdx"].as_u64().unwrap() as usize;
    doc.insert_text_in_footnote_native(0, 0, note_ctrl, 0, 0, "주\t석")
        .unwrap();
    doc
}

#[test]
fn codes_in_cells_headers_and_notes_are_found_and_erased() {
    let mut doc = tabs_everywhere();
    let tab = names(&["탭"]);
    let codes = doc.control_codes_native(&tab);
    let mut areas: Vec<&str> = codes
        .iter()
        .map(|c| c.path.last().map_or("body", |s| s.area))
        .collect();
    areas.sort_unstable();
    assert_eq!(areas, ["body", "cell", "footnote", "header"]);
    assert_eq!(doc.erase_control_codes_native(&tab, None).unwrap(), 4);
    assert!(doc.control_codes_native(&tab).is_empty());
    // 내용을 품은 개체는 내용째 지운다.
    let containers = names(&["표", "머리말", "각주"]);
    assert_eq!(
        doc.erase_control_codes_native(&containers, None).unwrap(),
        3
    );
    assert!(doc.control_codes_native(&containers).is_empty());
    let section = &doc.document().sections[0];
    assert_eq!(section.paragraphs[0].text, "본문끝");
    assert!(section.paragraphs.iter().all(|p| p
        .controls
        .iter()
        .all(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)))));
    // 표만 있던 문단은 빈 문단으로 남는다.
    assert!(section.paragraphs[1..].iter().all(|p| p.text.is_empty()));
    let reopened = DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    assert_eq!(texts(&reopened)[0], "본문끝");
}

#[test]
fn revision_marks_map_to_hancom_names() {
    // 한컴 12.30 교정 부호 넣기가 저장한 종류·명령과 그 목록 이름.
    let marks = [
        ("SIGN", "$RevisionSign;1;", "교정 부호:띄움표"),
        ("SIGN", "$RevisionSign?XX;15;", "교정 부호:넣음표"),
        ("SIGN", "$RevisionSign?.;14;", "교정 부호:부호 넣음표"),
        ("SIGN", "$RevisionSign;2;", "교정 부호:줄 바꿈표"),
        ("SIGN", "$RevisionSign;3;", "교정 부호:줄 비움표"),
        (
            "SIMPLECHANGE",
            "$RevisionSimpleChange?KK;",
            "교정 부호:고침표",
        ),
        ("CLIPPING", "$RevisionClipping;", "교정 부호:뺌표"),
        ("ATTACH", "$RevisionAttach;", "교정 부호:붙임표"),
        ("LINEATTACH", "$RevisionLineAttach;", "교정 부호:줄 붙임표"),
        ("SAWTOOTH", "$RevisionSawtooth;", "교정 부호:톱니표"),
        ("THINKING", "$RevisionThinking;", "교정 부호:생각표"),
        ("LINE", "$RevisionLine;", "교정 부호:줄표"),
        ("TRANSFER", "$RevisionTransfer", "교정 부호:자리 바꿈표"),
    ];
    let markers: Vec<String> = (0..marks.len()).map(|i| format!("M{i}")).collect();
    let doc = lines(&markers.iter().map(String::as_str).collect::<Vec<_>>());
    let core = rewrite_section(&doc, |xml| {
        let mut xml = xml.to_string();
        for (i, (ty, cmd, _)) in marks.iter().enumerate() {
            let replacement = format!(
                "<hp:t>M{i} </hp:t>{}",
                field(
                    100 + i as u32,
                    &format!("PROOFREADING_MARKS_{ty}"),
                    &command(cmd),
                    "abcd"
                )
            );
            xml = xml.replacen(&format!("<hp:t>M{i}</hp:t>"), &replacement, 1);
        }
        xml
    });
    let expected: Vec<&str> = marks.iter().map(|(_, _, name)| *name).collect();
    let found: Vec<&str> = core
        .control_codes_native(&names(&expected))
        .iter()
        .map(|c| c.kind)
        .collect();
    assert_eq!(found, expected);
}

#[test]
fn erasing_a_multi_paragraph_table_of_contents_keeps_its_text() {
    // 한컴 12.30 차례 만들기(필드로 넣기)는 차례 전체를 여러 문단에 걸친 TABLEOFCONTENTS 필드로 감싼다.
    // 조판 부호 지우기로 차례를 지우면 필드만 사라지고 차례 글은 남는다.
    let doc = lines(&["T1", "T2", "T3"]);
    let mut core = rewrite_section(&doc, |xml| {
        let begin = r#"<hp:ctrl><hp:fieldBegin id="9" type="TABLEOFCONTENTS" name="" editable="0" dirty="0" zorder="-1" fieldid="9" metaTag=""><hp:parameters cnt="1" name=""><hp:integerParam name="Prop">8</hp:integerParam></hp:parameters></hp:fieldBegin></hp:ctrl><hp:t>T1</hp:t>"#;
        let end = r#"<hp:t>T3</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="9" fieldid="9"/></hp:ctrl>"#;
        xml.replacen("<hp:t>T1</hp:t>", begin, 1)
            .replacen("<hp:t>T3</hp:t>", end, 1)
    });
    let toc = names(&["차례"]);
    assert_eq!(core.control_codes_native(&toc).len(), 1);
    assert_eq!(core.erase_control_codes_native(&toc, None).unwrap(), 1);
    let clean = |core: &DocumentCore| {
        core.document().sections[0].paragraphs.iter().all(|p| {
            p.orphan_field_ends.is_empty()
                && !p.controls.iter().any(|c| matches!(c, Control::Field(_)))
        })
    };
    assert_eq!(texts(&core), ["T1", "T2", "T3"]);
    assert!(clean(&core));
    for bytes in [
        core.export_hwpx_native().unwrap(),
        core.export_hwp_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(texts(&reopened), ["T1", "T2", "T3"]);
        assert!(clean(&reopened));
    }
}
