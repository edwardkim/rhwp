//! 문단 첫 글자 장식 방식을 읽고 바꾸며, HWP·HWPX 저장 뒤에도 남긴다.
//!
//! 한/글 12.30(Mac)의 서식 > 문단 첫 글자 장식…은 문단 첫 글자를 글상자(사각형)로 옮기고
//! 그 개체에 장식 방식을 둔다. 시험 문서는 같은 문단에 2줄·3줄·여백을 차례로 적용해 저장한
//! 한/글 저장본이다. HWPX는 `<hp:rect dropcapstyle="DoubleLine">`과 파라미터 셋 539를 싣고,
//! HWP는 CTRL_HEADER와 SHAPE_COMPONENT 아래 두 CTRL_DATA에 같은 파라미터 셋
//! (539 > 12291 > 28673 = 2줄 1·3줄 2·여백 3)을 싣는다. 인쇄 PDF는 `pdf/drop_cap/`에 있다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use std::io::{Cursor, Read};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parser::{cfb_reader::decompress_stream, record::Record, tags};
use serde_json::Value;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/drop_cap")
            .join(name),
    )
    .expect("read fixture")
}

/// 장식 글상자의 (문단, 개체) 위치.
fn drop_cap_box(core: &DocumentCore) -> (usize, usize) {
    core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(para, p)| {
            p.controls
                .iter()
                .position(|c| matches!(c, Control::Shape(_)))
                .map(|control| (para, control))
        })
        .expect("drop cap text box")
}

fn style(core: &DocumentCore) -> String {
    let (para, control) = drop_cap_box(core);
    let props: Value =
        serde_json::from_str(&core.get_shape_properties_native(0, para, control).unwrap()).unwrap();
    props["dropCapStyle"]
        .as_str()
        .unwrap_or_else(|| panic!("dropCapStyle 없음: {props}"))
        .to_string()
}

fn set_style(core: &mut DocumentCore, style: &str) {
    let (para, control) = drop_cap_box(core);
    core.set_shape_properties_native(
        0,
        para,
        control,
        &format!(r#"{{"dropCapStyle":"{style}"}}"#),
    )
    .expect("set drop cap");
}

fn section_xml(hwpx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(hwpx)).expect("hwpx zip");
    let mut xml = String::new();
    zip.by_name("Contents/section0.xml")
        .expect("section0")
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// HWP 본문 CTRL_DATA 가운데 첫 글자 장식 파라미터 셋의 값과 수준.
fn drop_cap_ctrl_data(hwp: &[u8]) -> Vec<(u32, u16)> {
    let mut cfb = cfb::CompoundFile::open(Cursor::new(hwp)).expect("open CFB");
    let mut raw = Vec::new();
    cfb.open_stream("/BodyText/Section0")
        .expect("section stream")
        .read_to_end(&mut raw)
        .unwrap();
    let section = decompress_stream(&raw).unwrap_or(raw);
    Record::read_all(&section)
        .expect("records")
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_CTRL_DATA && r.data.len() == 24)
        .filter(|r| {
            r.data
                .starts_with(&[0x1b, 0x02, 0x01, 0x00, 0x00, 0x00, 0x03, 0x30])
        })
        .map(|r| {
            (
                u32::from_le_bytes(r.data[20..24].try_into().unwrap()),
                r.level,
            )
        })
        .collect()
}

#[test]
fn hancom_drop_caps_are_read_from_hwp_and_hwpx() {
    for (name, expected) in [
        ("double-line.hwp", "DoubleLine"),
        ("triple-line.hwp", "TripleLine"),
        ("margin.hwp", "Margin"),
        ("double-line.hwpx", "DoubleLine"),
    ] {
        let core = DocumentCore::from_bytes(&fixture(name)).expect("open");
        assert_eq!(style(&core), expected, "{name}");
    }
}

#[test]
fn drop_cap_style_survives_hwp_and_hwpx_save() {
    for name in ["triple-line.hwp", "double-line.hwpx"] {
        let core = DocumentCore::from_bytes(&fixture(name)).expect("open");
        let expected = style(&core);
        let hwpx = core.export_hwpx_native().expect("hwpx");
        assert!(
            section_xml(&hwpx).contains(&format!(r#"dropcapstyle="{expected}""#)),
            "{name}: HWPX에 {expected} 없음"
        );
        let hwp = core.export_hwp_native().expect("hwp");
        for bytes in [hwpx, hwp] {
            assert_eq!(
                style(&DocumentCore::from_bytes(&bytes).expect("reopen")),
                expected
            );
        }
    }
}

#[test]
fn setting_the_style_rewrites_both_hwp_ctrl_data_copies() {
    let mut core = DocumentCore::from_bytes(&fixture("double-line.hwp")).expect("open");
    for (name, value) in [("TripleLine", 2u32), ("Margin", 3), ("DoubleLine", 1)] {
        set_style(&mut core, name);
        assert_eq!(style(&core), name);
        let hwp = core.export_hwp_native().expect("hwp");
        // 한/글처럼 CTRL_HEADER 아래(수준 2)와 SHAPE_COMPONENT 아래(수준 3)에 같은 값을 둔다.
        assert_eq!(drop_cap_ctrl_data(&hwp), [(value, 2), (value, 3)], "{name}");
        assert_eq!(style(&DocumentCore::from_bytes(&hwp).unwrap()), name);
        let hwpx = core.export_hwpx_native().expect("hwpx");
        assert!(section_xml(&hwpx).contains(&format!(r#"dropcapstyle="{name}""#)));
        assert_eq!(style(&DocumentCore::from_bytes(&hwpx).unwrap()), name);
    }

    set_style(&mut core, "None");
    let hwp = core.export_hwp_native().expect("hwp");
    assert!(drop_cap_ctrl_data(&hwp).is_empty());
    assert_eq!(style(&DocumentCore::from_bytes(&hwp).unwrap()), "None");
}
