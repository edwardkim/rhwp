//! Issue #1929 — HWP5 저장: raw 미보존 그림의 imgDim (0,0) 소실.
//!
//! HWP5 PICTURE 레코드의 extra 꼬리(18바이트)에는 원본 이미지 크기 칸
//! (offset 9..17)이 있는데, 종전에는 (1) HWP5 파서가 이를 `img_dim` 으로
//! 읽지 않았고 (2) 직렬화기 non-raw 경로가 crop 폴백만 기록했다. HWPX 파스
//! IR(hp:imgDim 보유, raw_picture_extra 없음)을 HWP5 로 저장-재파스하면
//! imgDim 이 (0,0) 으로 소실됐다 (#1916 과 같은 raw-부재 재구성 결손 계열).

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::image::Picture;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::style::CharShape;
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;

fn make_doc_with_pic(img_dim: (u32, u32)) -> Document {
    let mut pic = Picture::default();
    pic.image_attr.bin_data_id = 0; // 콘텐츠 없는 placeholder — img_dim 축만 검증
    pic.img_dim = img_dim;
    pic.common.width = 8000;
    pic.common.height = 6000;
    assert!(pic.raw_picture_extra.is_empty());

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
    doc
}

fn find_pic(doc: &Document) -> &Picture {
    for para in &doc.sections[0].paragraphs {
        for ctrl in &para.controls {
            if let Control::Picture(p) = ctrl {
                return p;
            }
        }
    }
    panic!("pic 미발견");
}

/// raw 미보존 그림의 imgDim 이 plain HWP5 왕복(2-round 포함)에서 보존된다.
#[test]
fn issue_1929_img_dim_survives_plain_hwp5_roundtrip() {
    let doc = make_doc_with_pic((117_780, 35_760));
    let bytes = serialize_document(&doc).expect("serialize");
    let doc2 = parse_document(&bytes).expect("reparse");
    assert_eq!(
        find_pic(&doc2).img_dim,
        (117_780, 35_760),
        "imgDim (0,0) 소실 회귀 (#1929)"
    );

    // 2-round 안정성: 재파스 IR(raw_picture_extra 채워짐)의 재직렬화도 보존.
    let bytes2 = serialize_document(&doc2).expect("serialize 2");
    let doc3 = parse_document(&bytes2).expect("reparse 2");
    assert_eq!(find_pic(&doc3).img_dim, (117_780, 35_760), "2-round 보존");
}

/// img_dim 없는(0,0) 그림은 종전 crop 폴백 동작이 유지된다.
#[test]
fn issue_1929_zero_img_dim_keeps_crop_fallback() {
    let mut doc = make_doc_with_pic((0, 0));
    if let Control::Picture(p) = &mut doc.sections[0].paragraphs[0].controls[0] {
        p.crop.right = 4321;
        p.crop.bottom = 1234;
    }
    let bytes = serialize_document(&doc).expect("serialize");
    let doc2 = parse_document(&bytes).expect("reparse");
    // crop 폴백값이 원본 크기 칸에 기록되고 재파스 img_dim 으로 읽힌다.
    assert_eq!(find_pic(&doc2).img_dim, (4321, 1234), "crop 폴백 유지");
}
