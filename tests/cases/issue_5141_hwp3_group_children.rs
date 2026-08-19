//! [#5141] HWP3 묶음(그리기 개체) 자식 도형이 IR 로 복원되는지 — 합성 최소 문서로 검증.
//!
//! 종전 파서는 컨테이너(type 0)의 세부 정보 길이 8바이트를 읽지 않아 자식 스트림이
//! 8바이트 어긋났고(첫 자식이 conn=0 가짜 컨테이너로 읽혀 자식 전체 소실), 빈티지
//! 공통 헤더의 선언 길이(header_length)가 플래그 유도 소비량보다 커도 전진하지 않아
//! 형제 스트림이 무너졌다. 두 계약을 공개 API(`parse_hwp3`) 경로에서 고정한다.
//!
//! 합성 문서 레이아웃은 `mydocs/tech/한글문서파일구조3.0.md`(§10.7 그림, §11.3 개체
//! 정보)와 파서 리더 구조를 따른다. src 쪽 단위 테스트는 unit-test-tier 정책
//! (source-side 총량 동결)에 따라 이 통합 테스트로 옮겼다.

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;

// ---------------------------------------------------------------------------
// 합성 HWP3 문서 빌더 (최소 골격)
// ---------------------------------------------------------------------------

fn u16le(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}

fn u32le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

/// 30B 시그니처 + 128B 문서 정보(비압축·비암호) + 1008B 요약 + 본문.
fn hwp3_doc(body: &[u8]) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05");
    assert_eq!(d.len(), 30);
    let info = [0u8; 128]; // compressed(124)=0, encrypted(96..98)=0, info_block_length(126..128)=0
    d.extend_from_slice(&info);
    d.extend_from_slice(&[0u8; 1008]);
    d.extend_from_slice(body);
    d
}

/// 글꼴 7개 언어 그룹(각 1개, "바탕") + 스타일 0개 + 문단들 + 리스트 종료.
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

/// 문단 리스트 종료 마커 — follow_prev=0, char_count=0 뒤 40B 패딩(총 43B).
fn paragraph_list_end() -> [u8; 43] {
    [0u8; 43]
}

/// 대표 글자 모양 31B — 크기 10pt(raw 250), 장평 100.
fn char_shape31() -> [u8; 31] {
    let mut cs = [0u8; 31];
    cs[..2].copy_from_slice(&u16le(250));
    for r in &mut cs[9..16] {
        *r = 100;
    }
    cs
}

/// 문단 모양 187B — 줄간격 160%, 나머지 0.
fn para_shape187() -> [u8; 187] {
    let mut ps = [0u8; 187];
    ps[6..8].copy_from_slice(&u16le(160));
    ps[172] = 1; // column count
    ps
}

/// 문단 헤더(정보 + 대표 글자 모양 + 문단 모양) — follow_prev=0 고정.
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

/// 줄 정보 14B.
fn line_info(pgy: u16) -> Vec<u8> {
    let mut l = Vec::new();
    l.extend_from_slice(&u16le(0)); // start_pos
    l.extend_from_slice(&u16le(0)); // space_correction
    l.extend_from_slice(&u16le(400)); // line_height
    l.extend_from_slice(&u16le(pgy));
    l.extend_from_slice(&u16le(0)); // sx
    l.extend_from_slice(&u16le(0)); // psx
    l.extend_from_slice(&u16le(0)); // break_flag
    l
}

// ---------------------------------------------------------------------------
// 그리기 개체 (§11.3)
// ---------------------------------------------------------------------------

/// 개체 블록 — 공통 헤더(92 + extra_header) + 세부 정보.
/// header_length 는 자기 4바이트를 제외한 (88 + extra_header)로 선언한다.
fn shape_block(object_type: u16, conn: u16, extra_header: usize, detail: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&u32le((88 + extra_header) as u32));
    b.extend_from_slice(&u16le(object_type));
    b.extend_from_slice(&u16le(conn)); // bit0=형제, bit1=자식
    b.extend_from_slice(&[0u8; 40]); // relative/size/absolute/bounds
    b.extend_from_slice(&[0u8; 44]); // basic_attr (options=0)
    b.extend_from_slice(&vec![0u8; extra_header]);
    b.extend_from_slice(detail);
    b
}

/// 그리기 프레임(28B) + 개체들 → 그림(ch=11) 컨트롤의 추가 정보(ext) 블록.
fn drawing_ext(objects: &[u8]) -> Vec<u8> {
    let mut e = Vec::new();
    e.extend_from_slice(&u32le(24)); // frame header_length (<=24: 하이퍼텍스트 없음)
    e.extend_from_slice(&u32le(0)); // z_order
    e.extend_from_slice(&u32le(1)); // object_count
    e.extend_from_slice(&[0u8; 16]); // bounds
    e.extend_from_slice(objects);
    e
}

/// 그림(ch=11) 컨트롤을 품은 문단 — [식별 8B][그림 정보 348B][ext][캡션 리스트][0x0D].
///
/// char_count 계상: 식별 블록이 4 hchar, 말미 0x0D 가 1 hchar → cc=5.
fn drawing_paragraph(ext: &[u8]) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&para_header(5, 1));
    p.extend_from_slice(&line_info(100));
    // 식별 정보: [hchar 11][dword 예약][hchar 11]
    p.extend_from_slice(&u16le(11));
    p.extend_from_slice(&u32le(0));
    p.extend_from_slice(&u16le(11));
    // 그림 정보 348B
    let mut info = [0u8; 348];
    info[..4].copy_from_slice(&u32le(ext.len() as u32)); // 추가 정보 길이
    info[8] = 0; // 기준 위치: 글자 (treat_as_char)
    info[42..44].copy_from_slice(&u16le(1000)); // 박스 가로
    info[44..46].copy_from_slice(&u16le(800)); // 박스 세로
    info[74] = 3; // pic_type 3 = 그리기 개체
    p.extend_from_slice(&info);
    p.extend_from_slice(ext);
    p.extend_from_slice(&paragraph_list_end()); // 캡션 문단 리스트 (빈)
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p
}

/// 문서에서 첫 Shape 컨트롤을 꺼낸다.
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

// 컨테이너 세부 정보 길이 8바이트를 소비하지 않으면 자식 스트림이 8바이트 어긋나
// 첫 자식이 conn=0 인 가짜 컨테이너로 읽히고 묶음 자식 전체가 소실된다.
// 컨테이너(자식 있음) → 사각형(형제 있음) → 직선의 최소 트리가 자식 2개로
// 복원되는지 공개 API 경로로 확인한다.
#[test]
fn hwp3_group_children_survive_container_detail_lengths() {
    let mut objects = Vec::new();
    objects.extend_from_slice(&shape_block(0, 0x0002, 0, &[0u8; 8])); // 컨테이너, has_child
    objects.extend_from_slice(&shape_block(2, 0x0001, 0, &[0u8; 8])); // 사각형, has_sibling
    objects.extend_from_slice(&shape_block(1, 0x0000, 0, &[0u8; 12])); // 직선, 끝

    let bytes = hwp3_doc(&hwp3_body(&drawing_paragraph(&drawing_ext(&objects))));
    let doc = rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패");

    match first_shape(&doc) {
        ShapeObject::Group(g) => {
            assert_eq!(g.children.len(), 2, "묶음 자식 2개가 복원되어야 함");
            assert!(matches!(g.children[0], ShapeObject::Rectangle(_)));
            assert!(matches!(g.children[1], ShapeObject::Line(_)));
        }
        other => panic!("루트가 묶음이어야 함: {}", other.shape_name()),
    }
}

// 빈티지 공통 헤더는 플래그 유도 소비량보다 큰 확장 길이를 header_length 로
// 선언한다. 선언 끝까지 전진하지 않으면 확장 필드가 세부 정보/형제 헤더로
// 오독되어 트리가 무너진다.
#[test]
fn hwp3_vintage_extended_header_is_skipped_by_declared_length() {
    let mut objects = Vec::new();
    objects.extend_from_slice(&shape_block(0, 0x0002, 0, &[0u8; 8])); // 컨테이너, has_child
    objects.extend_from_slice(&shape_block(2, 0x0001, 12, &[0u8; 8])); // 확장 12B 사각형
    objects.extend_from_slice(&shape_block(1, 0x0000, 0, &[0u8; 12])); // 직선, 끝

    let bytes = hwp3_doc(&hwp3_body(&drawing_paragraph(&drawing_ext(&objects))));
    let doc = rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패");

    match first_shape(&doc) {
        ShapeObject::Group(g) => {
            assert_eq!(g.children.len(), 2, "확장 헤더 뒤 형제까지 복원되어야 함");
            assert!(matches!(g.children[0], ShapeObject::Rectangle(_)));
            assert!(matches!(g.children[1], ShapeObject::Line(_)));
        }
        other => panic!("루트가 묶음이어야 함: {}", other.shape_name()),
    }
}
