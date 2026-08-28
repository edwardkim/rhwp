//! #6186 — 꼬리말 `hp:subList/@vertAlign` 이 HWPX 저장 왕복에서 보존되는지.
//!
//! 직렬화기가 `vertAlign="TOP"` 을 박아 내보내던 탓에, 원본이 아래 정렬인 꼬리말이
//! 저장본에서 위 정렬로 바뀌었다(`list_attr 0x00400000 → 0x00000000`). 꼬리말 세로 정렬을
//! 실제로 배치에 쓰기 시작하면서 이 손실이 시각 회귀로 드러났다 — 왕복 렌더가 원본보다
//! 꼬리말을 최대 34.7px 위에 그렸다(`visual_roundtrip_baseline` 9건).

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// LIST_HEADER `list_attr` bit 21~22 (0=위 1=가운데 2=아래).
fn footer_valigns(core: &DocumentCore) -> Vec<u32> {
    core.document()
        .sections
        .iter()
        .flat_map(|section| section.paragraphs.iter())
        .flat_map(|para| para.controls.iter())
        .filter_map(|ctrl| match ctrl {
            Control::Footer(footer) => Some((footer.list_attr >> 21) & 0x03),
            _ => None,
        })
        .collect()
}

#[test]
fn issue_6186_footer_vert_align_survives_hwpx_roundtrip() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("samples/hwpx/exam_social.hwpx");
    let bytes = std::fs::read(&path).expect("read exam_social.hwpx");

    let original = DocumentCore::from_bytes(&bytes).expect("parse original");
    let before = footer_valigns(&original);
    assert!(
        before.contains(&2),
        "픽스처 전제: 아래 정렬(2) 꼬리말이 있어야 한다 — 실제 {before:?}"
    );

    let saved = original.export_hwpx_native().expect("export hwpx");
    let reloaded = DocumentCore::from_bytes(&saved).expect("parse roundtrip");
    assert_eq!(
        footer_valigns(&reloaded),
        before,
        "꼬리말 세로 정렬이 HWPX 왕복에서 사라졌다 — subList/@vertAlign 미방출"
    );
}
