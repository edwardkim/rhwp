#![cfg(not(target_arch = "wasm32"))]

//! [#6807] 수식 속성 봉지에 `width`/`height` 가 없어도, `script`·`fontSize` 가 바뀌지 않았으면
//! 크기를 다시 계산하지 않아야 한다.
//!
//! 표본 `3-09월_교육_통합_2023.hwp` s0p1c0 — 한컴이 저장한 상자 900×2070, rhwp 계산 크기(`intrinsic_size_hwp`)
//! 855×2196. 종전에는 색만 바꾸는 봉지(`{"color":…}`)도 자동 크기 분기를 타서 900×2070 이 855×2196 이 되고,
//! `common` 이 바뀌어 #4495 봉인이 어긋나 원본 CTRL_HEADER 패스스루까지 잃었다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const SAMPLE: &str = "samples/3-09월_교육_통합_2023.hwp";
const AT: (usize, usize, usize) = (0, 1, 0);

fn load() -> DocumentCore {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(p).expect("표본 로드")).expect("파싱")
}

fn equation(core: &DocumentCore) -> &rhwp::model::control::Equation {
    match &core.document().sections[AT.0].paragraphs[AT.1].controls[AT.2] {
        Control::Equation(e) => e,
        _ => panic!("표본 전제 위반: 수식이 아니다"),
    }
}

fn set(core: &mut DocumentCore, bag: &str) {
    core.set_equation_properties_native(AT.0, AT.1, AT.2, None, None, bag)
        .expect("set_equation_properties_native");
}

fn assert_sample_premise(core: &DocumentCore) -> (u32, u32) {
    let e = equation(core);
    let (iw, ih) = rhwp::renderer::equation::intrinsic_size_hwp(&e.script, e.font_size);
    assert!(
        (iw, ih) != (e.common.width, e.common.height),
        "표본 전제: 자동 크기({iw}×{ih}) ≠ 저장 크기({}×{}) 여야 판정이 선다",
        e.common.width,
        e.common.height
    );
    assert!(
        e.raw_ctrl_seal.is_some(),
        "표본 전제: 파서를 거친 raw 봉인이 있어야 한다"
    );
    (e.common.width, e.common.height)
}

#[test]
fn color_only_bag_keeps_stored_size_and_raw_passthrough() {
    let mut core = load();
    let (w0, h0) = assert_sample_premise(&core);
    let raw0 = equation(&core).raw_ctrl_data.clone();
    let color = equation(&core).color;

    set(&mut core, &format!("{{\"color\":{}}}", color ^ 0x00ff_ffff));

    let e = equation(&core);
    assert_eq!(
        (e.common.width, e.common.height),
        (w0, h0),
        "색만 바꿨는데 상자 크기가 재계산됐다"
    );
    assert_eq!(e.raw_ctrl_data, raw0, "raw CTRL_HEADER 가 지워졌다");
    // 봉인은 `common` 다이제스트라, 크기가 그대로면 저장기가 원본 바이트를 그대로 쓴다.
    assert_eq!(
        e.raw_ctrl_seal,
        Some(rhwp::model::raw_provenance::record_digest(&e.common)),
        "봉인이 어긋나 저장 시 IR 합성으로 내려간다"
    );
}

#[test]
fn empty_bag_is_identity_for_size() {
    let mut core = load();
    let (w0, h0) = assert_sample_premise(&core);
    set(&mut core, "{}");
    let e = equation(&core);
    assert_eq!((e.common.width, e.common.height), (w0, h0));
}

#[test]
fn baseline_and_font_name_do_not_resize() {
    let mut core = load();
    let (w0, h0) = assert_sample_premise(&core);
    set(&mut core, r#"{"baseline":7,"fontName":"HancomEQN"}"#);
    let e = equation(&core);
    assert_eq!((e.common.width, e.common.height), (w0, h0));
    assert_eq!(e.baseline, 7);
}

#[test]
fn script_change_still_recomputes_size() {
    // 크기를 결정하는 값이 실제로 바뀌면 자동 크기는 종전대로 동작해야 한다.
    let mut core = load();
    let (w0, h0) = assert_sample_premise(&core);
    let new_script = format!("{} + 1 over 2", equation(&core).script);
    set(
        &mut core,
        &format!("{{\"script\":\"{}\"}}", new_script.replace('"', "\\\"")),
    );
    let e = equation(&core);
    let (iw, ih) = rhwp::renderer::equation::intrinsic_size_hwp(&e.script, e.font_size);
    assert_eq!(
        (e.common.width, e.common.height),
        (iw, ih),
        "스크립트가 바뀌면 자동 크기로 재계산돼야 한다"
    );
    assert_ne!(
        (iw, ih),
        (w0, h0),
        "표본 전제: 스크립트 변경이 크기를 바꾼다"
    );
}

#[test]
fn font_size_change_still_recomputes_size() {
    let mut core = load();
    assert_sample_premise(&core);
    let fs = equation(&core).font_size;
    set(&mut core, &format!("{{\"fontSize\":{}}}", fs * 2));
    let e = equation(&core);
    let (iw, ih) = rhwp::renderer::equation::intrinsic_size_hwp(&e.script, e.font_size);
    assert_eq!((e.common.width, e.common.height), (iw, ih));
}

#[test]
fn same_script_and_font_size_do_not_recompute() {
    // 게터가 낸 값과 같은 script/fontSize 를 되먹여도 재계산하지 않는다 — 부분 봉지 항등.
    let mut core = load();
    let (w0, h0) = assert_sample_premise(&core);
    let (script, fs) = (equation(&core).script.clone(), equation(&core).font_size);
    set(
        &mut core,
        &format!(
            "{{\"script\":\"{}\",\"fontSize\":{fs}}}",
            script.replace('"', "\\\"")
        ),
    );
    let e = equation(&core);
    assert_eq!((e.common.width, e.common.height), (w0, h0));
}

#[test]
fn explicit_size_is_still_respected() {
    // #6350 의 계약 — 명시 크기는 재계산에 덮이지 않는다.
    let mut core = load();
    assert_sample_premise(&core);
    let fs = equation(&core).font_size;
    set(
        &mut core,
        &format!("{{\"fontSize\":{},\"width\":5000,\"height\":4000}}", fs * 2),
    );
    let e = equation(&core);
    assert_eq!((e.common.width, e.common.height), (5000, 4000));
}
