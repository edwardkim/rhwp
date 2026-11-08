//! 관계형 및 고정형 회귀: 원본 번호 모양 코드와 표시 문자열의 대응·왕복 보존. 고정 문자열은 표시 의미 계약이며 절대 배치 좌표가 아니다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! note-formats.hwp (8536-note-formats-hwp-2022): 전 19쪽, Native 최저 99.98307%, fresh WASM 최저 99.98279%.
//! note-formats.hwpx (8536-note-formats-hwpx-2022): 전 19쪽, Native 최저 99.98307%, fresh WASM 최저 99.98279%.
//! numbering.hwp (8536-numbering-hwp-2022): 전 6쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! numbering.hwpx (8536-numbering-hwpx-2022): 전 6쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! numbering-wrap.hwp (8536-numbering-wrap-hwp-2022): 전 2쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 번호 모양 코드(문단 번호 표 41, 각주·미주·쪽 번호 표 134)의 그리기와 HWP↔HWPX 이름.
//!
//! 시험 문서는 한/글 12.30(Mac)이 연 뒤 HWP·HWPX로 저장한 것이다.
//! - `numbering.hwp`: 번호 모양마다 문단 번호 정의 하나를 두고 수준 1~5의 시작 번호를
//!   1·9·13·19·25로 정해 수준마다 문단 셋을 쓴다. `numbering-wrap.hwp`는 시작 번호
//!   28·42·56·70·84다. 한/글 인쇄 PDF(`pdf/number_shapes/`)의 번호를 아래 표에 옮겼다.
//!   글자 차례가 있는 모양은 차례를 넘으면 처음으로 돌아간다(⑳ 다음 ①, Z 다음 A,
//!   하 다음 거·허 다음 고처럼 가나다는 모음을 바꿔 84개를 돈다).
//! - `note-formats.hwp`: 구역마다 각주 번호 모양과 쪽 번호 모양을 같은 모양으로 바꾼 19구역.
//!   한/글은 각주 번호(본문 표시·각주 내용)와 쪽 번호를 같은 모양 코드로 그린다
//!   (7쪽 각주 Ⓐ)·쪽 번호 - Ⓖ -, 18쪽 *)·†)·‡)과 - ††††† -).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const DIR: &str = "tests/fixtures/number_shapes";

fn open(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR).join(name);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

/// 쪽의 본문·각주 영역·꼬리말 글자를 공백 없이 모은다.
fn collect(node: &RenderNode, area: usize, out: &mut [String; 3]) {
    let area = match node.node_type {
        RenderNodeType::FootnoteArea => 1,
        RenderNodeType::Footer => 2,
        _ => area,
    };
    match &node.node_type {
        RenderNodeType::TextRun(run) => out[area].extend(
            run.display_or_text()
                .chars()
                .filter(|ch| !ch.is_whitespace()),
        ),
        RenderNodeType::FootnoteMarker(marker) => out[area].push_str(&marker.text),
        _ => {}
    }
    for child in &node.children {
        collect(child, area, out);
    }
}

fn page_text(core: &DocumentCore, page: u32) -> [String; 3] {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = [String::new(), String::new(), String::new()];
    collect(&tree.root, 0, &mut out);
    out
}

fn body_text(core: &DocumentCore) -> String {
    (0..core.page_count())
        .map(|page| page_text(core, page)[0].clone())
        .collect()
}

const STARTS: [u32; 5] = [1, 9, 13, 19, 25];
const WRAP_STARTS: [u32; 5] = [28, 42, 56, 70, 84];

/// 한/글 인쇄 PDF의 번호(수준 1~5, 수준마다 시작 번호부터 셋).
const NUMBERING: [(&str, &str); 14] = [
    ("CIRCLED_LATIN_CAPITAL", "Ⓐ Ⓑ Ⓒ Ⓘ Ⓙ Ⓚ Ⓜ Ⓝ Ⓞ Ⓢ Ⓣ Ⓤ Ⓨ Ⓩ Ⓐ"),
    ("CIRCLED_LATIN_SMALL", "ⓐ ⓑ ⓒ ⓘ ⓙ ⓚ ⓜ ⓝ ⓞ ⓢ ⓣ ⓤ ⓨ ⓩ ⓐ"),
    (
        "CIRCLED_HANGUL_SYLLABLE",
        "㉮ ㉯ ㉰ ㉶ ㉷ ㉸ ㉺ ㉻ ㉮ ㉲ ㉳ ㉴ ㉸ ㉹ ㉺",
    ),
    (
        "CIRCLED_HANGUL_JAMO",
        "㉠ ㉡ ㉢ ㉨ ㉩ ㉪ ㉬ ㉭ ㉠ ㉤ ㉥ ㉦ ㉪ ㉫ ㉬",
    ),
    (
        "CIRCLED_IDEOGRAPH",
        "㊀ ㊁ ㊂ ㊈ ㊉ ㊀ ㊂ ㊃ ㊄ ㊈ ㊉ ㊀ ㊄ ㊅ ㊆",
    ),
    (
        "DECAGON_CIRCLE",
        "갑 을 병 임 계 갑 병 정 무 임 계 갑 무 기 경",
    ),
    (
        "DECAGON_CIRCLE_HANJA",
        "甲 乙 丙 壬 癸 甲 丙 丁 戊 壬 癸 甲 戊 己 庚",
    ),
    (
        "HANGUL_JAMO",
        "ㄱ ㄴ ㄷ ㅈ ㅊ ㅋ ㅍ ㅎ ㄱ ㅁ ㅂ ㅅ ㅋ ㅌ ㅍ",
    ),
    (
        "HANGUL_SYLLABLE",
        "가 나 다 자 차 카 파 하 거 머 버 서 커 터 퍼",
    ),
    (
        "HANGUL_PHONETIC",
        "일 이 삼 구 십 십일 십삼 십사 십오 십구 이십 이십일 이십오 이십육 이십칠",
    ),
    (
        "IDEOGRAPH",
        "一 二 三 九 十 十一 十三 十四 十五 十九 二十 二十一 二十五 二十六 二十七",
    ),
    (
        "ROMAN_CAPITAL",
        "I II III IX X XI XIII XIV XV XIX XX XXI XXV XXVI XXVII",
    ),
    ("LATIN_SMALL", "a b c i j k m n o s t u y z a"),
    ("CIRCLED_DIGIT", "① ② ③ ⑨ ⑩ ⑪ ⑬ ⑭ ⑮ ⑲ ⑳ ① ⑤ ⑥ ⑦"),
];

const NUMBERING_WRAP: [(&str, &str); 5] = [
    (
        "HANGUL_SYLLABLE",
        "허 고 노 호 구 누 후 그 느 흐 기 니 히 가 나",
    ),
    ("LATIN_CAPITAL", "B C D P Q R D E F R S T F G H"),
    ("CIRCLED_DIGIT", "⑧ ⑨ ⑩ ② ③ ④ ⑯ ⑰ ⑱ ⑩ ⑪ ⑫ ④ ⑤ ⑥"),
    (
        "HANGUL_JAMO",
        "ㅎ ㄱ ㄴ ㅎ ㄱ ㄴ ㅎ ㄱ ㄴ ㅎ ㄱ ㄴ ㅎ ㄱ ㄴ",
    ),
    ("CIRCLED_LATIN_SMALL", "ⓑ ⓒ ⓓ ⓟ ⓠ ⓡ ⓓ ⓔ ⓕ ⓡ ⓢ ⓣ ⓕ ⓖ ⓗ"),
];

fn assert_numbering(core: &DocumentCore, starts: [u32; 5], cases: &[(&str, &str)]) {
    let text = body_text(core);
    for (format, numbers) in cases {
        let numbers: Vec<&str> = numbers.split(' ').collect();
        for (level, start) in starts.iter().enumerate() {
            for step in 0..3 {
                let number = numbers[level * 3 + step];
                let line = format!("{number}.{format}수준{}시작{start}+{step}", level + 1);
                assert!(text.contains(&line), "{line:?} 없음");
            }
        }
    }
}

#[test]
fn numbering_draws_every_shape_like_hancom_print() {
    assert_numbering(&open("numbering.hwp"), STARTS, &NUMBERING);
}

#[test]
fn numbering_wraps_each_shape_like_hancom_print() {
    assert_numbering(&open("numbering-wrap.hwp"), WRAP_STARTS, &NUMBERING_WRAP);
}

fn num_formats(hwpx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip");
    let mut header = String::new();
    std::io::Read::read_to_string(
        &mut zip.by_name("Contents/header.xml").expect("header"),
        &mut header,
    )
    .expect("read header");
    header
        .split("<hh:paraHead ")
        .skip(1)
        .filter(|head| head.contains(r#" level="1" "#))
        .map(|head| {
            let start = head.find(r#"numFormat=""#).expect("numFormat") + 11;
            head[start..start + head[start..].find('"').unwrap()].to_string()
        })
        .collect()
}

#[test]
fn numbering_from_hwp_saves_hancom_hwpx_names() {
    let core = open("numbering.hwp");
    let ours = core.export_hwpx_native().expect("export hwpx");
    let hancom = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(DIR)
            .join("numbering.hwpx"),
    )
    .unwrap();
    let names = num_formats(&ours);
    assert_eq!(names, num_formats(&hancom));
    // 다시 열어도 같은 번호를 그린다.
    let reopened = DocumentCore::from_bytes(&ours).expect("reopen");
    assert_numbering(&reopened, STARTS, &NUMBERING);
}

/// 구역(쪽)마다 한/글 인쇄의 본문 각주 표시·각주 내용 번호·쪽 번호.
const NOTE_PAGES: [(&str, &str); 18] = [
    ("1 2 3", "1"),
    ("① ② ③", "②"),
    ("I II III", "III"),
    ("i ii iii", "iv"),
    ("A B C", "E"),
    ("a b c", "f"),
    ("Ⓐ Ⓑ Ⓒ", "Ⓖ"),
    ("ⓐ ⓑ ⓒ", "ⓗ"),
    ("가 나 다", "자"),
    ("㉮ ㉯ ㉰", "㉷"),
    ("ㄱ ㄴ ㄷ", "ㅋ"),
    ("㉠ ㉡ ㉢", "㉫"),
    ("일 이 삼", "십삼"),
    ("一 二 三", "十四"),
    ("㊀ ㊁ ㊂", "㊄"),
    ("갑 을 병", "기"),
    ("甲 乙 丙", "庚"),
    ("* † ‡", "†††††"),
];

fn assert_note_pages(core: &DocumentCore) {
    for (page, (notes, page_number)) in NOTE_PAGES.iter().enumerate() {
        let [body, footnotes, footer] = page_text(core, page as u32);
        for (index, number) in notes.split(' ').enumerate() {
            let marker = format!("글{}{number})", index + 1);
            assert!(
                body.contains(&marker),
                "{}쪽 본문 {marker:?}: {body}",
                page + 1
            );
            assert!(
                footnotes.contains(&format!("{number})")),
                "{}쪽 각주 {number}: {footnotes}",
                page + 1
            );
        }
        assert_eq!(footer, format!("-{page_number}-"), "{}쪽 쪽 번호", page + 1);
    }
}

#[test]
fn note_and_page_numbers_share_the_number_shape_codes() {
    assert_note_pages(&open("note-formats.hwp"));
    assert_note_pages(&open("note-formats.hwpx"));
}

/// 번호 서식 글자의 &·<·>는 HWPX에서 글자 참조로 온다. 다시 열 때 빠지면 안 된다.
#[test]
fn numbering_format_text_keeps_escaped_characters() {
    use std::io::{Read, Write};
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(DIR)
        .join("numbering.hwpx");
    let mut input =
        zip::ZipArchive::new(std::io::Cursor::new(std::fs::read(path).unwrap())).unwrap();
    let mut output = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        if file.name().expect("entry name") == "Contents/header.xml" {
            // 번호 정의 2(Ⓐ)의 수준 1 서식 "^1." → "<^1&>"
            let header = String::from_utf8(data).unwrap();
            let at = header.find(r#"<hh:numbering id="2""#).unwrap();
            let at = at + header[at..].find(">^1.</hh:paraHead>").unwrap();
            data = format!(
                "{}>&lt;^1&amp;&gt;</hh:paraHead>{}",
                &header[..at],
                &header[at + ">^1.</hh:paraHead>".len()..]
            )
            .into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(file.compression());
        output
            .start_file(file.name().expect("entry name"), options)
            .unwrap();
        output.write_all(&data).unwrap();
    }
    let bytes = output.finish().unwrap().into_inner();
    let text = body_text(&DocumentCore::from_bytes(&bytes).expect("open"));
    assert!(
        text.contains("<Ⓐ&>CIRCLED_LATIN_CAPITAL수준1시작1+0"),
        "{}",
        &text[..200.min(text.len())]
    );
}
