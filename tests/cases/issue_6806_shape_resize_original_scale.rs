#![cfg(not(target_arch = "wasm32"))]

//! [#6806 잔여] 도형 리사이즈가 `original_*`(생성 시 크기 = 렌더 스케일 분모)를 덮는다.
//!
//! 표본 `21_언어_기출_편집가능본.hwp` s0p4c0 — 한컴 저장 가로선 `common 31804×4`,
//! `original 100×100`. 폭을 절반으로 줄이면 424.05px 선이 1.33px(=100 HWPUNIT)로 붕괴했다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const LINE: &str = "samples/21_언어_기출_편집가능본.hwp";
const LINE_AT: (usize, usize, usize) = (0, 4, 0);

fn load() -> DocumentCore {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(LINE);
    DocumentCore::from_bytes(&std::fs::read(p).expect("표본 로드")).expect("파싱")
}

/// `(common.w, common.h, original_w, original_h, current_w, current_h)`.
fn geometry(core: &DocumentCore) -> (u32, u32, u32, u32, u32, u32) {
    let (si, pi, ci) = LINE_AT;
    let s = match &core.document().sections[si].paragraphs[pi].controls[ci] {
        Control::Shape(s) => s.as_ref(),
        _ => panic!("표본 전제 위반: s{si}p{pi}c{ci} 가 도형이 아니다"),
    };
    (
        s.common().width,
        s.common().height,
        s.shape_attr().original_width,
        s.shape_attr().original_height,
        s.shape_attr().current_width,
        s.shape_attr().current_height,
    )
}

fn resize_to(core: &mut DocumentCore, w: u32, h: u32) {
    let (si, pi, ci) = LINE_AT;
    core.set_shape_properties_native(si, pi, ci, &format!("{{\"width\":{w},\"height\":{h}}}"))
        .expect("set_shape_properties_native");
}

/// 0쪽 SVG 에서 그 가로선의 길이(px). 같은 y 대역의 가장 긴 `<line>` 을 고른다.
fn line_length(core: &DocumentCore) -> f64 {
    let svg = core.render_page_svg_native(0).expect("0쪽 svg");
    let attr = |head: &str, name: &str| -> Option<f64> {
        let key = format!("{name}=\"");
        let s = head.find(&key)? + key.len();
        let e = s + head[s..].find('"')?;
        head[s..e].parse().ok()
    };
    svg.split("<line ")
        .skip(1)
        .filter_map(|chunk| {
            let head = &chunk[..chunk.find('>')?];
            let (x1, x2, y1) = (attr(head, "x1")?, attr(head, "x2")?, attr(head, "y1")?);
            // 이 도형이 놓인 줄(y≈533)만 본다 — 같은 쪽의 다른 괘선과 섞이지 않게.
            ((532.0..535.0).contains(&y1)).then_some((x2 - x1).abs())
        })
        .fold(0.0f64, f64::max)
}

fn assert_premise(core: &DocumentCore) -> (u32, u32) {
    let (w, h, ow, oh, ..) = geometry(core);
    assert!(h < 200, "표본 전제: 200 미만 높이의 가로선 (h={h})");
    assert_eq!(
        (ow, oh),
        (100, 100),
        "표본 전제: 한컴이 저장한 생성 시 크기가 100×100 이어야 스케일 판정이 선다"
    );
    assert_ne!(ow, w, "표본 전제: original({ow}) ≠ common({w})");
    (w, h)
}

#[test]
fn real_resize_scales_the_drawn_line_proportionally() {
    let mut core = load();
    let (w0, h0) = assert_premise(&core);
    let before = line_length(&core);
    assert!(
        (before - 424.05).abs() < 1.0,
        "표본 전제: 원래 선 길이 424.05px (실측 {before:.2})"
    );

    resize_to(&mut core, w0 / 2, h0);

    let after = line_length(&core);
    assert!(
        (after - before / 2.0).abs() < 1.0,
        "폭을 절반으로 줄였는데 그려진 선이 {after:.2}px 다 — 절반인 {:.2}px 이어야 한다. \
         `original_*` 가 덮이면 스케일이 1 이 되어 100 HWPUNIT(=1.33px)로 붕괴한다",
        before / 2.0
    );
    assert_eq!(
        geometry(&core),
        (w0 / 2, h0, 100, 100, w0 / 2, h0),
        "`current_*` 만 새 크기를 따라가고 `original_*` 는 생성 시 값으로 남아야 한다"
    );
}

#[test]
fn real_resize_then_undo_restores_geometry_and_rendering() {
    // studio 의 `ResizeObjectCommand.undo` 는 게터가 냈던 원래 크기를 다시 넣는다
    // (`command.ts` `setProps` → `setShapeProperties`).
    let mut core = load();
    let (w0, h0) = assert_premise(&core);
    let before_geometry = geometry(&core);
    let before_svg = core.render_page_svg_native(0).expect("0쪽 svg");

    resize_to(&mut core, w0 / 2, h0);
    resize_to(&mut core, w0, h0);

    assert_eq!(
        geometry(&core),
        before_geometry,
        "리사이즈 되돌리기가 개체 기하를 원래대로 돌리지 못했다"
    );
    assert_eq!(
        core.render_page_svg_native(0).expect("0쪽 svg"),
        before_svg,
        "리사이즈 되돌리기 뒤 0쪽 렌더링이 원래와 다르다"
    );
}

#[test]
fn resize_still_applies() {
    // 대조 — 보존을 넣었다고 실제 리사이즈가 막히면 안 된다.
    let mut core = load();
    let (w0, h0) = assert_premise(&core);

    resize_to(&mut core, w0 + 4000, h0 + 40);

    let (w, h, ..) = geometry(&core);
    assert_eq!((w, h), (w0 + 4000, h0 + 40), "실제 변경은 그대로 적용된다");
}
