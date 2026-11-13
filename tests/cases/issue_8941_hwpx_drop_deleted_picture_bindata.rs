//! 본문에서 지운 그림은 HWPX 저장 파일의 `BinData/` 에도 남지 않는다.
//! 불러올 때부터 참조가 없던 항목은 원본 그대로 옮긴다.
//! 검사 유형: 구조·관계형 — 저장 패키지의 자원 소유·바이트 보존·참조 정합을 검사한다.
//! PDF 일치율은 비해당: 그림의 화면 좌표나 모양을 고정하지 않는 직렬화 계약이다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// 1x1 빨강 PNG
const RED_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0xc9, 0xfe, 0x92, 0xef, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
    0x44, 0xae, 0x42, 0x60, 0x82,
];

/// 1x1 파랑 PNG
const BLUE_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60, 0x60, 0xf8, 0x0f,
    0x00, 0x01, 0x03, 0x01, 0x00, 0x08, 0x89, 0xc2, 0xec, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
    0x44, 0xae, 0x42, 0x60, 0x82,
];

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");
    core
}

fn insert_picture(core: &mut DocumentCore, png: &[u8]) {
    core.insert_picture_native(0, 0, 0, &[], png, 7500, 7500, 1, 1, "png", "", None, None)
        .expect("insert picture");
}

/// 첫 문단의 그림 컨트롤 위치
fn picture_controls(core: &DocumentCore) -> Vec<usize> {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter(|(_, control)| matches!(control, Control::Picture(_)))
        .map(|(index, _)| index)
        .collect()
}

/// 저장 파일의 `BinData/` 항목 내용
fn saved_bin_data(hwpx: &[u8]) -> Vec<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(hwpx)).expect("zip");
    let mut contents = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        if entry.name().expect("entry name").starts_with("BinData/") {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("read entry");
            contents.push(bytes);
        }
    }
    contents
}

#[test]
fn deleted_picture_is_not_saved() {
    let mut core = blank();
    insert_picture(&mut core, RED_PNG);
    let picture = picture_controls(&core)[0];
    core.delete_picture_control_native(0, 0, picture)
        .expect("delete picture");

    let hwpx = core.export_hwpx_native().expect("export hwpx");
    assert!(saved_bin_data(&hwpx).is_empty());
}

#[test]
fn remaining_picture_keeps_its_own_bytes() {
    let mut core = blank();
    insert_picture(&mut core, RED_PNG);
    insert_picture(&mut core, BLUE_PNG);
    let red = picture_controls(&core)
        .into_iter()
        .find(|&index| {
            let Control::Picture(picture) =
                &core.document().sections[0].paragraphs[0].controls[index]
            else {
                unreachable!()
            };
            picture.image_attr.bin_data_id == 1
        })
        .expect("red picture");
    core.delete_picture_control_native(0, 0, red)
        .expect("delete red picture");

    let hwpx = core.export_hwpx_native().expect("export hwpx");
    assert_eq!(saved_bin_data(&hwpx), vec![BLUE_PNG.to_vec()]);

    // 다시 연 문서의 남은 그림이 파랑을 가리킨다.
    let reopened = DocumentCore::from_bytes(&hwpx).expect("reopen");
    let document = reopened.document();
    let Control::Picture(picture) = document.sections[0].paragraphs[0]
        .controls
        .iter()
        .find(|control| matches!(control, Control::Picture(_)))
        .expect("remaining picture")
    else {
        unreachable!()
    };
    let content = &document.bin_data_content[picture.image_attr.bin_data_id as usize - 1];
    assert_eq!(content.data.load(), BLUE_PNG);
}

#[test]
fn undo_after_save_brings_the_picture_back() {
    let mut core = blank();
    insert_picture(&mut core, RED_PNG);
    let before_delete = core.save_snapshot_native();
    let picture = picture_controls(&core)[0];
    core.delete_picture_control_native(0, 0, picture)
        .expect("delete picture");
    assert!(saved_bin_data(&core.export_hwpx_native().expect("export")).is_empty());

    core.restore_snapshot_native(before_delete).expect("undo");

    let hwpx = core.export_hwpx_native().expect("export after undo");
    assert_eq!(saved_bin_data(&hwpx), vec![RED_PNG.to_vec()]);
}

#[test]
fn bin_data_unreferenced_since_load_is_kept() {
    // 그림 2개 중 하나만 본문이 참조하는 원본
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue7816/odd_spacing_prev.hwpx"
    );
    let original = std::fs::read(path).expect("read sample");
    let core = DocumentCore::from_bytes(&original).expect("open sample");

    let saved = core.export_hwpx_native().expect("export hwpx");
    assert_eq!(saved_bin_data(&saved).len(), 2);
}

#[test]
fn picture_bullet_follows_its_image_when_an_earlier_picture_is_dropped() {
    let mut core = blank();
    insert_picture(&mut core, RED_PNG);
    core.create_bullet_native(r#"{"char":"●"}"#, Some((BLUE_PNG, "png")))
        .expect("picture bullet");
    let picture = picture_controls(&core)[0];
    core.delete_picture_control_native(0, 0, picture)
        .expect("delete picture");

    let hwpx = core.export_hwpx_native().expect("export hwpx");
    assert_eq!(saved_bin_data(&hwpx), vec![BLUE_PNG.to_vec()]);

    let reopened = DocumentCore::from_bytes(&hwpx).expect("reopen");
    let document = reopened.document();
    let image = document
        .doc_info
        .bullets
        .last()
        .expect("bullet")
        .image_reference();
    let content = &document.bin_data_content[usize::from(image.expect("bullet image")) - 1];
    assert_eq!(content.data.load(), BLUE_PNG);
}

/// 그림 컨트롤이 실제로 소유하는 바이트를 모은다. 저장 순번 자체를 기대값으로 고정하지 않는다.
fn owned_picture_bytes(core: &DocumentCore) -> Vec<Vec<u8>> {
    core.document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Picture(picture) => Some(
                core.document().bin_data_content[usize::from(picture.image_attr.bin_data_id - 1)]
                    .data
                    .load()
                    .to_vec(),
            ),
            _ => None,
        })
        .collect()
}

#[test]
fn remaining_picture_survives_two_saves_in_both_formats_without_mutating_live_resources() {
    for hwpx in [true, false] {
        let mut core = blank();
        insert_picture(&mut core, RED_PNG);
        insert_picture(&mut core, BLUE_PNG);
        let red = picture_controls(&core)
            .into_iter()
            .find(|&index| {
                matches!(&core.document().sections[0].paragraphs[0].controls[index],
                Control::Picture(picture) if picture.image_attr.bin_data_id == 1)
            })
            .expect("first picture owns the red payload");
        core.delete_picture_control_native(0, 0, red)
            .expect("delete first picture");
        let live_resources = core.document().bin_data_content.len();
        assert_eq!(owned_picture_bytes(&core), vec![BLUE_PNG.to_vec()]);
        let save = |document: &DocumentCore| {
            if hwpx {
                document.export_hwpx_native().expect("save HWPX")
            } else {
                document.export_hwp_native().expect("save HWP")
            }
        };
        let bytes = save(&core);
        assert_eq!(core.document().bin_data_content.len(), live_resources);
        if hwpx {
            assert_eq!(saved_bin_data(&bytes), vec![BLUE_PNG.to_vec()]);
        }
        let reopened = DocumentCore::from_bytes(&bytes).expect("first reopen");
        assert_eq!(owned_picture_bytes(&reopened), vec![BLUE_PNG.to_vec()]);
        let second = save(&reopened);
        if hwpx {
            assert_eq!(saved_bin_data(&second), vec![BLUE_PNG.to_vec()]);
        }
        let reopened_twice = DocumentCore::from_bytes(&second).expect("second reopen");
        assert_eq!(
            owned_picture_bytes(&reopened_twice),
            vec![BLUE_PNG.to_vec()]
        );
    }
}

#[test]
fn saving_after_deletion_keeps_the_internal_clipboard_picture_available() {
    let mut core = blank();
    insert_picture(&mut core, RED_PNG);
    let red = picture_controls(&core)[0];
    core.copy_control_native(0, 0, &[], red)
        .expect("copy picture");
    core.delete_picture_control_native(0, 0, red)
        .expect("delete copied picture");
    assert!(saved_bin_data(&core.export_hwpx_native().expect("save after deletion")).is_empty());
    core.paste_control_native(0, 0, 0)
        .expect("paste after save");
    assert_eq!(owned_picture_bytes(&core), vec![RED_PNG.to_vec()]);
    let saved = core.export_hwpx_native().expect("save restored picture");
    assert_eq!(saved_bin_data(&saved), vec![RED_PNG.to_vec()]);
    let reopened = DocumentCore::from_bytes(&saved).expect("reopen restored picture");
    assert_eq!(owned_picture_bytes(&reopened), vec![RED_PNG.to_vec()]);
}
