//! 흐름 표 칸의 글 뒤로 그림은 쪽 기준 글 뒤로 도형 위에 그려진다.
//!
//! 한글은 본문의 글 뒤로 도형을 먼저 그리고, 표는 칸 안 개체와 함께 그 위에 그린다.
//! 여권 발급 신청서의 사진 칸 얼굴 윤곽선(칸 문단의 글 뒤로 그림)은 같은 자리의 흰
//! 채우기 글 뒤로 사각형(용지 기준) 위에 보여야 하는데, 칸 그림이 쪽 전체 BehindText
//! plane 으로 올라가 사각형 아래에 깔려 사라졌다. 칸 그림은 칸 본문 텍스트 아래에는
//! 그대로 깔려야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Cursor;

use rhwp::model::bin_data::BinDataContent;
use rhwp::model::control::Control;
use rhwp::model::image::{ImageAttr, Picture};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{
    CommonObjAttr, DrawingObjAttr, HorzRelTo, RectangleShape, ShapeObject, TextWrap, VertRelTo,
};
use rhwp::model::style::{Fill, FillType, SolidFill};
use rhwp::model::table::{Cell, Table};
use rhwp::DocumentCore;

fn red_png() -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 4, image::Rgb([255, 0, 0])))
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

fn white_paper_rect() -> Control {
    Control::Shape(Box::new(ShapeObject::Rectangle(RectangleShape {
        common: CommonObjAttr {
            width: 20_000,
            height: 20_000,
            horizontal_offset: 8_000,
            vertical_offset: 5_000,
            z_order: 1,
            text_wrap: TextWrap::BehindText,
            horz_rel_to: HorzRelTo::Paper,
            vert_rel_to: VertRelTo::Paper,
            ..Default::default()
        },
        drawing: DrawingObjAttr {
            fill: Fill {
                fill_type: FillType::Solid,
                solid: Some(SolidFill {
                    background_color: 0x0000_FF00,
                    pattern_color: 0,
                    pattern_type: 0,
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    })))
}

fn table_with_behind_text_picture() -> Control {
    let picture = Control::Picture(Box::new(Picture {
        common: CommonObjAttr {
            width: 7_200,
            height: 7_200,
            z_order: 0,
            text_wrap: TextWrap::BehindText,
            horz_rel_to: HorzRelTo::Para,
            vert_rel_to: VertRelTo::Para,
            ..Default::default()
        },
        image_attr: ImageAttr {
            bin_data_id: 1,
            ..Default::default()
        },
        ..Default::default()
    }));
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            col_span: 1,
            row_span: 1,
            width: 30_000,
            height: 20_000,
            paragraphs: vec![Paragraph {
                text: "PHOTO".to_owned(),
                char_count: 5,
                char_offsets: (0..=5).collect(),
                line_segs: vec![LineSeg {
                    line_height: 1_000,
                    text_height: 1_000,
                    baseline_distance: 850,
                    segment_width: 30_000,
                    ..Default::default()
                }],
                controls: vec![picture],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    table.common.width = 30_000;
    table.common.height = 20_000;
    table.common.treat_as_char = true;
    table.rebuild_grid();
    Control::Table(Box::new(table))
}

#[test]
fn flow_cell_behind_text_picture_paints_above_paper_behind_text_shape() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    let para = &mut doc.sections[0].paragraphs[0];
    para.controls.push(white_paper_rect());
    para.controls.push(table_with_behind_text_picture());
    doc.bin_data_content.push(BinDataContent {
        id: 1,
        data: red_png().into(),
        extension: "png".to_owned(),
    });
    core.set_document(doc);

    let svg = core.render_page_svg_layer_native(0).unwrap();
    let rect = svg
        .find("fill=\"#00ff00\"")
        .expect("쪽 기준 사각형이 그려져야 한다");
    let picture = svg.find("<image").expect("칸 그림이 그려져야 한다");
    let text = svg.find(">P<").expect("칸 본문 텍스트가 그려져야 한다");
    assert!(
        rect < picture,
        "칸 그림은 쪽 기준 글 뒤로 사각형 위에 그려져야 한다"
    );
    assert!(
        picture < text,
        "칸 그림은 칸 본문 텍스트 아래에 깔려야 한다"
    );
}
