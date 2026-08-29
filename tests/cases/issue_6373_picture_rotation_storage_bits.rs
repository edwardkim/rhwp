//! [#6373] 회전 편집은 `flip` 저장 워드와 `rotate_image` 를 건드리지 않는다.
//!
//! 종전 `refresh_picture_rotation_layout_for_save` 는 각도와 무관하게
//! `rotate_image = true` 와 `flip |= 0x0008_0000`(bit19) 을 세웠다. 회전을 0 으로 되돌려도
//! 남아 되돌릴 경로가 없었다.
//!
//! 두 값은 회전 상태의 함수가 아니다 — `tools/hangul_rotation_oracle/EVIDENCE.md` 실측:
//! 한컴 저장본 5660개 개체에서 bit19 는 회전 개체 569건 중 559건이 꺼져 있고 비회전 개체
//! 5091건 중 4416건이 켜져 있다. 한글 2024 는 회전 0 그림의 bit19 를 켜 두고 34° 회전
//! 그림의 `rotateimage` 를 0 으로 둔다. 그래서 세우는 것도 지우는 것도 근거가 없다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;

/// 한컴이 만든 34° 회전 그림 — 표 셀 안.
/// `tests/issue_1279_picture_rotation_save.rs` 가 같은 샘플로 회전 행렬 계약을 고정한다.
const SAMPLE: &str = "samples/ta-pic-001-r.hwp";
const CELL_PATH: &str = r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#;
const ROTATE_IMAGE_BIT: u32 = 0x0008_0000;

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn cell_picture(core: &DocumentCore) -> &Picture {
    let table = match &core.document().sections[0].paragraphs[0].controls[2] {
        Control::Table(table) => table,
        other => panic!("샘플 좌표가 표가 아니다: {other:?}"),
    };
    table.cells[2].paragraphs[0]
        .controls
        .iter()
        .find_map(|ctrl| match ctrl {
            Control::Picture(pic) => Some(&**pic),
            _ => None,
        })
        .expect("셀에 그림이 없다")
}

fn set_rotation(core: &mut DocumentCore, degrees: i32) {
    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        CELL_PATH,
        0,
        &format!(r#"{{"rotationAngle":{degrees}}}"#),
    )
    .unwrap_or_else(|e| panic!("회전 {degrees} 적용: {e}"));
}

/// 회전 제거·복귀 왕복에서 두 값이 보존된다.
#[test]
fn issue_6373_rotation_edit_preserves_storage_flip_and_rotate_image() {
    let mut core = DocumentCore::from_bytes(&read_fixture(SAMPLE)).expect("파싱");

    let (flip, rotate_image, angle) = {
        let pic = cell_picture(&core);
        (
            pic.shape_attr.flip,
            pic.shape_attr.rotate_image,
            pic.shape_attr.rotation_angle,
        )
    };
    assert_eq!(angle, 34, "전제: 한컴 원본은 34° 회전");
    assert_ne!(
        flip & ROTATE_IMAGE_BIT,
        0,
        "전제: 원본에 bit19 가 켜져 있다"
    );
    assert!(
        !rotate_image,
        "전제: 한컴은 회전 그림에도 rotateImage=0 을 쓴다"
    );

    // 회전 제거 — 종전에는 이 시점에 rotate_image=true 가 되고 bit19 가 세워졌다.
    set_rotation(&mut core, 0);
    {
        let pic = cell_picture(&core);
        assert_eq!(pic.shape_attr.rotation_angle, 0, "회전이 제거됐다");
        assert_eq!(
            pic.shape_attr.flip, flip,
            "회전 편집이 flip 저장 워드를 바꾸지 말아야 한다"
        );
        assert_eq!(
            pic.shape_attr.rotate_image, rotate_image,
            "회전 편집이 rotate_image 를 바꾸지 말아야 한다"
        );
    }

    // 되돌리기도 성립한다.
    set_rotation(&mut core, 34);
    let pic = cell_picture(&core);
    assert_eq!(pic.shape_attr.rotation_angle, 34);
    assert_eq!(pic.shape_attr.flip, flip, "왕복 후에도 flip 이 보존된다");
    assert_eq!(pic.shape_attr.rotate_image, rotate_image);
}

/// 저장·재파싱에서도 보존된다 — HWP5 저장에 나가는 것은 `flip` 워드다.
#[test]
fn issue_6373_preserved_bits_survive_save_and_reparse() {
    let mut core = DocumentCore::from_bytes(&read_fixture(SAMPLE)).expect("파싱");
    let flip = cell_picture(&core).shape_attr.flip;

    set_rotation(&mut core, 0);
    let saved = core.export_hwp_native().expect("저장");

    let reparsed = DocumentCore::from_bytes(&saved).expect("재파싱");
    let pic = cell_picture(&reparsed);
    assert_eq!(
        pic.shape_attr.rotation_angle, 0,
        "회전 제거가 저장에 반영됐다"
    );
    assert_eq!(
        pic.shape_attr.flip, flip,
        "회전 제거가 flip 저장 워드를 바꾸지 말아야 한다"
    );
}
