//! [Issue #6511] 글뒤로(BehindText) 전면 배경 그림이 #1995 다중 전면 이미지
//! 낱장 배치로 오인되어 강제 새 쪽을 얻는 결함의 회귀.
//!
//! #1995 낱장 배치 후보 필터는 `treat_as_char` 와 높이(본문 60%+)만 보고 wrap
//! 종류를 보지 않았다. 그래서 전면 크기 배경 프레임 2장을 깐 한 쪽짜리 안내문이
//! 배경마다 `force_new_page()` 를 얻어 3쪽이 된다. 표에는 #703/#1955 가 "글뒤로/
//! 글앞으로 개체는 본문 흐름에서 제외"를 이미 강제한다 — 그림 낱장 경로만 이
//! 불변식을 놓쳤다. 다운스트림 운영 문서 코호트 103건 실측에서 같은 제외로
//! 쪽 일치 82→86, 개선 4건·퇴행 0건이 확인됐다.
//!
//! 픽스처는 합성이다(운영 문서는 개인정보로 첨부 불가): A4 빈 문단에 전면 크기
//! 그림을 wrap 만 바꿔 싣고, HWP5 직렬화 왕복 후 쪽수를 계약으로 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::image::Picture;
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::TextWrap;
use rhwp::model::style::CharShape;
use rhwp::{parse_document, serialize_document, DocumentCore};

/// A4 본문 높이 74267HU(84188 − 위 5669 − 아래 4252)의 81% — 전면 판정(60%) 초과.
const FULLPAGE_H: u32 = 60000;
const FULLPAGE_W: u32 = 40000;

fn fullpage_picture(wrap: TextWrap) -> Control {
    let mut pic = Picture::default();
    pic.common.treat_as_char = false;
    pic.common.text_wrap = wrap;
    pic.common.width = FULLPAGE_W;
    pic.common.height = FULLPAGE_H;
    Control::Picture(Box::new(pic))
}

fn doc_with_fullpage_pictures(wraps: &[TextWrap]) -> Document {
    let host = Paragraph {
        text: String::new(),
        char_count: 1,
        line_segs: vec![LineSeg {
            line_height: 1000,
            line_spacing: 600,
            ..Default::default()
        }],
        controls: wraps.iter().map(|w| fullpage_picture(*w)).collect(),
        ..Default::default()
    };
    let mut doc = Document::default();
    doc.doc_info.char_shapes.push(CharShape::default());
    let mut section = Section::default();
    section.section_def.page_def = PageDef::a4_default();
    section.paragraphs.push(host);
    doc.sections.push(section);
    doc
}

/// 직렬화 왕복 뒤 그림과 wrap 이 살아 있는지 먼저 증명하고 쪽수를 잰다 —
/// 왕복에서 컨트롤이 유실되면 쪽수 계약은 결함이 아니라 유실을 재는 셈이다.
fn page_count_for(wraps: &[TextWrap]) -> u32 {
    let doc = doc_with_fullpage_pictures(wraps);
    let bytes = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&bytes).expect("reparse");
    let survived: Vec<TextWrap> = reparsed.sections[0].paragraphs[0]
        .controls
        .iter()
        .filter_map(|c| match c {
            Control::Picture(pic) => Some(pic.common.text_wrap),
            _ => None,
        })
        .collect();
    assert_eq!(survived, wraps, "왕복에서 그림 컨트롤 또는 wrap 유실");
    let core = DocumentCore::from_bytes(&bytes).expect("open");
    core.page_count()
}

/// 글뒤로 전면 배경 2장은 앵커 쪽에 남는다 — 낱장 배치 대상이 아니다.
/// 수정 전에는 배경마다 강제 새 쪽을 얻어 3쪽이 됐다.
#[test]
fn behind_text_fullpage_backgrounds_stay_on_anchor_page() {
    assert_eq!(
        page_count_for(&[TextWrap::BehindText, TextWrap::BehindText]),
        1
    );
}

/// #1995 보존: 흐름에 참여하는 전면 스캔 그림 2장은 여전히 각각 한 쪽을 받는다.
#[test]
fn square_fullpage_scans_keep_single_page_each() {
    assert_eq!(page_count_for(&[TextWrap::Square, TextWrap::Square]), 3);
}

/// 분모 검증: 배경 2장이 같은 문단에 있어도 스캔 2장의 "전면 과반"(#4654)은
/// 유지된다 — 스캔만 낱장을 받고 배경은 앵커에 남아 3쪽. 배경을 분모에만
/// 남겨두면 2×2 > 4 가 거짓이 되어 정당한 낱장 배치까지 꺼진다.
#[test]
fn behind_text_does_not_dilute_scan_majority() {
    assert_eq!(
        page_count_for(&[
            TextWrap::Square,
            TextWrap::Square,
            TextWrap::BehindText,
            TextWrap::BehindText,
        ]),
        3
    );
}
