//! Synthetic V3 header/footer field contracts through parse_hwp3; not a visual fixture.
use rhwp::model::control::Control;
use rhwp::model::header_footer::HeaderFooterApply;
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

fn parsed(kind: u8, apply: u8) -> Control {
    let text = "source footer";
    let mut nested = para_header(text.len() as u16 + 1, 1);
    nested.extend_from_slice(&line_info(0));
    for ch in text.encode_utf16().chain(std::iter::once(13)) {
        nested.extend_from_slice(&u16le(ch));
    }
    nested.extend_from_slice(&paragraph_list_end());
    let mut host = para_header(5, 1);
    host.extend_from_slice(&line_info(100));
    host.extend_from_slice(&u16le(16));
    host.extend_from_slice(&u32le(0));
    host.extend_from_slice(&u16le(16));
    let mut info = [0; 10];
    info[8] = kind;
    info[9] = apply;
    host.extend_from_slice(&info);
    host.extend_from_slice(&nested);
    host.extend_from_slice(&u16le(13));
    let doc = rhwp::parser::hwp3::parse_hwp3(&hwp3_doc(&hwp3_body(&host)))
        .expect("synthetic V3 header/footer");
    let controls: Vec<_> = doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::Header(_) | Control::Footer(_)))
        .cloned()
        .collect();
    assert_eq!(
        controls.len(),
        1,
        "one source control without loss or duplication"
    );
    controls.into_iter().next().unwrap()
}
#[test]
fn footer_kind_owns_bottom_alignment_and_keeps_scope_and_content() {
    let Control::Footer(footer) = parsed(1, 1) else {
        panic!("footer source kind")
    };
    assert_eq!((footer.list_attr >> 21) & 3, 2);
    assert_eq!(footer.apply_to, HeaderFooterApply::Even);
    assert_eq!(&footer.raw_ctrl_extra[8..], &[1, 1]);
    assert_eq!(footer.paragraphs.len(), 1);
    assert_eq!(footer.paragraphs[0].text, "source footer");
    assert_eq!(footer.text_height, 0, "no invented band height");
}
#[test]
fn header_kind_keeps_its_top_anchor_instead_of_footer_alignment() {
    let Control::Header(header) = parsed(0, 2) else {
        panic!("header source kind")
    };
    assert_eq!((header.list_attr >> 21) & 3, 0);
    assert_eq!(header.apply_to, HeaderFooterApply::Odd);
    assert_eq!(&header.raw_ctrl_extra[8..], &[0, 2]);
    assert_eq!(header.paragraphs[0].text, "source footer");
}
