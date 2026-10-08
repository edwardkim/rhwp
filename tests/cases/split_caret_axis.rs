//! 문단 나누기는 캐럿 자리에서 나눈다.
//!
//! 캐럿은 글자 하나와, 글자처럼 취급하는 개체·각주·미주 하나마다 한 칸 나아간다. 떠 있는
//! 그림·도형·표는 칸이 없고, 자동 번호는 자리표 글자가 이미 한 칸이다. 문단 나누기가 이들까지
//! 한 칸으로 세면 그 개체 뒤의 Enter·쪽/단 나누기·여러 문단 붙이기·합치기 되돌리기가 개체
//! 하나마다 한 글자씩 앞에서 나눈다.

#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{ColumnBreakType, Paragraph};

const TEXT: &str = "가나다라마";

/// 문단 1 에 `TEXT` 를 둔 문서. 구역 정의를 지는 첫 문단은 피한다.
fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫 문단").unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.insert_text_native(0, 1, 0, TEXT).unwrap();
    core
}

#[derive(Clone, Copy, Debug)]
enum Obj {
    FrontRect,
    SquareRect,
    FloatPicture,
    InlineRect,
    InlinePicture,
    Footnote,
}

impl Obj {
    /// 캐럿이 한 칸 지나가는 개체인가.
    fn inline(self) -> bool {
        matches!(self, Obj::InlineRect | Obj::InlinePicture | Obj::Footnote)
    }
}

/// 문단 1 의 `at` 번째 글자 앞에 개체를 둔다.
fn put(core: &mut DocumentCore, obj: Obj, at: usize) {
    match obj {
        Obj::FrontRect | Obj::SquareRect | Obj::InlineRect => {
            let wrap = if matches!(obj, Obj::SquareRect) {
                "Square"
            } else {
                "InFrontOfText"
            };
            core.create_shape_control_native(
                0,
                1,
                at,
                3000,
                3000,
                20000,
                15000,
                matches!(obj, Obj::InlineRect),
                wrap,
                "rectangle",
                false,
                false,
                &[],
            )
            .unwrap();
        }
        Obj::FloatPicture | Obj::InlinePicture => {
            let png = std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/logo/logo-16.png"
            ))
            .unwrap();
            let result = core
                .insert_picture_native(
                    0,
                    1,
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
            if matches!(obj, Obj::InlinePicture) {
                let result: serde_json::Value = serde_json::from_str(&result).unwrap();
                let control = result["controlIdx"].as_u64().unwrap() as usize;
                core.set_picture_properties_native(0, 1, control, r#"{"treatAsChar":true}"#)
                    .unwrap();
            }
        }
        Obj::Footnote => {
            core.insert_footnote_native(0, 1, at).unwrap();
        }
    }
}

/// 개체·각주·자동 번호의 (종류, 글자 위치). 구역·단 정의는 뺀다.
///
/// 자동 번호의 자리는 자리표 글자다. 자리표가 문단 끝 글자면 엔진은 번호를 글자 길이
/// 자리로 보고하므로(파일에서 연 문단도 같다) 자리표 자리로 맞춘다.
fn objects(p: &Paragraph) -> Vec<(&'static str, usize)> {
    let last = p.text.chars().count().saturating_sub(1);
    p.controls
        .iter()
        .zip(p.control_text_positions())
        .filter_map(|(control, at)| {
            Some(match control {
                Control::Shape(_) => ("도형", at),
                Control::Picture(_) => ("그림", at),
                Control::Table(_) => ("표", at),
                Control::Footnote(_) => ("각주", at),
                Control::AutoNumber(_) => ("번호", at.min(last)),
                _ => return None,
            })
        })
        .collect()
}

/// 캐럿 `caret` 에서 나눌 때 앞 문단에 남는 글자 수와 개체 수.
///
/// `anchors` 는 흐름 순서대로 (글자 위치, 한 칸 개체인가)다. 캐럿은 글자와 한 칸 개체마다
/// 한 칸 나아가고, 캐럿보다 앞에 있는 개체만 앞 문단에 남는다.
fn expected(len: usize, anchors: &[(usize, bool)], caret: usize) -> (usize, usize) {
    let mut at = 0;
    let mut kept = 0;
    for chars in 0..=len {
        while kept < anchors.len() && anchors[kept].0 == chars {
            if at == caret {
                return (chars, kept);
            }
            at += usize::from(anchors[kept].1);
            kept += 1;
        }
        if at == caret {
            return (chars, kept);
        }
        at += 1;
    }
    (len, kept)
}

/// 나눈 두 문단이 캐럿 자리에서 갈렸는지 본다. 앞 문단에 남은 개체는 제자리여야 한다.
fn check_split(
    broken: &mut Vec<String>,
    label: String,
    before: (&str, &[(&'static str, usize)]),
    (chars, kept): (usize, usize),
    got: (&Paragraph, &Paragraph),
) {
    let (text, objs) = before;
    let head: String = text.chars().take(chars).collect();
    let tail: String = text.chars().skip(chars).collect();
    let ok = got.0.text == head
        && got.1.text == tail
        && objects(got.0) == objs[..kept]
        && objects(got.1).len() == objs.len() - kept;
    if !ok {
        broken.push(format!(
            "{label}: {:?}{:?} | {:?}{:?}",
            got.0.text,
            objects(got.0),
            got.1.text,
            objects(got.1)
        ));
    }
}

/// 편집 중 문서와, HWP·HWPX 로 저장했다 다시 연 문서를 나눌 때마다 새로 만든다.
fn variants(
    make: impl Fn() -> DocumentCore + 'static,
) -> Vec<(&'static str, Box<dyn Fn() -> DocumentCore>)> {
    let hwp = make().export_hwp_native().unwrap();
    let hwpx = make().export_hwpx_native().unwrap();
    vec![
        ("편집 중", Box::new(make)),
        (
            "HWP",
            Box::new(move || DocumentCore::from_bytes(&hwp).unwrap()),
        ),
        (
            "HWPX",
            Box::new(move || DocumentCore::from_bytes(&hwpx).unwrap()),
        ),
    ]
}

fn body(core: &DocumentCore, para: usize) -> &Paragraph {
    &core.document().sections[0].paragraphs[para]
}

#[test]
fn enter_after_each_object_kind_splits_at_caret() {
    let mut broken = Vec::new();
    for obj in [
        Obj::FrontRect,
        Obj::SquareRect,
        Obj::FloatPicture,
        Obj::InlineRect,
        Obj::InlinePicture,
        Obj::Footnote,
    ] {
        let make = move || {
            let mut core = blank();
            put(&mut core, obj, 2);
            core
        };
        let objs = objects(body(&make(), 1));
        assert_eq!(objs.len(), 1, "{obj:?}");
        assert_eq!(objs[0].1, 2, "{obj:?}");
        for (format, open) in variants(make) {
            for caret in 0..=5 + usize::from(obj.inline()) {
                let mut core = open();
                core.split_paragraph_native(0, 1, caret, None).unwrap();
                check_split(
                    &mut broken,
                    format!("{obj:?} {format} Enter {caret}"),
                    (TEXT, &objs),
                    expected(5, &[(2, obj.inline())], caret),
                    (body(&core, 1), body(&core, 2)),
                );
            }
        }
    }
    assert!(
        broken.is_empty(),
        "캐럿 자리에서 나누지 않은 경우: {broken:#?}"
    );
}

/// 떠 있는 사각형 둘과 글자처럼 취급하는 그림이 섞인 "[도형]가[그림]나다[도형]라마".
fn mixed() -> DocumentCore {
    let mut core = blank();
    put(&mut core, Obj::FrontRect, 3);
    put(&mut core, Obj::InlinePicture, 1);
    put(&mut core, Obj::FrontRect, 0);
    core
}

#[test]
fn enter_page_and_column_break_after_mixed_anchors_split_at_caret() {
    let objs = objects(body(&mixed(), 1));
    assert_eq!(objs, [("도형", 0), ("그림", 1), ("도형", 3)]);
    let anchors = [(0, false), (1, true), (3, false)];
    let mut broken = Vec::new();
    for (format, open) in variants(mixed) {
        for caret in 0..=6 {
            for (op, column_type) in [
                ("Enter", ColumnBreakType::None),
                ("쪽 나누기", ColumnBreakType::Page),
                ("단 나누기", ColumnBreakType::Column),
            ] {
                let mut core = open();
                match column_type {
                    ColumnBreakType::Page => core.insert_page_break_native(0, 1, caret),
                    ColumnBreakType::Column => core.insert_column_break_native(0, 1, caret),
                    _ => core.split_paragraph_native(0, 1, caret, None),
                }
                .unwrap();
                let label = format!("{format} {op} {caret}");
                if body(&core, 2).column_type != column_type {
                    broken.push(format!("{label}: 나누기 종류"));
                }
                check_split(
                    &mut broken,
                    label,
                    (TEXT, &objs),
                    expected(5, &anchors, caret),
                    (body(&core, 1), body(&core, 2)),
                );
            }
        }
    }
    assert!(
        broken.is_empty(),
        "캐럿 자리에서 나누지 않은 경우: {broken:#?}"
    );
}

#[test]
fn split_after_anchors_survives_save_and_reopen() {
    // "[도형]가[그림]나다[도형]라" | "마" — 개체는 저장본에서도 제자리에 있다.
    let mut core = mixed();
    core.split_paragraph_native(0, 1, 5, None).unwrap();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(body(&reopened, 1).text, "가나다라");
        assert_eq!(
            objects(body(&reopened, 1)),
            [("도형", 0), ("그림", 1), ("도형", 3)]
        );
        assert_eq!(body(&reopened, 2).text, "마");
        assert!(objects(body(&reopened, 2)).is_empty());
    }
}

/// 저장한 HWPX 의 첫 구역 XML 을 고쳐 다시 연다.
fn rewrite_section(core: &DocumentCore, edit: impl Fn(String) -> String) -> DocumentCore {
    let bytes = core.export_hwpx_native().unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        if entry.name() == "Contents/section0.xml" {
            data = edit(String::from_utf8(data).unwrap()).into_bytes();
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

/// `xml` 에서 `open` 으로 시작해 `close` 로 끝나는 첫 조각을 떼어 `TEXT` 의 2번 글자 앞에 넣는다.
fn move_into_text(xml: String, open: &str, close: &str) -> String {
    let start = xml.find(open).unwrap();
    let end = start + xml[start..].find(close).unwrap() + close.len();
    let snippet = xml[start..end].to_string();
    put_into_text(xml.replacen(&snippet, "", 1), &snippet)
}

fn put_into_text(xml: String, snippet: &str) -> String {
    let target = format!("<hp:t>{TEXT}</hp:t>");
    assert_eq!(xml.matches(&target).count(), 1);
    xml.replacen(
        &target,
        &format!("<hp:t>가나</hp:t>{snippet}<hp:t>다라마</hp:t>"),
        1,
    )
}

/// 다른 문단에 만든 떠 있는 표를 문단 1 의 글 가운데로 옮긴 문서.
fn floating_table() -> DocumentCore {
    let mut core = blank();
    core.split_paragraph_native(0, 1, 5, None).unwrap();
    core.create_table_ex_native(0, 2, 0, 1, 2, false, None, None)
        .unwrap();
    rewrite_section(&core, |xml| move_into_text(xml, "<hp:tbl ", "</hp:tbl>"))
}

/// 문단 1 의 글 가운데 쪽 번호 자동 번호를 둔 문서. 글은 자리표 공백을 더해 "가나 다라마"다.
fn auto_number() -> DocumentCore {
    rewrite_section(&blank(), |xml| {
        put_into_text(
            xml,
            r#"<hp:ctrl><hp:autoNum num="1" numType="PAGE"><hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar="" supscript="0"/></hp:autoNum></hp:ctrl>"#,
        )
    })
}

#[test]
fn floating_table_and_auto_number_split_at_caret() {
    let mut broken = Vec::new();
    for (name, make) in [
        ("떠 있는 표", floating_table as fn() -> DocumentCore),
        ("자동 번호", auto_number),
    ] {
        let core = make();
        let text = body(&core, 1).text.clone();
        let objs = objects(body(&core, 1));
        assert_eq!(objs.len(), 1, "{name}");
        assert_eq!(objs[0].1, 2, "{name}");
        let len = text.chars().count();
        for (format, open) in variants(make) {
            for caret in 0..=len {
                let mut core = open();
                core.split_paragraph_native(0, 1, caret, None).unwrap();
                check_split(
                    &mut broken,
                    format!("{name} {format} Enter {caret}"),
                    (&text, &objs),
                    expected(len, &[(2, false)], caret),
                    (body(&core, 1), body(&core, 2)),
                );
            }
        }
    }
    assert!(
        broken.is_empty(),
        "캐럿 자리에서 나누지 않은 경우: {broken:#?}"
    );
}

#[test]
fn enter_right_after_auto_number_keeps_it_in_first_paragraph() {
    // 번호 바로 뒤 Enter: "가나[번호]" | "다라마". 저장했다 다시 열어도 같다.
    let mut core = auto_number();
    core.split_paragraph_native(0, 1, 3, None).unwrap();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(body(&reopened, 1).text, "가나 ");
        assert_eq!(objects(body(&reopened, 1)), [("번호", 2)]);
        assert_eq!(body(&reopened, 2).text, "다라마");
        assert!(objects(body(&reopened, 2)).is_empty());
    }
}

/// 본문 첫 문단의 떠 있는 사각형을, `build` 가 만든 글 `TEXT` 의 2번 글자 앞으로 옮긴 문서.
fn rect_moved_into(build: fn(&mut DocumentCore)) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫 문단").unwrap();
    core.create_shape_control_native(
        0,
        0,
        1,
        3000,
        3000,
        20000,
        15000,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    )
    .unwrap();
    build(&mut core);
    rewrite_section(&core, |xml| move_into_text(xml, "<hp:rect ", "</hp:rect>"))
}

fn with_cell_text(core: &mut DocumentCore) {
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.create_table_ex_native(0, 1, 0, 1, 2, false, None, None)
        .unwrap();
    core.insert_text_in_cell_native(0, 1, 0, 0, 0, 0, TEXT)
        .unwrap();
}

fn with_header_text(core: &mut DocumentCore) {
    core.create_header_footer_native(0, true, 0).unwrap();
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, TEXT)
        .unwrap();
}

fn cell_paragraphs(core: &DocumentCore) -> &[Paragraph] {
    match &body(core, 1).controls[0] {
        Control::Table(table) => &table.cells[0].paragraphs,
        other => panic!("표가 아니다: {other:?}"),
    }
}

fn header_paragraphs(core: &DocumentCore) -> &[Paragraph] {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|control| match control {
            Control::Header(header) => Some(header.paragraphs.as_slice()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn cell_and_header_paragraphs_split_at_caret_after_floating_anchor() {
    let mut broken = Vec::new();
    for in_cell in [true, false] {
        let name = if in_cell { "셀" } else { "머리말" };
        let make = move || {
            rect_moved_into(if in_cell {
                with_cell_text
            } else {
                with_header_text
            })
        };
        let paragraphs = move |core: &DocumentCore| -> Vec<Paragraph> {
            if in_cell {
                cell_paragraphs(core).to_vec()
            } else {
                header_paragraphs(core).to_vec()
            }
        };
        let objs = objects(&paragraphs(&make())[0]);
        assert_eq!(objs, [("도형", 2)], "{name}");
        for (format, open) in variants(make) {
            for caret in 0..=5 {
                let mut core = open();
                if in_cell {
                    core.split_paragraph_in_cell_native(0, 1, 0, 0, 0, caret, None)
                        .unwrap();
                } else {
                    core.split_paragraph_in_header_footer_native(0, true, 0, 0, caret, None)
                        .unwrap();
                }
                let got = paragraphs(&core);
                check_split(
                    &mut broken,
                    format!("{name} {format} Enter {caret}"),
                    (TEXT, &objs),
                    expected(5, &[(2, false)], caret),
                    (&got[0], &got[1]),
                );
            }
        }
    }
    assert!(
        broken.is_empty(),
        "캐럿 자리에서 나누지 않은 경우: {broken:#?}"
    );
}

/// 문단 1 의 각주. 첫 문단은 "[번호] 주석글"이다 — 번호 자리표 공백, 띄움 공백, 글.
fn footnote() -> DocumentCore {
    let mut core = blank();
    core.insert_footnote_native(0, 1, 2).unwrap();
    core.insert_text_in_footnote_native(0, 1, footnote_control(&core), 0, 2, "주석글")
        .unwrap();
    core
}

fn footnote_control(core: &DocumentCore) -> usize {
    body(core, 1)
        .controls
        .iter()
        .position(|c| matches!(c, Control::Footnote(_)))
        .unwrap()
}

fn footnote_paragraphs(core: &DocumentCore) -> &[Paragraph] {
    match &body(core, 1).controls[footnote_control(core)] {
        Control::Footnote(note) => &note.paragraphs,
        _ => unreachable!(),
    }
}

#[test]
fn footnote_paragraph_splits_at_caret_after_its_number() {
    let first = footnote_paragraphs(&footnote())[0].clone();
    assert_eq!(first.text, "  주석글");
    let objs = objects(&first);
    assert_eq!(objs, [("번호", 0)]);
    let mut broken = Vec::new();
    for (format, open) in variants(footnote) {
        for caret in 1..=5 {
            let mut core = open();
            let control = footnote_control(&core);
            core.split_paragraph_in_footnote_native(0, 1, control, 0, caret, None)
                .unwrap();
            let got = footnote_paragraphs(&core);
            check_split(
                &mut broken,
                format!("{format} Enter {caret}"),
                (&first.text, &objs),
                expected(first.text.chars().count(), &[(0, false)], caret),
                (&got[0], &got[1]),
            );
        }
    }
    assert!(
        broken.is_empty(),
        "캐럿 자리에서 나누지 않은 경우: {broken:#?}"
    );
}

#[test]
fn merge_undo_paste_and_copy_after_floating_anchor_use_caret() {
    // 합치기 되돌리기: "[도형]가나다" + "라마" 를 합쳤다가 합친 자리에서 다시 나눈다.
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫 문단").unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.insert_text_native(0, 1, 0, "가나다라마").unwrap();
    core.split_paragraph_native(0, 1, 3, None).unwrap();
    put(&mut core, Obj::FrontRect, 0);
    let merged: serde_json::Value =
        serde_json::from_str(&core.merge_paragraph_native(0, 2).unwrap()).unwrap();
    let merge_point = merged["charOffset"].as_u64().unwrap() as usize;
    core.split_paragraph_native(0, 1, merge_point, None)
        .unwrap();
    assert_eq!(
        (body(&core, 1).text.as_str(), body(&core, 2).text.as_str()),
        ("가나다", "라마")
    );
    assert_eq!(objects(body(&core, 1)), [("도형", 0)]);

    // 여러 문단 HTML 붙이기: 캐럿 3 은 "가나다" 뒤다.
    let mut core = blank();
    put(&mut core, Obj::FrontRect, 2);
    core.paste_html_native(0, 1, 3, "<p>A</p><p>B</p>").unwrap();
    assert_eq!(
        (body(&core, 1).text.as_str(), body(&core, 2).text.as_str()),
        ("가나다A", "B라마")
    );

    // 여러 문단 내부 붙이기: "문단"·"첫" 두 문단을 캐럿 3 에 붙인다.
    let mut core = blank();
    put(&mut core, Obj::FrontRect, 2);
    core.copy_selection_native(0, 0, 2, 1, 1).unwrap();
    core.paste_internal_native(0, 1, 3).unwrap();
    assert_eq!(
        (body(&core, 1).text.as_str(), body(&core, 2).text.as_str()),
        ("가나다문단", "가라마")
    );

    // 복사 범위는 글자 좌표다. 앞의 떠 있는 개체 때문에 한 글자 밀리지 않는다.
    let mut core = blank();
    put(&mut core, Obj::FrontRect, 2);
    let copied: serde_json::Value =
        serde_json::from_str(&core.copy_selection_native(0, 1, 3, 1, 5).unwrap()).unwrap();
    assert_eq!(copied["text"], "라마");
}

fn sample_paragraph(name: &str, section: usize, para: usize) -> Paragraph {
    let bytes = std::fs::read(format!("{}/samples/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    rhwp::parser::parse_document(&bytes).unwrap().sections[section].paragraphs[para].clone()
}

#[test]
fn real_document_end_enter_keeps_last_character() {
    // (파일, 구역, 문단): 앞머리 떠 있는 도형, 미주·수식 뒤의 떠 있는 표·그림, 앞머리 자동 번호.
    for (name, section, para) in [
        ("143E433F503322BD33.hwp", 0, 1),
        ("3-09월_교육_통합_2023.hwp", 0, 258),
        ("3-09월_교육_통합_2023.hwp", 0, 369),
        ("eq-002.hwp", 0, 0),
    ] {
        let source = sample_paragraph(name, section, para);
        let end = source.text.chars().count()
            + source
                .controls
                .iter()
                .filter(|c| c.is_logical_inline())
                .count();
        let last = source.text.chars().last().unwrap().to_string();

        let mut head = source.clone();
        let tail = head.split_at(end);
        assert_eq!(head.text, source.text, "{name} 문단 {para} End");
        assert_eq!(tail.text, "", "{name} 문단 {para} End");

        let mut head = source.clone();
        let tail = head.split_at(end - 1);
        assert_eq!(tail.text, last, "{name} 문단 {para} End-1");
    }
}

#[test]
fn char_overlap_keeps_its_caret_slot() {
    // 글자겹침은 조판·캐럿에서 한 글자 칸이다. "  [겹침] 제3항…" 의 캐럿 3 은 겹침 바로 뒤다.
    let mut head = sample_paragraph("issue1880_takeplace_oracle_p13.hwpx", 1, 46);
    assert!(head
        .controls
        .iter()
        .any(|c| matches!(c, Control::CharOverlap(_))));
    let tail = head.split_at(3);
    assert_eq!(head.text, "  ");
    assert!(matches!(
        head.controls.as_slice(),
        [Control::CharOverlap(_)]
    ));
    assert!(tail.text.starts_with(" 제3항"));
    assert!(tail.controls.is_empty());
}
