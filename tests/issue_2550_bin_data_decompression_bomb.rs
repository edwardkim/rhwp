//! [#2550] BinData deflate bomb — 저장·클립보드·렌더 경로 상한 회귀.
//!
//! `/BinData/BIN0001.*` 에 zeros 를 deflate 한 작은 스트림(해제 시 수 GB)을 넣으면,
//! 파싱은 지연 등록이라 저렴하게 성공하고 **저장하거나 그림을 복사·렌더하는 순간**
//! 무제한 `load()` 가 전량 materialize 해 OOM 이 났다. wasm32 에서는 모듈 abort 로
//! 열려 있는 다른 문서까지 함께 죽는 실패 양상이다.
//!
//! 수정 방향은 이슈 합의(C안: 경로별 차등)다.
//!
//! - **저장(HWP5)**: 상한 초과 시 압축 해제를 포기하고 **원본 저장 바이트를 그대로**
//!   기록한다. 정상 대용량 개체는 무손실이고, 폭탄은 애초에 해제하지 않는다.
//! - **렌더·클립보드·질의**: 상한 초과는 placeholder(빈 값/None) — 이미지 누락과 같은 경로.
//!
//! 공격 문서는 저장소에 커밋하지 않고 **시험 시점에 합성한다**
//! (`tests/security_corpus_regression.rs` 와 같은 방침).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use rhwp::model::bin_data::MAX_BIN_DATA_BYTES;
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::serializer::{mini_cfb, serialize_hwpx};
use rhwp::{parse_document, serialize_document, DocumentCore};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// BinData 를 가진 실문서 — 폭탄을 심을 숙주다.
const HOST_SAMPLE: &str = "samples/143E433F503322BD33.hwp";

/// 폭탄의 압축 해제 크기. 상한(256MB)의 4배로, 해제되면 즉시 관측 가능한 규모다.
const BOMB_PLAIN_BYTES: usize = 1024 * 1024 * 1024;

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// zeros 를 raw deflate 로 압축해 폭탄 스트림을 만든다.
///
/// HWP5 BinData 스트림의 압축 형식과 같은 raw deflate(wbits=-15)다. 입력이 전부
/// 0 이라 산출물은 수 KB 이며, 해제하면 [`BOMB_PLAIN_BYTES`] 가 된다.
fn deflate_bomb_with_plain_bytes(plain_bytes: usize) -> Vec<u8> {
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    let chunk = vec![0_u8; 1024 * 1024];
    let mut written = 0;
    while written < plain_bytes {
        let n = chunk.len().min(plain_bytes - written);
        encoder.write_all(&chunk[..n]).expect("deflate write");
        written += n;
    }
    encoder.finish().expect("deflate finish")
}

fn deflate_bomb() -> Vec<u8> {
    deflate_bomb_with_plain_bytes(BOMB_PLAIN_BYTES)
}

/// ZIP writer에 0 바이트를 스트리밍해 실제 ZIP deflate bomb 엔트리를 만든다.
///
/// HWP5 CFB의 raw-deflate 스트림과 달리 HWPX는 `ZipWriter`가 압축을 담당한다.
/// 이미 압축한 raw deflate를 다시 쓰면 260KB짜리 정상 엔트리가 되므로, 평문을
/// 직접 흘려보내 central directory의 비압축 크기도 공격 조건과 일치시킨다.
fn write_zeroes(writer: &mut impl Write, plain_bytes: usize) {
    let chunk = [0_u8; 1024 * 1024];
    let mut written = 0;
    while written < plain_bytes {
        let len = chunk.len().min(plain_bytes - written);
        writer.write_all(&chunk[..len]).expect("0 바이트 쓰기");
        written += len;
    }
}

/// 숙주 문서의 첫 `/BinData/*` 스트림을 폭탄으로 갈아끼운 CFB 를 만든다.
///
/// DocInfo 는 그대로 두므로 `HWPTAG_BIN_DATA` 레코드와 스트림 대응이 유지된다 —
/// 파서가 정상 이미지로 지연 등록하고, 실제 폭발은 소비 경로에서 일어난다.
fn synthesize_bomb_document() -> (Vec<u8>, String, Vec<u8>) {
    let host = std::fs::read(repo(HOST_SAMPLE)).expect("숙주 표본 읽기");
    let mut reader = CfbReader::open(&host).expect("숙주 CFB 열기");

    let bin_name = reader
        .list_bin_data()
        .into_iter()
        .next()
        .expect("숙주에 BinData 스트림이 있어야 함");
    let bomb_path = format!("/BinData/{}", bin_name);
    let bomb = deflate_bomb();

    let paths = reader.list_streams();
    let mut streams: Vec<(String, Vec<u8>)> = Vec::new();
    for path in paths {
        let data = if path == bomb_path {
            bomb.clone()
        } else {
            reader.read_stream_raw(&path).expect("스트림 읽기")
        };
        streams.push((path, data));
    }

    let named: Vec<(&str, &[u8])> = streams
        .iter()
        .map(|(p, d)| (p.as_str(), d.as_slice()))
        .collect();
    let bytes = mini_cfb::build_cfb(&named).expect("공격 문서 CFB 조립");
    (bytes, bomb_path, bomb)
}

/// HWP 숙주를 HWPX로 직렬화한 뒤 첫 BinData ZIP 엔트리만 폭탄으로 갈아낀다.
///
/// HWPX parser는 BinData를 lazy resolver로 등록하므로, 공격 파일을 만드는 과정에서
/// 폭탄 payload를 다시 읽지 않는다. ZIP central directory에 기록된 비압축 크기는
/// `MAX_BIN_DATA_BYTES + 1`로 실제 상한 초과를 재현한다.
fn synthesize_hwpx_bomb_document() -> (Vec<u8>, String) {
    let host = std::fs::read(repo(HOST_SAMPLE)).expect("숙주 표본 읽기");
    let document = parse_document(&host).expect("숙주 파싱");
    let hwpx = serialize_hwpx(&document).expect("HWPX 숙주 직렬화");
    let mut archive = ZipArchive::new(Cursor::new(hwpx)).expect("HWPX 숙주 ZIP 열기");
    let bomb_path = (0..archive.len())
        .find_map(|index| {
            let entry = archive.by_index(index).ok()?;
            entry
                .name()
                .starts_with("BinData/")
                .then(|| entry.name().to_string())
        })
        .expect("HWPX 숙주에 BinData ZIP 엔트리가 있어야 함");
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("숙주 ZIP 엔트리 읽기");
        let path = entry.name().to_string();
        let compression = if path == "mimetype" {
            CompressionMethod::Stored
        } else {
            entry.compression()
        };
        let options = SimpleFileOptions::default().compression_method(compression);
        writer
            .start_file(&path, options)
            .expect("공격 ZIP 엔트리 시작");
        if path == bomb_path {
            write_zeroes(&mut writer, MAX_BIN_DATA_BYTES + 1);
        } else {
            std::io::copy(&mut entry, &mut writer).expect("숙주 ZIP 엔트리 복사");
        }
    }
    let attack = writer.finish().expect("공격 HWPX ZIP 마감").into_inner();
    let mut verify = ZipArchive::new(Cursor::new(attack.as_slice())).expect("공격 ZIP 재확인");
    assert_eq!(
        verify
            .by_name(&bomb_path)
            .expect("폭탄 ZIP 엔트리 재확인")
            .size(),
        (MAX_BIN_DATA_BYTES + 1) as u64,
        "합성 ZIP은 중앙 디렉터리에 상한 초과 비압축 크기를 기록해야 한다"
    );
    (attack, bomb_path)
}

/// 파싱은 폭탄을 해제하지 않는다 (지연 등록) — 공격 전제의 확인.
#[test]
fn parsing_a_bomb_document_stays_cheap() {
    let (attack, _, _) = synthesize_bomb_document();
    let document = parse_document(&attack).expect("공격 문서 파싱은 성공해야 함");
    assert!(
        !document.bin_data_content.is_empty(),
        "폭탄 항목이 BinData 로 등록되어야 소비 경로 시험이 의미를 갖는다"
    );
}

/// 저장 경로는 OOM 없이 **원본 압축 바이트를 그대로** 보존한다.
///
/// 수정 전에는 `cfb_writer` 가 무제한 `load()` 로 1GB 를 materialize 했다.
#[test]
fn saving_a_bomb_document_preserves_original_stream_without_decompressing() {
    let (attack, bomb_path, bomb) = synthesize_bomb_document();
    let document = parse_document(&attack).expect("공격 문서 파싱");

    let saved = serialize_document(&document).expect("저장은 성공해야 함");

    let mut reader = CfbReader::open(&saved).expect("저장 결과 CFB 열기");
    let written = reader
        .read_stream_raw(&bomb_path)
        .expect("폭탄 스트림이 저장 결과에도 있어야 함");
    assert_eq!(
        written, bomb,
        "상한 초과 항목은 해제·재압축 없이 원본 저장 바이트가 그대로 보존되어야 한다 \
         (빈 값으로 유실되면 데이터 손실)"
    );
}

/// 렌더·질의 경로는 상한 초과를 placeholder 로 접는다.
#[test]
fn render_and_query_paths_fold_an_oversized_entry_to_placeholder() {
    let (attack, _, _) = synthesize_bomb_document();
    let core = DocumentCore::from_bytes(&attack).expect("DocumentCore 적재");

    assert_eq!(
        core.get_bin_data(0),
        None,
        "상한 초과 항목의 바이트 질의는 항목 없음과 같아야 한다"
    );

    // 렌더가 폭탄 항목을 만나도 해제하지 않고 완주한다 (쪽 번호 0-based).
    let svg = core
        .render_page_svg_native(0)
        .expect("첫 쪽 렌더는 성공해야 함");
    assert!(!svg.is_empty(), "렌더 산출물이 비어서는 안 된다");
}

/// 클립보드 이미지 질의는 상한 초과를 오류로 돌려준다 (materialize 하지 않는다).
#[test]
fn clipboard_image_queries_reject_an_oversized_entry() {
    let (attack, _, _) = synthesize_bomb_document();
    let core = DocumentCore::from_bytes(&attack).expect("DocumentCore 적재");

    let error = core
        .get_bin_data_image_data_native(1)
        .expect_err("상한 초과 항목은 바이트를 돌려주지 않아야 한다");
    assert!(
        error.to_string().contains("상한"),
        "상한 초과임이 오류 메시지에 드러나야 한다: {error}"
    );

    assert!(
        core.get_bin_data_image_mime_native(1).is_err(),
        "MIME 판별도 전체 해제 없이 실패해야 한다"
    );
}

/// 정상 문서는 상한 도입 전후로 왕복 결과가 같다 (데이터 손실 회귀 가드).
#[test]
fn normal_documents_round_trip_unchanged_under_the_limit() {
    let host = std::fs::read(repo(HOST_SAMPLE)).expect("숙주 표본 읽기");
    let document = parse_document(&host).expect("숙주 파싱");

    let before: Vec<Vec<u8>> = document
        .bin_data_content
        .iter()
        .map(|c| c.data.load())
        .collect();
    assert!(
        before
            .iter()
            .all(|b| !b.is_empty() && b.len() <= MAX_BIN_DATA_BYTES),
        "숙주 표본의 BinData 는 상한 이내의 정상 데이터여야 한다"
    );

    let saved = serialize_document(&document).expect("숙주 저장");
    let reparsed = parse_document(&saved).expect("저장 결과 재파싱");
    let after: Vec<Vec<u8>> = reparsed
        .bin_data_content
        .iter()
        .map(|c| c.data.load())
        .collect();

    assert_eq!(
        after, before,
        "상한 이내 정상 BinData 는 왕복에서 바이트가 보존되어야 한다"
    );
}

/// HWPX ZIP BinData도 같은 상한 아래에서 HWP/HWPX 저장과 질의를 모두 안전하게
/// placeholder로 접는다.
///
/// 수정 전에는 HWP5 저장기의 `load_raw() == None` fallback이 `load()`를 호출했고,
/// HWPX resolver의 `len()`/`is_empty()`도 무제한 `resolve()`로 ZIP bomb를 풀었다.
#[test]
fn hwpx_bomb_is_bounded_for_query_and_both_save_targets() {
    let (attack, bomb_path) = synthesize_hwpx_bomb_document();
    let document = parse_document(&attack).expect("공격 HWPX 파싱");
    let bomb_index = document
        .bin_data_content
        .iter()
        .position(|content| {
            matches!(
                &content.data,
                rhwp::model::bin_data::BinDataBytes::Lazy { key, .. } if key == &bomb_path
            )
        })
        .expect("폭탄 BinData가 lazy 항목으로 등록되어야 함");
    let bomb_content = &document.bin_data_content[bomb_index];

    assert_eq!(
        bomb_content.data.len(),
        0,
        "HWPX ZIP 상한 초과 항목의 길이는 materialize 없이 0이어야 한다: {bomb_path}"
    );
    assert!(
        bomb_content.data.is_empty(),
        "HWPX ZIP 상한 초과 항목은 존재 질의에서도 placeholder여야 한다"
    );
    assert!(
        bomb_content.data.load_limited(MAX_BIN_DATA_BYTES).is_none(),
        "HWPX ZIP 상한 초과 항목은 bounded load를 통과하면 안 된다"
    );

    let core = DocumentCore::from_bytes(&attack).expect("공격 HWPX DocumentCore 적재");
    assert_eq!(
        core.get_bin_data(bomb_index),
        None,
        "HWPX ZIP 상한 초과 항목의 공개 질의는 None이어야 한다"
    );

    let hwp = serialize_document(&document).expect("HWPX→HWP 저장");
    let mut hwp_reader = CfbReader::open(&hwp).expect("HWP 저장 결과 CFB 열기");
    assert!(
        hwp_reader
            .list_bin_data()
            .into_iter()
            .map(|name| format!("/BinData/{name}"))
            .any(|path| hwp_reader
                .read_stream_raw(&path)
                .expect("HWP BinData 읽기")
                .is_empty()),
        "HWPX 폭탄은 HWP 저장에서 raw passthrough 없이 빈 placeholder가 되어야 한다"
    );

    let saved_hwpx = serialize_hwpx(&document).expect("HWPX→HWPX 저장");
    let mut saved_archive = ZipArchive::new(Cursor::new(saved_hwpx)).expect("저장 HWPX ZIP 열기");
    let mut written = Vec::new();
    saved_archive
        .by_name(&bomb_path)
        .expect("폭탄 BinData 엔트리 보존")
        .read_to_end(&mut written)
        .expect("저장 HWPX BinData 읽기");
    assert!(
        written.is_empty(),
        "HWPX 폭탄은 HWPX 저장에서 빈 placeholder가 되어야 한다"
    );
}
