//! [#6919] `META_SETTEXTALIGN` 이 없는 metafile 의 DC 기본 세로 정렬은 `TA_TOP` 이다.
//!
//! [MS-WMF] 2.1.2.18 의 기본값은 `TA_LEFT | TA_TOP | TA_NOUPDATECP`(= `0x0000`)이고
//! `TA_BASELINE`(0x0018)이 아니다. 곧 `META_EXTTEXTOUT` 의 `y` 는 **글자 셀의 위끝**이며
//! baseline 은 `y + ascent` 다.
//!
//! rhwp 의 DC 기본값은 `VTA_BASELINE` 이었다. `SetTextAlign` 을 부르는 metafile 은
//! 그 핸들러(`v_bits == 0 → VTA_TOP`)가 옳게 매핑했지만, **부르지 않는 metafile** 은
//! `#965` 가 구현해 둔 cell-top 보정(`+0.8em`)에 도달하지 못해 글자가 통째로 한 줄
//! 위로 올라갔다.
//!
//! `148726703`(산업활동동향 브리핑) 6쪽 차트 실측 — WMF viewBox 341×228 기준:
//!
//! ```text
//!   제목 레코드  META_EXTTEXTOUT x=110 y=13,  font height = -12
//!   GDI+ (정답지)   글자 잉크 y =  9.5 .. 20.0   테두리(y=5) 아래
//!   수정 전         <text y="13">                테두리를 글자가 가로지른다
//!   수정 후         <text y="22">                = 13 + 0.8 × 12
//! ```
//!
//! 이 시험은 문서를 쓰지 않고 **합성 metafile 세 개**로 계약을 직접 잠근다 —
//! `SetTextAlign` 없음 ≡ 명시 `TA_TOP(0x0000)`, 그리고 `TA_BASELINE(0x0018)` 은 불변.

use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

const META_SETTEXTALIGN: u16 = 0x012E;
const META_EXTTEXTOUT: u16 = 0x0A32;
const META_CREATEFONTINDIRECT: u16 = 0x02FB;
const META_SELECTOBJECT: u16 = 0x012D;

/// 글자 높이(HWP 아닌 metafile 논리 단위). 부호는 셀/문자 해석만 바꾼다.
const FONT_HEIGHT: i16 = -12;
/// `META_EXTTEXTOUT` 의 세로 기준점.
const TEXT_Y: i16 = 13;

fn push_header(b: &mut Vec<u8>) {
    b.extend_from_slice(&1u16.to_le_bytes()); // Type: memory metafile
    b.extend_from_slice(&9u16.to_le_bytes()); // HeaderSize (words)
    b.extend_from_slice(&0x0300u16.to_le_bytes()); // Version
    b.extend_from_slice(&0u16.to_le_bytes()); // SizeLow
    b.extend_from_slice(&0u16.to_le_bytes()); // SizeHigh
    b.extend_from_slice(&1u16.to_le_bytes()); // NumberOfObjects — 글꼴 한 개
    b.extend_from_slice(&0u32.to_le_bytes()); // MaxRecord
    b.extend_from_slice(&0u16.to_le_bytes()); // NumberOfMembers
}

fn push_eof(b: &mut Vec<u8>) {
    b.extend_from_slice(&3u32.to_le_bytes());
    b.extend_from_slice(&0u16.to_le_bytes());
}

fn push_create_font(b: &mut Vec<u8>) {
    // RecordSize(2) + Function(1) + Font(25 words) = 28 words
    b.extend_from_slice(&28u32.to_le_bytes());
    b.extend_from_slice(&META_CREATEFONTINDIRECT.to_le_bytes());
    b.extend_from_slice(&FONT_HEIGHT.to_le_bytes()); // Height
    b.extend_from_slice(&0i16.to_le_bytes()); // Width
    b.extend_from_slice(&0i16.to_le_bytes()); // Escapement
    b.extend_from_slice(&0i16.to_le_bytes()); // Orientation
    b.extend_from_slice(&400i16.to_le_bytes()); // Weight
    b.extend_from_slice(&[0u8; 8]); // Italic..Quality, PitchAndFamily
    b.extend_from_slice(&[0u8; 32]); // Facename
}

fn push_select_object(b: &mut Vec<u8>, index: u16) {
    b.extend_from_slice(&4u32.to_le_bytes());
    b.extend_from_slice(&META_SELECTOBJECT.to_le_bytes());
    b.extend_from_slice(&index.to_le_bytes());
}

fn push_set_text_align(b: &mut Vec<u8>, mode: u16) {
    b.extend_from_slice(&4u32.to_le_bytes());
    b.extend_from_slice(&META_SETTEXTALIGN.to_le_bytes());
    b.extend_from_slice(&mode.to_le_bytes());
}

fn push_ext_text_out(b: &mut Vec<u8>) {
    // RecordSize(2) + Function(1) + Y + X + Len + Opts + String(1 word) = 8 words
    b.extend_from_slice(&8u32.to_le_bytes());
    b.extend_from_slice(&META_EXTTEXTOUT.to_le_bytes());
    b.extend_from_slice(&TEXT_Y.to_le_bytes());
    b.extend_from_slice(&110i16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes()); // StringLength
    b.extend_from_slice(&0u16.to_le_bytes()); // fwOpts
    b.extend_from_slice(b"AB");
}

fn metafile(align: Option<u16>) -> Vec<u8> {
    let mut b = Vec::new();
    push_header(&mut b);
    push_create_font(&mut b);
    push_select_object(&mut b, 0);
    if let Some(mode) = align {
        push_set_text_align(&mut b, mode);
    }
    push_ext_text_out(&mut b);
    push_eof(&mut b);
    b
}

fn text_y(align: Option<u16>) -> f64 {
    let bytes = metafile(align);
    let out = WMFConverter::new(&*bytes, SVGPlayer::new())
        .run()
        .expect("WMF 변환");
    let svg = String::from_utf8_lossy(&out).into_owned();
    let idx = svg.find("<text").expect("text 노드가 있어야 한다");
    let tail = &svg[idx..];
    let key = " y=\"";
    let at = tail.find(key).expect("text 의 y 속성");
    let rest = &tail[at + key.len()..];
    let end = rest.find('"').expect("y 값 끝");
    rest[..end].parse::<f64>().expect("y 는 수")
}

#[test]
fn issue_6919_missing_set_text_align_matches_explicit_ta_top() {
    // [MS-WMF] 2.1.2.18: 기본값 = TA_LEFT | TA_TOP | TA_NOUPDATECP = 0x0000.
    // 곧 `SetTextAlign` 이 없는 것과 명시적 `0x0000` 은 **같은 상태**여야 한다.
    let implicit = text_y(None);
    let explicit_top = text_y(Some(0x0000));
    assert!(
        (implicit - explicit_top).abs() < 0.001,
        "SetTextAlign 없음과 명시 TA_TOP 은 같아야 한다: {implicit} vs {explicit_top}"
    );

    // cell-top 보정이 실제로 걸린다 — y 는 레코드 값보다 아래(큰 값)다.
    assert!(
        implicit > f64::from(TEXT_Y) + 0.5,
        "기본값에서 cell-top 보정이 걸려야 한다: y={implicit} (레코드 {TEXT_Y})"
    );
}

#[test]
fn issue_6919_explicit_ta_baseline_is_unchanged() {
    // **음성 대조** — `TA_BASELINE`(0x0018)을 명시한 metafile 은 종전 그대로
    // 레코드 `y` 를 baseline 으로 쓴다. 이 수정이 기본값 경로만 건드린다는 것을 잠근다.
    let baseline = text_y(Some(0x0018));
    assert!(
        (baseline - f64::from(TEXT_Y)).abs() < 0.001,
        "명시 TA_BASELINE 은 레코드 y 를 그대로 쓴다: {baseline} (기대 {TEXT_Y})"
    );
}

/// SVG 안 모든 `<text>` 의 y 를 모은다.
fn text_ys(bytes: &[u8]) -> Vec<f64> {
    let out = WMFConverter::new(bytes, SVGPlayer::new())
        .run()
        .expect("WMF 변환");
    let svg = String::from_utf8_lossy(&out).into_owned();
    svg.match_indices("<text")
        .filter_map(|(i, _)| {
            let tail = &svg[i..];
            let at = tail.find(" y=\"")? + 4;
            let rest = &tail[at..];
            let end = rest.find('"')?;
            rest[..end].parse::<f64>().ok()
        })
        .collect()
}

#[test]
fn issue_6919_real_chart_preview_drops_every_glyph_by_the_cell_top_correction() {
    // 실제 재현물 — `148726703` 6쪽 차트의 OLE 미리보기 WMF 다.
    // `META_SETTEXTALIGN` 이 하나도 없고 글자 레코드가 9개다(추출 시 실측).
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6919/148726703-chart-preview.wmf");
    let original = std::fs::read(path).expect("정식 재현물");

    // 같은 metafile 머리 뒤에 `SetTextAlign(TA_BASELINE)` 하나만 끼운 변형.
    // 종전 기본값이 하던 일을 명시적으로 시킨 것이라, 그 y 가 곧 **수정 전 출력**이다.
    let mut baseline_variant = Vec::with_capacity(original.len() + 8);
    baseline_variant.extend_from_slice(&original[..18]); // 표준 WMF 머리(9 words)
    baseline_variant.extend_from_slice(&4u32.to_le_bytes());
    baseline_variant.extend_from_slice(&META_SETTEXTALIGN.to_le_bytes());
    baseline_variant.extend_from_slice(&0x0018u16.to_le_bytes());
    baseline_variant.extend_from_slice(&original[18..]);

    let now = text_ys(&original);
    let before = text_ys(&baseline_variant);
    assert!(!now.is_empty(), "차트 글자가 방출돼야 한다");
    assert_eq!(now.len(), before.len(), "글자 수는 같아야 한다");

    // 기본값 경로가 cell-top 보정을 태우므로 **모든 글자가 내려간다** — 한 줄 위로
    // 올라가 테두리·축선·범례 표식과 겹치던 것이 이 차이다.
    for (after_y, before_y) in now.iter().zip(before.iter()) {
        assert!(
            *after_y > *before_y + 0.5,
            "기본값에서 글자가 셀 위끝 기준으로 내려가야 한다: {after_y} vs {before_y}"
        );
    }
}
