//! Issue #3706 (#3676 후속): HWP3 → HWP5 변환본의 FileHeader 에 버전 3 이 기록된다.
//!
//! ## 증상
//!
//! `rhwp convert` 로 만든 HWP3 → HWP5 변환본의 FileHeader 버전 바이트가
//! `00000003`(major=3) 이다. HWP5 컨테이너의 FileHeader 는 5.x 버전을 선언해야
//! 하며(같은 문서의 한컴 저장본 `samples/hwp3-sample-hwp5.hwp` 는 `01000105`
//! = 5.1.0.1), 버전 3 은 규격 위반이다. #3676 조사에서 한컴 열기 거부의
//! 근인은 아니었지만(버전 4종 실험으로 반증) 규격 위반 자체는 확정됐다.
//!
//! ## 근인
//!
//! HWP3 파서(`parser/hwp3/mod.rs`)는 `version.major = 3` 을 메모리 전용
//! 표시로 두고 **실제 저장용 HWP5 헤더(5.0.3.0)는 `header.raw_data` 에**
//! 기록한다 — 직렬화(`serialize_file_header`)가 raw_data 를 우선 쓰는 것을
//! 전제한 설계다. 그런데 저장 정규화
//! (`document_core/converters/hwpx_to_hwp.rs::normalize_file_header_for_hwp`)가
//! 압축 플래그를 필드로 실체화하면서 `raw_data` 를 무조건 버려, 직렬화가
//! 필드 경로로 떨어지고 major=3 이 그대로 디스크에 기록된다.
//!
//! ## 이 테스트가 지키는 계약
//!
//! 변환본 FileHeader 는 (1) 5.x 버전을 선언하고 (2) 압축 플래그가 실제 스트림
//! 압축과 일치하며 (3) rhwp 재로드에서도 5.x 로 읽혀야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::cfb_reader::CfbReader;

/// HWP3 표본 — #3676 표본군과 같은 계열.
const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// HWP3 원본을 HWP5 바이트로 변환한다 (CLI `convert` 와 같은 경로:
/// parse_hwp3 → convert_if_hwpx_source → serialize_hwp).
fn convert_to_hwp5_bytes() -> Vec<u8> {
    let raw = std::fs::read(sample_path()).expect("표본 읽기");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).expect("HWP3 파싱");
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        rhwp::parser::FileFormat::Hwp3,
    );
    rhwp::serializer::serialize_hwp(&doc).expect("HWP5 직렬화")
}

fn converted_file_header() -> Vec<u8> {
    let bytes = convert_to_hwp5_bytes();
    let mut cfb = CfbReader::open(&bytes).expect("CFB 열기");
    cfb.read_file_header().expect("FileHeader 스트림 읽기")
}

/// ① 변환본 FileHeader 가 HWP5 버전(5.x)을 선언한다.
///
/// HWP3 파서가 raw_data 에 기록해 둔 저장용 버전은 5.0.3.0 이다. 저장 정규화가
/// raw_data 를 버리더라도 이 버전은 필드로 회수돼야 한다.
#[test]
fn converted_file_header_declares_hwp5_version() {
    let fh = converted_file_header();
    assert!(fh.len() >= 40, "FileHeader 는 최소 40바이트여야 한다");
    let (revision, build, minor, major) = (fh[32], fh[33], fh[34], fh[35]);
    assert_eq!(
        major, 5,
        "HWP5 컨테이너의 FileHeader 버전이 {major}.{minor}.{build}.{revision} 이다. \
         HWP5 규격은 major=5 를 요구한다(한컴 저장본 hwp3-sample-hwp5.hwp 는 5.1.0.1). \
         HWP3 파서가 raw_data 에 기록한 5.0.3.0 이 저장 정규화에서 유실되면 \
         메모리 전용 표시값 major=3 이 그대로 디스크에 기록된다."
    );
    assert_eq!(
        (major, minor, build, revision),
        (5, 0, 3, 0),
        "회수된 버전은 HWP3 파서가 raw_data 에 기록한 5.0.3.0 그대로여야 한다"
    );
}

/// ② 버전 회수가 압축 플래그 실체화를 깨지 않는다.
///
/// 정규화는 raw_data(flags=0 비압축)를 버리고 압축 플래그를 필드로 실체화한다.
/// 버전을 raw_data 에서 회수하더라도 flags 는 raw_data 값이 아니라 실체화된
/// 값(bit0=압축)이어야 한다 — 아니면 "헤더는 비압축, 스트림은 압축" 불일치가 생긴다.
#[test]
fn converted_file_header_keeps_compressed_flag_materialized() {
    let fh = converted_file_header();
    assert_eq!(
        fh[36] & 0x01,
        0x01,
        "FileHeader 압축 플래그(bit0)가 꺼져 있다. 저장기는 DocInfo/BodyText 를 \
         압축 기록하므로 플래그가 꺼지면 컨테이너 자기모순이다."
    );
}

/// ③ 왕복: 변환본을 rhwp 로 재로드해도 5.x 버전으로 읽힌다.
#[test]
fn converted_reloads_with_hwp5_version() {
    let bytes = convert_to_hwp5_bytes();
    let fh =
        rhwp::parser::header::parse_file_header(&converted_file_header()).expect("FileHeader 파싱");
    assert_eq!(fh.version.major, 5, "재파스한 FileHeader 버전 major=5");
    // 문서 전체 재로드도 성공해야 한다 (컨테이너 왕복 무결성).
    let core = rhwp::document_core::DocumentCore::from_bytes(&bytes).expect("변환본 재로드");
    assert_eq!(
        core.document().header.version.major,
        5,
        "재로드된 문서의 header.version.major 는 5 여야 한다"
    );
}
