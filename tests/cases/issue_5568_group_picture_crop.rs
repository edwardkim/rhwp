//! [#5568] 묶음(container) 자식 그림의 자르기(imgClip) 렌더 계약.
//!
//! 묶음은 원본 하나를 imgClip 띠로 나눠 쓰는 문서가 있다(같은 bin_data 를
//! 자식마다 다른 영역으로 자름 — 20211008101929000 8쪽 인포그래픽). 그룹 자식
//! 경로가 `ImageNode.crop`/`original_size_hu` 를 싣지 않으면 원본 전체가 대상
//! 상자에 압착돼 비율이 깨진다. SVG 렌더러는 crop 이 있으면 중첩
//! `<svg x=.. viewBox=..>` 로 잘린 영역만 표시한다 — 그 방출 여부가 판정이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::image::CropInfo;
use rhwp::model::shape::{GroupShape, ShapeObject};
use rhwp::wasm_api::HwpDocument;

/// 8x8 빨강 단색 PNG (crop 판정은 이미지 픽셀 크기 파싱이 필요해 실제 PNG 를 쓴다).
const PNG_8X8: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, 0x08, 0x02, 0x00, 0x00, 0x00, 0x4B, 0x6D, 0x29,
    0xDC, 0x00, 0x00, 0x00, 0x12, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0x80,
    0x15, 0x61, 0x17, 0x1D, 0xB4, 0x12, 0x00, 0x28, 0xFF, 0x3F, 0xC1, 0x6E, 0xEC, 0xDF, 0x61, 0x00,
    0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// 그림 하나를 삽입한 뒤 그 그림을 묶음(Group) 자식으로 감싼 문서를 만든다.
/// `crop` 이 Some 이면 자식 그림에 자르기(아래 절반)와 imgDim 좌표 기준을 싣는다.
fn doc_with_grouped_picture(crop: Option<CropInfo>) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.insert_picture(
        0, 0, 0, "[]", PNG_8X8, 6000, 6000, 8, 8, "png", "", None, None,
    )
    .expect("그림 삽입");

    let paragraph = &mut doc.document_mut().sections[0].paragraphs[0];
    let ctrl_idx = paragraph
        .controls
        .iter()
        .position(|c| matches!(c, Control::Picture(_)))
        .expect("삽입된 그림 컨트롤");
    let Control::Picture(mut pic) = paragraph.controls.remove(ctrl_idx) else {
        unreachable!()
    };

    if let Some(c) = crop {
        pic.crop = c;
        // crop 좌표 기준 범위(imgDim) — HWPX imgClip 계약과 동일 축.
        pic.img_dim = (600, 600);
    }

    let mut group = GroupShape::default();
    group.common = pic.common.clone();
    group.shape_attr = pic.shape_attr.clone();
    group.children = vec![ShapeObject::Picture(pic)];
    paragraph.controls.insert(
        ctrl_idx,
        Control::Shape(Box::new(ShapeObject::Group(group))),
    );
    doc
}

fn nested_crop_svg_count(svg: &str) -> usize {
    // 루트 `<svg xmlns=..>` 와 달리 crop 방출은 `<svg x="..` 로 시작한다.
    svg.matches("<svg x=").count()
}

#[test]
fn group_child_picture_crop_emits_nested_viewbox_svg() {
    // 아래 절반만 표시하는 자르기: imgDim(600x600) 기준 y 300..600.
    let doc = doc_with_grouped_picture(Some(CropInfo {
        left: 0,
        top: 300,
        right: 600,
        bottom: 600,
    }));
    let svg = doc.render_page_svg(0).expect("SVG 렌더");
    assert!(
        svg.contains("<image") || nested_crop_svg_count(&svg) > 0,
        "그림이 렌더되지 않았다:\n{svg}"
    );
    assert!(
        nested_crop_svg_count(&svg) > 0,
        "묶음 자식 그림의 crop 이 소실됐다 — 중첩 <svg x=.. viewBox=..> 미방출:\n{svg}"
    );
}

#[test]
fn group_child_picture_without_crop_keeps_plain_image() {
    let doc = doc_with_grouped_picture(None);
    let svg = doc.render_page_svg(0).expect("SVG 렌더");
    assert!(svg.contains("<image"), "그림이 렌더되지 않았다:\n{svg}");
    assert_eq!(
        nested_crop_svg_count(&svg),
        0,
        "자르기 없는 그림에 crop 경로가 발동했다:\n{svg}"
    );
}
