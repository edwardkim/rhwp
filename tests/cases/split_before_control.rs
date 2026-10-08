//! 개체 바로 앞에서 나눈 문단은 그 개체로 시작한다.
//!
//! `앞[각주]뒤` 를 각주 바로 앞 캐럿에서 나누면 `앞` | `[각주]뒤` 다. 새 문단 첫 글자 앞에
//! 개체 자리를 남기지 않으면 개체가 새 문단 끝으로 밀려 `뒤[각주]` 가 되고, 저장본도 그렇게
//! 남는다. 합치기를 되돌려도, 개체부터 복사해 붙여도 같다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

#[derive(Clone, Copy, Debug)]
enum Obj {
    Footnote,
    Endnote,
    InlinePicture,
    InlineRect,
    FloatRect,
}

const OBJS: [Obj; 5] = [
    Obj::Footnote,
    Obj::Endnote,
    Obj::InlinePicture,
    Obj::InlineRect,
    Obj::FloatRect,
];

impl Obj {
    fn kind(self) -> &'static str {
        match self {
            Obj::Footnote => "각주",
            Obj::Endnote => "미주",
            Obj::InlinePicture => "그림",
            Obj::InlineRect | Obj::FloatRect => "도형",
        }
    }
}

/// 본문 문단 `para` 의 `at` 번째 글자 앞에 개체를 둔다.
fn put(core: &mut DocumentCore, para: usize, obj: Obj, at: usize) {
    match obj {
        Obj::Footnote => {
            core.insert_footnote_native(0, para, at).unwrap();
        }
        Obj::Endnote => {
            core.insert_endnote_native(0, para, at).unwrap();
        }
        Obj::InlinePicture => {
            let png = std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/logo/logo-16.png"
            ))
            .unwrap();
            let result = core
                .insert_picture_native(
                    0,
                    para,
                    at,
                    &[],
                    &png,
                    1200,
                    1200,
                    16,
                    16,
                    "png",
                    "그림",
                    None,
                    None,
                )
                .unwrap();
            let result: serde_json::Value = serde_json::from_str(&result).unwrap();
            let control = result["controlIdx"].as_u64().unwrap() as usize;
            core.set_picture_properties_native(0, para, control, r#"{"treatAsChar":true}"#)
                .unwrap();
        }
        Obj::InlineRect | Obj::FloatRect => {
            core.create_shape_control_native(
                0,
                para,
                at,
                3000,
                3000,
                20000,
                15000,
                matches!(obj, Obj::InlineRect),
                "InFrontOfText",
                "rectangle",
                false,
                false,
                &[],
            )
            .unwrap();
        }
    }
}

/// 문단의 글과, 개체·각주·미주·자동 번호·제목 차례 표시의 (종류, 글자 위치).
type Content = (String, Vec<(&'static str, usize)>);

fn content(p: &Paragraph) -> Content {
    let controls = p
        .controls
        .iter()
        .zip(p.control_text_positions())
        .filter_map(|(control, at)| {
            Some(match control {
                Control::Shape(_) => ("도형", at),
                Control::Picture(_) => ("그림", at),
                Control::Footnote(_) => ("각주", at),
                Control::Endnote(_) => ("미주", at),
                Control::AutoNumber(_) => ("번호", at),
                _ => return None,
            })
        });
    let marks = p.title_marks.iter().map(|m| ("차례", m.char_idx));
    (p.text.clone(), controls.chain(marks).collect())
}

type Objects<'a> = &'a [(&'static str, usize)];

fn pair((a, a_objects): (&str, Objects), (b, b_objects): (&str, Objects)) -> [Content; 2] {
    [
        (a.to_string(), a_objects.to_vec()),
        (b.to_string(), b_objects.to_vec()),
    ]
}

fn body(core: &DocumentCore, para: usize) -> &Paragraph {
    &core.document().sections[0].paragraphs[para]
}

/// 문단 1·2 가 `want` 인지 편집 중 문서와, HWP·HWPX 로 저장했다 다시 연 문서에서 본다.
fn check(broken: &mut Vec<String>, label: String, core: &DocumentCore, want: &[Content; 2]) {
    let hwp = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    for (state, doc) in [("편집 중", core), ("HWP 저장", &hwp), ("HWPX 저장", &hwpx)] {
        let got = [content(body(doc, 1)), content(body(doc, 2))];
        if &got != want {
            broken.push(format!("{label} → {state}: {got:?}"));
        }
    }
}

/// 문서 그대로와, HWP·HWPX 로 저장했다 다시 연 문서.
fn variants(core: DocumentCore) -> Vec<(&'static str, DocumentCore)> {
    let hwp = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    vec![("원본", core), ("HWP 저장본", hwp), ("HWPX 저장본", hwpx)]
}

/// 첫 문단 뒤에 `texts` 문단들을 둔 문서. 구역 정의를 지는 첫 문단은 피한다.
fn paragraphs(texts: &[&str]) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫 문단").unwrap();
    let mut end = 4;
    for (i, text) in texts.iter().enumerate() {
        core.split_paragraph_native(0, i, end, None).unwrap();
        if !text.is_empty() {
            core.insert_text_native(0, i + 1, 0, text).unwrap();
        }
        end = text.chars().count();
    }
    core
}

/// 문단 1 이 "앞[개체]뒤" 이고 문단 2 가 빈 문서.
fn object_between(obj: Obj) -> DocumentCore {
    let mut core = paragraphs(&["앞뒤", ""]);
    put(&mut core, 1, obj, 1);
    core
}

#[test]
fn enter_right_before_object_starts_new_paragraph_with_it() {
    let mut broken = Vec::new();
    for obj in OBJS {
        for (source, mut core) in variants(object_between(obj)) {
            assert_eq!(
                content(body(&core, 1)),
                ("앞뒤".to_string(), vec![(obj.kind(), 1)]),
                "{obj:?} {source}"
            );
            core.split_paragraph_native(0, 1, 1, None).unwrap();
            let want = pair(("앞", &[]), ("뒤", &[(obj.kind(), 0)]));
            check(
                &mut broken,
                format!("{obj:?} {source} Enter 1"),
                &core,
                &want,
            );
        }
    }
    assert!(
        broken.is_empty(),
        "개체가 새 문단 맨 앞에 서지 않은 경우: {broken:#?}"
    );
}

#[test]
fn enter_right_after_inline_object_keeps_it_in_first_paragraph() {
    let mut broken = Vec::new();
    for obj in [
        Obj::Footnote,
        Obj::Endnote,
        Obj::InlinePicture,
        Obj::InlineRect,
    ] {
        for (source, mut core) in variants(object_between(obj)) {
            core.split_paragraph_native(0, 1, 2, None).unwrap();
            let want = pair(("앞", &[(obj.kind(), 1)]), ("뒤", &[]));
            check(
                &mut broken,
                format!("{obj:?} {source} Enter 2"),
                &core,
                &want,
            );
        }
    }
    assert!(
        broken.is_empty(),
        "개체 뒤 나누기가 달라진 경우: {broken:#?}"
    );
}

#[test]
fn merge_undo_restores_paragraph_that_starts_with_object() {
    let mut broken = Vec::new();
    for obj in OBJS {
        // "앞" 과 "[개체]뒤" 를 합쳤다가, 합친 자리에서 다시 나눈다.
        let mut core = paragraphs(&["앞", "뒤"]);
        put(&mut core, 2, obj, 0);
        let want = pair(("앞", &[]), ("뒤", &[(obj.kind(), 0)]));
        assert_eq!(
            [content(body(&core, 1)), content(body(&core, 2))],
            want,
            "{obj:?}"
        );
        let merged: serde_json::Value =
            serde_json::from_str(&core.merge_paragraph_native(0, 2).unwrap()).unwrap();
        let merge_point = merged["charOffset"].as_u64().unwrap() as usize;
        core.split_paragraph_native(0, 1, merge_point, None)
            .unwrap();
        check(
            &mut broken,
            format!("{obj:?} 합치기 되돌리기"),
            &core,
            &want,
        );
    }
    assert!(
        broken.is_empty(),
        "합치기를 되돌려도 원래 문단이 아닌 경우: {broken:#?}"
    );
}

#[test]
fn copy_starting_at_object_pastes_it_first() {
    let mut broken = Vec::new();
    for obj in OBJS {
        // "앞[개체]뒤" 에서 "[개체]뒤" 를 복사해 빈 문단 2 에 붙인다.
        let mut core = object_between(obj);
        core.copy_selection_native(0, 1, 1, 1, 2).unwrap();
        core.paste_internal_native(0, 2, 0).unwrap();
        let want = pair(("앞뒤", &[(obj.kind(), 1)]), ("뒤", &[(obj.kind(), 0)]));
        check(&mut broken, format!("{obj:?} 복사해 붙이기"), &core, &want);
    }
    assert!(
        broken.is_empty(),
        "붙인 문단이 개체로 시작하지 않은 경우: {broken:#?}"
    );
}

/// HWPX 로 저장한 본문 XML 을 고쳐 다시 연다.
fn rewrite_section(core: &DocumentCore, edit: impl Fn(&str) -> String) -> DocumentCore {
    let bytes = core.export_hwpx_native().unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        if entry.name() == "Contents/section0.xml" {
            let xml = String::from_utf8(data).unwrap();
            let edited = edit(&xml);
            assert_ne!(edited, xml, "본문 XML 을 고치지 못했다");
            data = edited.into_bytes();
        }
        output
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        output.write_all(&data).unwrap();
    }
    DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap()
}

/// 본문 자동 번호 컨트롤. 문단 글에는 자리표 공백 한 글자로 들어간다.
const AUTO_NUMBER: &str = concat!(
    r#"<hp:ctrl><hp:autoNum num="1" numType="PAGE"><hp:autoNumFormat type="DIGIT" "#,
    r#"userChar="" prefixChar="" suffixChar="" supscript="0"/></hp:autoNum></hp:ctrl>"#
);

#[test]
fn split_leaves_auto_number_on_its_placeholder_character() {
    let mut broken = Vec::new();

    // "앞[번호]뒤": 번호는 갭이 아니라 자리표 글자에 있다. 번호 앞에서 나누면 새 문단은
    // 자리표 글자로 시작하고, 그 앞에 갭을 두지 않는다.
    let numbered = || {
        rewrite_section(&paragraphs(&["앞뒤", ""]), |xml| {
            let parts = format!("<hp:t>앞</hp:t>{AUTO_NUMBER}<hp:t>뒤</hp:t>");
            xml.replacen("<hp:t>앞뒤</hp:t>", &parts, 1)
        })
    };
    assert_eq!(
        content(body(&numbered(), 1)),
        ("앞 뒤".to_string(), vec![("번호", 1)])
    );
    for (source, mut core) in variants(numbered()) {
        core.split_paragraph_native(0, 1, 1, None).unwrap();
        let want = pair(("앞", &[]), (" 뒤", &[("번호", 0)]));
        check(&mut broken, format!("번호 {source} Enter 1"), &core, &want);
    }

    // "앞[각주][번호]뒤": 둘 사이에서 나누면 각주 자리는 앞 문단에 남고, 각주 앞에서
    // 나누면 각주 자리만 새 문단 첫 글자 앞으로 간다. HWP 저장은 나누기 전부터 각주 바로
    // 뒤 자동 번호를 공백 글자로 쓰므로 편집 중과 HWPX 저장만 본다.
    let footnoted = || {
        rewrite_section(&object_between(Obj::Footnote), |xml| {
            let parts = format!("</hp:footNote></hp:ctrl>{AUTO_NUMBER}");
            xml.replacen("</hp:footNote></hp:ctrl>", &parts, 1)
        })
    };
    assert_eq!(
        content(body(&footnoted(), 1)),
        ("앞 뒤".to_string(), vec![("각주", 1), ("번호", 1)])
    );
    for (caret, want) in [
        (1, pair(("앞", &[]), (" 뒤", &[("각주", 0), ("번호", 0)]))),
        (2, pair(("앞", &[("각주", 1)]), (" 뒤", &[("번호", 0)]))),
    ] {
        let mut core = footnoted();
        core.split_paragraph_native(0, 1, caret, None).unwrap();
        let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
        for (state, doc) in [("편집 중", &core), ("HWPX 저장", &hwpx)] {
            let got = [content(body(doc, 1)), content(body(doc, 2))];
            if got != want {
                broken.push(format!("각주·번호 Enter {caret} → {state}: {got:?}"));
            }
        }
    }

    assert!(
        broken.is_empty(),
        "자동 번호를 자리표 글자에서 떼어 낸 경우: {broken:#?}"
    );
}

#[test]
fn split_moves_title_mark_slot_with_its_character() {
    // 제목 차례 표시는 컨트롤 없이 "뒤" 앞 갭의 한 칸을 차지하고 "뒤" 와 함께 간다. 각주 앞에서
    // 나누면 새 문단 첫 글자 앞 갭은 각주와 표시 두 칸이다.
    let titled = rewrite_section(&object_between(Obj::Footnote), |xml| {
        let marked = r#"<hp:t><hp:titleMark ignore="1"/>뒤</hp:t>"#;
        xml.replacen("<hp:t>뒤</hp:t>", marked, 1)
    });
    assert_eq!(
        content(body(&titled, 1)),
        ("앞뒤".to_string(), vec![("각주", 1), ("차례", 1)])
    );
    let mut broken = Vec::new();
    for (source, mut core) in variants(titled) {
        core.split_paragraph_native(0, 1, 1, None).unwrap();
        let want = pair(("앞", &[]), ("뒤", &[("각주", 0), ("차례", 0)]));
        check(&mut broken, format!("{source} Enter 1"), &core, &want);
    }

    // 개체 없이 표시만 있는 제목 문단 맨 앞에서 나누면 표시는 글과 함께 새 문단으로 간다.
    let heading = rewrite_section(&paragraphs(&["앞뒤", ""]), |xml| {
        let marked = r#"<hp:t><hp:titleMark ignore="1"/>앞뒤</hp:t>"#;
        xml.replacen("<hp:t>앞뒤</hp:t>", marked, 1)
    });
    for (source, mut core) in variants(heading) {
        core.split_paragraph_native(0, 1, 0, None).unwrap();
        let want = pair(("", &[]), ("앞뒤", &[("차례", 0)]));
        check(&mut broken, format!("제목 {source} Enter 0"), &core, &want);
    }

    assert!(
        broken.is_empty(),
        "제목 차례 표시가 자기 글자와 함께 가지 않은 경우: {broken:#?}"
    );
}
