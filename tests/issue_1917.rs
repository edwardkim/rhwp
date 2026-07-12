//! Issue #1917 — HWPX BinData 64MB 엔트리 상한이 실문서를 거부.
//!
//! 정부 보도자료 계열에 비압축 BMP/TIF 대형 이미지가 실재한다 (10k 서베이:
//! 압축 7.5MB/원본 103.7MB BMP — 한글은 정상 열람). 종전 64MB 상한은 로드
//! 거부(그림 소실) + 재직렬화 pic 컨트롤 드롭(왕복 데이터 손실)으로 이어졌다.
//! 상한을 512MB 로 상향 (zip-bomb 무제한 팽창 차단 목적은 유지).

use rhwp::model::bin_data::BinDataContent;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::image::Picture;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::style::CharShape;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

/// 압축 해제 크기 70MB(종전 상한 64MB 초과) BinData 가 HWPX 왕복에서 보존된다.
#[test]
fn issue_1917_large_bindata_survives_hwpx_roundtrip() {
    const BIG: usize = 70 * 1024 * 1024; // 70MB > 종전 64MB 상한

    let mut pic = Picture::default();
    pic.image_attr.bin_data_id = 1;
    pic.common.width = 8000;
    pic.common.height = 6000;
    pic.shape_attr.original_width = 8000;
    pic.shape_attr.original_height = 6000;
    pic.shape_attr.current_width = 8000;
    pic.shape_attr.current_height = 6000;

    let host = Paragraph {
        char_count: 1,
        line_segs: vec![LineSeg {
            line_height: 1000,
            line_spacing: 600,
            ..Default::default()
        }],
        controls: vec![Control::Picture(Box::new(pic))],
        ..Default::default()
    };

    let mut doc = Document::default();
    doc.doc_info.char_shapes.push(CharShape::default());
    let mut section = Section::default();
    section.paragraphs.push(host);
    doc.sections.push(section);
    doc.bin_data_content.push(BinDataContent {
        id: 1,
        data: vec![0u8; BIG], // ZIP 으로는 소형 압축 — 압축 해제 상한 검증에 적합
        extension: "bmp".to_string(),
    });

    let bytes = serialize_hwpx(&doc).expect("serialize");
    let doc2 = parse_hwpx(&bytes).expect("parse (종전: 64MB 상한으로 BinData 로드 거부)");

    let content = doc2
        .bin_data_content
        .iter()
        .find(|b| b.data.len() == BIG)
        .expect("70MB BinData 가 로드되어야 함 (상한 512MB)");
    assert_eq!(content.data.len(), BIG);

    // pic 컨트롤 보존 (종전: 콘텐츠 미등록 → 재직렬화에서 pic 드롭)
    let out2 = serialize_hwpx(&doc2).expect("re-serialize");
    let doc3 = parse_hwpx(&out2).expect("re-parse");
    let pic_count = doc3.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .filter(|c| matches!(c, Control::Picture(_)))
        .count();
    assert_eq!(pic_count, 1, "대형 BinData 참조 pic 컨트롤 왕복 보존");
}
