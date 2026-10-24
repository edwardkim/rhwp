#![cfg(not(target_arch = "wasm32"))]

//! 수식 흐름 프레임·paint 조각 캐시와 미주 단 측정 재사용이 편집 뒤 낡은 결과를
//! 내지 않는다. 먼저 그린 문서를 편집한 출력은, 같은 편집만 적용한 새 문서를
//! 캐시가 빈 다른 스레드에서 처음 그린 출력과 같아야 한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Equation};

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn all_pages_svg(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| core.render_page_svg_native(page).expect("render page"))
        .collect()
}

/// 수식 캐시는 스레드마다 따로 두므로 새 스레드는 모든 수식을 처음부터 계산한다.
fn fresh_render(
    bytes: &[u8],
    edit: impl FnOnce(&mut DocumentCore) + Send + 'static,
) -> Vec<String> {
    let bytes = bytes.to_vec();
    std::thread::spawn(move || {
        let mut core = DocumentCore::from_bytes(&bytes).expect("parse fixture again");
        edit(&mut core);
        all_pages_svg(&core)
    })
    .join()
    .expect("fresh render thread")
}

fn first_body_equation(core: &DocumentCore) -> (usize, usize, usize) {
    for (section, value) in core.document().sections.iter().enumerate() {
        for (para, paragraph) in value.paragraphs.iter().enumerate() {
            for (control, item) in paragraph.controls.iter().enumerate() {
                if matches!(item, Control::Equation(_)) {
                    return (section, para, control);
                }
            }
        }
    }
    panic!("fixture has no body equation");
}

/// (구역, 본문 문단, 미주 컨트롤, 미주 문단, 미주 문단 안 수식 컨트롤)
fn first_endnote_equation(core: &DocumentCore) -> (usize, usize, usize, usize, usize) {
    for (section, value) in core.document().sections.iter().enumerate() {
        for (para, paragraph) in value.paragraphs.iter().enumerate() {
            for (note_control, item) in paragraph.controls.iter().enumerate() {
                let Control::Endnote(note) = item else {
                    continue;
                };
                for (note_para, inner) in note.paragraphs.iter().enumerate() {
                    for (inner_control, control) in inner.controls.iter().enumerate() {
                        if matches!(control, Control::Equation(_)) {
                            return (section, para, note_control, note_para, inner_control);
                        }
                    }
                }
            }
        }
    }
    panic!("fixture has no endnote equation");
}

fn body_equation(
    core: &DocumentCore,
    (section, para, control): (usize, usize, usize),
) -> &Equation {
    match &core.document().sections[section].paragraphs[para].controls[control] {
        Control::Equation(equation) => equation,
        _ => unreachable!(),
    }
}

fn endnote_equation(
    core: &DocumentCore,
    (section, para, note_control, note_para, inner_control): (usize, usize, usize, usize, usize),
) -> &Equation {
    let Control::Endnote(note) =
        &core.document().sections[section].paragraphs[para].controls[note_control]
    else {
        unreachable!()
    };
    match &note.paragraphs[note_para].controls[inner_control] {
        Control::Equation(equation) => equation,
        _ => unreachable!(),
    }
}

/// 한 값만 바꾸고 저장 상자 크기는 원래 값으로 고정한다. 크기가 함께 바뀌면
/// 캐시 키의 다른 값이 달라져 바꾼 값의 누락을 드러내지 못한다.
fn single_value_edits(equation: &Equation) -> Vec<String> {
    let size = format!(
        r#""width":{},"height":{}"#,
        equation.common.width, equation.common.height
    );
    let script = if equation.script.contains('x') {
        equation.script.replace('x', "y")
    } else {
        format!("{} + 1", equation.script)
    };
    vec![
        format!(r#"{{"script":{script:?},{size}}}"#),
        format!(r#"{{"fontSize":{},{size}}}"#, equation.font_size + 400),
        format!(r#"{{"color":{}}}"#, equation.color ^ 0x0000_00ff),
        format!(r#"{{"baseline":{}}}"#, (equation.baseline + 20) % 100),
    ]
}

#[test]
fn body_equation_edit_after_render_matches_fresh_render() {
    let bytes = read_fixture("samples/equation-lim.hwp");
    let probe = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    let coord = first_body_equation(&probe);
    for props in single_value_edits(body_equation(&probe, coord)) {
        let (section, para, control) = coord;
        let mut rendered = DocumentCore::from_bytes(&bytes).expect("parse fixture");
        let before = all_pages_svg(&rendered);
        rendered
            .set_equation_properties_native(section, para, control, None, None, &props)
            .expect("edit rendered equation");
        let edited = all_pages_svg(&rendered);

        let fresh_props = props.clone();
        let fresh = fresh_render(&bytes, move |core| {
            core.set_equation_properties_native(section, para, control, None, None, &fresh_props)
                .expect("edit fresh equation");
        });

        assert_ne!(edited, before, "{props}: 편집이 출력에 반영되지 않았다");
        assert_eq!(
            edited, fresh,
            "{props}: 먼저 그린 문서의 편집 출력이 새 문서와 다르다"
        );
    }
}

#[test]
fn endnote_equation_edit_after_render_matches_fresh_render() {
    let bytes = read_fixture("samples/3-11월_실전_통합_2022.hwp");
    let mut rendered = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    let before = all_pages_svg(&rendered);
    let coord = first_endnote_equation(&rendered);
    let props = single_value_edits(endnote_equation(&rendered, coord)).swap_remove(0);
    let (section, para, note_control, note_para, inner_control) = coord;
    rendered
        .set_note_equation_properties_native(
            "endnote",
            section,
            para,
            note_control,
            note_para,
            inner_control,
            &props,
        )
        .expect("edit rendered endnote equation");
    let edited = all_pages_svg(&rendered);

    let fresh_props = props.clone();
    let fresh = fresh_render(&bytes, move |core| {
        core.set_note_equation_properties_native(
            "endnote",
            section,
            para,
            note_control,
            note_para,
            inner_control,
            &fresh_props,
        )
        .expect("edit fresh endnote equation");
    });

    assert_ne!(
        edited, before,
        "{props}: 미주 수식 편집이 출력에 반영되지 않았다"
    );
    assert_eq!(
        edited, fresh,
        "{props}: 먼저 그린 문서의 미주 편집 출력이 새 문서와 다르다"
    );
}
