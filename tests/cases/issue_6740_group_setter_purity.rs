#![cfg(not(target_arch = "wasm32"))]

//! [#6740] 묶음(Group) 속성 setter 의 변환 파생 상태 순수성.
//!
//! 직렬화기는 `raw_rendering` 이 비어 있을 때만 변환 행렬을 새로 만든다
//! (`serializer/control.rs` rendering 블록). 재생성본이 원본과 바이트 동일한 문서도
//! 있지만(파싱된 `render_*` 로 완전 복원되는 경우), 그렇지 않은 문서에서는 한컴 원본
//! 행렬이 사라진다 — 속성 bag 에 원본 바이트가 없어 되돌릴 길이 없다(#5890).
//!
//! 종전에는 `raw_rendering = Vec::new()` 가 `if let Some(new_w/new_h)` 가드 **밖**에
//! 있어 분기 조건이 "Group 인가" 하나뿐이었다 — 크기 키가 없는 속성이나 같은 값
//! 재적용에도 원본이 사라졌다. #6355 가 그림에서 쓴 지문 판정을 묶음에도 적용한다.
//!
//! 표본은 `group-box.hwp` 다 — 이 문서의 묶음은 `raw_rendering` 파괴가 저장 바이트를
//! 실제로 바꾼다(수정 전후 오프셋 632에서 갈림). 손실이 드러나지 않는 문서도 있으므로
//! (`draw-group.hwp`) 판정력이 있는 표본을 골랐다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const GROUP_SAMPLE: &str = "samples/group-box.hwp";
const SEC: usize = 0;
const PARA: usize = 0;
const CTRL: usize = 2;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(GROUP_SAMPLE);
    let bytes = std::fs::read(path).expect("표본 로드");
    DocumentCore::from_bytes(&bytes).expect("파싱")
}

fn group_raw_rendering_len(core: &DocumentCore) -> usize {
    match &core.document().sections[SEC].paragraphs[PARA].controls[CTRL] {
        Control::Shape(sh) => sh.as_ref().shape_attr().raw_rendering.len(),
        _ => panic!("표본 전제 위반: ctrl{CTRL} 가 Shape 가 아니다"),
    }
}

fn first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    let n = a.len().min(b.len());
    (0..n)
        .find(|&i| a[i] != b[i])
        .or((a.len() != b.len()).then_some(n))
}

/// 대조군 — 편집이라면 무엇이든 하는 구역 패스스루 무효화만 적용한 저장 바이트.
/// 무변경 속성 적용은 이 대조군을 넘어서는 비용을 내면 안 된다.
fn passthrough_only_export() -> Vec<u8> {
    let mut core = load();
    core.document_mut().sections[SEC].raw_stream = None;
    core.export_hwp_native().expect("대조군 export")
}

/// 크기 키가 없는 속성 적용은 변환을 바꾸지 않으므로 원본 행렬을 보존해야 한다.
#[test]
fn group_props_without_size_keys_preserves_raw_rendering() {
    let mut core = load();
    let before_len = group_raw_rendering_len(&core);
    assert!(
        before_len > 0,
        "표본 전제: 원본 raw_rendering 이 있어야 판정이 의미를 갖는다"
    );

    core.set_shape_properties_native(SEC, PARA, CTRL, "{}")
        .expect("빈 속성 적용");

    assert_eq!(
        group_raw_rendering_len(&core),
        before_len,
        "값 변화가 없는데 raw_rendering 이 사라졌다 — 한컴 원본 변환 행렬 손실"
    );
    assert_eq!(
        first_diff(
            &core.export_hwp_native().expect("export"),
            &passthrough_only_export()
        ),
        None,
        "무변경 속성 적용이 패스스루 무효화 이상의 바이트 비용을 냈다"
    );
}

/// 다이얼로그에서 아무것도 고치지 않고 확인을 누른 것과 동형(get∘set 항등).
#[test]
fn group_props_identity_roundtrip_preserves_raw_rendering() {
    let mut core = load();
    let before_len = group_raw_rendering_len(&core);

    let props = core
        .get_shape_properties_native(SEC, PARA, CTRL)
        .expect("현재 속성 조회");
    core.set_shape_properties_native(SEC, PARA, CTRL, &props)
        .expect("같은 값 재적용");

    assert_eq!(
        group_raw_rendering_len(&core),
        before_len,
        "get∘set 항등이 raw_rendering 을 파괴했다"
    );
    assert_eq!(
        first_diff(
            &core.export_hwp_native().expect("export"),
            &passthrough_only_export()
        ),
        None,
        "get∘set 항등이 패스스루 무효화 이상의 바이트 비용을 냈다"
    );
}

/// 과교정 방지 — 실제로 크기가 바뀌면 파생 행렬은 여전히 무효화돼야 한다.
#[test]
fn group_props_real_resize_still_invalidates_raw_rendering() {
    let mut core = load();
    assert!(group_raw_rendering_len(&core) > 0, "표본 전제");

    let props = core
        .get_shape_properties_native(SEC, PARA, CTRL)
        .expect("현재 속성 조회");
    let cur: serde_json::Value = serde_json::from_str(&props).expect("getter JSON");
    let w = cur["width"].as_u64().expect("width") as u32;
    let h = cur["height"].as_u64().expect("height") as u32;

    core.set_shape_properties_native(
        SEC,
        PARA,
        CTRL,
        &format!("{{\"width\":{},\"height\":{}}}", w + 1000, h + 1000),
    )
    .expect("크기 변경 적용");

    assert_eq!(
        group_raw_rendering_len(&core),
        0,
        "실제 크기 변경인데 옛 변환 행렬이 남았다 — 지문 판정이 과하게 보존한다"
    );
}
