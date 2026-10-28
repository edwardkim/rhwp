//! [parser] `LenientCfbReader` 가 CFB 스트림을 **경로가 아니라 이름으로** 찾던 회귀.
//!
//! `BodyText/Section0` 과 `ViewText/Section0` 은 **이름이 같다**. 종전 `find_entry_idx` 는
//! 경로의 마지막 세그먼트만 떼어 전체 엔트리에서 첫 일치를 돌려주었으므로, 명시적으로
//! `/BodyText/Section0` 을 물어도 디렉터리에 먼저 나오는 `ViewText/Section0` 이 왔다.
//! (주석의 근거 "HWP에서는 이름이 유일하므로 단순 매칭" 이라는 전제가 성립하지 않는다.)
//!
//! 표본 `samples/issue5169_viewtext_changetracking.hwp` 의 두 스트림:
//!   `ViewText/Section0` = 30,738 B (디렉터리에서 **먼저** 나온다)
//!   `BodyText/Section0` =  6,974 B
//!
//! 계약: 두 경로는 서로 **다른** 스트림으로 풀려야 하고, 각각 제 스토리지 아래의 것이어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::cfb_reader::LenientCfbReader;
use std::io::{Cursor, Write};

const SAMPLE: &str = "samples/issue5169_viewtext_changetracking.hwp";
const CAP: usize = 64 * 1024 * 1024;
const BODY_LEN: usize = 6_974;
const VIEW_LEN: usize = 30_738;

fn sample() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

#[test]
fn lenient_cfb_resolves_same_named_streams_by_path() {
    let data = sample();
    let lenient = LenientCfbReader::open(&data).expect("lenient open");

    // 표본이 이 회귀를 실제로 태우는지 먼저 확인한다 — 이름이 같은 스트림이 둘이어야 한다.
    let same_named = lenient
        .list_entries()
        .iter()
        .filter(|(name, _, _, _)| name == "Section0")
        .count();
    assert_eq!(
        same_named, 2,
        "표본에 이름이 Section0 인 스트림이 둘이어야 이 테스트가 의미가 있다"
    );

    let body = lenient
        .read_stream_raw_limited("/BodyText/Section0", CAP)
        .expect("/BodyText/Section0 를 읽을 수 있어야 한다");
    let view = lenient
        .read_stream_raw_limited("/ViewText/Section0", CAP)
        .expect("/ViewText/Section0 를 읽을 수 있어야 한다");

    assert_ne!(
        body.len(),
        view.len(),
        "두 경로가 같은 스트림으로 풀렸다 — 이름만 비교하고 있다"
    );
    assert_eq!(
        body.len(),
        BODY_LEN,
        "/BodyText/Section0 가 ViewText 쪽으로 풀렸다"
    );
    assert_eq!(view.len(), VIEW_LEN, "/ViewText/Section0 크기가 다르다");
}

#[test]
fn lenient_body_text_section_reads_bodytext_storage() {
    let data = sample();
    let lenient = LenientCfbReader::open(&data).expect("lenient open");

    let raw = lenient
        .read_body_text_section_raw_limited(0, CAP)
        .expect("BodyText Section0 raw");
    assert_eq!(
        raw.len(),
        BODY_LEN,
        "본문 섹션 읽기가 ViewText 스트림을 집었다"
    );
}

// 아래 합성 CFB는 directory entry가 root 포함 4개 이하인 v3 문서다.
// 실제 문서나 암호화 내용을 바꾸지 않고 디렉터리 링크 손상만 재현한다.
fn small_cfb(streams: &[(&str, &[u8])]) -> Vec<u8> {
    let mut cfb =
        cfb::CompoundFile::create_with_version(cfb::Version::V3, Cursor::new(Vec::new())).unwrap();
    for &(path, contents) in streams {
        let parent = std::path::Path::new(path).parent().unwrap();
        cfb.create_storage_all(parent).unwrap();
        cfb.create_stream(path)
            .unwrap()
            .write_all(contents)
            .unwrap();
    }
    cfb.into_inner().into_inner()
}

fn directory_offset(bytes: &[u8]) -> usize {
    assert_eq!(u16::from_le_bytes(bytes[30..32].try_into().unwrap()), 9);
    let sid = u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize;
    (sid + 1) * 512
}

fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn lenient_cfb_invalid_named_slot_falls_back_to_valid_stream() {
    for invalid_type in [0, 255] {
        let mut bytes = small_cfb(&[("/FileHeader", b"valid header")]);
        let dir = directory_offset(&bytes);
        let valid_id = u32::from_le_bytes(bytes[dir + 76..dir + 80].try_into().unwrap());
        assert_eq!(valid_id, 1);
        let entry = bytes[dir + 128..dir + 256].to_vec();
        bytes[dir + 256..dir + 384].copy_from_slice(&entry);
        bytes[dir + 256 + 66] = invalid_type;
        set_u32(&mut bytes, dir + 76, 2);

        let reader = LenientCfbReader::open(&bytes).unwrap();
        assert_eq!(reader.read_file_header().unwrap(), b"valid header");
    }
}

#[test]
fn lenient_cfb_broken_child_link_recovers_only_unique_names() {
    let mut bytes = small_cfb(&[("/FileHeader", b"valid header")]);
    let dir = directory_offset(&bytes);
    set_u32(&mut bytes, dir + 76, 10_000);
    let reader = LenientCfbReader::open(&bytes).unwrap();
    assert_eq!(reader.read_file_header().unwrap(), b"valid header");

    let mut bytes = sample();
    let dir = directory_offset(&bytes);
    set_u32(&mut bytes, dir + 76, 10_000);
    let reader = LenientCfbReader::open(&bytes).unwrap();
    // 트리 밖에 Section0이 둘이면 임의의 것을 선택하면 안 된다.
    assert!(!reader.has_stream("/BodyText/Section0"));
    assert!(reader
        .read_stream_raw_limited("/BodyText/Section0", CAP)
        .is_err());
}

#[test]
fn lenient_cfb_stream_cannot_be_used_as_parent_storage() {
    let mut bytes = small_cfb(&[("/BodyText/Section0", b"body"), ("/Section0", b"other")]);
    let dir = directory_offset(&bytes);
    // create 순서상 entry 1은 BodyText storage다. child pointer를 유지한 채
    // stream으로 손상시켜도 이를 부모 storage로 통과시켜서는 안 된다.
    assert_eq!(bytes[dir + 128 + 66], 1);
    bytes[dir + 128 + 66] = 2;
    let reader = LenientCfbReader::open(&bytes).unwrap();
    assert!(reader
        .read_stream_raw_limited("/BodyText/Section0", CAP)
        .is_err());
}

#[test]
fn lenient_cfb_cyclic_sibling_link_terminates_and_recovers_unique_name() {
    let mut bytes = small_cfb(&[("/FileHeader", b"valid header")]);
    let dir = directory_offset(&bytes);
    let entry = bytes[dir + 128..dir + 256].to_vec();
    bytes[dir + 256..dir + 384].copy_from_slice(&entry);
    bytes[dir + 256 + 66] = 0;
    // 다른 이름의 삭제 슬롯만 순환하도록 만들어 유일 이름 fallback을 검사한다.
    bytes[dir + 256] = b'X';
    set_u32(&mut bytes, dir + 256 + 68, 2);
    set_u32(&mut bytes, dir + 256 + 72, 2);
    set_u32(&mut bytes, dir + 76, 2);
    let reader = LenientCfbReader::open(&bytes).unwrap();
    assert_eq!(reader.read_file_header().unwrap(), b"valid header");
}
