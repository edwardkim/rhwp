//! [#5838] HWPX 책갈피의 이름이 HWP5 저장에서 사라진다.
//!
//! HWPX 는 책갈피를 `<hp:fieldBegin type="BOOKMARK" name="_top">` 으로 싣고, rhwp 는 그것을
//! `Field{field_type: Bookmark}` 로 읽어 `%bmk` 컨트롤까지 정확히 방출한다. 그런데 이름을
//! 담는 `HWPTAG_CTRL_DATA` 는 `ClickHere`(누름틀)에만 붙었으므로 **이름 없는 책갈피**가
//! 저장됐다 — 상호참조·하이퍼링크가 가리킬 대상이 없어진다.
//!
//! 정답지는 같은 문서의 한컴 저작본이다. `samples/aift.hwp` 의 `%bmk` 아래에는
//! `ParameterSet ps_id=0x021b · item id=0x4000(String) = "_top"` 이 있고, 이는 누름틀
//! 경로가 이미 쓰던 바이트 모양과 같다(스펙 §4.2.10.11 "책갈피는 이름 밖에 없다").
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const HWPTAG_CTRL_HEADER: u16 = 0x010 + 55;
const HWPTAG_CTRL_DATA: u16 = 0x010 + 71;
/// ctrl_id 는 big-endian 으로 조립해 u32 LE 로 실린다.
const CTRL_BOOKMARK: u32 =
    ((b'%' as u32) << 24) | ((b'b' as u32) << 16) | ((b'm' as u32) << 8) | b'k' as u32;

fn records(buf: &[u8]) -> Vec<(u16, u16, Vec<u8>)> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off + 4 <= buf.len() {
        let header = u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        off += 4;
        let tag = (header & 0x3FF) as u16;
        let level = ((header >> 10) & 0x3FF) as u16;
        let mut size = ((header >> 20) & 0xFFF) as usize;
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

/// 저장된 HWP5 에서 `%bmk` 컨트롤 바로 아래의 `CTRL_DATA` 페이로드를 모은다.
fn bookmark_ctrl_data(hwp: &[u8]) -> Vec<Vec<u8>> {
    let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(hwp.to_vec())).expect("CFB 열기");
    let compressed = {
        let mut stream = cfb.open_stream("/FileHeader").expect("FileHeader");
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut stream, &mut buf).expect("FileHeader 읽기");
        u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]) & 1 == 1
    };
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
            let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
            std::io::Read::read_to_end(&mut decoder, &mut decoded).expect("Section 압축 해제");
            decoded
        } else {
            raw
        };
        // level → 그 level 에서 마지막으로 본 컨트롤 이름.
        let mut open: std::collections::BTreeMap<u16, u32> = std::collections::BTreeMap::new();
        for (tag, level, data) in records(&body) {
            if tag == HWPTAG_CTRL_HEADER && data.len() >= 4 {
                open.insert(
                    level,
                    u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                );
            } else if tag == HWPTAG_CTRL_DATA
                && level > 0
                && open.get(&(level - 1)) == Some(&CTRL_BOOKMARK)
            {
                out.push(data);
            }
        }
    }
    out
}

fn convert_hwpx_to_hwp_bytes(sample: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let mut doc = rhwp::parse_document(&data).unwrap_or_else(|e| panic!("parse {sample}: {e:?}"));
    rhwp::document_core::converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir(&mut doc);
    rhwp::serializer::serialize_document(&doc).unwrap_or_else(|e| panic!("serialize: {e:?}"))
}

/// 변환본의 책갈피 이름 레코드가 한컴 정답지와 **바이트 동일**해야 한다.
#[test]
fn bookmark_name_survives_hwpx_to_hwp_and_matches_the_oracle() {
    let converted = convert_hwpx_to_hwp_bytes("samples/hwpx/aift.hwpx");
    let ours = bookmark_ctrl_data(&converted);

    let oracle_bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/aift.hwp"))
            .expect("정답지 읽기");
    let oracle = bookmark_ctrl_data(&oracle_bytes);

    assert_eq!(
        oracle.len(),
        1,
        "정답지에는 이름 붙은 책갈피가 하나 있어야 한다"
    );
    assert_eq!(
        ours.len(),
        oracle.len(),
        "변환본에서 책갈피 이름 레코드가 사라졌다 (#5838): {}건",
        ours.len()
    );
    assert_eq!(
        ours[0], oracle[0],
        "책갈피 CTRL_DATA 가 정답지와 다르다\n  변환: {:02x?}\n  정답: {:02x?}",
        ours[0], oracle[0]
    );

    // 실린 값이 실제로 이름인지까지 본다 — ParameterSet 0x021b · item 0x4000(String).
    let payload = &ours[0];
    assert_eq!(
        u16::from_le_bytes([payload[0], payload[1]]),
        0x021b,
        "ParameterSet 머리(0x021b)가 아니다"
    );
    assert_eq!(
        u16::from_le_bytes([payload[6], payload[7]]),
        0x4000,
        "item id 가 0x4000(필드 이름)이 아니다"
    );
    let chars = u16::from_le_bytes([payload[10], payload[11]]) as usize;
    let name: String = char::decode_utf16(
        payload[12..12 + chars * 2]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
    )
    .map(|ch| ch.unwrap_or('\u{fffd}'))
    .collect();
    assert_eq!(name, "_top", "책갈피 이름이 원본과 다르다");
}
