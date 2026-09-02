//! [Issue #6577] EMF 플레이어가 `EMR_POLYLINETO16` · `EMR_POLYBEZIERTO16` 를 처리하지
//! 않아, **패스 기반 EMF** 의 도형이 통째로 사라지던 결함의 가드.
//!
//! 패스 기반 EMF(Office·Illustrator 산출물)는 도형을 거의 전부
//! `BeginPath → MoveTo → {PolylineTo16 | PolyBezierTo16}* → CloseFigure → FillPath`
//! 로 표현한다. 종전에는
//!
//! - 두 `…To16` 레코드가 `Record::Unknown` 으로 **버려지고**,
//! - `MoveToEx` 는 `current_pos` 만 갱신하고 `path_d` 를 건드리지 않아
//!
//! `FillPath` 가 **빈 패스**를 채웠다. 즉 채움 도형이 하나도 안 나온다.
//!
//! 실측 — `156627451` 의 WMF 안에 실린 EMF(`META_ESCAPE_ENHANCED_METAFILE` 재조립,
//! 306,980 bytes)의 레코드 분포:
//!
//! ```text
//! POLYBEZIERTO16  758      POLYLINETO16  644      (합 1,402건이 전부 버려졌다)
//! MOVETOEX        693      CLOSEFIGURE   689      BEGINPATH/ENDPATH 각 116
//! ```
//!
//! ⚠ 이 수정은 EMF 렌더러 자체를 고친 것이고, WMF 안에 실린 EMF 를 **쓰도록 배선한
//! 것은 아니다**(#6577 은 그 배선이 본체). 배선을 켜기 전에 색·클리핑·텍스트가 더
//! 필요하다 — 지금 켜면 도형이 단색 굵은 획으로 뭉친다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::emf::parser::records::Record;

/// `BeginPath → MoveTo(10,10) → PolylineTo16{(20,10)} → PolyBezierTo16{…} → EndPath`
/// 형태의 최소 EMF 를 만들어, 두 `…To16` 이 `Unknown` 이 아니라 제 변형으로 파싱되는지
/// 확인한다.
fn record_types(emf: &[u8]) -> Vec<String> {
    rhwp::emf::parser::parse(emf)
        .map(|records| {
            records
                .iter()
                .map(|record| match record {
                    Record::PolylineTo16 { .. } => "PolylineTo16".to_string(),
                    Record::PolyBezierTo16 { .. } => "PolyBezierTo16".to_string(),
                    Record::Unknown { .. } => "Unknown".to_string(),
                    other => format!("{other:?}")
                        .split_whitespace()
                        .next()
                        .unwrap_or("?")
                        .trim_end_matches('(')
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_record(out: &mut Vec<u8>, kind: u32, payload: &[u8]) {
    let size = 8 + payload.len() as u32;
    push_u32(out, kind);
    push_u32(out, size);
    out.extend_from_slice(payload);
}

#[test]
fn polyline_to16_and_polybezier_to16_are_parsed() {
    // EMR_HEADER: iType=1, 최소 88바이트. offset 40 에 " EMF" 서명.
    let mut emf = Vec::new();
    let mut header = vec![0u8; 80];
    header[32..36].copy_from_slice(b" EMF"); // payload 기준 offset 40
    push_record(&mut emf, 1, &header);

    // EMR_POLYLINETO16 (0x59): bounds(16) + count(4) + points
    let mut payload = vec![0u8; 16];
    push_u32(&mut payload, 1);
    payload.extend_from_slice(&20i16.to_le_bytes());
    payload.extend_from_slice(&10i16.to_le_bytes());
    push_record(&mut emf, 0x59, &payload);

    // EMR_POLYBEZIERTO16 (0x58): 제어점 2 + 끝점 1
    let mut payload = vec![0u8; 16];
    push_u32(&mut payload, 3);
    for (x, y) in [(25i16, 5i16), (35, 5), (40, 10)] {
        payload.extend_from_slice(&x.to_le_bytes());
        payload.extend_from_slice(&y.to_le_bytes());
    }
    push_record(&mut emf, 0x58, &payload);

    // EMR_EXTCREATEPEN (0x5F): ihPen + offBmi/cbBmi/offBits/cbBits + LogPenEx 앞 4필드
    let mut payload = Vec::new();
    push_u32(&mut payload, 7); // ihPen
    for _ in 0..4 {
        push_u32(&mut payload, 0);
    }
    push_u32(&mut payload, 0); // PenStyle = PS_SOLID
    push_u32(&mut payload, 3); // Width
    push_u32(&mut payload, 0); // BrushStyle = BS_SOLID
    push_u32(&mut payload, 0x0000_00FF); // ColorRef
    push_u32(&mut payload, 0); // BrushHatch
    push_u32(&mut payload, 0); // NumStyleEntries
    push_record(&mut emf, 0x5F, &payload);

    push_record(&mut emf, 14, &[]); // EMR_EOF (내용 없이도 파서가 멈추면 됨)

    let types = record_types(&emf);
    assert!(
        types.iter().any(|t| t == "PolylineTo16"),
        "EMR_POLYLINETO16 이 Unknown 으로 버려졌다 — #6577 회귀. 파싱 결과: {types:?}"
    );
    assert!(
        types.iter().any(|t| t == "PolyBezierTo16"),
        "EMR_POLYBEZIERTO16 이 Unknown 으로 버려졌다 — #6577 회귀. 파싱 결과: {types:?}"
    );
    // [#6577 ③] 이 파일군의 비-스톡 펜은 전부 EXTCREATEPEN 이다(EMR_CREATEPEN 0건).
    assert!(
        types.iter().any(|t| t == "CreatePen"),
        "EMR_EXTCREATEPEN 이 Unknown 으로 버려졌다 — #6577 회귀. 파싱 결과: {types:?}"
    );
}
