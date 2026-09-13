//! [#4680] HWP3 "항상 홀수쪽으로 시작"(제어문자 21, 종류 0)이 HWP5 저장에서
//! 구조 오염이 된다 — 한글이 문서를 열지 못한다.
//!
//! ## 증상
//!
//! 264쪽 HWP3 문서(`1170000-200500003 독일의 법령체계와 입법심사기준`)를
//! `rhwp convert` 로 저장하면 한글이 **열지 못한다**(open=False). 그 문서에서
//! 해당 제어문자가 있는 42쪽 한 장만 `extract-pages` 로 뽑아도 같은 거부가
//! 재현된다. rhwp 자신은 산출물을 원본과 동일하게 되읽으므로 `--verify`
//! 자기검증으로는 보이지 않는다 — 판정자는 한글뿐이다.
//!
//! ## 재현 (코퍼스 문서 — 저장소에는 넣지 않는다)
//!
//! ```text
//! hwpdocs_10k_share/prism_downloads/법제처/
//!   1170000-200500003_D0150004-1-001_독일의 법령체계와 입법심사기준(최종본).hwp
//!
//! rhwp extract-pages <원본> p42.hwp --from 42 --to 42   # 5문단 1쪽
//! # 한글로 p42.hwp 열기 → 수정 전 open=False / 수정 후 open=True
//! ```
//!
//! ## 근인
//!
//! HWP3 파서의 제어문자 18..=21 arm 에서 21 은 종류 1(감춤)만 다루고 종류 0은
//! catch-all 로 떨어져 `Control::Unknown { ctrl_id: 21 }` 이 됐다. `UnknownControl`
//! 의 `ctrl_id` 는 **HWP5 의 네 글자 코드**를 실어 나르는 자리인데(알 수 없는 HWP5
//! 컨트롤의 왕복 보존용), HWP3 파서가 거기에 HWP3 제어문자 코드 21을 넣었다.
//! HWP5 저장기는 그 값을 개체 제어문자 자리(0x000B)와 `CTRL_HEADER` 에 그대로
//! 쓰므로, 저장본에는 `ctrl_id = 0x00000015` 라는 존재할 수 없는 컨트롤이 실린다.
//!
//! ## 기대값의 출처
//!
//! 규격(`mydocs/tech/한글문서파일구조3.0.md` §10.15 표 56)이 종류 0 = "홀수로 시작",
//! 1 = "감춤" 이라고 적는다. 값은 같은 문서를 한/글이 저장한 HWP5 변환본에서 왔다 —
//! `pgct`(PageNumCtrl) 4건이 모두 payload 2(= 홀수 쪽)이고 개수가 우리 쪽 4건과
//! 일치한다.
//!
//! ## 이 테스트가 잠그는 것
//!
//! 합성 최소 HWP3 문서를 만들어 (1) IR 이 `PageNumCtrl(Odd)` 가 되는지, (2) 저장한
//! HWP5 의 `CTRL_HEADER` 가 **전부 네 글자 코드**인지를 본다. (2)가 개방 거부의
//! 실제 형상이다 — 수정 전에는 `0x00000015` 가 실려 깨진다. 반례로 종류 1(감춤)이
//! 계속 `PageHide` 로 남는 것도 함께 잠근다.
//!
//! 합성 문서 골격은 `issue_5557_hwp3_cell_margin` 과 동형이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::{Control, PageStartsOn};
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;

// ---------------------------------------------------------------------------
// 합성 HWP3 문서 빌더 (최소 골격)
// ---------------------------------------------------------------------------

fn u16le(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}

fn u32le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

fn hwp3_doc(body: &[u8]) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05");
    assert_eq!(d.len(), 30);
    d.extend_from_slice(&[0u8; 128]); // 문서 정보 (비압축·비암호)
    d.extend_from_slice(&[0u8; 1008]); // 요약
    d.extend_from_slice(body);
    d
}

fn hwp3_body(paragraphs: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    for _ in 0..7 {
        b.extend_from_slice(&u16le(1));
        let mut name = [0u8; 40];
        name[..4].copy_from_slice(&[0xB9, 0xD9, 0xC5, 0xC1]); // "바탕" (EUC-KR)
        b.extend_from_slice(&name);
    }
    b.extend_from_slice(&u16le(0)); // nstyles
    b.extend_from_slice(paragraphs);
    b.extend_from_slice(&paragraph_list_end());
    b
}

fn paragraph_list_end() -> [u8; 43] {
    [0u8; 43]
}

fn char_shape31() -> [u8; 31] {
    let mut cs = [0u8; 31];
    cs[..2].copy_from_slice(&u16le(250));
    for r in &mut cs[9..16] {
        *r = 100;
    }
    cs
}

fn para_shape187() -> [u8; 187] {
    let mut ps = [0u8; 187];
    ps[6..8].copy_from_slice(&u16le(160));
    ps[172] = 1;
    ps
}

fn para_header(char_count: u16, line_count: u16) -> Vec<u8> {
    let mut h = Vec::new();
    h.push(0u8);
    h.extend_from_slice(&u16le(char_count));
    h.extend_from_slice(&u16le(line_count));
    h.push(0u8); // include_char_shape
    h.push(0u8); // flags
    h.extend_from_slice(&u32le(0)); // special_char_flags
    h.push(0u8); // style_index
    h.extend_from_slice(&char_shape31());
    h.extend_from_slice(&para_shape187());
    h
}

fn line_info(pgy: u16) -> Vec<u8> {
    let mut l = Vec::new();
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(400));
    l.extend_from_slice(&u16le(pgy));
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(0));
    l
}

/// 홀수쪽시작/감추기(21) 한 개만 담은 문단 — 규격 §10.15 표 56.
///
///   [hchar 21][word 종류][word 감출 대상][hchar 21] = 8바이트 = hchar 4개,
///   뒤에 문단 끝(0x0D) 1개.
fn odd_or_hide_paragraph(kind: u16, hide_mask: u16) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&para_header(5, 1));
    p.extend_from_slice(&line_info(100));
    p.extend_from_slice(&u16le(21));
    p.extend_from_slice(&u16le(kind));
    p.extend_from_slice(&u16le(hide_mask));
    p.extend_from_slice(&u16le(21));
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p
}

fn parse(kind: u16, hide_mask: u16) -> rhwp::model::document::Document {
    let bytes = hwp3_doc(&hwp3_body(&odd_or_hide_paragraph(kind, hide_mask)));
    rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패")
}

fn controls(doc: &rhwp::model::document::Document) -> Vec<&Control> {
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .collect()
}

/// 저장한 HWP5 본문의 `CTRL_HEADER` ctrl_id 를 네 글자 코드 문자열로 모은다.
/// 네 글자 ASCII 가 아니면 `None` 자리에 원값을 16진수로 남긴다.
fn body_ctrl_ids(doc: &rhwp::model::document::Document) -> Vec<Result<String, u32>> {
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(doc).expect("HWP5 직렬화 실패");
    let mut cfb = CfbReader::open(&bytes).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header.get(36).is_some_and(|b| b & 0x01 != 0);
    let section = cfb
        .read_body_text_section(0, compressed, false)
        .expect("Section0 읽기");
    Record::read_all(&section)
        .expect("Section0 record 파싱")
        .into_iter()
        .filter(|r| r.tag_id == rhwp::parser::tags::HWPTAG_CTRL_HEADER)
        .filter_map(|r| {
            let raw = r.data.get(..4)?;
            let id = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
            // HWP5 ctrl_id 는 네 글자 코드를 빅엔디언으로 담는다 ('pgct' → 0x70676374).
            let code: String = raw.iter().rev().map(|b| *b as char).collect();
            Some(if code.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
                Ok(code)
            } else {
                Err(id)
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------

/// 종류 0 = "항상 홀수쪽으로 시작" → `PageNumCtrl(Odd)`.
/// 한/글 HWP5 변환본이 같은 자리에 `pgct` payload 2 를 쓴다.
#[test]
fn hwp3_odd_page_start_becomes_page_num_ctrl() {
    let doc = parse(0, 0);
    let ctrls = controls(&doc);
    let page_num_ctrls: Vec<_> = ctrls
        .iter()
        .filter_map(|c| match c {
            Control::PageNumCtrl(p) => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(
        page_num_ctrls.len(),
        1,
        "홀수쪽시작(21/종류 0)은 PageNumCtrl 하나가 되어야 한다 — 실제 컨트롤: {:?}",
        ctrls
    );
    assert_eq!(
        page_num_ctrls[0].page_starts_on,
        PageStartsOn::Odd,
        "규격 §10.15 종류 0 = 홀수로 시작 (한/글 변환본 pgct payload 2)"
    );
    assert!(
        !ctrls.iter().any(|c| matches!(c, Control::Unknown(_))),
        "HWP3 제어문자 코드가 Unknown 의 ctrl_id 로 새면 안 된다 — {:?}",
        ctrls
    );
}

/// 저장본의 `CTRL_HEADER` 는 전부 네 글자 코드여야 한다. 수정 전에는 여기에
/// `0x00000015`(HWP3 제어문자 코드 21)가 실렸고, 한글은 그 문서를 열지 못했다.
#[test]
fn saved_hwp5_ctrl_ids_are_all_four_char_codes() {
    let doc = parse(0, 0);
    let ids = body_ctrl_ids(&doc);
    let bogus: Vec<_> = ids.iter().filter_map(|r| r.as_ref().err()).collect();
    assert!(
        bogus.is_empty(),
        "네 글자 코드가 아닌 ctrl_id 가 저장본에 실렸다 (한글 개방 거부) — {:?} / 전체 {:?}",
        bogus,
        ids
    );
    assert!(
        ids.iter().any(|r| r.as_deref() == Ok("pgct")),
        "홀수쪽시작은 'pgct' 로 저장되어야 한다 — 전체 {:?}",
        ids
    );
}

/// 반례 — 종류 1(감춤)은 종전대로 `PageHide` 다. 감출 대상 비트도 그대로 옮긴다.
/// 00472 실측: HWP3 감출 대상 7 ↔ 한/글 `pghd` 0x23(머리말·꼬리말·쪽번호).
#[test]
fn hwp3_hide_kind_still_becomes_page_hide() {
    let doc = parse(1, 0b0000_0111);
    let ctrls = controls(&doc);
    let hides: Vec<_> = ctrls
        .iter()
        .filter_map(|c| match c {
            Control::PageHide(h) => Some(h),
            _ => None,
        })
        .collect();
    assert_eq!(
        hides.len(),
        1,
        "감추기(21/종류 1)는 PageHide 로 남아야 한다 — 실제 컨트롤: {:?}",
        ctrls
    );
    assert_eq!(
        (
            hides[0].hide_header,
            hides[0].hide_footer,
            hides[0].hide_page_num,
            hides[0].hide_border
        ),
        (true, true, true, false),
        "감출 대상 비트 0·1·2 만 선다 (규격 §10.15)"
    );
    assert!(
        !ctrls
            .iter()
            .any(|c| matches!(c, Control::PageNumCtrl(_) | Control::Unknown(_))),
        "감추기가 홀수쪽시작으로 새면 안 된다 — {:?}",
        ctrls
    );
}
