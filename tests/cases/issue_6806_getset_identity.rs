#![cfg(not(target_arch = "wasm32"))]

//! [#6806] objectProps get∘set 항등 — 게터가 낸 봉지를 그대로 되먹이면 IR·raw·렌더링이
//! 바뀌지 않아야 한다.
//!
//! 표본은 전부 한컴 저장본이고, 각각 결함의 한 층을 판정력 있게 드러낸다.
//! - 그림 `3-09월_교육_통합_2023.hwp` s0p236c0: 파싱값이 `current_height(7295) ≠ common.height(7296)`.
//!   종전에는 `rotationAngle` 키 존재만으로 refresh 가 돌고 `height` 키로 `current` 가 덮여
//!   지문이 흔들려 한컴 원본 렌더링 행렬(146B)이 지워졌다.
//! - 가로선 `21_언어_기출_편집가능본.hwp` s0p4c0: 높이 4, `original 100×100`. 종전에는 높이가 200 으로
//!   부풀고 `original` 이 `common` 으로 덮여 렌더 스케일이 바뀌어 424px 선이 1.3px 로 무너졌다.
//! - 묶음 `text_footnote_tail_overpagination.hwp` s0p4352c1: 높이 76, `current ≠ common`.
//! - 도형 `143E433F503322BD33.hwp` s0p1c0: 「쪽 영역 제한」과 「겹침 허용」이 동시에 켜진 채 저장됨.
//!
//! 대조군은 같은 setter 에 빈 봉지 `{}` — 부수효과(recompose·paginate)는 같고 대입만 없다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

fn load(rel: &str) -> DocumentCore {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(p).expect("표본 로드")).expect("파싱")
}

fn shape(core: &DocumentCore, si: usize, pi: usize, ci: usize) -> &rhwp::model::shape::ShapeObject {
    match &core.document().sections[si].paragraphs[pi].controls[ci] {
        Control::Shape(s) => s.as_ref(),
        _ => panic!("표본 전제 위반: s{si}p{pi}c{ci} 가 도형이 아니다"),
    }
}

// ───────────────────────── 그림 ─────────────────────────

// ───────────────────────── 가로선 ─────────────────────────

// ───────────────────────── 묶음 ─────────────────────────

#[test]
fn group_getset_keeps_raw_rendering() {
    let rel = "samples/task1725/text_footnote_tail_overpagination.hwp";
    let (si, pi, ci) = (0, 4352, 1);
    let mut core = load(rel);
    let g0 = shape(&core, si, pi, ci);
    assert!(
        matches!(g0, rhwp::model::shape::ShapeObject::Group(_)),
        "표본 전제: 묶음"
    );
    assert_ne!(
        g0.shape_attr().current_height,
        g0.common().height,
        "표본 전제: current ≠ common"
    );
    let (raw0, h0, cur_h0) = (
        g0.shape_attr().raw_rendering.clone(),
        g0.common().height,
        g0.shape_attr().current_height,
    );
    assert!(raw0.len() >= 146);

    let bag = core.get_shape_properties_native(si, pi, ci).expect("get");
    core.set_shape_properties_native(si, pi, ci, &bag)
        .expect("set");

    let g1 = shape(&core, si, pi, ci);
    assert_eq!(
        g1.shape_attr().raw_rendering,
        raw0,
        "묶음 원본 변환 행렬(#6740)이 지워졌다"
    );
    assert_eq!(g1.common().height, h0, "높이 76 이 200 으로 부풀었다");
    assert_eq!(g1.shape_attr().current_height, cur_h0);
}

// ───────────────────────── 겹침 허용 ─────────────────────────

#[test]
fn restrict_in_page_does_not_force_allow_overlap_off() {
    let (si, pi, ci) = (0, 1, 0);
    let mut core = load("samples/143E433F503322BD33.hwp");
    let c0 = shape(&core, si, pi, ci).common();
    assert!(
        c0.flow_with_text && c0.allow_overlap,
        "표본 전제: 한컴이 둘을 동시에 켜서 저장한 개체"
    );
    let attr0 = c0.attr;

    let bag = core.get_shape_properties_native(si, pi, ci).expect("get");
    assert!(bag.contains("\"allowOverlap\":true"));
    core.set_shape_properties_native(si, pi, ci, &bag)
        .expect("set");

    let c1 = shape(&core, si, pi, ci).common();
    assert!(c1.allow_overlap, "같은 봉지 되먹임이 겹침 허용을 껐다");
    assert_eq!(c1.attr, attr0, "attr bit 14 가 지워졌다");
    let bag2 = core.get_shape_properties_native(si, pi, ci).expect("get");
    assert!(bag2.contains("\"allowOverlap\":true"));
}

#[test]
fn allow_overlap_can_still_be_turned_off_explicitly() {
    let (si, pi, ci) = (0, 1, 0);
    let mut core = load("samples/143E433F503322BD33.hwp");
    core.set_shape_properties_native(si, pi, ci, r#"{"allowOverlap":false}"#)
        .expect("명시 해제");
    let c = shape(&core, si, pi, ci).common();
    assert!(!c.allow_overlap);
    assert_eq!(c.attr & (1 << 14), 0);
}
