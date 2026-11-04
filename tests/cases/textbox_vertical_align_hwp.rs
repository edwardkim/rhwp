//! 관계형 저장/API 회귀: 편집한 글상자의 세로 정렬과 비정렬 raw 비트를 HWP/HWPX 왕복에서 보존한다.
//! 주석 갱신: 2026-11-02T09:51:29+09:00; 직전 파일 커밋: 2026-11-02T09:44:44+09:00
//! (67501ea78bd34c7f6b1f0dd9c4bd99bdaae9282c).
//! 입력은 아래 helper가 API로 생성한 빈 글상자다. enum과 비트 값은 저장 계약이며 화면 px 핀이 아니다.
//! 이 세 검사는 실제 글자 배치를 측정하지 않으므로 쪽별 raw/DPI는 비해당이다. 별도의 글자 있는
//! Top/Center/Bottom 저장본·독립 한컴 Print·Native/fresh WASM 전쪽 TSV와 확대 판독은
//! mydocs/pr/assets/pr8309-latestbase-20261102/README.md에 구분해 보존한다.
//!
//! 글상자 세로 정렬은 HWP 저장본과 HWPX 저장본에서 같다.
//!
//! HWP LIST_HEADER 는 세로 정렬을 list_attr bit 5~6 에 싣는다. 모델의 정렬 값은
//! `TextBox::vertical_align` 이므로 HWP 저장기는 그 비트를 `vertical_align` 에서 만든다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::TextBox;
use rhwp::model::table::VerticalAlign;

/// 빈 문서 첫 문단에 글상자 하나를 넣고 (para, control) 을 돌려준다.
fn with_textbox() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let out = core
        .create_shape_control_native(
            0,
            0,
            0,
            8000,
            4000,
            0,
            0,
            false,
            "InFrontOfText",
            "textbox",
            false,
            false,
            &[],
        )
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let para = v["paraIdx"].as_u64().unwrap() as usize;
    let control = v["controlIdx"].as_u64().unwrap() as usize;
    (core, para, control)
}

fn textbox(core: &DocumentCore, para: usize, control: usize) -> &TextBox {
    let Control::Shape(shape) = &core.document().sections[0].paragraphs[para].controls[control]
    else {
        panic!("도형이 아님")
    };
    shape.drawing().unwrap().text_box.as_ref().expect("글상자")
}

fn textbox_mut(core: &mut DocumentCore, para: usize, control: usize) -> &mut TextBox {
    let Control::Shape(shape) =
        &mut core.document_mut().sections[0].paragraphs[para].controls[control]
    else {
        panic!("도형이 아님")
    };
    shape
        .drawing_mut()
        .unwrap()
        .text_box
        .as_mut()
        .expect("글상자")
}

/// HWP·HWPX 로 저장해 다시 읽은 글상자의 (형식, 세로 정렬, list_attr).
fn saved(
    core: &DocumentCore,
    para: usize,
    control: usize,
) -> Vec<(&'static str, VerticalAlign, u32)> {
    [
        ("hwp", core.export_hwp_native().unwrap()),
        ("hwpx", core.export_hwpx_native().unwrap()),
    ]
    .into_iter()
    .map(|(ext, bytes)| {
        let reread = DocumentCore::from_bytes(&bytes).unwrap();
        let tb = textbox(&reread, para, control);
        (ext, tb.vertical_align, tb.list_attr)
    })
    .collect()
}

#[test]
fn new_textbox_is_top_aligned_in_hwp_and_hwpx() {
    let (core, para, control) = with_textbox();
    let tb = textbox(&core, para, control);
    assert_eq!(
        (tb.vertical_align, tb.list_attr & (0b11 << 5)),
        (VerticalAlign::Top, 0)
    );
    for (ext, align, _) in saved(&core, para, control) {
        assert_eq!(align, VerticalAlign::Top, "{ext}");
    }
}

#[test]
fn tb_vertical_align_change_reaches_hwp() {
    let (mut core, para, control) = with_textbox();
    for (name, want) in [
        ("Center", VerticalAlign::Center),
        ("Bottom", VerticalAlign::Bottom),
        ("Top", VerticalAlign::Top),
    ] {
        core.set_shape_properties_native(
            0,
            para,
            control,
            &format!(r#"{{"tbVerticalAlign":"{name}"}}"#),
        )
        .unwrap();
        for (ext, align, _) in saved(&core, para, control) {
            assert_eq!(align, want, "{ext} {name}");
        }
    }
}

/// 세로 정렬 비트만 다시 만든다 — 같은 list_attr 의 글자 방향(bit 0~2)은 그대로 남는다.
#[test]
fn hwp_keeps_other_list_attr_bits() {
    let (mut core, para, control) = with_textbox();
    let tb = textbox_mut(&mut core, para, control);
    tb.list_attr = 1;
    tb.vertical_align = VerticalAlign::Bottom;
    let (ext, align, list_attr) = saved(&core, para, control).remove(0);
    assert_eq!(ext, "hwp");
    assert_eq!((align, list_attr), (VerticalAlign::Bottom, 1 | (2 << 5)));
}
