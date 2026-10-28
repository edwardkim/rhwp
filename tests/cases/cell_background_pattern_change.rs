#![cfg(not(target_arch = "wasm32"))]
//! 칸 배경의 면 색을 그대로 두고 무늬 종류나 무늬 색만 바꿔도 칸 배경이 바뀐다.
//!
//! 테두리/배경 정의를 재사용할 때 면 색만 비교하면, 무늬만 다른 정의를 같은 것으로 보고 옛
//! 정의를 돌려준다. 그러면 무늬를 넣거나 바꾸거나 지워도 칸 배경이 그대로다. 바꾼 무늬는
//! HWP·HWPX로 저장해 다시 열어도 남아야 한다. 그러데이션·그림·투명도만 다른 정의도 같다.
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::style::{BorderFill, Fill, FillType, GradientFill, ImageFill, SolidFill};
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

/// 머리 글 아래 문단 1에 1행 `cols`열 표를 만든다.
fn table_doc(cols: u32) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, "머리").unwrap();
    doc.split_paragraph(0, 0, 2, None).unwrap();
    doc.create_table(0, 1, 0, 1, cols).unwrap();
    doc
}

fn props(doc: &HwpDocument, cell: u32) -> Value {
    serde_json::from_str(&doc.get_cell_properties(0, 1, 0, cell).unwrap()).unwrap()
}

/// 첫 칸의 `(무늬 종류, 무늬 색)`.
fn pattern(doc: &HwpDocument) -> (i64, String) {
    let props = props(doc, 0);
    assert_eq!(props["fillColor"], "#ffff00", "면 색은 그대로다");
    (
        props["patternType"].as_i64().unwrap(),
        props["patternColor"].as_str().unwrap().to_string(),
    )
}

/// 셀 속성 대화상자처럼 현재 속성을 읽어 배경만 바꿔 다시 적용한다.
fn set_pattern(doc: &mut HwpDocument, pattern_type: i64, pattern_color: &str) {
    let mut props = props(doc, 0);
    props["fillType"] = "solid".into();
    props["fillColor"] = "#ffff00".into();
    props["patternType"] = pattern_type.into();
    props["patternColor"] = pattern_color.into();
    doc.set_cell_properties(0, 1, 0, 0, &props.to_string())
        .unwrap();
}

#[test]
fn changing_only_the_cell_pattern_changes_the_cell_background() {
    let mut doc = table_doc(1);

    // 무늬 없음 → 세로줄 → 같은 색의 격자 → 무늬 색만 파랑 → 다시 무늬 없음.
    for (pattern_type, pattern_color) in [
        (-1, "#000000"),
        (2, "#ff0000"),
        (5, "#ff0000"),
        (5, "#0000ff"),
        (-1, "#000000"),
    ] {
        set_pattern(&mut doc, pattern_type, pattern_color);
        let expected = (pattern_type, pattern_color.to_string());
        assert_eq!(pattern(&doc), expected, "편집 직후");
        for (format, bytes) in [
            ("HWP", doc.export_hwp().unwrap()),
            ("HWPX", doc.export_hwpx().unwrap()),
        ] {
            let reopened = HwpDocument::new(&bytes).unwrap();
            assert_eq!(pattern(&reopened), expected, "{format} 다시 열기");
        }
    }
}

fn table(document: &mut Document) -> &mut Table {
    match document.sections[0].paragraphs[1].controls.first_mut() {
        Some(Control::Table(table)) => table,
        _ => panic!("표를 찾지 못했다"),
    }
}

fn cell_fill(doc: &mut HwpDocument, cell: usize) -> Fill {
    let document = doc.document_mut();
    let id = table(document).cells[cell].border_fill_id;
    document.doc_info.border_fills[id as usize - 1].fill.clone()
}

/// 채우기만 다른 두 칸에 차례로 같은 굵은 테두리를 준다. 나중 칸이 먼저 칸의 새 정의를
/// 재사용하면 먼저 칸의 그러데이션·그림·투명도까지 넘겨받는다.
#[test]
fn giving_two_cells_the_same_border_keeps_each_cells_fill() {
    let gradient = |color| Fill {
        fill_type: FillType::Gradient,
        gradient: Some(GradientFill {
            gradient_type: 1,
            colors: vec![color, 0xffffff],
            positions: vec![0, 100],
            ..Default::default()
        }),
        ..Default::default()
    };
    let image = |bin_data_id| Fill {
        fill_type: FillType::Image,
        image: Some(ImageFill {
            bin_data_id,
            ..Default::default()
        }),
        ..Default::default()
    };
    let solid = |alpha| Fill {
        fill_type: FillType::Solid,
        solid: Some(SolidFill {
            background_color: 0x00ffff,
            pattern_color: 0,
            pattern_type: -1,
        }),
        alpha,
        ..Default::default()
    };

    let mut taken_over = Vec::new();
    for (kind, fills) in [
        ("그러데이션", [gradient(0x0000ff), gradient(0xff0000)]),
        ("그림", [image(1), image(2)]),
        ("투명도", [solid(255), solid(128)]),
    ] {
        let mut doc = table_doc(2);
        let document = doc.document_mut();
        for (cell, fill) in fills.iter().enumerate() {
            let base = table(document).cells[cell].border_fill_id as usize - 1;
            let bf = BorderFill {
                raw_data: None,
                fill: fill.clone(),
                ..document.doc_info.border_fills[base].clone()
            };
            document.doc_info.border_fills.push(bf);
            table(document).cells[cell].border_fill_id =
                document.doc_info.border_fills.len() as u16;
        }

        // 채우기 키 없이 테두리만 굵게 바꾼다.
        for cell in [0, 1] {
            let mut props = props(&doc, cell);
            for key in ["fillType", "fillColor", "patternColor", "patternType"] {
                props.as_object_mut().unwrap().remove(key);
            }
            for side in ["borderLeft", "borderRight", "borderTop", "borderBottom"] {
                props[side]["width"] = 3.into();
            }
            doc.set_cell_properties(0, 1, 0, cell, &props.to_string())
                .unwrap();
        }

        if [cell_fill(&mut doc, 0), cell_fill(&mut doc, 1)] != fills {
            taken_over.push(kind);
        }
    }
    assert!(
        taken_over.is_empty(),
        "앞 칸의 채우기를 넘겨받았다: {taken_over:?}"
    );
}
