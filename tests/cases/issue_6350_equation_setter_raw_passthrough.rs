#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Equation};

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn first_equation_coord(core: &DocumentCore) -> (usize, usize, usize) {
    for (section_idx, section) in core.document().sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            for (control_idx, control) in para.controls.iter().enumerate() {
                if matches!(control, Control::Equation(_)) {
                    return (section_idx, para_idx, control_idx);
                }
            }
        }
    }
    panic!("fixture has no body equation control");
}

fn equation(core: &DocumentCore, coord: (usize, usize, usize)) -> &Equation {
    let (section_idx, para_idx, control_idx) = coord;
    match &core.document().sections[section_idx].paragraphs[para_idx].controls[control_idx] {
        Control::Equation(equation) => equation,
        _ => panic!("fixture coordinate is not an equation"),
    }
}

fn equation_mut(core: &mut DocumentCore, coord: (usize, usize, usize)) -> &mut Equation {
    let (section_idx, para_idx, control_idx) = coord;
    match &mut core.document_mut().sections[section_idx].paragraphs[para_idx].controls[control_idx]
    {
        Control::Equation(equation) => equation,
        _ => panic!("fixture coordinate is not an equation"),
    }
}

#[test]
fn issue_6350_getter_setter_roundtrip_preserves_hancom_samples() {
    for path in [
        "samples/equation-lim.hwp",
        "samples/atop-equation-01.hwp",
        "samples/issue-505-equations.hwp",
    ] {
        let bytes = read_fixture(path);
        let mut base = DocumentCore::from_bytes(&bytes).expect("parse fixture");
        let coord = first_equation_coord(&base);
        let original = equation(&base, coord);
        assert!(
            !original.raw_ctrl_data.is_empty() && original.raw_ctrl_seal.is_some(),
            "{path}: parsed equation should have sealed raw CTRL_HEADER bytes"
        );

        let (section_idx, para_idx, control_idx) = coord;
        let bag = base
            .get_equation_properties_native(section_idx, para_idx, control_idx, None, None)
            .expect("read equation properties");
        base.document_mut().sections[section_idx].raw_stream = None;
        let baseline = base.export_hwp_native().expect("export baseline");

        let mut edited = DocumentCore::from_bytes(&bytes).expect("parse fixture again");
        edited
            .set_equation_properties_native(section_idx, para_idx, control_idx, None, None, &bag)
            .expect("reapply getter bag");
        let roundtripped = edited.export_hwp_native().expect("export edited document");

        assert_eq!(
            baseline, roundtripped,
            "{path}: get-equation-properties followed by set-equation-properties changed HWP bytes"
        );
    }
}

#[test]
fn issue_6350_explicit_size_survives_save_and_reparse() {
    let bytes = read_fixture("samples/equation-lim.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    let coord = first_equation_coord(&core);
    let (section_idx, para_idx, control_idx) = coord;

    core.set_equation_properties_native(
        section_idx,
        para_idx,
        control_idx,
        None,
        None,
        r#"{"width":5000,"height":4000}"#,
    )
    .expect("set explicit equation size");
    let exported = core.export_hwp_native().expect("export resized equation");
    let reparsed = DocumentCore::from_bytes(&exported).expect("reparse resized equation");

    assert_eq!(
        (
            equation(&reparsed, coord).common.width,
            equation(&reparsed, coord).common.height,
        ),
        (5000, 4000),
        "explicit equation size should survive HWP save and reparse"
    );
}

#[test]
fn issue_6350_unsealed_equation_raw_is_cleared_by_public_setter() {
    let bytes = read_fixture("samples/equation-lim.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    let coord = first_equation_coord(&core);
    let (section_idx, para_idx, control_idx) = coord;

    {
        let equation = equation_mut(&mut core, coord);
        assert!(
            !equation.raw_ctrl_data.is_empty(),
            "fixture should carry raw bytes"
        );
        equation.raw_ctrl_seal = None;
    }

    core.set_equation_properties_native(
        section_idx,
        para_idx,
        control_idx,
        None,
        None,
        r#"{"width":5000,"height":4000}"#,
    )
    .expect("set explicit equation size");

    assert!(
        equation(&core, coord).raw_ctrl_data.is_empty(),
        "unsealed raw CTRL_HEADER bytes should be cleared so the edit can be serialized"
    );
}

#[test]
fn issue_6350_sealed_equation_raw_is_kept_by_public_setter() {
    let bytes = read_fixture("samples/equation-lim.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    let coord = first_equation_coord(&core);
    let (section_idx, para_idx, control_idx) = coord;
    let original = equation(&core, coord).raw_ctrl_data.clone();
    assert!(
        equation(&core, coord).raw_ctrl_seal.is_some(),
        "fixture should have a raw seal"
    );

    core.set_equation_properties_native(
        section_idx,
        para_idx,
        control_idx,
        None,
        None,
        r#"{"script":"1 over 3"}"#,
    )
    .expect("set equation script");

    assert_eq!(
        equation(&core, coord).raw_ctrl_data,
        original,
        "sealed raw CTRL_HEADER bytes should stay available for serializer seal checks"
    );
}
