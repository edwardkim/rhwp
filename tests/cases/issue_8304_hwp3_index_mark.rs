//! [Issue #8304] HWP3 찾아보기 표시는 공통 IR 의 찾아보기 표시로 읽고, HWP5·HWPX 저장이
//! 함께 보존한다.
//!
//! HWP3 특수 문자 26(스펙 §10.20 표 61)의 키워드는 `hchar array[60]` 두 개다. rhwp 는 이를
//! 바이트 단위 조합형으로 읽어 키워드가 깨졌고(`씽b로?`), `Unknown` 필드로 만들어 HWP5
//! 저장에서 사라졌다(`convert --verify` exit 3). 기대 키워드는 같은 문서를 한컴 오피스 2024
//! (13.0.0.3901)가 HWP5 로 저장한 `samples/HWP5-nopassword-123456.hwp` 의 찾아보기 표시다.
//!
//! 이 문서는 화면 배치가 아니라 IR·저장 계약만 검사한다 — 찾아보기 표시는 그려지지 않는
//! 컨트롤이며, 원문의 쪽 배치는 #7445 의 피델리티 범위다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parser::hwp3::encoding::{decode_hchar_array, decode_hwp3_string};

const HWP3: &str = "samples/HWP3-password-123456.hwp";
const HANCOM_HWP5: &str = "samples/HWP5-nopassword-123456.hwp";

fn open(path: &str, password: Option<&str>) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    match password {
        Some(p) => DocumentCore::from_bytes_with_password(&bytes, p.as_bytes()),
        None => DocumentCore::from_bytes(&bytes),
    }
    .unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

/// 본문 문단마다 (구역, 문단, 첫째 키, 둘째 키).
fn index_marks(core: &DocumentCore) -> Vec<(usize, usize, String, String)> {
    let mut out = Vec::new();
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for control in &para.controls {
                if let Control::IndexMark(im) = control {
                    out.push((si, pi, im.first_key.clone(), im.second_key.clone()));
                }
            }
        }
    }
    out
}

/// HWP3 원문 본문에는 컨트롤 자리 표지(U+FFFC)가 있고 HWP5 에는 없다 — 표지를 빼고 본다.
fn paragraph_text(core: &DocumentCore, si: usize, pi: usize) -> String {
    core.document().sections[si].paragraphs[pi]
        .text
        .chars()
        .filter(|&c| c != '\u{FFFC}')
        .collect()
}

#[test]
fn hwp3_index_marks_match_hancom_hwp5_keywords() {
    let hwp3 = open(HWP3, Some("123456"));
    let hancom = open(HANCOM_HWP5, None);
    let expected = index_marks(&hancom);
    assert_eq!(
        expected
            .iter()
            .map(|(_, _, k, _)| k.as_str())
            .collect::<Vec<_>>(),
        ["입력기", "바탕글", "팩스 머지"],
        "한컴 HWP5 의 찾아보기 키워드"
    );
    assert_eq!(index_marks(&hwp3), expected, "HWP3 에서 읽은 찾아보기 표시");
    for (si, pi, _, _) in &expected {
        assert_eq!(
            paragraph_text(&hwp3, *si, *pi),
            paragraph_text(&hancom, *si, *pi),
            "구역 {si} 문단 {pi}: 표시가 놓인 문단"
        );
    }
}

#[test]
fn hwp3_index_marks_survive_hwp5_and_hwpx_save() {
    let hwp3 = open(HWP3, Some("123456"));
    let marks = index_marks(&hwp3);
    assert_eq!(marks.len(), 3);

    let hwp5 = DocumentCore::from_bytes(&hwp3.export_hwp_with_adapter().expect("HWP5 저장"))
        .expect("HWP5 다시 열기");
    assert_eq!(index_marks(&hwp5), marks, "HWP5 저장 뒤");
    assert_eq!(hwp5.page_count(), hwp3.page_count(), "HWP5 저장 뒤 쪽수");

    let hwpx = DocumentCore::from_bytes(&hwp3.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기");
    assert_eq!(index_marks(&hwpx), marks, "HWPX 저장 뒤");
    assert_eq!(hwpx.page_count(), hwp3.page_count(), "HWPX 저장 뒤 쪽수");
}

/// `hchar array` 는 글자마다 리틀 엔디언 2바이트다 — ASCII 도 2바이트를 차지하고 0 에서
/// 끝난다. 바이트 단위 조합형으로 읽으면 다른 글자가 된다.
#[test]
fn hchar_array_reads_little_endian_internal_codes() {
    // 'A'(0x0041), '가'(조합형 0x8861), 끝(0), 'B'
    let bytes = [0x41, 0x00, 0x61, 0x88, 0x00, 0x00, 0x42, 0x00];
    assert_eq!(decode_hchar_array(&bytes), "A가");
    assert_ne!(decode_hwp3_string(&bytes), "A가");
}

// ---------- 같은 문서의 렌더링 정합 (#8304 PR 범위) ----------
//
// 이 문서의 Print PDF(`pdf/HWP3-password-123456.pdf`) 대비 24쪽 중 23쪽이 90% 미만이던 원인을
// 한컴 HWP5 변환본과 대조해 고쳤다. 기대값은 모두 그 변환본의 저장값이다.

/// 문단 앞뒤 간격과 첫 줄 위치가 문단 365개 모두 한컴 HWP5 와 같다. 종전 암호 전용 ×4
/// 간격은 쪽 맨 위 제목의 앞 간격을 저장 vpos 보다 작게 만들어 그 쪽 본문을 4px 올렸고,
/// 글자처럼 취급 표 문단의 앞 간격 생략은 3쪽 표 뒤 내용을 9px 올렸다.
#[test]
fn password_hwp3_paragraph_spacing_and_line_positions_match_hancom_hwp5() {
    let hwp3 = open(HWP3, Some("123456"));
    let hancom = open(HANCOM_HWP5, None);
    let (a, b) = (hwp3.document(), hancom.document());
    let (pa, pb) = (&a.sections[0].paragraphs, &b.sections[0].paragraphs);
    assert_eq!(pa.len(), pb.len(), "문단 수");
    for (i, (x, y)) in pa.iter().zip(pb).enumerate() {
        let sx = &a.doc_info.para_shapes[x.para_shape_id as usize];
        let sy = &b.doc_info.para_shapes[y.para_shape_id as usize];
        assert_eq!(
            (sx.spacing_before, sx.spacing_after),
            (sy.spacing_before, sy.spacing_after),
            "문단 {i} 앞뒤 간격"
        );
        assert_eq!(
            x.line_segs.first().map(|l| l.vertical_pos),
            y.line_segs.first().map(|l| l.vertical_pos),
            "문단 {i} 첫 줄 위치"
        );
    }
    assert_eq!(hwp3.page_count(), hancom.page_count(), "쪽수");
}

/// 추가 정보 블록 #6 의 쪽 배경 그림은 구역 첫 쪽에만 그린다 — 한컴 HWP5 의 구역 정의
/// bit 9(0x0200). 종전에는 모든 쪽 본문 위에 그림이 깔렸다.
#[test]
fn password_hwp3_page_background_is_first_page_only() {
    let hwp3 = open(HWP3, Some("123456"));
    let hancom = open(HANCOM_HWP5, None);
    let (a, b) = (
        &hwp3.document().sections[0].section_def,
        &hancom.document().sections[0].section_def,
    );
    assert!(b.first_page_fill && b.flags & 0x0200 != 0, "한컴 HWP5 기준");
    assert_eq!(
        (a.first_page_fill, a.first_page_border, a.flags & 0x0300),
        (b.first_page_fill, b.first_page_border, b.flags & 0x0300)
    );
}

/// 글이 든 하이퍼텍스트 상자는 상자와 글을 보존한다(한컴 HWP5: 글상자). 종전에는 빈
/// 하이퍼링크만 남아 17쪽 `홈페이지()`, 24쪽 `인터넷:` 뒤 글이 사라졌다.
#[test]
fn password_hwp3_hypertext_box_keeps_its_text() {
    let hwp3 = open(HWP3, Some("123456"));
    let pages: Vec<String> = (0..hwp3.page_count())
        .map(|p| hwp3.extract_page_text_native(p).expect("쪽 글"))
        .collect();
    assert!(pages[16].contains("http://www.hnc.co.kr"), "17쪽");
    assert!(
        pages[23].contains("고객지원(http://haansoft.netian.com)"),
        "24쪽"
    );
}

/// 반례: 빈 하이퍼텍스트 상자(hwp3-sample16 목차)는 한컴 Print 에 아무것도 그려지지 않으므로
/// 종전대로 하이퍼링크 슬롯으로 남는다. 이 문서는 기존 90% 미만 쪽이 있어 배치(쪽수)는
/// 고정하지 않고 컨트롤 구조만 본다 — 시각 무변화는 PR 의 전쪽 TSV 로 확인했다.
#[test]
fn empty_hypertext_box_stays_a_hyperlink_slot() {
    let core = open("samples/hwp3-sample16.hwp", None);
    let mut hyperlinks = 0;
    for para in &core.document().sections[0].paragraphs {
        for control in &para.controls {
            if matches!(control, Control::Hyperlink(_)) {
                hyperlinks += 1;
            }
        }
    }
    assert_eq!(hyperlinks, 4);
}
