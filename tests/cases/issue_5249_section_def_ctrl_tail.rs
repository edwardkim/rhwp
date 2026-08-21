//! [#5249] `secd` CTRL_HEADER 확장 tail 길이는 **저장될 파일 형식 버전**이 정한다.
//!
//! 종전 어댑터는 바탕쪽 유무로 갈랐다(`master_pages.is_empty()` → 10 byte tail = secd 38,
//! 있으면 19 byte = secd 47). `samples/**/*.hwp` 한컴 저작 517구역 전수 실측
//! (`scripts/secd_tail_survey.py`)은 그 게이트를
//! 양방향으로 반증한다 — 5.0.4.0 이상에서 **바탕쪽 0인데 47인 구역이 284개**, 5.0.4.0
//! 미만에서 **바탕쪽이 있는데 38인 구역이 10개**다. 버전으로 가르면 예외가 0이다.
//!
//! HWPX 파서는 FileHeader 에 5.1.0.0 을 적으므로, HWPX 출처 저장본은 **버전이 47을
//! 약속하고 내용은 38을 내보내는** 상태였다. 이 계약은 그 둘을 묶는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

/// HWP5 레코드 헤더를 풀어 `(tag_id, level, data)` 를 순회한다.
fn records(buf: &[u8]) -> Vec<(u16, u16, Vec<u8>)> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off + 4 <= buf.len() {
        let h = u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        off += 4;
        let tag = (h & 0x3FF) as u16;
        let level = ((h >> 10) & 0x3FF) as u16;
        let mut size = ((h >> 20) & 0xFFF) as usize;
        if size == 0xFFF {
            if off + 4 > buf.len() {
                break;
            }
            size =
                u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]) as usize;
            off += 4;
        }
        let end = (off + size).min(buf.len());
        out.push((tag, level, buf[off..end].to_vec()));
        off = end;
    }
    out
}

/// 저장된 HWP5 바이트에서 모든 `secd` CTRL_HEADER 레코드의 데이터를 뽑는다.
fn section_def_records(hwp: &[u8]) -> Vec<Vec<u8>> {
    const HWPTAG_CTRL_HEADER: u16 = 0x010 + 55;
    // ctrl_id 는 big-endian 조립 후 u32 LE 로 실린다 — 파일 바이트로는 "dces".
    const SECD: u32 =
        ((b's' as u32) << 24) | ((b'e' as u32) << 16) | ((b'c' as u32) << 8) | b'd' as u32;

    let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(hwp.to_vec())).expect("CFB 열기");
    let compressed = {
        let mut header = cfb.open_stream("/FileHeader").expect("FileHeader");
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut header, &mut buf).expect("FileHeader 읽기");
        u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]) & 1 == 1
    };

    // walk() 가 cfb 를 빌리므로 경로만 먼저 모은 뒤 스트림을 연다.
    let section_paths: Vec<std::path::PathBuf> = cfb
        .walk()
        .filter(|entry| {
            let path = entry.path().to_string_lossy().into_owned();
            entry.is_stream() && path.contains("BodyText") && path.contains("Section")
        })
        .map(|entry| entry.path().to_path_buf())
        .collect();

    let mut out = Vec::new();
    for section_path in section_paths {
        let mut stream = cfb.open_stream(&section_path).expect("Section 스트림");
        let mut raw = Vec::new();
        std::io::Read::read_to_end(&mut stream, &mut raw).expect("Section 읽기");
        let body = if compressed {
            let mut decoded = Vec::new();
            let mut dec = flate2::read::DeflateDecoder::new(&raw[..]);
            std::io::Read::read_to_end(&mut dec, &mut decoded).expect("Section 압축 해제");
            decoded
        } else {
            raw
        };
        for (tag, _level, data) in records(&body) {
            if tag == HWPTAG_CTRL_HEADER
                && data.len() >= 4
                && u32::from_le_bytes([data[0], data[1], data[2], data[3]]) == SECD
            {
                out.push(data);
            }
        }
    }
    out
}

/// HWPX 를 HWP5 로 저장한 바이트.
fn convert_hwpx_to_hwp_bytes(sample: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let mut doc = rhwp::parse_document(&data).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
    rhwp::document_core::converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir(&mut doc);
    rhwp::serializer::serialize_document(&doc).unwrap_or_else(|e| panic!("serialize: {e:?}"))
}

/// 바탕쪽이 **없는** HWPX 도 5.1.0.0 저장본이면 19 byte tail(secd 47)이다.
///
/// 종전 게이트에서는 이 문서가 secd 38 로 나갔다 — 버전 선언(5.1.0.0)과 어긋난 값이다.
#[test]
fn hwpx_without_master_pages_still_gets_the_versioned_tail() {
    let hwp = convert_hwpx_to_hwp_bytes("samples/hwpx/143E433F503322BD33.hwpx");
    let secds = section_def_records(&hwp);
    assert!(!secds.is_empty(), "secd 레코드를 찾지 못했다");
    for data in &secds {
        assert_eq!(
            data.len(),
            47,
            "5.1.0.0 저장본의 secd 는 47 byte 여야 한다 (바탕쪽 유무와 무관): {} byte",
            data.len()
        );
        assert!(
            data[28..].iter().all(|b| *b == 0),
            "합성 tail 은 전부 0 이어야 한다: {:02x?}",
            &data[28..]
        );
    }
}

/// 한컴 정답지 대조 — 같은 문서의 한컴 저작 HWP(5.1.0.1) 가 47 byte all-zero tail 이다.
#[test]
fn converted_tail_matches_the_hancom_oracle_for_the_same_document() {
    let hwp = convert_hwpx_to_hwp_bytes("samples/hwpx/exam_social-p1.hwpx");
    let ours = section_def_records(&hwp);
    let oracle = section_def_records(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_social-p1.hwp"))
            .expect("정답지 읽기"),
    );

    assert_eq!(oracle.len(), 1, "정답지 구역 수");
    assert_eq!(oracle[0].len(), 47, "정답지(5.1.0.1) 는 secd 47 이다");
    assert_eq!(ours.len(), oracle.len(), "변환 구역 수가 정답지와 다르다");
    assert_eq!(
        ours[0].len(),
        oracle[0].len(),
        "변환 secd 크기가 정답지와 다르다"
    );
    assert_eq!(
        &ours[0][28..],
        &oracle[0][28..],
        "변환 tail 바이트가 정답지와 다르다"
    );
}
