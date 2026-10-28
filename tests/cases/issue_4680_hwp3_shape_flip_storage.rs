//! [#4680] HWP3 의 선·다각형이 `SHAPE_COMPONENT` storage flip 을 0 으로 내보내
//! 한글이 문서를 열지 못한다.
//!
//! ## 증상
//!
//! 264쪽 HWP3 문서(`1170000-200500003 독일의 법령체계와 입법심사기준`)의 53쪽에는
//! 표 칸 안에 묶음 개체가 있다(`$con` → `$rec`·`$lin`·`$pol`). 그 쪽 한 장만
//! `extract-pages` 로 뽑아도 한글이 `open=False` 로 거부한다. 표를 지우면 열린다.
//!
//! ## 근인 — 한 필드
//!
//! 한/글 저장본과 우리 산출물은 그 표 서브트리에서 레코드 225개의 태그·레벨 시퀀스가
//! **완전히 같다**. 그래서 레코드 단위로 이등분할 수 있었고, 이등분은 크기가 어긋난
//! `$lin`/`$pol` 의 `SHAPE_COMPONENT` 로 좁혀졌다. 그 레코드를 다시 바이트 구간으로
//! 좁히면 갈림점은 **offset 32..36 의 storage flip 4바이트 하나**다:
//!
//! - 그 4바이트만 한/글 값으로 바꾸면 `open=True`
//! - 나머지(행렬·선/채우기 꼬리 전부)를 한/글 값으로 바꿔도 그 4바이트가 0 이면 `open=False`
//!
//! `hwpx_to_hwp` 는 이미 같은 계약을 **사각형에만** 걸고 있었다(`#3930` 계열, 주석에
//! "이 storage 비트가 없으면 한컴이 개체 이후 레코드 스트림을 이어 읽지 못한다"고
//! 적혀 있다). 선·다각형·타원·호·곡선은 그 arm 을 타지 않아 0 으로 나갔다.
//!
//! ## 이 테스트가 잠그는 것
//!
//! HWP3 원본 경로(`convert_if_hwpx_source(_, FileFormat::Hwp3)`)를 지난 뒤
//! (1) 사각형이 아닌 도형도 storage flip 이 서고, (2) 글상자 유무로 값이 갈리며,
//! (3) 이미 값이 있는 도형(HWP5/HWPX 경로)은 건드리지 않는지를 본다.
//! 마지막으로 저장 바이트의 `SHAPE_COMPONENT` 해당 4바이트가 0 이 아닌지까지 본다 —
//! 그게 개방 거부의 실제 형상이다.
//!
//! 회전중심은 세우지 않는다. 사각형 arm 은 함께 세우지만, 이 결함에서는 그 구간만
//! 한/글 값으로 바꿔도 여전히 거부였다(`open=False`) — 근거 없는 값은 쓰지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{
    DrawingObjAttr, GroupShape, LineShape, PolygonShape, ShapeComponentAttr, ShapeObject,
};
use rhwp::parser::record::Record;
use rhwp::parser::tags;
use rhwp::parser::FileFormat;

/// HWP3 파서가 내놓는 모양 — storage flip 이 0 이고 크기만 있다.
fn shape_attr(flip: u32) -> ShapeComponentAttr {
    ShapeComponentAttr {
        original_width: 2000,
        original_height: 1000,
        current_width: 2000,
        current_height: 1000,
        flip,
        ..Default::default()
    }
}

fn line(flip: u32) -> ShapeObject {
    ShapeObject::Line(LineShape {
        drawing: DrawingObjAttr {
            shape_attr: shape_attr(flip),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn polygon_with_text_box() -> ShapeObject {
    let mut drawing = DrawingObjAttr {
        shape_attr: shape_attr(0),
        ..Default::default()
    };
    drawing.text_box = Some(Default::default());
    ShapeObject::Polygon(PolygonShape {
        drawing,
        ..Default::default()
    })
}

/// 실제 결함 자리와 같은 모양으로 담는다 — 도형은 **묶음 자식**이다.
/// 최상위 도형은 `SHAPE_COMPONENT` 가 ctrl_id 를 두 번 쓰므로 바이트 오프셋이 다르다.
fn doc_with(shapes: Vec<ShapeObject>) -> Document {
    let mut para = Paragraph::default();
    let group = ShapeObject::Group(GroupShape {
        children: shapes,
        ..Default::default()
    });
    para.controls.push(Control::Shape(Box::new(group)));
    let mut section = Section::default();
    section.paragraphs.push(para);
    let mut doc = Document::default();
    doc.sections.push(section);
    doc
}

/// CLI `convert` 가 HWP3 원본에 적용하는 것과 같은 경로.
fn convert_as_hwp3(doc: &mut Document) {
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(doc, FileFormat::Hwp3);
}

fn flips(doc: &Document) -> Vec<u32> {
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Group(g) => Some(g.children.iter().map(|c| c.shape_attr().flip)),
                _ => None,
            },
            _ => None,
        })
        .flatten()
        .collect()
}

/// 저장 바이트의 `SHAPE_COMPONENT` storage flip(offset 32..36)을 모은다.
fn saved_storage_flips(doc: &Document) -> Vec<u32> {
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(doc).expect("HWP5 직렬화 실패");
    let mut cfb = rhwp::parser::cfb_reader::CfbReader::open(&bytes).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header.get(36).is_some_and(|b| b & 0x01 != 0);
    let section = cfb
        .read_body_text_section(0, compressed, false)
        .expect("Section0 읽기");
    Record::read_all(&section)
        .expect("Section0 record 파싱")
        .into_iter()
        .filter(|r| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT)
        // 묶음 자식만 본다 — 최상위(묶음 자신)는 ctrl_id 를 두 번 써서 오프셋이 4 밀린다.
        .filter(|r| r.data.get(0..4) != r.data.get(4..8))
        .filter_map(|r| {
            let raw = r.data.get(32..36)?;
            Some(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
        })
        .collect()
}

/// 글상자 없는 선은 `0x0008_0000` 을 받는다 — 사각형 arm 과 같은 값.
#[test]
fn hwp3_line_gets_storage_flip() {
    let mut doc = doc_with(vec![line(0)]);
    convert_as_hwp3(&mut doc);
    assert_eq!(
        flips(&doc),
        vec![0x0008_0000],
        "사각형이 아닌 도형도 storage flip 을 받아야 한다 (한글 개방 조건)"
    );
}

/// 글상자가 있는 다각형은 글상자 비트까지 받는다.
#[test]
fn hwp3_polygon_with_text_box_gets_text_box_bit() {
    let mut doc = doc_with(vec![polygon_with_text_box()]);
    convert_as_hwp3(&mut doc);
    assert_eq!(
        flips(&doc),
        vec![0x0108_0000],
        "글상자가 있으면 글상자 비트(0x0100_0000)까지 선다"
    );
}

/// 반례 — 이미 값이 있는 도형(HWP5·HWPX 경로)은 건드리지 않는다.
#[test]
fn existing_storage_flip_is_preserved() {
    let mut doc = doc_with(vec![line(0x0002_0000)]);
    convert_as_hwp3(&mut doc);
    assert_eq!(
        flips(&doc),
        vec![0x0002_0000],
        "원본이 준 flip 은 덮어쓰지 않는다"
    );
}

/// 반례 — HWP3 원본이 아니면 이 보정은 걸리지 않는다.
#[test]
fn non_hwp3_source_is_untouched() {
    let mut doc = doc_with(vec![line(0)]);
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(&mut doc, FileFormat::Hwp);
    assert_eq!(
        flips(&doc),
        vec![0],
        "HWP5(Hwp) 원본 경로는 storage 보정 대상이 아니다"
    );
}

/// 개방 거부의 실제 형상 — 저장 바이트의 해당 4바이트가 0 이면 안 된다.
#[test]
fn saved_shape_component_storage_flip_is_not_zero() {
    let mut doc = doc_with(vec![line(0), polygon_with_text_box()]);
    convert_as_hwp3(&mut doc);
    let saved = saved_storage_flips(&doc);
    assert!(
        !saved.is_empty(),
        "SHAPE_COMPONENT 가 저장되지 않았다 — 테스트 전제가 깨졌다"
    );
    assert!(
        saved.iter().all(|v| *v != 0),
        "storage flip 0 인 SHAPE_COMPONENT 가 저장본에 남았다 (한글 개방 거부) — {:?}",
        saved
    );
}
