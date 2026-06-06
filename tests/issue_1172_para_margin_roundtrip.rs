//! Issue #1172: rhwp-studio 문단모양 왼쪽 여백 round-trip 2배 결함.
//!
//! 증상: 문단모양 대화상자에서 왼쪽 여백 10.0pt 설정 후 다시 열면 20.0pt 로 표시.
//!
//! 근본: ParaShape margin/indent 의 IR 값은 2× 스케일이다 (HWP5 바이너리 원본
//! 스케일, HWPX 도 parser val2x 로 통일). 즉 1pt = 200 HWPUNIT.
//! `build_para_properties_json` 이 dialog 표시 px 를 만들 때 이 2× 스케일을 1× 로
//! 환산(÷2)하지 않고 표준 hwpunit_to_px(7200/inch) 를 그대로 적용해 2배로 표시했다.
//!
//! 정답지 (작업지시자 한컴 편집기): para-001 첫 문단 margin_left IR=2000 → 10.0pt.
//!
//! 검증: getParaPropertiesAt 의 marginLeft(px) 가 10pt 에 해당하는 px(13.33 @96dpi)인지.
//! frontend 는 이 px 를 pxToPt(px*72/96) 로 pt 표시하므로 13.33px → 10.0pt.

use std::fs;
use std::path::Path;

fn load(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn para_prop_number(
    doc: &rhwp::wasm_api::HwpDocument,
    sec: usize,
    para: usize,
    key_name: &str,
) -> f64 {
    let json = doc
        .get_para_properties_at_native(sec, para)
        .expect("para props");
    let key = format!("\"{}\":", key_name);
    let start = json.find(&key).unwrap_or_else(|| panic!("{key_name} key")) + key.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| c == ',' || c == '}').expect("delim");
    rest[..end]
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("parse {key_name}"))
}

fn margin_left_px(doc: &rhwp::wasm_api::HwpDocument, sec: usize, para: usize) -> f64 {
    para_prop_number(doc, sec, para, "marginLeft")
}

/// px → pt (frontend pxToPt 와 동일: px * 72 / 96)
fn px_to_pt(px: f64) -> f64 {
    px * 72.0 / 96.0
}

#[test]
fn para_001_first_paragraph_left_margin_is_10pt() {
    // para-001 첫 문단(0,0): IR margin_left = 2000 HWPUNIT (2× 스케일) = 한컴 10.0pt.
    for rel in ["samples/hwpx/para-001.hwpx", "samples/para-001.hwp"] {
        let doc = load(rel);
        let px = margin_left_px(&doc, 0, 0);
        let pt = px_to_pt(px);
        assert!(
            (pt - 10.0).abs() < 0.1,
            "{rel}: 첫 문단 왼쪽 여백이 10.0pt 여야 함 (표시 px={px:.2} → {pt:.2}pt). \
             2× 스케일 IR 의 ÷2 환산 누락 시 20pt 로 나타나는 round-trip 결함."
        );
    }
}

#[test]
fn para_unit_hanging_indent_does_not_bind_to_left_margin() {
    // Hancom 2022 paragraph dialog resets display units to pt after reopen.
    // The sample keeps paragraph left margin at 0 and varies only hanging indent.
    // Regression: build_para_properties_json exposed effective_left = margin_left + indent,
    // so the dialog showed negative left margin instead of 0.
    for rel in ["samples/para-unit-01.hwp", "samples/hwpx/para-unit-01.hwpx"] {
        let doc = load(rel);

        for para in [0usize, 13usize] {
            let margin_left_pt = px_to_pt(para_prop_number(&doc, 0, para, "marginLeft"));
            let indent_pt = px_to_pt(para_prop_number(&doc, 0, para, "indent"));
            assert!(
                margin_left_pt.abs() < 0.1,
                "{rel} para {para}: 왼쪽 여백은 0.0pt 여야 함 (actual {margin_left_pt:.2}pt)"
            );
            assert!(
                indent_pt < -0.1,
                "{rel} para {para}: 내어쓰기 값은 음수로 별도 표시되어야 함 (actual {indent_pt:.2}pt)"
            );
        }
    }
}

fn fill_type(doc: &rhwp::wasm_api::HwpDocument, sec: usize, para: usize) -> String {
    let json = doc.get_para_properties_at_native(sec, para).expect("props");
    let key = "\"fillType\":\"";
    let s = json.find(key).expect("fillType") + key.len();
    let rest = &json[s..];
    let e = rest.find('"').expect("end");
    rest[..e].to_string()
}

#[test]
fn para_001_hwpx_first_paragraph_has_no_fill() {
    // [Issue #1172] HWPX 첫 문단 border_fill_id=2 = winBrush faceColor="none".
    // 파서가 이를 FillType::None 으로 정합해야 한다 (faceColor=none + 무늬 없음 → 빈 채우기).
    // 종전엔 winBrush 존재만으로 Solid 흰배경 으로 잘못 파싱해 문단모양/렌더 배경이 생겼다.
    let fill = fill_type(&load("samples/hwpx/para-001.hwpx"), 0, 0);
    assert_eq!(
        fill, "none",
        "HWPX 첫 문단 배경은 none 이어야 함 (실제 {fill})"
    );
}
