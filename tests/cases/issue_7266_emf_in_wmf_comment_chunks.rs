//! [Issue #7266] `OlePres000` 이 EMF-in-WMF 면 주석 청크 헤더가 EMF 안에 박힌다.
//!
//! ## 개체 안 구조 (2817919 실측)
//!
//! ```text
//!   BinData/ole1.ole  178,180 = [u32 길이 접두] + CFB
//!     \x01CompObj         88   "Picture (Enhanced Metafile)"
//!     \x02OlePres000  119,146   ← 미리보기. **EMF 가 아니라 WMF** 다.
//!     CONTENTS         54,672
//! ```
//!
//! `OlePres000` 은 offset 40 부터 표준 WMF(`01 00 09 00 00 03`)이고, EMF 는 그 안에
//! `META_ESCAPE`(func `0x0626`) + `META_ESCAPE_ENHANCED_METAFILE`(escape `0x000F`) 의
//! `WMFC` 주석 청크로 쪼개져 들어 있다.
//!
//! ```text
//!   rec 0 at 58     8,236B = 44B 헤더 + 8,192B 데이터  (CommentRecordCount=7,
//!   rec 1 at 8,294  8,236B                              CurrentRecordSize=8192,
//!   ...                                                 EnhancedMetafileDataSize=54,560)
//!   rec 6 at 49,474 5,452B = 44B 헤더 + 5,408B 데이터
//! ```
//!
//! 44B = WMF 레코드 헤더 6B + escape/count 4B + `EmfComment` 헤더 34B.
//!
//! ## 종전 결함
//!
//! `strip_ole_presentation_header` 가 `" EMF"` 를 **바이트 스캔해 그 뒤를 통째로 복사**했다.
//! 그래서 8,192바이트마다 청크 헤더 44B 가 박힌 가짜 EMF 가 나온다.
//!
//! - 2817919: `EMR_STRETCHDIBITS` 의 bottom-up DIB 안으로 44B 가 여러 번 들어가
//!   44B ÷ 2B/px = **22px** 씩 아래에서 위로 누적해 밀린다 — 로고가 띠로 뭉개진다.
//!   (사용자 신고 문구: "이미지가 뒤집어짐")
//! - 156564340(#6896 fixture): 선언 6,022,292B 중 6,022,272B 에서 레코드가 끊긴다.
//!   #6896 이 "잘린 EMF" 로 본 것이 사실은 이 섞임이었다.
//!
//! ## 수정
//!
//! `WMFC` 청크의 34B `EmfComment` 헤더를 걷어 데이터만 이어 붙인다. 복원본이
//! `EnhancedMetafileDataSize` 선언값과 `EMR_HEADER` 서명을 함께 만족할 때만 채택하고,
//! 아니면 종전 바이트 스캔으로 내려간다(단일 청크·순수 EMF 는 동작이 같다).
//!
//! 실측: 2817919 의 청크를 이어 붙이면 54,560B 가 나오고 같은 OLE 의 `CONTENTS` 가
//! 담은 EMF 와 **바이트 단위로 일치**한다.
//!
//! ## 곁가지 ② — 페이지 변환
//!
//! de-chunk 로 156564340 의 EMF 가 살아나자 4쪽이 **백지**가 됐다. 그 EMF 는
//! `MM_ANISOTROPIC` 에 window 37,094×52,391 / viewport 1,191×1,684 인데, EMF 플레이어가
//! `SetMapMode`·`SetWindow*`·`SetViewport*` 를 DC 에 담아만 두고 **아무도 소비하지
//! 않았다**. 논리 좌표가 장치 좌표인 양 31배로 그려져 셀에는 그림 왼쪽 위 3%(흰 여백)만
//! 들어왔다. 종전에는 이 EMF 가 파싱에 실패해 WMF 폴백으로 내려가 가려져 있었다.
//! `anisotropic_geometry_respects_physical_frame_and_target`가 이 계약을 잠근다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::ole_container::{
    contents_emf_payload, raw_contents_is_emf, strip_ole_presentation_header,
};

/// 실문서 `OlePres000` 의 선두 40바이트 (2817919). WMF 헤더는 그 뒤에서 시작한다.
const PRESENTATION_HEADER: [u8; 40] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0xFF, 0xFF, 0xFF, 0xFF, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCB, 0x21, 0x00, 0x00,
    0x9D, 0x06, 0x00, 0x00, 0xFC, 0xD0, 0x01, 0x00,
];

/// 실문서와 같은 청크 크기 (`CurrentRecordSize`).
const CHUNK: usize = 8_192;

/// `EMR_HEADER`(108B) + `EMR_COMMENT`(가변) + `EMR_EOF`(20B) 의 최소 EMF.
///
/// 기대값을 구현과 독립으로 두기 위해 **여기서 만든 바이트 그대로**가 복원 결과여야 한다.
fn emf_bytes(total: usize) -> Vec<u8> {
    assert!(total > 108 + 20 && total.is_multiple_of(4));
    let comment = total - 108 - 20;

    let mut emf = Vec::with_capacity(total);
    let push = |value: u32, out: &mut Vec<u8>| out.extend_from_slice(&value.to_le_bytes());

    // EMR_HEADER
    push(1, &mut emf); // iType
    push(108, &mut emf); // nSize
    for bound in [0u32, 0, 10, 10, 0, 0, 1000, 1000] {
        push(bound, &mut emf); // rclBounds + rclFrame
    }
    emf.extend_from_slice(b" EMF"); // dSignature  (offset 40)
    push(0x0001_0000, &mut emf); // nVersion
    push(total as u32, &mut emf); // nBytes
    push(3, &mut emf); // nRecords
    emf.extend_from_slice(&0u16.to_le_bytes()); // nHandles
    emf.extend_from_slice(&0u16.to_le_bytes()); // sReserved
    for tail in [0u32, 0, 0, 1920, 1080, 527, 296, 0, 0, 0] {
        push(tail, &mut emf); // nDescription..szlMillimeters + 예약부
    }
    emf.resize(108, 0);

    // EMR_COMMENT — 청크 경계를 넘기는 덩치. 데이터는 위치를 알아볼 수 있게 순번으로 채운다.
    push(70, &mut emf);
    push(comment as u32, &mut emf);
    push((comment - 12) as u32, &mut emf);
    for i in 0..(comment - 12) {
        emf.push((i % 251) as u8);
    }

    // EMR_EOF
    push(14, &mut emf);
    push(20, &mut emf);
    push(0, &mut emf);
    push(0x10, &mut emf);
    push(20, &mut emf);

    assert_eq!(emf.len(), total);
    emf
}

/// EMF 를 `WMFC` 주석 청크로 쪼개 표준 WMF 로 감싸고, OLE 프레젠테이션 헤더를 붙인다.
fn presentation_with_emf_in_wmf(emf: &[u8]) -> Vec<u8> {
    let chunks: Vec<&[u8]> = emf.chunks(CHUNK).collect();

    let mut wmf = Vec::new();
    // METAHEADER (18B)
    wmf.extend_from_slice(&1u16.to_le_bytes()); // mtType = memory
    wmf.extend_from_slice(&9u16.to_le_bytes()); // mtHeaderSize (words)
    wmf.extend_from_slice(&0x0300u16.to_le_bytes()); // mtVersion
    wmf.extend_from_slice(&0u32.to_le_bytes()); // mtSize (뒤에서 채운다)
    wmf.extend_from_slice(&0u16.to_le_bytes()); // mtNoObjects
    wmf.extend_from_slice(&0u32.to_le_bytes()); // mtMaxRecord
    wmf.extend_from_slice(&0u16.to_le_bytes()); // mtNoParameters

    let mut remaining = emf.len();
    for chunk in &chunks {
        remaining -= chunk.len();
        let count = 34 + chunk.len();
        let record = 10 + count;
        assert!(record.is_multiple_of(2));
        wmf.extend_from_slice(&((record / 2) as u32).to_le_bytes()); // rdSize (words)
        wmf.extend_from_slice(&0x0626u16.to_le_bytes()); // META_ESCAPE
        wmf.extend_from_slice(&0x000Fu16.to_le_bytes()); // ENHANCED_METAFILE
        wmf.extend_from_slice(&(count as u16).to_le_bytes()); // byte count
        wmf.extend_from_slice(b"WMFC"); // CommentIdentifier
        wmf.extend_from_slice(&1u32.to_le_bytes()); // CommentType
        wmf.extend_from_slice(&0x0001_0000u32.to_le_bytes()); // Version
        wmf.extend_from_slice(&0u16.to_le_bytes()); // Checksum
        wmf.extend_from_slice(&0u32.to_le_bytes()); // Flags
        wmf.extend_from_slice(&(chunks.len() as u32).to_le_bytes()); // CommentRecordCount
        wmf.extend_from_slice(&(chunk.len() as u32).to_le_bytes()); // CurrentRecordSize
        wmf.extend_from_slice(&(remaining as u32).to_le_bytes()); // RemainingBytes
        wmf.extend_from_slice(&(emf.len() as u32).to_le_bytes()); // EnhancedMetafileDataSize
        wmf.extend_from_slice(chunk);
    }
    // META_EOF
    wmf.extend_from_slice(&3u32.to_le_bytes());
    wmf.extend_from_slice(&0u16.to_le_bytes());

    let size_words = (wmf.len() / 2) as u32;
    wmf[6..10].copy_from_slice(&size_words.to_le_bytes());

    let mut out = PRESENTATION_HEADER.to_vec();
    out.extend_from_slice(&wmf);
    out
}

/// 여러 청크로 쪼개진 EMF 를 이어 붙여 **원본 그대로** 복원한다.
///
/// 수정 전에는 `" EMF"` 바이트 스캔 결과(= 청크 헤더 44B 가 박힌 스트림 전체)를 돌려줘
/// 길이부터 어긋났다.
#[test]
fn emf_in_wmf_comment_chunks_are_reassembled() {
    let emf = emf_bytes(20_000);
    assert!(
        emf.len() > CHUNK * 2,
        "청크가 셋 이상이어야 경계가 검증된다"
    );
    let presentation = presentation_with_emf_in_wmf(&emf);

    let restored = strip_ole_presentation_header(&presentation).expect("EMF 미리보기 복원");

    assert_eq!(
        restored.len(),
        emf.len(),
        "복원 길이가 원본 EMF 와 다르다 — 청크 헤더 44B 가 섞였다는 뜻이다 \
         (스트림 전체 길이는 {})",
        presentation.len()
    );
    assert_eq!(restored, emf, "복원 바이트가 원본 EMF 와 다르다");
}

/// 청크가 하나뿐(= EMF 가 8KB 이하)이어도 EMF 만 정확히 잘라 낸다 — 경계 하한.
///
/// 이때 종전 바이트 스캔은 청크 헤더를 **안** 섞지만 `WMFC` 뒤의 WMF 꼬리
/// (`META_EOF` 6B)를 EMF 에 붙여 돌려줬다. 렌더는 `EMR_EOF` 에서 멈춰 무해했으나
/// 경계는 이쪽이 정확하다.
#[test]
fn a_single_chunk_presentation_is_unchanged() {
    let emf = emf_bytes(4_096);
    assert!(emf.len() <= CHUNK, "한 청크에 들어가야 경계가 성립한다");
    let presentation = presentation_with_emf_in_wmf(&emf);

    let restored = strip_ole_presentation_header(&presentation).expect("EMF 미리보기 복원");
    assert_eq!(restored, emf);
}

/// `WMFC` 주석이 없는 평범한 EMF 프레젠테이션은 종전 바이트 스캔 그대로 동작한다.
#[test]
fn a_plain_emf_presentation_still_falls_back_to_the_byte_scan() {
    let emf = emf_bytes(4_096);
    let mut presentation = PRESENTATION_HEADER.to_vec();
    presentation.extend_from_slice(&emf);

    let restored = strip_ole_presentation_header(&presentation).expect("EMF 미리보기 복원");
    assert_eq!(restored, emf);
}

/// [#7266 곁가지] `CONTENTS` 의 `u32` 길이 접두 변형도 EMF 로 인식한다.
///
/// 2817919 의 `CONTENTS` 는 `6C 00 00 00`(= 뒤따르는 `EMR_HEADER` 사본 108B) + 사본 +
/// 본 EMF 로 시작한다. `data[0..4] == 1` 만 보면 이 갈래를 통째로 놓쳐, 미리보기가
/// 실패해도 폴백이 못 받는다.
#[test]
fn contents_with_a_u32_length_prefix_is_recognised_as_emf() {
    let emf = emf_bytes(4_096);

    // 접두 없는 평범한 CONTENTS
    assert!(raw_contents_is_emf(&emf));
    assert_eq!(contents_emf_payload(&emf), Some(emf.as_slice()));

    // 2817919 변형: [u32 = 108] + EMR_HEADER 사본 + 본 EMF
    let mut prefixed = 108u32.to_le_bytes().to_vec();
    prefixed.extend_from_slice(&emf[..108]);
    prefixed.extend_from_slice(&emf);
    assert!(
        raw_contents_is_emf(&prefixed),
        "u32 길이 접두가 붙은 CONTENTS 를 EMF 로 못 알아본다"
    );
    assert_eq!(contents_emf_payload(&prefixed), Some(&prefixed[4..]));

    // EMF 가 아닌 페이로드는 그대로 거른다.
    assert!(!raw_contents_is_emf(b"Hwp 5.0 Equation Editor(HwpEq5x)"));
    assert!(contents_emf_payload(&[0u8; 200]).is_none());
}

/// [#7266 ②] EMF 의 페이지 변환(map mode + window/viewport)이 실제 좌표에 걸린다.
///
/// 156564340 의 포스터 EMF 는 `MM_ANISOTROPIC` 에 window 37,094×52,391 /
/// viewport 1,191×1,684 이다. 종전에는 이 레코드를 DC 에 담아만 두고 **아무도 쓰지
/// 않아서** 논리 좌표가 장치 좌표인 양 31배로 그려졌다 — de-chunk 로 EMF 가 살아나자
/// 그림의 왼쪽 위 3%만 셀에 들어와 4쪽이 백지가 됐다(WMF 폴백일 때는 가려져 있었다).
///
/// 여기서는 배율 10:1 로 축약해, 논리 1000×1000 사각형이 장치 100×100 에 정확히
/// 들어앉는지 **합성 변환을 실제로 계산해** 확인한다. 수정 전에는 1000×1000 이 나온다.
#[test]
fn anisotropic_geometry_respects_physical_frame_and_target() {
    for (window, viewport, target, frame) in [
        (1000, 100, 100.0_f32, 1000),
        (600, 150, 75.0, 1000),
        (1000, 100, 200.0, 2000),
    ] {
        let emf = emf_with_anisotropic_page(window, viewport, frame);
        let svg = rhwp::emf::convert_to_svg(&emf, (0.0, 0.0, target, target)).expect("EMF → SVG");
        let (x, y, w, h) = device_geometry(&svg);
        // 참조 장치는 10mm당 viewport 픽셀이다. Frame은 0.01mm 단위이므로
        // 논리 사각형이 채우는 viewport의 물리 10mm를 목표 영역에 대응시킨다.
        // 세 번째 사례는 잉크 Bounds 대신 물리 Frame을 사용해야 절반을 차지한다.
        let expected = target * 1000.0 / frame as f32;
        let bad = format!(
            "window={window}, viewport={viewport}, frame={frame}, target={target}: \
             도형 경계 {x} {y} {w} {h}, 기대 크기 {expected} — svg={svg}"
        );
        assert!(x.abs() < 0.5 && y.abs() < 0.5, "{bad}");
        assert!(
            (w - expected).abs() < 0.5 && (h - expected).abs() < 0.5,
            "{bad}"
        );
    }
}

/// `MM_ANISOTROPIC` + window/viewport + 논리 사각형 하나를 담은 최소 EMF.
fn emf_with_anisotropic_page(window: i32, viewport: i32, frame: i32) -> Vec<u8> {
    const MM_ANISOTROPIC: u32 = 8;
    let total = 108 + 12 + 16 + 16 + 16 + 24 + 20;

    let mut emf = Vec::with_capacity(total);
    let push = |value: i32, out: &mut Vec<u8>| out.extend_from_slice(&value.to_le_bytes());

    // EMR_HEADER — Bounds는 장치 픽셀, Frame은 물리 0.01mm 단위다.
    push(1, &mut emf);
    push(108, &mut emf);
    for bound in [0, 0, viewport, viewport, 0, 0, frame, frame] {
        push(bound, &mut emf);
    }
    emf.extend_from_slice(b" EMF");
    push(0x0001_0000, &mut emf);
    push(total as i32, &mut emf);
    push(6, &mut emf);
    emf.extend_from_slice(&0u16.to_le_bytes());
    emf.extend_from_slice(&0u16.to_le_bytes());
    for tail in [0, 0, 0, viewport, viewport, 10, 10, 0, 0, 0] {
        push(tail, &mut emf);
    }
    emf.resize(108, 0);

    // EMR_SETMAPMODE / SETWINDOWORGEX / SETWINDOWEXTEX / SETVIEWPORTEXTEX
    push(17, &mut emf);
    push(12, &mut emf);
    emf.extend_from_slice(&MM_ANISOTROPIC.to_le_bytes());
    push(10, &mut emf);
    push(16, &mut emf);
    push(0, &mut emf);
    push(0, &mut emf);
    push(9, &mut emf);
    push(16, &mut emf);
    push(window, &mut emf);
    push(window, &mut emf);
    push(11, &mut emf);
    push(16, &mut emf);
    push(viewport, &mut emf);
    push(viewport, &mut emf);

    // EMR_RECTANGLE — 논리 좌표로 window 를 꽉 채운다.
    push(43, &mut emf);
    push(24, &mut emf);
    push(0, &mut emf);
    push(0, &mut emf);
    push(window, &mut emf);
    push(window, &mut emf);

    // EMR_EOF
    push(14, &mut emf);
    push(20, &mut emf);
    push(0, &mut emf);
    push(0x10, &mut emf);
    push(20, &mut emf);

    assert_eq!(emf.len(), total);
    emf
}

/// 도형 태그나 행렬 문자열의 형태 대신 최종 기하 경계를 검사한다.
fn device_geometry(svg: &str) -> (f32, f32, f32, f32) {
    fn collect(group: &usvg::Group, boxes: &mut Vec<usvg::Rect>) {
        for node in group.children() {
            match node {
                usvg::Node::Group(child) => collect(child, boxes),
                usvg::Node::Path(path) => boxes.push(path.abs_bounding_box()),
                _ => panic!("합성 사각형 외의 출력 노드: {node:?}"),
            }
        }
    }
    let document = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"1000\">{svg}</svg>"
    );
    let tree = usvg::Tree::from_str(&document, &usvg::Options::default()).expect("SVG 기하 해석");
    let mut boxes = Vec::new();
    collect(tree.root(), &mut boxes);
    assert_eq!(boxes.len(), 1, "합성 사각형이 누락되거나 중복됐다");
    let rect = boxes[0];
    (rect.x(), rect.y(), rect.width(), rect.height())
}
