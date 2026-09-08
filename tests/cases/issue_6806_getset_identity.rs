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

fn picture(core: &DocumentCore, si: usize, pi: usize, ci: usize) -> &rhwp::model::image::Picture {
    match &core.document().sections[si].paragraphs[pi].controls[ci] {
        Control::Picture(p) => p,
        _ => panic!("표본 전제 위반: s{si}p{pi}c{ci} 가 그림이 아니다"),
    }
}

fn shape(core: &DocumentCore, si: usize, pi: usize, ci: usize) -> &rhwp::model::shape::ShapeObject {
    match &core.document().sections[si].paragraphs[pi].controls[ci] {
        Control::Shape(s) => s.as_ref(),
        _ => panic!("표본 전제 위반: s{si}p{pi}c{ci} 가 도형이 아니다"),
    }
}

// ───────────────────────── 그림 ─────────────────────────

const PIC: &str = "samples/3-09월_교육_통합_2023.hwp";
const PIC_AT: (usize, usize, usize) = (0, 236, 0);

#[test]
fn picture_getset_keeps_raw_rendering_and_current_size() {
    let (si, pi, ci) = PIC_AT;
    let mut core = load(PIC);
    let before = picture(&core, si, pi, ci);
    assert_ne!(
        before.shape_attr.current_height, before.common.height,
        "표본 전제: current ≠ common 이어야 판정이 의미를 갖는다"
    );
    let (raw0, cur_h0, common_h0) = (
        before.shape_attr.raw_rendering.clone(),
        before.shape_attr.current_height,
        before.common.height,
    );
    assert!(
        raw0.len() >= 146,
        "표본 전제: 한컴 렌더링 행렬이 있어야 한다"
    );

    let bag = core.get_picture_properties_native(si, pi, ci).expect("get");
    core.set_picture_properties_native(si, pi, ci, &bag)
        .expect("set 같은 봉지");

    let after = picture(&core, si, pi, ci);
    assert_eq!(
        after.shape_attr.raw_rendering, raw0,
        "같은 봉지 되먹임이 원본 행렬을 지웠다"
    );
    assert_eq!(
        after.shape_attr.current_height, cur_h0,
        "current_height 가 common 으로 덮였다"
    );
    assert_eq!(after.common.height, common_h0);
}

#[test]
fn picture_same_rotation_angle_does_not_refresh_layout() {
    let (si, pi, ci) = PIC_AT;
    let mut core = load(PIC);
    let raw0 = picture(&core, si, pi, ci).shape_attr.raw_rendering.clone();
    let vo0 = picture(&core, si, pi, ci).common.vertical_offset;
    let angle = picture(&core, si, pi, ci).shape_attr.rotation_angle;

    core.set_picture_properties_native(si, pi, ci, &format!("{{\"rotationAngle\":{angle}}}"))
        .expect("같은 각도");

    let after = picture(&core, si, pi, ci);
    assert_eq!(
        after.shape_attr.raw_rendering, raw0,
        "같은 각도인데 회전 재배치가 돌았다"
    );
    assert_eq!(
        after.common.vertical_offset, vo0,
        "재배치가 오프셋을 재중심화했다"
    );
}

#[test]
fn picture_undo_size_bag_is_identity() {
    // ResizeObjectCommand.before = { width, height } — 값은 게터의 common.* 다.
    let (si, pi, ci) = PIC_AT;
    let mut core = load(PIC);
    let p0 = picture(&core, si, pi, ci);
    let (w, h, cur_h0, raw0) = (
        p0.common.width,
        p0.common.height,
        p0.shape_attr.current_height,
        p0.shape_attr.raw_rendering.clone(),
    );

    core.set_picture_properties_native(si, pi, ci, &format!("{{\"width\":{w},\"height\":{h}}}"))
        .expect("undo 봉지");

    let after = picture(&core, si, pi, ci);
    assert_eq!(after.shape_attr.current_height, cur_h0);
    assert_eq!(after.shape_attr.raw_rendering, raw0);
}

#[test]
fn picture_changed_size_still_applies() {
    // 되먹임을 막았다고 실제 변경까지 막히면 안 된다.
    let (si, pi, ci) = PIC_AT;
    let mut core = load(PIC);
    let h = picture(&core, si, pi, ci).common.height;
    core.set_picture_properties_native(si, pi, ci, &format!("{{\"height\":{}}}", h + 400))
        .expect("실제 변경");
    let after = picture(&core, si, pi, ci);
    assert_eq!(after.common.height, h + 400);
    assert_eq!(
        after.shape_attr.current_height,
        h + 400,
        "실제 변경은 current 도 따라가야 한다"
    );
    assert!(
        after.shape_attr.raw_rendering.is_empty(),
        "변환이 실제로 바뀌면 원본 행렬은 무효화된다"
    );
}

// ───────────────────────── 가로선 ─────────────────────────

const LINE: &str = "samples/21_언어_기출_편집가능본.hwp";
const LINE_AT: (usize, usize, usize) = (0, 4, 0);

#[test]
fn thin_line_getset_keeps_height_and_original_size() {
    let (si, pi, ci) = LINE_AT;
    let mut core = load(LINE);
    let s0 = shape(&core, si, pi, ci);
    let (h0, ow0, oh0) = (
        s0.common().height,
        s0.shape_attr().original_width,
        s0.shape_attr().original_height,
    );
    assert!(h0 < 200, "표본 전제: 200 미만 높이의 가로선");
    assert_ne!(
        ow0,
        s0.common().width,
        "표본 전제: original ≠ common 이어야 스케일 판정이 선다"
    );

    let bag = core.get_shape_properties_native(si, pi, ci).expect("get");
    core.set_shape_properties_native(si, pi, ci, &bag)
        .expect("set 같은 봉지");

    let s1 = shape(&core, si, pi, ci);
    assert_eq!(s1.common().height, h0, "200 미만 높이가 클램프에 부풀었다");
    assert_eq!(
        s1.shape_attr().original_width,
        ow0,
        "original_width 가 common 으로 덮였다 — 렌더 스케일 파괴"
    );
    assert_eq!(s1.shape_attr().original_height, oh0);
}

#[test]
fn thin_line_getset_renders_identically() {
    // 렌더 스케일 분모(original_*)가 보존되면 SVG 도 같아야 한다 — 대조군은 set({}).
    let (si, pi, ci) = LINE_AT;
    let mut control = load(LINE);
    control
        .set_shape_properties_native(si, pi, ci, "{}")
        .expect("대조군");
    let mut core = load(LINE);
    let bag = core.get_shape_properties_native(si, pi, ci).expect("get");
    core.set_shape_properties_native(si, pi, ci, &bag)
        .expect("set");

    let (a, b) = (
        control.render_page_svg_native(0).expect("svg"),
        core.render_page_svg_native(0).expect("svg"),
    );
    assert_eq!(a, b, "같은 봉지 되먹임이 0쪽 렌더링을 바꿨다");
}

#[test]
fn zero_size_still_clamps_to_min() {
    // 클램프의 원 목적(핸들을 반대편으로 넘겨 0 이 오는 경우)은 유지한다.
    let (si, pi, ci) = LINE_AT;
    let mut core = load(LINE);
    core.set_shape_properties_native(si, pi, ci, r#"{"width":0,"height":0}"#)
        .expect("0 크기");
    let s = shape(&core, si, pi, ci);
    assert!(
        s.common().width >= 200 && s.common().height >= 200,
        "퇴화값 0 은 최소 크기로 올라야 한다"
    );
}

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
