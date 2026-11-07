//! 관계형·고정형 — 띄운 그림을 다른 문단·칸 문단으로 옮겨 붙이면 소속과 오프셋이 바뀌고 저장 후에도 남으며,
//! 다른 쪽 칸으로 옮긴 그림은 그 칸 안에 그려진다 (#8557). 고정형은 독립 한컴 인쇄의 전체 쪽수(4)뿐이다.
//! 주석 갱신: 2026-11-06T22:17:14+09:00 (Asia/Seoul; 메인테이너 경계 검사 추가)
//! 직전 파일 변경: a1146f76a0cbafcb11691f73b576dcb428bd62e0 2026-11-05T09:16:56+09:00 (Git 시각; 아래 측정일과 구분).
//! 추가 합성 검사는 실패 원자성·기본 인수·반환 소유 경로만 검사하며 새 배치를 고정하지 않는다.
//! 앞 세 검사는 합성 문서의 소속·개수·오프셋과 그림 데이터 보존만 본다(시각 측정 해당 없음).
//! 쪽·칸 소속 검사: 2026-11-05 측정, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`
//! raw 실루엣(2px 이웃, Native 는 스크립트의 웹폰트 자동 공급),
//! 입력 `samples/issue8557/21868765_relocated_picture.hwp`(같은 편집, 192px 그림),
//! 기준 `pdf/issue8557/21868765_relocated_picture-hwp-mac-12.30.pdf`(한글 2022 맥 12.30.0 인쇄, 4쪽).
//! Native 95.36·97.64·96.13·97.17, fresh WASM 98.57·99.69·98.83·99.40 (각 4쪽, 최저 95.36·98.57).
//! 증거: `mydocs/pr/assets/issue_8557_relocate_dragged_picture/`.
//! 기대값 근거: 한컴은 개체를 다른 쪽으로 끌어 놓으면 앵커(조판 부호)를 놓은 자리의 문단으로 옮긴다.
//! 옮기는 동안 그림은 하나뿐이어야 하고(복제·소실 없음), 그림 데이터(BinData)는 그대로 쓰인다.
//! 칸 문단으로 옮긴 그림은 한컴 칸 그림 저장 형태(세로 문단·가로 단 기준, 본문 영역 제한)를 따른다.
//! 한글 2022(맥 12.30)에서 종이 기준·제한 없음 그대로 칸에 옮긴 그림을 인쇄하면 칸이 아니라 1쪽 맨 위에 그려졌다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::{HorzRelTo, VertRelTo};

/// 1×1 PNG (투명 픽셀 하나).
const PNG: [u8; 65] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60,
    0x82,
];

/// 문서 안 모든 그림의 (칸 안인가, 가로 오프셋, 세로 오프셋, 그림 데이터 번호).
fn pictures(doc: &Document) -> Vec<(bool, i32, i32, u16)> {
    let mut out = Vec::new();
    for para in &doc.sections[0].paragraphs {
        for control in &para.controls {
            match control {
                Control::Picture(pic) => out.push((
                    false,
                    pic.common.horizontal_offset as i32,
                    pic.common.vertical_offset as i32,
                    pic.image_attr.bin_data_id,
                )),
                Control::Table(table) => {
                    for cell in &table.cells {
                        for cell_para in &cell.paragraphs {
                            for inner in &cell_para.controls {
                                if let Control::Picture(pic) = inner {
                                    out.push((
                                        true,
                                        pic.common.horizontal_offset as i32,
                                        pic.common.vertical_offset as i32,
                                        pic.image_attr.bin_data_id,
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// 칸 문단 안 그림 하나의 (세로 기준, 가로 기준, 본문 영역 제한).
fn cell_picture_anchor(doc: &Document) -> (VertRelTo, HorzRelTo, bool) {
    doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|c| match c {
            Control::Table(t) => t
                .cells
                .iter()
                .flat_map(|cell| &cell.paragraphs)
                .find_map(|cp| {
                    cp.controls.iter().find_map(|inner| match inner {
                        Control::Picture(pic) => Some((
                            pic.common.vert_rel_to,
                            pic.common.horz_rel_to,
                            pic.common.flow_with_text,
                        )),
                        _ => None,
                    })
                }),
            _ => None,
        })
        .expect("칸 문단 그림")
}

fn control_index(core: &DocumentCore, para: usize, pick: fn(&Control) -> bool) -> usize {
    core.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .position(pick)
        .expect("컨트롤")
}

#[test]
fn floating_picture_relocates_between_body_and_cell_paragraphs() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("유효한 글자·문단 모양 정의");
    core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    let para = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .expect("표 문단");
    core.insert_picture_native(
        0,
        para,
        0,
        &[],
        &PNG,
        4000,
        2000,
        1,
        1,
        "png",
        "서명",
        Some(100),
        Some(200),
    )
    .expect("본문 그림");
    let [(false, 100, 200, bin_id)] = pictures(core.document())[..] else {
        panic!("전제: 본문 그림 하나 {:?}", pictures(core.document()));
    };
    let pic = control_index(&core, para, |c| matches!(c, Control::Picture(_)));
    let table = control_index(&core, para, |c| matches!(c, Control::Table(_)));

    // 본문 → 칸 문단 (문단 위로 걸친 음수 오프셋 포함)
    core.relocate_floating_picture_native(
        0,
        para,
        &[],
        pic,
        0,
        para,
        0,
        &[(table, 0, 0)],
        1200,
        -300,
    )
    .expect("칸으로 옮기기");
    assert_eq!(pictures(core.document()), vec![(true, 1200, -300, bin_id)]);
    let in_cell =
        DocumentCore::from_bytes(&core.export_hwp_native().expect("칸 저장")).expect("다시 열기");
    assert_eq!(
        cell_picture_anchor(in_cell.document()),
        (VertRelTo::Para, HorzRelTo::Column, true),
        "칸 문단 그림은 칸 기준·본문 영역 제한"
    );

    // 칸 문단 → 본문
    let table = control_index(&core, para, |c| matches!(c, Control::Table(_)));
    core.relocate_floating_picture_native(0, para, &[(table, 0, 0)], 0, 0, para, 0, &[], 700, 900)
        .expect("본문으로 되돌리기");
    assert_eq!(pictures(core.document()), vec![(false, 700, 900, bin_id)]);

    let reopened =
        DocumentCore::from_bytes(&core.export_hwp_native().expect("저장")).expect("다시 열기");
    let saved = pictures(reopened.document());
    assert_eq!(saved.len(), 1, "저장 후에도 그림은 하나");
    assert_eq!((saved[0].0, saved[0].1, saved[0].2), (false, 700, 900));
}

#[test]
fn relocation_rejects_a_missing_control_without_changing_the_document() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("유효한 글자·문단 모양 정의");
    core.insert_picture_native(
        0,
        0,
        0,
        &[],
        &PNG,
        4000,
        2000,
        1,
        1,
        "png",
        "서명",
        Some(10),
        Some(20),
    )
    .expect("본문 그림");
    let before = pictures(core.document());
    assert!(core
        .relocate_floating_picture_native(0, 0, &[], 99, 0, 0, 0, &[], 0, 0)
        .is_err());
    assert!(
        core.relocate_floating_picture_native(0, 0, &[], 0, 0, 0, 0, &[(9, 0, 0)], 0, 0)
            .is_err(),
        "없는 도착 칸도 거절한다"
    );
    assert_eq!(pictures(core.document()), before);
}

#[test]
fn moving_a_picture_out_of_its_paragraph_renumbers_the_destination_table_path() {
    // 그림이 같은 문단의 표보다 앞 번호이면, 그림이 빠진 뒤 칸 경로의 표 번호가 하나 당겨진다.
    // 호출자는 옮기기 전 번호로 경로를 준다(끌어 놓기 시점의 적중 결과).
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("유효한 글자·문단 모양 정의");
    core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    let para = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .expect("표 문단");
    core.insert_picture_native(
        0,
        para,
        0,
        &[],
        &PNG,
        4000,
        2000,
        1,
        1,
        "png",
        "서명",
        Some(5),
        Some(6),
    )
    .expect("본문 그림");
    let pic = control_index(&core, para, |c| matches!(c, Control::Picture(_)));
    let table = control_index(&core, para, |c| matches!(c, Control::Table(_)));
    // 그림을 표 앞 자리로 둔다(한컴 문서에서 그림 조판 부호가 표보다 앞에 있는 경우).
    if pic > table {
        let p = &mut core.document_mut().sections[0].paragraphs[para];
        p.controls.swap(pic, table);
        if p.ctrl_data_records.len() > pic.max(table) {
            p.ctrl_data_records.swap(pic, table);
        }
    }
    let pic = control_index(&core, para, |c| matches!(c, Control::Picture(_)));
    let table = control_index(&core, para, |c| matches!(c, Control::Table(_)));
    assert!(pic < table, "전제: 그림이 표보다 앞 번호");

    let reply = core
        .relocate_floating_picture_native(0, para, &[], pic, 0, para, 0, &[(table, 0, 0)], 300, 400)
        .expect("같은 문단의 뒤쪽 표 칸으로 옮기기");
    let reply: serde_json::Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(
        reply["cellPath"],
        serde_json::json!([{
            "controlIndex": table - 1, "cellIndex": 0, "cellParaIndex": 0
        }]),
        "삭제 전 hit-test 경로 대신 실제 도착 소유 경로를 반환한다"
    );
    assert_eq!(
        pictures(core.document())
            .iter()
            .map(|p| (p.0, p.1, p.2))
            .collect::<Vec<_>>(),
        vec![(true, 300, 400)],
        "그림은 칸 문단 안에 하나"
    );
}

/// 쪽마다 그림 노드의 (bbox, 그림을 품은 칸의 bbox, 그 칸의 글).
fn page_images(doc: &rhwp::wasm_api::HwpDocument) -> Vec<Vec<([f64; 4], [f64; 4], String)>> {
    fn bbox(node: &serde_json::Value) -> [f64; 4] {
        let b = &node["bbox"];
        let v = |k: &str| b[k].as_f64().unwrap();
        [v("x"), v("y"), v("x") + v("w"), v("y") + v("h")]
    }
    fn texts(node: &serde_json::Value, out: &mut String) {
        if node["type"] == "TextRun" {
            out.push_str(node["text"].as_str().unwrap_or_default());
        }
        for child in node["children"].as_array().into_iter().flatten() {
            texts(child, out);
        }
    }
    fn walk<'a>(
        node: &'a serde_json::Value,
        cell: Option<&'a serde_json::Value>,
        out: &mut Vec<([f64; 4], [f64; 4], String)>,
    ) {
        let cell = if node["type"] == "Cell" {
            Some(node)
        } else {
            cell
        };
        if node["type"] == "Image" {
            let cell = cell.expect("그림이 칸 밖에 그려졌다");
            let mut text = String::new();
            texts(cell, &mut text);
            out.push((bbox(node), bbox(cell), text));
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, cell, out);
        }
    }
    (0..doc.page_count())
        .map(|page| {
            let tree: serde_json::Value =
                serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
            let mut out = Vec::new();
            walk(&tree, None, &mut out);
            out
        })
        .collect()
}

#[test]
fn picture_moved_to_a_later_page_cell_is_drawn_inside_that_cell() {
    // 1쪽 제목 문단에 띄운 그림을 3쪽 표 80행 내용 칸(「9. 통합건강증진사업에 관한 업무」)으로 옮긴다.
    // 한글 2022(맥 12.31) 인쇄: 전체 4쪽, 그림은 3쪽 그 칸 안에 한 번만 그려진다.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21868765_별표2_보건소_분장사무.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).expect("표본");
    core.insert_picture_native(
        0,
        2,
        0,
        &[],
        &PNG,
        2800,
        2800,
        1,
        1,
        "png",
        "그림",
        Some(30000),
        Some(0),
    )
    .expect("본문 그림");
    let pic = control_index(&core, 2, |c| matches!(c, Control::Picture(_)));
    core.relocate_floating_picture_native(0, 2, &[], pic, 0, 4, 0, &[(0, 242, 0)], 22000, 0)
        .expect("3쪽 칸으로 옮기기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&core.export_hwp_native().unwrap())
        .expect("저장본");
    let pages = page_images(&doc);
    assert_eq!(pages.len(), 4, "한컴 인쇄와 같은 4쪽");
    let drawn: Vec<usize> = pages
        .iter()
        .enumerate()
        .flat_map(|(page, images)| std::iter::repeat_n(page, images.len()))
        .collect();
    assert_eq!(drawn, vec![2], "그림은 3쪽에 한 번만");
    let (image, cell, text) = &pages[2][0];
    assert!(
        text.contains("통합건강증진사업에"),
        "그림을 품은 칸은 옮겨 놓은 9번 행 칸: {text}"
    );
    assert!(
        image[0] >= cell[0] - 0.5
            && image[1] >= cell[1] - 0.5
            && image[2] <= cell[2] + 0.5
            && image[3] <= cell[3] + 0.5,
        "그림 {image:?} 이 칸 {cell:?} 안에 있다"
    );
}

#[test]
fn unsupported_caption_destination_preserves_the_source_picture() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_picture_native(
        0,
        0,
        0,
        &[],
        &PNG,
        4000,
        2000,
        1,
        1,
        "png",
        "서명",
        Some(100),
        Some(200),
    )
    .unwrap();
    let picture = control_index(&core, 0, |c| matches!(c, Control::Picture(_)));
    core.set_picture_properties_native(0, 0, picture, r#"{"hasCaption":true}"#)
        .unwrap();
    let before = core.export_hwp_native().unwrap();
    assert!(core
        .relocate_floating_picture_native(0, 0, &[], picture, 0, 0, 0, &[(picture, 0, 0)], 0, 0)
        .is_err());
    assert_eq!(
        core.export_hwp_native().unwrap(),
        before,
        "지원하지 않는 캡션/순환 경로는 그림 제거 전에 거절한다"
    );
    assert_eq!(pictures(core.document()).len(), 1);
}

#[test]
fn relocation_reports_and_preserves_nested_cell_ownership() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.create_table_native(0, 0, 0, 1, 1).unwrap();
    let para = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .unwrap();
    let table = control_index(&core, para, |c| matches!(c, Control::Table(_)));
    let nested = core
        .create_table_in_cell_by_path_native(0, para, &[(table, 0, 0)], 0, 1, 1, None, None, true)
        .unwrap();
    let nested: serde_json::Value = serde_json::from_str(&nested).unwrap();
    let wire = nested["cellPath"].as_array().unwrap();
    let path: Vec<(usize, usize, usize)> = wire
        .iter()
        .map(|e| {
            (
                e["controlIndex"]
                    .as_u64()
                    .or_else(|| e["controlIdx"].as_u64())
                    .unwrap() as usize,
                e["cellIndex"]
                    .as_u64()
                    .or_else(|| e["cellIdx"].as_u64())
                    .unwrap() as usize,
                e["cellParaIndex"]
                    .as_u64()
                    .or_else(|| e["cellParaIdx"].as_u64())
                    .unwrap() as usize,
            )
        })
        .collect();
    assert_eq!(path.len(), 2, "실제 중첩 칸 경로");
    core.insert_picture_native(
        0,
        0,
        0,
        &[],
        &PNG,
        1000,
        1000,
        1,
        1,
        "png",
        "중첩 칸 이동",
        Some(100),
        Some(200),
    )
    .unwrap();
    let picture = control_index(&core, 0, |c| matches!(c, Control::Picture(_)));
    let reply = core
        .relocate_floating_picture_native(0, 0, &[], picture, 0, para, 0, &path, -100, 200)
        .unwrap();
    let reply: serde_json::Value = serde_json::from_str(&reply).unwrap();
    let expected: Vec<_> = path
        .iter()
        .map(|&(c, cell, p)| {
            serde_json::json!({
                "controlIndex": c, "cellIndex": cell, "cellParaIndex": p
            })
        })
        .collect();
    assert_eq!(reply["cellPath"], serde_json::json!(expected));
    let mut owner = &core.document().sections[0].paragraphs[para];
    for &(control, cell, paragraph) in &path {
        let Control::Table(table) = &owner.controls[control] else {
            panic!("표 경로")
        };
        owner = &table.cells[cell].paragraphs[paragraph];
    }
    let Control::Picture(picture) = &owner.controls[reply["controlIdx"].as_u64().unwrap() as usize]
    else {
        panic!("중첩 칸 소유 그림")
    };
    assert_eq!(picture.common.horizontal_offset as i32, -100);
    assert_eq!(picture.common.vertical_offset as i32, 200);
    assert!(picture.common.flow_with_text);
}
