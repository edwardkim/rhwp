//! Issue #3570: 표·연결선 레코드의 0 패딩 길이가 한컴 저장본과 다르다.
//!
//! 한컴이 같은 원본을 저장한 파일을 정답지로 삼아 맞췄다.
//!
//! | 레코드 | 종전 | 한컴 | 조치 |
//! |---|---|---|---|
//! | `TABLE`(77) | rhwp 가 **+2** | zone 개수까지만 | 여분 2바이트 제거 |
//! | `SC_LINE`(78) 연결선 | rhwp 가 **−4** | 제어점 뒤 0 4바이트 | 4바이트 채움 |
//!
//! 둘 다 0 으로 채워진 패딩이라 정보 손실은 없었지만, 길이가 달라 다른 도구와의 대조·이식이
//! 어긋났다. 표 쪽은 종전에 어댑터가 `raw_table_record_extra = [0, 0]` 을 **의도적으로**
//! 넣던 계약을 되돌린 것이라, 한컴 개방까지 실측해 확인했다(387쪽 편람 → 384쪽 정상 개방).
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

/// 표와 연결선을 함께 가진 표본.
const SAMPLE: &str = "samples/task1771/nested_group_vectors.hwpx";

const TAG_TABLE: u16 = 77;
const TAG_SC_LINE: u16 = 78;

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 저장 바이트에서 BodyText 전 구역의 (태그, 데이터) 를 읽는다.
fn body_records(hwp: &[u8]) -> Vec<(u16, Vec<u8>)> {
    let cursor = std::io::Cursor::new(hwp);
    let mut comp = cfb::CompoundFile::open(cursor).expect("cfb 열기");
    let mut fh = comp.open_stream("/FileHeader").expect("FileHeader");
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).expect("FileHeader 읽기");
    let compressed = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);

    let names: Vec<String> = comp
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_string_lossy().replace('\\', "/"))
        .filter(|p| p.starts_with("/BodyText/Section"))
        .collect();

    let mut out = Vec::new();
    for name in names {
        let mut s = comp.open_stream(&name).expect("Section");
        let mut raw = Vec::new();
        s.read_to_end(&mut raw).expect("Section 읽기");
        drop(s);
        let data = if compressed {
            let mut d = flate2::read::DeflateDecoder::new(&raw[..]);
            let mut v = Vec::new();
            d.read_to_end(&mut v).expect("inflate");
            v
        } else {
            raw
        };
        let mut pos = 0usize;
        while pos + 4 <= data.len() {
            let header = u32::from_le_bytes(data[pos..pos + 4].try_into().expect("헤더"));
            let tag = (header & 0x3FF) as u16;
            let mut size = ((header >> 20) & 0xFFF) as usize;
            pos += 4;
            if size == 0xFFF {
                if pos + 4 > data.len() {
                    break;
                }
                size = u32::from_le_bytes(data[pos..pos + 4].try_into().expect("확장")) as usize;
                pos += 4;
            }
            if pos + size > data.len() {
                break;
            }
            out.push((tag, data[pos..pos + size].to_vec()));
            pos += size;
        }
    }
    out
}

/// CLI `convert` 와 동일한 경로로 저장한다.
///
/// 표의 여분 2바이트는 **HWPX→HWP 어댑터**(`materialize_table_record_extra`)가 넣던
/// 것이라, 어댑터를 건너뛰고 직렬화하면 이 테스트가 수정 여부와 무관하게 통과한다.
fn saved_records() -> Vec<(u16, Vec<u8>)> {
    use rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source;
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let mut doc = rhwp::parser::parse_document(&bytes).expect("파싱");
    let _ = convert_if_hwpx_source(&mut doc, rhwp::parser::FileFormat::Hwpx);
    let saved = rhwp::serializer::serialize_document(&doc).expect("직렬화");
    body_records(&saved)
}

/// 표 레코드는 zone 개수(u16)까지만 쓰고 끝난다 — 여분 2바이트가 붙지 않는다.
///
/// 배치: `속성(4) + 행(2) + 열(2) + 셀간격(2) + 안쪽여백(8) + 행별셀수(2×N) +
/// 테두리배경(2) + zone 개수(2) + zone(10×M)`.
#[test]
fn table_record_ends_at_zone_count() {
    let recs = saved_records();
    let tables: Vec<&Vec<u8>> = recs
        .iter()
        .filter(|(t, _)| *t == TAG_TABLE)
        .map(|(_, d)| d)
        .collect();
    assert!(
        !tables.is_empty(),
        "표 레코드를 찾지 못했다 — 표본이 바뀌었는지 확인하라"
    );

    let mut offenders = Vec::new();
    for (i, d) in tables.iter().enumerate() {
        if d.len() < 20 {
            offenders.push(format!("표 {i}: 길이 {} < 최소 20", d.len()));
            continue;
        }
        let rows = u16::from_le_bytes(d[4..6].try_into().expect("행")) as usize;
        // 속성4 + 행2 + 열2 + 셀간격2 + 안쪽여백8 = 18, + 행별셀수 2×N,
        // + 테두리배경2 + zone 개수2 → zone 개수 필드의 끝
        let base = 22 + 2 * rows;
        if d.len() < base {
            offenders.push(format!("표 {i}: 길이 {} < 필요 {base}", d.len()));
            continue;
        }
        let zones = u16::from_le_bytes(d[base - 2..base].try_into().expect("zone")) as usize;
        let expected = base + 10 * zones;
        if d.len() != expected {
            offenders.push(format!(
                "표 {i}: 길이 {} 기대 {expected} (행 {rows}, zone {zones}) — 여분 {}바이트",
                d.len(),
                d.len() as i64 - expected as i64
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "표 레코드에 한컴이 쓰지 않는 여분 바이트가 붙었다 ({}건):\n  {}",
        offenders.len(),
        offenders
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// 연결선 레코드는 제어점 뒤에 4바이트를 더 쓴다 (한컴 실측 20건 전부 `00000000`).
///
/// 배치: `시작점(8) + 끝점(8) + 연결정보(20) + 제어점수(4) + 제어점(10×N) + 꼬리(4)`.
///
/// 추적 표본에는 연결선이 없어(일반 직선뿐) 합성 문서로 계약을 고정한다.
#[test]
fn connector_record_has_trailing_word() {
    use rhwp::model::control::Control;
    use rhwp::model::document::{Document, Section};
    use rhwp::model::paragraph::Paragraph;
    use rhwp::model::shape::{
        ConnectorControlPoint, ConnectorData, GroupShape, LineShape, ShapeObject,
    };

    let mut connector = LineShape::default();
    connector.connector = Some(ConnectorData {
        control_points: vec![ConnectorControlPoint::default(); 2],
        ..Default::default()
    });

    let group = GroupShape {
        children: vec![ShapeObject::Line(connector)],
        ..Default::default()
    };
    let mut para = Paragraph::default();
    para.controls
        .push(Control::Shape(Box::new(ShapeObject::Group(group))));
    para.char_count = 9; // 확장 컨트롤 8 + 문단 종결자

    let mut doc = Document::default();
    doc.sections.push(Section {
        paragraphs: vec![para],
        ..Default::default()
    });
    doc.doc_properties.section_count = 1;

    let saved = rhwp::serializer::serialize_document(&doc).expect("직렬화");
    let lines: Vec<Vec<u8>> = body_records(&saved)
        .into_iter()
        .filter(|(t, _)| *t == TAG_SC_LINE)
        .map(|(_, d)| d)
        .collect();
    assert_eq!(lines.len(), 1, "연결선 레코드가 하나여야 한다");

    let d = &lines[0];
    let n = u32::from_le_bytes(d[36..40].try_into().expect("제어점수")) as usize;
    assert_eq!(n, 2, "제어점 수가 어긋난다");
    let expected = 40 + 10 * n + 4;
    assert_eq!(
        d.len(),
        expected,
        "연결선 레코드 길이가 한컴과 다르다 — 제어점 뒤 꼬리 4바이트가 빠졌다"
    );
    assert_eq!(
        &d[d.len() - 4..],
        &[0, 0, 0, 0],
        "꼬리 4바이트는 0 이어야 한다"
    );
}
