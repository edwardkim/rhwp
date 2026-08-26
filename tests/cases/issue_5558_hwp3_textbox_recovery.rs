//! [#5558] 공통 헤더 뒤 글상자 정보(스펙 §11.3 optional 블록, 표 78 형식)로 실려 온
//! 내부 문단 리스트가 회수되는지 — 합성 최소 문서로 검증.
//!
//! "글상자로 만들기"가 걸린 사각형 등 비-글상자 개체는 공통 헤더 뒤에
//! `[정보1 길이][정보2 길이][문단 리스트]` 블록을 갖고, 빈티지 코퍼스는 그 전체
//! 길이를 header_length 로 선언한다. #5141 의 선언-길이 전진만으로는 이 블록을
//! 통째로 건너뛰어 묶음 안 상자 라벨이 전멸했다(07615: 정답지 drawText 134 vs 1).
//!
//! src 쪽 단위 테스트는 unit-test-tier 정책(source-side 총량 동결)에 따라
//! 공개 API(`parse_hwp3`) 경로의 이 통합 테스트로 옮겼다.

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;

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

/// 텍스트 문단 — 문단 리스트(글상자 내부용)에 넣을 "ja" 라벨.
fn text_paragraph(text: &str) -> Vec<u8> {
    let chars: Vec<u16> = text.chars().map(|c| c as u16).collect();
    let mut p = Vec::new();
    p.extend_from_slice(&para_header((chars.len() + 1) as u16, 1));
    p.extend_from_slice(&line_info(100));
    for ch in chars {
        p.extend_from_slice(&u16le(ch));
    }
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p
}

// ---------------------------------------------------------------------------
// 그리기 개체 (§11.3) — 글상자 정보를 품은 사각형
// ---------------------------------------------------------------------------

fn shape_block(object_type: u16, conn: u16, detail: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&u32le(88));
    b.extend_from_slice(&u16le(object_type));
    b.extend_from_slice(&u16le(conn));
    b.extend_from_slice(&[0u8; 40]);
    b.extend_from_slice(&[0u8; 44]);
    b.extend_from_slice(detail);
    b
}

/// 글상자 정보(표 78: [정보1 길이=0][정보2 길이=n][문단 리스트 n])를 품은 사각형 —
/// header_length 는 (88 + 글상자 정보 길이)로 선언된다(빈티지 계약).
fn textbox_rect_block(conn: u16, inner_list: &[u8]) -> Vec<u8> {
    let mut tbox = Vec::new();
    tbox.extend_from_slice(&u32le(0)); // 정보 1의 길이
    tbox.extend_from_slice(&u32le(inner_list.len() as u32)); // 정보 2의 길이
    tbox.extend_from_slice(inner_list);

    let mut b = Vec::new();
    b.extend_from_slice(&u32le((88 + tbox.len()) as u32));
    b.extend_from_slice(&u16le(2)); // 사각형
    b.extend_from_slice(&u16le(conn));
    b.extend_from_slice(&[0u8; 40]);
    b.extend_from_slice(&[0u8; 44]);
    b.extend_from_slice(&tbox); // 글상자 정보 (header_length 에 포함)
    b.extend_from_slice(&[0u8; 8]); // 사각형 세부 정보 길이
    b
}

fn drawing_ext(objects: &[u8]) -> Vec<u8> {
    let mut e = Vec::new();
    e.extend_from_slice(&u32le(24));
    e.extend_from_slice(&u32le(0));
    e.extend_from_slice(&u32le(1));
    e.extend_from_slice(&[0u8; 16]);
    e.extend_from_slice(objects);
    e
}

fn drawing_paragraph(ext: &[u8]) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&para_header(5, 1));
    p.extend_from_slice(&line_info(100));
    p.extend_from_slice(&u16le(11));
    p.extend_from_slice(&u32le(0));
    p.extend_from_slice(&u16le(11));
    let mut info = [0u8; 348];
    info[..4].copy_from_slice(&u32le(ext.len() as u32));
    info[42..44].copy_from_slice(&u16le(1000));
    info[44..46].copy_from_slice(&u16le(800));
    info[74] = 3; // pic_type 3 = 그리기 개체
    p.extend_from_slice(&info);
    p.extend_from_slice(ext);
    p.extend_from_slice(&paragraph_list_end()); // 캡션 (빈)
    p.extend_from_slice(&u16le(13));
    p
}

fn first_shape(doc: &rhwp::model::document::Document) -> ShapeObject {
    for section in &doc.sections {
        for para in &section.paragraphs {
            for control in &para.controls {
                if let Control::Shape(shape) = control {
                    return (**shape).clone();
                }
            }
        }
    }
    panic!("Shape 컨트롤이 없음");
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------

// 잉여 구간이 표 78 과 정확히 맞으면 문단 리스트를 회수해 사각형의 글상자
// 텍스트로 붙인다 — 묶음(컨테이너 자식) 안 상자 라벨이 살아나야 한다.
#[test]
fn hwp3_textbox_info_paragraph_list_is_recovered() {
    let mut inner = Vec::new();
    inner.extend_from_slice(&text_paragraph("ja"));
    inner.extend_from_slice(&paragraph_list_end());

    let mut objects = Vec::new();
    objects.extend_from_slice(&shape_block(0, 0x0002, &[0u8; 8])); // 컨테이너, has_child
    objects.extend_from_slice(&textbox_rect_block(0x0001, &inner)); // 글상자 사각형, 형제
    objects.extend_from_slice(&shape_block(1, 0x0000, &[0u8; 12])); // 직선, 끝

    let bytes = hwp3_doc(&hwp3_body(&drawing_paragraph(&drawing_ext(&objects))));
    let doc = rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패");

    match first_shape(&doc) {
        ShapeObject::Group(g) => {
            assert_eq!(
                g.children.len(),
                2,
                "글상자 회수 후 형제 스트림이 이어져야 함"
            );
            let ShapeObject::Rectangle(rect) = &g.children[0] else {
                panic!("첫 자식이 사각형이어야 함");
            };
            let text_box = rect
                .drawing
                .text_box
                .as_ref()
                .expect("글상자 문단 리스트가 회수되어야 함");
            assert_eq!(
                text_box.paragraphs.first().map(|p| p.text.as_str()),
                Some("ja"),
                "회수된 글상자 라벨 텍스트"
            );
            assert!(matches!(g.children[1], ShapeObject::Line(_)));
        }
        other => panic!("루트가 묶음이어야 함: {}", other.shape_name()),
    }
}
