//! [#5557] HWP3 기본 셀 안쪽 여백(35 hunit ×4 = 140 균일)이 한글 표준 셀 여백
//! (좌우 0.18cm=510 · 상하 0.05cm=141)으로 사상되는지 — 합성 최소 문서로 검증.
//!
//! 한글 2022 HWP3 임포터의 실측 규칙(SaveAs HWPX 정답지 2개 문서 2,351셀 전수에서
//! 예외 0)이다. 510 은 hunit×4 로 표현 불가(127.5)한 값이라 파일 유래일 수 없다.
//! 기본값 튜플만 사상하고 사용자 지정 여백은 원값(×4)을 보존해야 한다.
//!
//! 합성 문서 레이아웃은 `mydocs/tech/한글문서파일구조3.0.md`(§10.6 표, 표 40~42)와
//! 파서 리더 구조를 따른다. src 쪽 단위 테스트는 unit-test-tier 정책(source-side
//! 총량 동결)에 따라 이 통합 테스트로 옮겼다.

use rhwp::model::control::Control;
use rhwp::model::table::Table;

// ---------------------------------------------------------------------------
// 합성 HWP3 문서 빌더 (최소 골격 — issue_5141 테스트와 동형)
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
    h.push(0u8);
    h.push(0u8);
    h.extend_from_slice(&u32le(0));
    h.push(0u8);
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

// ---------------------------------------------------------------------------
// 표(ch=10) 컨트롤 (§10.6, 표 40~42)
// ---------------------------------------------------------------------------

/// 1×1 표 컨트롤 문단 — [식별 8B][표 정보 84B][셀 정보 27B][셀 문단 리스트]
/// [캡션 리스트][0x0D]. 셀 여백 raw(hunit)를 인자로 받는다.
fn table_paragraph(cell_margin_raw: [u16; 4]) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&para_header(5, 1));
    p.extend_from_slice(&line_info(100));
    // 식별 정보: [hchar 10][dword 예약][hchar 10]
    p.extend_from_slice(&u16le(10));
    p.extend_from_slice(&u32le(0));
    p.extend_from_slice(&u16le(10));
    // 표 정보 84B (표 40)
    let mut info = [0u8; 84];
    info[16..18].copy_from_slice(&u16le(10)); // 특수 문자 코드
    for (k, v) in cell_margin_raw.iter().enumerate() {
        info[34 + k * 2..36 + k * 2].copy_from_slice(&u16le(*v)); // 셀 여백
    }
    info[42..44].copy_from_slice(&u16le(2000)); // 박스 가로
    info[44..46].copy_from_slice(&u16le(400)); // 박스 세로
    info[78..80].copy_from_slice(&u16le(0)); // 박스 종류 = 표
    info[80..82].copy_from_slice(&u16le(1)); // 셀 개수
    p.extend_from_slice(&info);
    // 셀 정보 27B (표 42)
    let mut cell = [0u8; 27];
    cell[8..10].copy_from_slice(&u16le(2000)); // 셀 가로
    cell[10..12].copy_from_slice(&u16le(400)); // 셀 세로
    p.extend_from_slice(&cell);
    // 셀 문단 리스트 (빈) + 캡션 문단 리스트 (빈)
    p.extend_from_slice(&paragraph_list_end());
    p.extend_from_slice(&paragraph_list_end());
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p
}

fn first_table(doc: &rhwp::model::document::Document) -> Table {
    for section in &doc.sections {
        for para in &section.paragraphs {
            for control in &para.controls {
                if let Control::Table(table) = control {
                    return (**table).clone();
                }
            }
        }
    }
    panic!("표 컨트롤이 없음");
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------

// 기본값 튜플(35 hunit ×4 = 140 균일)은 한글 표준(510/510/141/141)으로 사상된다.
#[test]
fn hwp3_default_cell_margin_maps_to_hangul_standard() {
    let bytes = hwp3_doc(&hwp3_body(&table_paragraph([35, 35, 35, 35])));
    let doc = rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패");
    let table = first_table(&doc);
    let cell = table.cells.first().expect("셀이 없음");
    assert_eq!(
        (
            cell.padding.left,
            cell.padding.right,
            cell.padding.top,
            cell.padding.bottom
        ),
        (510, 510, 141, 141),
        "기본 셀 여백은 한글 표준으로 사상되어야 함"
    );
}

// 사용자 지정 여백은 원값(×4)을 보존한다.
#[test]
fn hwp3_custom_cell_margin_is_preserved() {
    let bytes = hwp3_doc(&hwp3_body(&table_paragraph([50, 50, 35, 35])));
    let doc = rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패");
    let table = first_table(&doc);
    let cell = table.cells.first().expect("셀이 없음");
    assert_eq!(
        (
            cell.padding.left,
            cell.padding.right,
            cell.padding.top,
            cell.padding.bottom
        ),
        (200, 200, 140, 140),
        "사용자 지정 셀 여백은 원값 ×4 를 보존해야 함"
    );
}
