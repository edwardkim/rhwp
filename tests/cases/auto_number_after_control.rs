//! 다른 컨트롤 바로 뒤 자동 번호는 HWP 로 저장해도 자기 자리표 글자에 남는다.
//!
//! `앞[각주][번호]뒤` 를 HWP 로 저장해 다시 열면 `앞 뒤 ` 가 된다. 번호 자리표가 그냥 공백
//! 글자로 써지고 번호는 문단 끝에 붙는다. 구역 첫 문단 맨 앞 번호도 구역·단 정의 뒤라 같다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

/// 본문 자동 번호 컨트롤. 문단 글에는 자리표 공백 한 글자로 들어간다.
const AUTO_NUMBER: &str = concat!(
    r#"<hp:ctrl><hp:autoNum num="1" numType="PAGE"><hp:autoNumFormat type="DIGIT" "#,
    r#"userChar="" prefixChar="" suffixChar="" supscript="0"/></hp:autoNum></hp:ctrl>"#
);

/// 문단의 글과, 각주·미주·그림·자동 번호의 (종류, 글자 위치).
type Content = (String, Vec<(&'static str, usize)>);

fn content(p: &Paragraph) -> Content {
    let controls = p
        .controls
        .iter()
        .zip(p.control_text_positions())
        .filter_map(|(control, at)| {
            Some(match control {
                Control::Footnote(_) => ("각주", at),
                Control::Endnote(_) => ("미주", at),
                Control::Picture(_) => ("그림", at),
                Control::AutoNumber(_) => ("번호", at),
                _ => return None,
            })
        });
    (p.text.clone(), controls.collect())
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

/// 본문 문단 `para` 가 `want` 인지 편집 중 문서와, HWP·HWPX 로 저장했다 다시 연 문서에서 본다.
fn check(broken: &mut Vec<String>, label: &str, core: &DocumentCore, para: usize, want: &Content) {
    let hwp = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    for (state, doc) in [("편집 중", core), ("HWP 저장", &hwp), ("HWPX 저장", &hwpx)] {
        let got = content(&doc.document().sections[0].paragraphs[para]);
        if &got != want {
            broken.push(format!("{label} → {state}: {got:?}"));
        }
    }
}

/// "첫 문단" 뒤에 "앞뒤" 문단을 둔 문서.
fn two_paragraphs() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫 문단").unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.insert_text_native(0, 1, 0, "앞뒤").unwrap();
    core
}

#[test]
fn auto_number_right_after_object_keeps_its_placeholder() {
    let picture = |core: &mut DocumentCore| {
        let png = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/logo/logo-16.png"
        ))
        .unwrap();
        let result = core
            .insert_picture_native(
                0,
                1,
                1,
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
        core.set_picture_properties_native(0, 1, control, r#"{"treatAsChar":true}"#)
            .unwrap();
    };
    let cases: [(&str, &str, &dyn Fn(&mut DocumentCore)); 3] = [
        ("각주", "</hp:footNote></hp:ctrl>", &|core| {
            core.insert_footnote_native(0, 1, 1).unwrap();
        }),
        ("미주", "</hp:endNote></hp:ctrl>", &|core| {
            core.insert_endnote_native(0, 1, 1).unwrap();
        }),
        ("그림", "</hp:pic>", &picture),
    ];

    let mut broken = Vec::new();
    for (kind, object_end, put) in cases {
        // "앞[개체][번호]뒤": 번호는 개체 바로 뒤 자리표 공백에 있다.
        let mut core = two_paragraphs();
        put(&mut core);
        let core = rewrite_section(&core, |xml| {
            xml.replacen(object_end, &format!("{object_end}{AUTO_NUMBER}"), 1)
        });
        let want = ("앞 뒤".to_string(), vec![(kind, 1), ("번호", 1)]);
        check(&mut broken, kind, &core, 1, &want);
    }
    assert!(
        broken.is_empty(),
        "개체 바로 뒤 자동 번호가 자리표 글자를 잃은 경우: {broken:#?}"
    );
}

#[test]
fn auto_number_opening_section_keeps_its_placeholder() {
    // 구역 첫 문단 " 첫 문단": 번호 자리표 앞 갭에 구역·단 정의가 있다.
    let core = rewrite_section(&two_paragraphs(), |xml| {
        xml.replacen(
            "<hp:t>첫 문단</hp:t>",
            &format!("{AUTO_NUMBER}<hp:t>첫 문단</hp:t>"),
            1,
        )
    });
    let mut broken = Vec::new();
    let want = (" 첫 문단".to_string(), vec![("번호", 0)]);
    check(&mut broken, "구역 첫 문단", &core, 0, &want);
    assert!(
        broken.is_empty(),
        "구역·단 정의 뒤 자동 번호가 자리표 글자를 잃은 경우: {broken:#?}"
    );
}
