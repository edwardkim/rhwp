//! 구역 나누기·합치기(`splitSection`·`mergeSection`) — 한/글 12.30(Mac) [쪽-구역 나누기]와 구역 경계 지우기.
//!
//! 한/글에서 직접 확인한 동작(빈 문서에 세 문단 "첫째 문단 가나다라"·"둘째 문단 마바사아"·"셋째 문단
//! 자차카타"를 넣고 실행):
//! - 둘째 문단 "둘째 문단 |마바사아"에서 나누면 문단이 나뉘고 "마바사아"가 2쪽 2구역의 첫 문단이 된다.
//!   캐럿은 그 처음(1줄 1칸, 2/2구역)이다. 한/글 저장본은 HWP `BodyText/Section1`, HWPX
//!   `section1.xml`로 구역을 나눠 쓰고, 둘째 구역 첫 문단의 문단 머리 나누기 바이트는 0x03이다.
//! - 문단 처음이면 그 문단부터, 문단 끝이면 다음 문단부터 새 구역이다(빈 문단이 생기지 않는다).
//!   빈 문단은 처음으로 친다. 구역 첫 문단의 처음에서 나누면 앞 구역에 빈 문단이 남고, 문서 끝에서
//!   나누면 새 구역이 빈 문단으로 시작한다.
//! - 가로 용지 구역에서 나누면 새 구역도 가로다. 시작 쪽 번호 5(사용자)·표 시작 번호 3인 구역에서
//!   나누면 새 구역은 둘 다 '이어서'(쪽 번호 6)이고, 홀수 시작·빈 줄 감추기·각주 새로 시작 7은 잇는다.
//! - 새 구역의 처음에서 Backspace(또는 앞 구역 끝에서 Delete)를 누르면 구역과 문단이 함께 합쳐지고,
//!   세로 용지·두 단이던 뒤 구역 글이 앞 구역의 가로 용지·한 단을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const TEXTS: [&str; 3] = [
    "첫째 문단 가나다라",
    "둘째 문단 마바사아",
    "셋째 문단 자차카타",
];

fn three_paragraphs() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    for (para, text) in TEXTS.iter().enumerate() {
        if para > 0 {
            let previous = TEXTS[para - 1].chars().count();
            doc.split_paragraph_native(0, para - 1, previous, None)
                .unwrap();
        }
        doc.insert_text_native(0, para, 0, text).unwrap();
    }
    doc
}

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// 구역마다 문단 글.
fn texts(doc: &HwpDocument) -> Vec<Vec<String>> {
    (0..doc.get_section_count() as usize)
        .map(|section| {
            (0..doc.get_paragraph_count_native(section).unwrap())
                .map(|para| {
                    doc.get_text_range_native(section, para, 0, usize::MAX)
                        .unwrap()
                })
                .collect()
        })
        .collect()
}

fn page_def(doc: &HwpDocument, section: usize) -> Value {
    json(&doc.get_page_def_native(section).unwrap())
}

fn section_def(doc: &HwpDocument, section: usize) -> Value {
    json(&doc.get_section_def_native(section).unwrap())
}

/// 쪽마다 (구역, 쪽 번호).
fn pages(doc: &HwpDocument) -> Vec<(u64, u64)> {
    (0..doc.page_count())
        .map(|page| {
            let info = json(&doc.get_page_info_native(page).unwrap());
            (
                info["sectionIndex"].as_u64().unwrap(),
                info["pageNumber"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn split(doc: &mut HwpDocument, section: usize, para: usize, offset: usize) -> Value {
    json(&doc.split_section_native(section, para, offset).unwrap())
}

fn reopened(doc: &HwpDocument) -> [HwpDocument; 2] {
    [
        HwpDocument::from_bytes(&doc.export_hwp_native().unwrap()).unwrap(),
        HwpDocument::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap(),
    ]
}

#[test]
fn split_in_paragraph_starts_new_section_on_new_page_like_hancom() {
    let mut doc = three_paragraphs();
    doc.set_page_def_native(0, r#"{"landscape":true}"#).unwrap();
    let landscape = page_def(&doc, 0);

    let caret = split(&mut doc, 0, 1, 6);
    assert_eq!(caret["section"], 1);
    assert_eq!(caret["para"], 0);
    assert_eq!(
        texts(&doc),
        vec![
            vec![TEXTS[0].to_string(), "둘째 문단 ".to_string()],
            vec!["마바사아".to_string(), TEXTS[2].to_string()],
        ]
    );
    // 새 구역은 앞 구역의 용지를 잇고 새 쪽에서 시작한다.
    assert_eq!(page_def(&doc, 1), landscape);
    assert_eq!(pages(&doc), vec![(0, 1), (1, 2)]);

    for saved in reopened(&doc) {
        assert_eq!(texts(&saved), texts(&doc));
        assert_eq!(page_def(&saved, 0), landscape);
        assert_eq!(page_def(&saved, 1), landscape);
        assert_eq!(pages(&saved), vec![(0, 1), (1, 2)]);
    }
}

#[test]
fn split_position_decides_paragraph_split_like_hancom() {
    let owned = |rows: &[&[&str]]| -> Vec<Vec<String>> {
        rows.iter()
            .map(|row| row.iter().map(|text| text.to_string()).collect())
            .collect()
    };
    // 문단 처음: 그 문단부터 새 구역이다.
    let mut doc = three_paragraphs();
    split(&mut doc, 0, 1, 0);
    assert_eq!(texts(&doc), owned(&[&[TEXTS[0]], &[TEXTS[1], TEXTS[2]]]));
    // 문단 끝: 다음 문단부터 새 구역이다.
    let mut doc = three_paragraphs();
    split(&mut doc, 0, 0, TEXTS[0].chars().count());
    assert_eq!(texts(&doc), owned(&[&[TEXTS[0]], &[TEXTS[1], TEXTS[2]]]));
    // 문서 끝: 새 구역이 빈 문단으로 시작한다.
    let mut doc = three_paragraphs();
    split(&mut doc, 0, 2, TEXTS[2].chars().count());
    assert_eq!(
        texts(&doc),
        owned(&[&[TEXTS[0], TEXTS[1], TEXTS[2]], &[""]])
    );
    // 구역 첫 문단의 처음: 앞 구역에 빈 문단이 남는다.
    let mut doc = three_paragraphs();
    let caret = split(&mut doc, 0, 0, 0);
    assert_eq!(caret["section"], 1);
    assert_eq!(
        texts(&doc),
        owned(&[&[""], &[TEXTS[0], TEXTS[1], TEXTS[2]]])
    );
    for saved in reopened(&doc) {
        assert_eq!(texts(&saved), texts(&doc));
    }
    // 빈 문단은 처음으로 친다. 둘째 구역 첫 문단의 처음도 같다.
    let mut doc = three_paragraphs();
    doc.split_paragraph_native(0, 1, TEXTS[1].chars().count(), None)
        .unwrap();
    split(&mut doc, 0, 2, 0);
    assert_eq!(
        texts(&doc),
        owned(&[&[TEXTS[0], TEXTS[1]], &["", TEXTS[2]]])
    );
    split(&mut doc, 1, 0, 0);
    assert_eq!(
        texts(&doc),
        owned(&[&[TEXTS[0], TEXTS[1]], &[""], &["", TEXTS[2]]])
    );
    for saved in reopened(&doc) {
        assert_eq!(texts(&saved), texts(&doc));
    }
}

#[test]
fn new_section_continues_start_numbers_and_keeps_other_settings() {
    let mut doc = three_paragraphs();
    doc.set_section_def_native(
        0,
        r#"{"pageNum":5,"tableNum":3,"pictureNum":2,"equationNum":4,"hideMasterPage":true,"defaultTabSpacing":12000}"#,
    )
    .unwrap();
    split(&mut doc, 0, 1, 0);
    let first = section_def(&doc, 0);
    let second = section_def(&doc, 1);
    for key in ["pageNum", "tableNum", "pictureNum", "equationNum"] {
        assert_ne!(first[key], 0, "{key}");
        assert_eq!(second[key], 0, "{key}");
    }
    for key in ["hideMasterPage", "defaultTabSpacing", "pageNumType"] {
        assert_eq!(second[key], first[key], "{key}");
    }
    for saved in reopened(&doc) {
        assert_eq!(section_def(&saved, 1)["pageNum"], 0);
        assert_eq!(section_def(&saved, 1)["defaultTabSpacing"], 12000);
    }
}

#[test]
fn new_section_keeps_columns_at_split_point() {
    let mut doc = three_paragraphs();
    doc.set_column_def_native(0, 2, 0, true, 2268).unwrap();
    split(&mut doc, 0, 1, 3);
    let columns = json(&doc.get_column_def(1).unwrap());
    assert_eq!(columns["columnCount"], 2);
    assert_eq!(columns["spacing"], 2268);
    for saved in reopened(&doc) {
        assert_eq!(json(&saved.get_column_def(1).unwrap())["columnCount"], 2);
    }
}

#[test]
fn merged_section_follows_previous_section_shape() {
    let mut doc = three_paragraphs();
    doc.set_page_def_native(0, r#"{"landscape":true}"#).unwrap();
    let landscape = page_def(&doc, 0);
    split(&mut doc, 0, 1, 6);
    doc.set_page_def_native(1, r#"{"landscape":false}"#)
        .unwrap();
    doc.set_column_def_native(1, 2, 0, true, 2268).unwrap();
    assert_ne!(page_def(&doc, 1), landscape);

    let merged = json(&doc.merge_section_native(1).unwrap());
    assert_eq!(merged["section"], 0);
    assert_eq!(merged["para"], 2);
    assert_eq!(doc.get_section_count(), 1);
    assert_eq!(
        texts(&doc),
        vec![vec![
            TEXTS[0].to_string(),
            "둘째 문단 ".to_string(),
            "마바사아".to_string(),
            TEXTS[2].to_string(),
        ]]
    );
    assert_eq!(page_def(&doc, 0), landscape);
    assert_eq!(json(&doc.get_column_def(0).unwrap())["columnCount"], 1);
    assert_eq!(pages(&doc), vec![(0, 1)]);

    // 한/글의 Backspace처럼 이어서 문단을 잇는다.
    doc.merge_paragraph_native(0, 2).unwrap();
    assert_eq!(texts(&doc)[0][1], TEXTS[1]);
    for saved in reopened(&doc) {
        assert_eq!(saved.get_section_count(), 1);
        assert_eq!(texts(&saved), texts(&doc));
        assert_eq!(page_def(&saved, 0), landscape);
        assert_eq!(pages(&saved), vec![(0, 1)]);
    }
}

#[test]
fn merging_hancom_sections_keeps_every_paragraph() {
    let bytes = std::fs::read("samples/2026_oss_rst.hwp").unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    let sections = doc.get_section_count() as usize;
    assert!(sections >= 2);
    let before: Vec<String> = texts(&doc).into_iter().flatten().collect();
    let first_paragraphs = doc.get_paragraph_count_native(0).unwrap();
    let page_def_0 = page_def(&doc, 0);

    let merged = json(&doc.merge_section_native(1).unwrap());
    assert_eq!(merged["para"].as_u64().unwrap() as usize, first_paragraphs);
    assert_eq!(doc.get_section_count() as usize, sections - 1);
    assert_eq!(
        texts(&doc).into_iter().flatten().collect::<Vec<_>>(),
        before
    );
    assert_eq!(page_def(&doc, 0), page_def_0);
    for saved in reopened(&doc) {
        assert_eq!(saved.get_section_count() as usize, sections - 1);
        assert_eq!(
            texts(&saved).into_iter().flatten().collect::<Vec<_>>(),
            before
        );
    }

    // 다시 나누면 구역 수가 돌아온다.
    split(&mut doc, 0, first_paragraphs, 0);
    assert_eq!(doc.get_section_count() as usize, sections);
    assert_eq!(
        texts(&doc).into_iter().flatten().collect::<Vec<_>>(),
        before
    );
}

/// 한/글 12.30이 나눈 문서(1구역 가로: "첫째 문단 가나다라"·"둘째 문단 " / 2구역 세로: "마바사아"·
/// "셋째 문단 자차카타"). 저장 줄 위치는 구역마다 0에서 시작한다.
fn hancom_two_sections() -> [HwpDocument; 2] {
    ["hwp", "hwpx"].map(|ext| {
        let path = format!("tests/fixtures/section_split/hancom-two-sections.{ext}");
        HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
    })
}

fn caret(doc: &HwpDocument, section: usize, para: usize) -> (u64, f64) {
    let rect = json(&doc.get_cursor_rect_native(section, para, 0).unwrap());
    (
        rect["pageIndex"].as_u64().unwrap(),
        rect["y"].as_f64().unwrap(),
    )
}

#[test]
fn hancom_sections_merge_into_one_page_and_split_back_to_page_top() {
    for mut doc in hancom_two_sections() {
        assert_eq!(
            texts(&doc),
            vec![
                vec![TEXTS[0].to_string(), "둘째 문단 ".to_string()],
                vec!["마바사아".to_string(), TEXTS[2].to_string()],
            ]
        );
        let landscape = page_def(&doc, 0);
        assert_eq!(landscape["landscape"], true);
        assert_eq!(page_def(&doc, 1)["landscape"], false);
        let top = caret(&doc, 0, 0).1;

        // 2구역 처음의 Backspace: 구역을 합치고 두 문단을 잇는다.
        let merged = json(&doc.merge_section_native(1).unwrap());
        doc.merge_paragraph_native(0, merged["para"].as_u64().unwrap() as usize)
            .unwrap();
        assert_eq!(doc.get_section_count(), 1);
        assert_eq!(texts(&doc), vec![TEXTS.map(String::from).to_vec()]);
        assert_eq!(page_def(&doc, 0), landscape);
        // 뒤 구역 글은 새 쪽을 열지 않고 앞 글에 이어진다.
        assert_eq!(pages(&doc), vec![(0, 1)]);
        let (page, y) = caret(&doc, 0, 2);
        assert_eq!(page, 0);
        assert!(y > caret(&doc, 0, 1).1);

        // 셋째 문단 처음에서 다시 나누면 새 쪽 맨 위에서 시작한다.
        split(&mut doc, 0, 2, 0);
        assert_eq!(pages(&doc), vec![(0, 1), (1, 2)]);
        assert_eq!(page_def(&doc, 1), landscape);
        let (page, y) = caret(&doc, 1, 0);
        assert_eq!(page, 1);
        assert!(
            (y - top).abs() < 0.5,
            "새 구역 첫 줄 y {y}, 1쪽 첫 줄 y {top}"
        );
        for saved in reopened(&doc) {
            assert_eq!(texts(&saved), texts(&doc));
            assert_eq!(pages(&saved), vec![(0, 1), (1, 2)]);
            assert!((caret(&saved, 1, 0).1 - top).abs() < 0.5);
        }
    }
}

#[test]
fn section_with_raw_memo_list_is_refused_without_change() {
    let bytes = std::fs::read("samples/issue5866/memo_field_hwp5.hwp").unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    let before = texts(&doc);
    assert!(doc.split_section_native(0, 0, 1).is_err());
    assert_eq!(texts(&doc), before);
    assert!(doc.merge_section_native(1).is_err());
}

/// HWPX 저장본 `section{n}.xml`의 본문 문단마다 첫 줄의 저장 세로 위치(`vertpos`). 표·글상자 안
/// 문단(`hp:subList` 안)은 건너뛴다. 줄 캐시를 생략한 문단(편집한 문단)은 `None`이다.
fn saved_first_line_vpos(doc: &HwpDocument, section: usize) -> Vec<Option<i64>> {
    let bytes = doc.export_hwpx_native().unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(
        &mut zip
            .by_name(&format!("Contents/section{section}.xml"))
            .unwrap(),
        &mut xml,
    )
    .unwrap();
    let mut body = Vec::new();
    let mut depth = 0i32;
    for (index, part) in xml.split("<hp:p ").enumerate() {
        if index > 0 && depth == 0 {
            // 하위 목록 앞까지가 이 문단 자신의 부분이다.
            let head = &part[..part.find("<hp:subList").unwrap_or(part.len())];
            body.push(head.find("vertpos=\"").and_then(|at| {
                let value = &head[at + "vertpos=\"".len()..];
                value[..value.find('"')?].parse().ok()
            }));
        }
        depth += part.matches("<hp:subList").count() as i32;
        depth -= part.matches("</hp:subList>").count() as i32;
    }
    body
}

#[test]
fn saved_lines_of_new_section_start_at_page_top() {
    // 한/글 저장본처럼 구역마다 저장 줄 위치는 쪽 맨 위 0에서 시작한다.
    let bytes = std::fs::read("samples/2026_oss_rst.hwp").unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    // 2쪽 가운데의 문단 5에서 나눈다. 바로 뒤 문단 6은 같은 쪽의 저장 위치를 지닌다.
    assert_eq!(caret(&doc, 0, 5).0, 1);
    let stored = saved_first_line_vpos(&doc, 0);
    assert_eq!(stored.len(), doc.get_paragraph_count_native(0).unwrap());
    let (first, second) = (stored[5].unwrap(), stored[6].unwrap());
    assert!(first > 0 && second > first);
    split(&mut doc, 0, 5, 0);
    let lines = saved_first_line_vpos(&doc, 1);
    assert!(lines[0].is_none_or(|top| top == 0), "{lines:?}");
    assert_eq!(lines[1], Some(second - first), "{lines:?}");
}

#[test]
fn new_section_keeps_master_pages_through_save() {
    // 한/글은 새 구역에도 앞 구역의 바탕쪽을 잇는다(도움말 `format/break/section_break.htm`).
    for path in [
        "samples/master-page-section-carry.hwpx",
        "samples/hwpx/exam-kor-1p.hwpx",
    ] {
        let mut doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        let sections = doc.get_section_count() as usize;
        let last = sections - 1;
        let para = doc.get_paragraph_count_native(last).unwrap() - 1;
        split(&mut doc, last, para, 0);
        assert_eq!(doc.get_section_count() as usize, sections + 1, "{path}");
        for saved in reopened(&doc) {
            assert_eq!(saved.get_section_count() as usize, sections + 1, "{path}");
            assert_eq!(texts(&saved), texts(&doc), "{path}");
            assert_eq!(pages(&saved), pages(&doc), "{path}");
        }
    }
}

/// HWPX `bytes`의 `section0.xml`에서 본문 문단 `para`의 저장 줄(`hp:linesegarray`)을 지운다.
/// rhwp가 고친 문단을 저장 줄 없이 쓴 HWPX처럼, 다시 열면 구역 줄 위치를 다시 계산한다.
fn without_saved_lines(bytes: &[u8], para: usize) -> Vec<u8> {
    use std::io::{Read, Write};
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..zip.len() {
        let mut file = zip.by_index(index).unwrap();
        let name = file.name().unwrap().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).unwrap();
            let mut parts: Vec<String> = xml.split("<hp:p ").map(String::from).collect();
            let mut depth = 0i32;
            let mut body = 0usize;
            for part in parts.iter_mut().skip(1) {
                if depth == 0 {
                    if body == para {
                        let start = part.find("<hp:linesegarray>").unwrap();
                        let end =
                            part.find("</hp:linesegarray>").unwrap() + "</hp:linesegarray>".len();
                        part.replace_range(start..end, "");
                    }
                    body += 1;
                }
                depth += part.matches("<hp:subList").count() as i32;
                depth -= part.matches("</hp:subList>").count() as i32;
            }
            data = parts.join("<hp:p ").into_bytes();
        }
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        out.start_file(name, options).unwrap();
        out.write_all(&data).unwrap();
    }
    out.finish().unwrap().into_inner()
}

#[test]
fn split_moves_file_line_positions_of_recomputed_hwpx_like_hancom_pages() {
    // 저장 줄 없는 문단이 있는 HWPX는 열 때 구역 줄 위치를 다시 계산하고(메모리는 구역 누적일 수 있다)
    // 파일 좌표는 따로 둔다. 나눈 뒤 저장하는 좌표도 새 쪽 맨 위부터 쪽 안 좌표여야 한다.
    let hwp = HwpDocument::from_bytes(&std::fs::read("samples/2026_oss_rst.hwp").unwrap()).unwrap();
    let lined = (1..5)
        .find(|&para| saved_first_line_vpos(&hwp, 0)[para].is_some())
        .unwrap();
    let source = without_saved_lines(&hwp.export_hwpx_native().unwrap(), lined);
    let mut doc = HwpDocument::from_bytes(&source).unwrap();
    let stored = saved_first_line_vpos(&doc, 0);
    assert!(stored[lined].is_none());
    let (first, second) = (stored[5].unwrap(), stored[6].unwrap());
    assert!(first > 0 && second > first);
    split(&mut doc, 0, 5, 0);
    let lines = saved_first_line_vpos(&doc, 1);
    assert!(lines[0].is_none_or(|top| top == 0), "{lines:?}");
    assert_eq!(lines[1], Some(second - first), "{lines:?}");
    assert!(lines.iter().flatten().all(|&vpos| vpos >= 0), "{lines:?}");
}

#[test]
fn split_before_paragraph_running_onto_next_page_stacks_its_lines_from_page_top() {
    // SO-SUEOP.hwpx 본문 문단 179는 쪽 끝(62660)에서 시작해 다음 쪽 맨 위(0)로 이어진다.
    let mut doc =
        HwpDocument::from_bytes(&std::fs::read("samples/SO-SUEOP.hwpx").unwrap()).unwrap();
    let before = saved_first_line_vpos(&doc, 0);
    assert_eq!(before[179], Some(62660));
    split(&mut doc, 0, 179, 0);
    let lines = saved_first_line_vpos(&doc, 1);
    assert!(lines[0].is_none_or(|top| top == 0), "{:?}", &lines[..3]);
    // 새 쪽의 다음 줄은 앞 문단 두 줄 뒤에 온다. 음수나 옛 쪽 끝 좌표가 남지 않는다.
    let next = lines[1..].iter().flatten().next().copied().unwrap();
    assert!(next > 0 && next < 62660, "{:?}", &lines[..3]);
    assert!(lines.iter().flatten().all(|&vpos| vpos >= 0));
    for saved in reopened(&doc) {
        assert_eq!(saved.get_section_count(), doc.get_section_count());
        assert_eq!(texts(&saved), texts(&doc));
    }
}

#[test]
fn merged_text_is_rewrapped_to_previous_section_width() {
    // 세로 구역의 여러 줄 문단을 가로 구역에 합치면 가로 폭으로 다시 접힌다.
    let long = "가나다라마바사아자차카타파하 ".repeat(12);
    let lines_of = |doc: &HwpDocument, section: usize, para: usize, chars: usize| {
        let first = json(&doc.get_cursor_rect_native(section, para, 0).unwrap());
        let last = json(&doc.get_cursor_rect_native(section, para, chars).unwrap());
        last["y"].as_f64().unwrap() - first["y"].as_f64().unwrap()
    };
    let chars = long.chars().count();
    let mut wide = three_paragraphs();
    wide.set_page_def_native(0, r#"{"landscape":true}"#)
        .unwrap();
    wide.insert_text_native(0, 2, 0, &long).unwrap();
    let expected = lines_of(&wide, 0, 2, chars);

    let mut doc = three_paragraphs();
    doc.set_page_def_native(0, r#"{"landscape":true}"#).unwrap();
    split(&mut doc, 0, 2, 0);
    doc.set_page_def_native(1, r#"{"landscape":false}"#)
        .unwrap();
    doc.insert_text_native(1, 0, 0, &long).unwrap();
    let narrow = lines_of(&doc, 1, 0, chars);
    assert!(narrow > expected, "세로 {narrow}, 가로 {expected}");
    doc.merge_section_native(1).unwrap();
    assert_eq!(lines_of(&doc, 0, 2, chars), expected);
    for saved in reopened(&doc) {
        assert_eq!(lines_of(&saved, 0, 2, chars), expected);
    }
}

#[test]
fn paragraph_already_holding_a_section_definition_becomes_the_new_section() {
    // 한 구역 안에 둘째 구역 정의가 든 문서(옛 breakAtCursor 결과·HWPX 한 파일의 여러 secPr)에서
    // 그 문단 처음을 나누면 정의를 겹쳐 넣지 않고 그 정의로 새 구역을 연다.
    let mut doc = three_paragraphs();
    doc.set_page_def_native(0, r#"{"landscape":true}"#).unwrap();
    let mut probe = three_paragraphs();
    probe
        .set_page_def_native(0, r#"{"landscape":true}"#)
        .unwrap();
    split(&mut probe, 0, 2, 0);
    probe
        .set_page_def_native(1, r#"{"landscape":false}"#)
        .unwrap();
    // 둘째 구역을 첫 구역의 문단으로 붙인 문서를 만든다(구역 정의는 그 문단에 남는다).
    let saved = probe.export_hwpx_native().unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(saved)).unwrap();
    let mut read = |name: &str| {
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut zip.by_name(name).unwrap(), &mut xml).unwrap();
        xml
    };
    let first = read("Contents/section0.xml");
    let second = read("Contents/section1.xml");
    let body = |xml: &str| {
        let start = xml.find("<hp:p ").unwrap();
        let end = xml.rfind("</hs:sec>").unwrap();
        xml[start..end].to_string()
    };
    let joined = first.replace("</hs:sec>", &format!("{}</hs:sec>", body(&second)));
    let _ = &mut doc;
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(probe.export_hwpx_native().unwrap())).unwrap();
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..zip.len() {
        use std::io::{Read, Write};
        let mut file = zip.by_index(index).unwrap();
        let name = file.name().unwrap().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        if name == "Contents/section1.xml" {
            continue;
        }
        if name == "Contents/section0.xml" {
            data = joined.clone().into_bytes();
        }
        if name == "Contents/content.hpf" || name == "Contents/header.xml" {
            data = String::from_utf8(data)
                .unwrap()
                .replace(r#"<opf:item id="section1" href="Contents/section1.xml" media-type="application/xml"/>"#, "")
                .replace(r#"<opf:itemref idref="section1" linear="yes"/>"#, "")
                .replace(r#"secCnt="2""#, r#"secCnt="1""#)
                .into_bytes();
        }
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        out.start_file(name, options).unwrap();
        out.write_all(&data).unwrap();
    }
    let joined_doc = out.finish().unwrap().into_inner();
    let mut doc = HwpDocument::from_bytes(&joined_doc).unwrap();
    assert_eq!(doc.get_section_count(), 1);
    assert_eq!(doc.get_paragraph_count_native(0).unwrap(), 3);
    split(&mut doc, 0, 2, 0);
    assert_eq!(doc.get_section_count(), 2);
    assert_eq!(page_def(&doc, 1)["landscape"], false);
    for saved in reopened(&doc) {
        assert_eq!(saved.get_section_count(), 2);
        assert_eq!(page_def(&saved, 0)["landscape"], true);
        assert_eq!(page_def(&saved, 1)["landscape"], false);
        assert_eq!(texts(&saved), texts(&doc));
    }
}

#[test]
fn caret_past_the_end_of_an_empty_paragraph_counts_as_its_start() {
    let mut doc = three_paragraphs();
    doc.split_paragraph_native(0, 1, TEXTS[1].chars().count(), None)
        .unwrap();
    split(&mut doc, 0, 2, 5);
    assert_eq!(texts(&doc)[1], vec!["".to_string(), TEXTS[2].to_string()]);
}

#[test]
fn paragraph_start_with_page_break_leaves_the_break_in_previous_section() {
    // 한/글 12.30: 둘째 문단 처음에 쪽 나누기를 넣고(2쪽) 그 자리에서 구역을 나누면 3쪽이 된다.
    // 쪽 나누기를 지닌 빈 문단이 1구역 2쪽에 남고, 둘째 문단은 3쪽 2구역에서 시작한다.
    // 엔진의 쪽 나누기는 문단 처음에서도 문단을 나눠 둘째 문단(2)이 나누기를 진다.
    let mut doc = three_paragraphs();
    doc.insert_page_break_native(0, 1, 0).unwrap();
    assert_eq!(texts(&doc)[0][2], TEXTS[1]);
    assert_eq!(pages(&doc), vec![(0, 1), (0, 2)]);
    let caret = split(&mut doc, 0, 2, 0);
    assert_eq!(caret["section"], 1);
    assert_eq!(
        texts(&doc),
        vec![
            vec![TEXTS[0].to_string(), String::new(), String::new()],
            vec![TEXTS[1].to_string(), TEXTS[2].to_string()],
        ]
    );
    assert_eq!(pages(&doc), vec![(0, 1), (0, 2), (1, 3)]);
    for saved in reopened(&doc) {
        assert_eq!(texts(&saved), texts(&doc));
        assert_eq!(pages(&saved), vec![(0, 1), (0, 2), (1, 3)]);
    }
}
