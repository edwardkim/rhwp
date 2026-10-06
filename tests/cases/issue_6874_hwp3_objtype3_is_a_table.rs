//! [#6874] HWP3 `obj_type=3` 을 표로 보존한다 — 종전에는 표를 버리고 버튼만 남겼다.
//!
//! ## 증상
//!
//! 코퍼스 `2955289`(HWP3)를 `convert` 하면 표 2개 중 1개와 본문 12자(`- 581-13 -`)를
//! 잃는다. 둘은 **같은 개체 하나**다. 한글은 원본에서 그 글자를 보여 주는데 저장본에는
//! 없다(한글 2024 추출 텍스트 372자 대 362자).
//!
//! ## 근인
//!
//! HWP3 제어문자 10(표/글상자/수식/버튼)의 `obj_type=3` 은 **캡션을 담은 1x1 표**
//! 구조로 저장되고, 한글도 그것을 표로 만든다 — 정본 HWP3 -> HWPX 대조에서
//! `hp:tbl 2 / hp:btn 0` 이다(현행은 `hp:tbl 1 / hp:btn 1`). 파서는 그 표를 버리고
//! `FormObject{PushButton}` 만 남겨, 캡션이 본문 글자가 아니라 **개체 속성**이 됐다.
//!
//! 바로 위 `obj_type == 1`(글상자)은 같은 이유로 이미 Table IR 을 보존한다 —
//! `obj_type = 3` 만 버렸다.
//!
//! ## 배치 계약과의 관계
//!
//! 표로 두면 세로 기준이 `VertRelTo::Paper` 경로로 간다. 그 경로의 기준 높이가
//! 상·하 여백이 같다는 가정(`col_area.y * 2 + col_area.height`)이면 `#6266` 이 한글
//! 2024 COM PDF 로 잠근 배치가 +37.8px 아래로 밀린다. 같은 커밋이 그 기준을 실제 용지
//! 높이로 바로잡는다 — 실문서 계약은 `issue_6266_form_object_placement` 가 잠근다.
//! 이 파일은 **파서 축만** 합성 최소 문서로 잠근다.
//!
//! 합성 문서 골격은 `issue_5557_hwp3_cell_margin` 과 동형이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;

// ---------------------------------------------------------------------------
// 합성 HWP3 문서 빌더
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
    d.extend_from_slice(&[0u8; 128]);
    d.extend_from_slice(&[0u8; 1008]);
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
    b.extend_from_slice(&u16le(0));
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
    h.push(0u8); // follow_prev_para_shape
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

/// 글자만 담은 셀 문단 하나 + 리스트 끝.
fn cell_paragraph_list(text: &str) -> Vec<u8> {
    let chars: Vec<u16> = text.chars().map(|c| c as u16).collect();
    let mut p = Vec::new();
    p.extend_from_slice(&para_header(chars.len() as u16 + 1, 1));
    p.extend_from_slice(&line_info(0));
    for c in &chars {
        p.extend_from_slice(&u16le(*c));
    }
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p.extend_from_slice(&paragraph_list_end());
    p
}

/// 제어문자 10 개체 문단 — `box_type` 이 표(0)/글상자(1)/수식(2)/버튼(3)을 가른다.
fn object_paragraph(box_type: u16, cell_text: &str) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&para_header(5, 1));
    p.extend_from_slice(&line_info(100));
    // 식별 정보: [hchar 10][dword 예약][hchar 10]
    p.extend_from_slice(&u16le(10));
    p.extend_from_slice(&u32le(0));
    p.extend_from_slice(&u16le(10));
    // 개체 정보 84B
    let mut info = [0u8; 84];
    info[16..18].copy_from_slice(&u16le(10)); // 특수 문자 코드
    for k in 0..4 {
        info[34 + k * 2..36 + k * 2].copy_from_slice(&u16le(35)); // 셀 여백
    }
    info[42..44].copy_from_slice(&u16le(2000)); // 박스 가로
    info[44..46].copy_from_slice(&u16le(400)); // 박스 세로
    info[78..80].copy_from_slice(&u16le(box_type)); // 박스 종류
    info[80..82].copy_from_slice(&u16le(1)); // 셀 개수
    p.extend_from_slice(&info);
    // 셀 정보 27B
    let mut cell = [0u8; 27];
    cell[8..10].copy_from_slice(&u16le(2000));
    cell[10..12].copy_from_slice(&u16le(400));
    p.extend_from_slice(&cell);
    p.extend_from_slice(&cell_paragraph_list(cell_text)); // 셀 문단 리스트
    p.extend_from_slice(&paragraph_list_end()); // 캡션 문단 리스트
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p
}

fn parse(box_type: u16, cell_text: &str) -> rhwp::model::document::Document {
    let bytes = hwp3_doc(&hwp3_body(&object_paragraph(box_type, cell_text)));
    rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패")
}

fn controls(doc: &rhwp::model::document::Document) -> Vec<&Control> {
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .collect()
}

/// 문서 전체에서 표 셀 안 글자를 모은다.
fn table_cell_text(doc: &rhwp::model::document::Document) -> String {
    let mut out = String::new();
    for c in controls(doc) {
        if let Control::Table(t) = c {
            for cell in &t.cells {
                for para in &cell.paragraphs {
                    out.push_str(&para.text);
                }
            }
        }
    }
    out
}

fn kinds(doc: &rhwp::model::document::Document) -> Vec<&'static str> {
    controls(doc)
        .into_iter()
        .filter_map(|c| match c {
            Control::Table(_) => Some("Table"),
            Control::Form(_) => Some("Form"),
            Control::Shape(_) => Some("Shape"),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------

/// `obj_type=3`(버튼)은 표로 보존된다 — 한글도 같은 개체를 표로 만든다.
#[test]
fn hwp3_objtype3_becomes_a_table() {
    let doc = parse(3, "581");
    assert_eq!(
        kinds(&doc),
        vec!["Table"],
        "obj_type=3 은 Table 이어야 한다 (정본 HWP3->HWPX: hp:tbl 2 / hp:btn 0)"
    );
}

/// 캡션이 **본문 글자**로 남는다 — 종전에는 개체 속성이라 저장본에서 사라졌다.
#[test]
fn hwp3_objtype3_keeps_its_caption_as_body_text() {
    let doc = parse(3, "581");
    assert!(
        table_cell_text(&doc).contains("581"),
        "표 셀 글자가 사라졌다 — 실제 문서에서 본문 12자가 없어지던 결함"
    );
}

/// 반례 — 표(0)와 글상자(1)는 종전 그대로 Table 이다.
#[test]
fn hwp3_table_and_textbox_are_unchanged() {
    for box_type in [0u16, 1u16] {
        let doc = parse(box_type, "581");
        assert_eq!(
            kinds(&doc),
            vec!["Table"],
            "obj_type={box_type} 의 종전 동작이 바뀌면 안 된다"
        );
        assert!(
            table_cell_text(&doc).contains("581"),
            "obj_type={box_type} 의 셀 글자가 사라졌다"
        );
    }
}
