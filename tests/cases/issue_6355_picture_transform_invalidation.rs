//! [#6355] 그림 변환 파생 상태 무효화는 값 변화로 판정한다.
//!
//! 종전 `picture_props_touch_shape_transform` 은 7개 변환 키를 `props_json.contains(key)`
//! 로 **텍스트 등장**만 봤다. 값이 그대로여도, 심지어 키 이름이 문자열 값 안에 우연히
//! 등장하기만 해도 한컴 원본 렌더링 행렬(`raw_rendering`)을 파괴했다. 저장기는
//! `raw_rendering` 이 비어 있을 때만 행렬을 새로 만들므로(`src/serializer/control.rs` 의
//! rendering 블록) 파괴는 곧 원본 손실이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;

/// 한컴 편집기가 만든 34° 회전 그림 — 표 셀 안.
/// `tests/issue_1279_picture_rotation_save.rs` 가 같은 샘플로 회전 행렬 계약(146B)을 고정한다.
const SAMPLE: &str = "samples/ta-pic-001-r.hwp";
const CELL_PATH: &str = r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#;

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

/// 원본 행렬을 함께 돌려준다 — 판정은 "이 바이트가 그대로 남았는가" 다.
fn loaded() -> (DocumentCore, Vec<u8>) {
    let core = DocumentCore::from_bytes(&read_fixture(SAMPLE)).expect("파싱");
    let raw_rendering = cell_picture(&core).shape_attr.raw_rendering.clone();
    assert!(
        raw_rendering.len() >= 146,
        "전제: 한컴 원본이 회전 행렬을 담고 있다"
    );
    (core, raw_rendering)
}

/// 같은 값을 다시 지정하는 것만으로 한컴 원본 렌더링 행렬을 파괴하지 않는다.
///
/// 수정 전에는 146B 가 0B 로 날아갔다(직렬화기가 자체 생성 행렬로 대체).
#[test]
fn same_valued_transform_props_keep_raw_rendering() {
    let (mut core, original) = loaded();
    let bag = {
        let pic = cell_picture(&core);
        format!(
            r#"{{"width":{},"height":{},"horzOffset":{},"vertOffset":{},"horzFlip":{},"vertFlip":{}}}"#,
            pic.common.width,
            pic.common.height,
            pic.common.horizontal_offset as i32,
            pic.common.vertical_offset as i32,
            pic.shape_attr.horz_flip,
            pic.shape_attr.vert_flip,
        )
    };

    core.set_cell_picture_properties_by_path_native(0, 0, CELL_PATH, 0, &bag)
        .expect("동일 값 재적용");

    assert_eq!(
        cell_picture(&core).shape_attr.raw_rendering,
        original,
        "값이 그대로면 원본 렌더링 행렬을 유지해야 한다"
    );
}

/// 키 이름이 **문자열 값 안에** 등장하기만 해도 파괴되던 텍스트 스캔 판정 회귀.
#[test]
fn transform_key_quoted_inside_a_value_keeps_raw_rendering() {
    let (mut core, original) = loaded();

    // `note` 는 setter 가 소비하지 않는 키다 — 변환 필드는 하나도 바뀌지 않는다.
    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        CELL_PATH,
        0,
        r#"{"note":"\"width\" 언급"}"#,
    )
    .expect("무관 속성 적용");

    assert_eq!(
        cell_picture(&core).shape_attr.raw_rendering,
        original,
        "변환 키가 문자열 값 안에 있을 뿐이면 행렬을 건드리지 않는다"
    );
}

/// 변환과 무관한 속성(밝기)은 렌더링 행렬을 건드리지 않는다.
#[test]
fn non_transform_props_keep_raw_rendering() {
    let (mut core, original) = loaded();

    core.set_cell_picture_properties_by_path_native(0, 0, CELL_PATH, 0, r#"{"brightness":20}"#)
        .expect("밝기 변경");

    assert_eq!(
        cell_picture(&core).shape_attr.raw_rendering,
        original,
        "밝기는 변환이 아니다"
    );
}

/// 실제 변환 변경은 종전대로 파생 상태를 무효화한다 — 직렬화기가 새 크기에 맞는 행렬을
/// 다시 만들어야 하므로 원본 바이트를 비우는 계약은 유지된다.
#[test]
fn changed_transform_props_still_invalidate_raw_rendering() {
    let (mut core, _original) = loaded();
    let wider = cell_picture(&core).common.width + 2000;

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        CELL_PATH,
        0,
        &format!(r#"{{"width":{wider}}}"#),
    )
    .expect("폭 변경");

    assert!(
        cell_picture(&core).shape_attr.raw_rendering.is_empty(),
        "실제 크기 변경은 원본 행렬을 비워 직렬화기가 재생성하게 해야 한다"
    );
}
