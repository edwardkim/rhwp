//! 변환 계보와 실제 저장 컨테이너의 조판 정책을 구분하는 메타데이터 계약.
//! 픽셀 위치·SVG·쪽수·렌더링 golden을 고정하지 않는다.

use rhwp::model::provenance::SourceFormat;
use rhwp::parser::parse_document;
use std::io::Read;

#[test]
fn issue7642_converted_hwp3_keeps_lineage_and_uses_the_hwp5_container_contract() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample-hwp5.hwp");
    let bytes = std::fs::read(path).expect("공식 HWP3 → HWP5 변환 표본");
    // 모델의 계보 플래그와 별도로 CFB FileHeader의 실제 저장 버전을 확인한다.
    let mut container = cfb::CompoundFile::open(std::io::Cursor::new(&bytes)).expect("CFB");
    let mut header = Vec::new();
    container
        .open_stream("/FileHeader")
        .expect("FileHeader")
        .read_to_end(&mut header)
        .expect("FileHeader 바이트");
    assert!(header.starts_with(b"HWP Document File"));
    let stored_version = u32::from_le_bytes(header[32..36].try_into().expect("버전 DWORD"));
    assert_eq!(stored_version >> 24, 5);
    let doc = parse_document(&bytes).expect("변환 표본 파싱");
    assert_eq!(doc.header.version.major, 5, "원 HWP3가 아닌 HWP5 저장본");
    assert_eq!(doc.provenance.format, SourceFormat::Hwp5);
    assert!(doc.provenance.hwp3_lineage, "원 HWP3 계보는 보존한다");
    assert_eq!(doc.provenance.hwp3_lineage, doc.is_hwp3_variant);

    let profile = doc.layout_profile();
    assert!(profile.hwp5_container());
    assert!(profile.native_hwp5_layout());
    assert!(profile.hwp5_stored_pagination_layout());
    assert!(!profile.hwp3_native_layout());
    assert!(
        !profile.hwp3_layout(),
        "계보만으로 HWP3 좌표 단위를 적용하지 않는다"
    );
    assert!(!profile.hwpx_stored_layout());
}

/// HWP5 원본을 HWPX 컨테이너로 옮겨도 저장 조판의 단위와 계보 계약을 보존한다.
/// 실제 렌더링 좌표나 쪽수 대신 공개 출처·프로필을 검사한다.
#[test]
fn issue7642_hwp5_origin_export_keeps_the_source_layout_profile() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample-hwp5.hwp");
    let input = std::fs::read(path).expect("한컴 HWP5 변환본");
    let source = parse_document(&input).expect("원본 파싱");
    let core = rhwp::document_core::DocumentCore::from_bytes(&input).expect("공개 문서 열기");
    let bytes = core.export_hwpx_native().expect("공개 HWPX 저장");
    let reopened = parse_document(&bytes).expect("HWPX 재파싱");
    assert_eq!(reopened.provenance.format, SourceFormat::Hwpx);
    assert_eq!(
        reopened.provenance.hwp3_lineage,
        source.provenance.hwp3_lineage
    );
    assert!(reopened.layout_profile().hwp5_origin_hwpx());
    assert!(reopened.layout_profile().hwp5_stored_pagination_layout());
    assert_eq!(
        reopened.layout_profile().hwp3_layout(),
        source.layout_profile().hwp3_layout(),
        "저장 컨테이너 변경만으로 HWP3 전용 좌표 계약을 켜지 않는다"
    );
    assert_eq!(
        reopened.layout_profile().hwp3_origin_flow_spacing_before(),
        source.layout_profile().hwp3_origin_flow_spacing_before(),
        "직렬화한 원본 문단 간격을 재열람에서 다시 배율 변환하지 않는다"
    );
}
