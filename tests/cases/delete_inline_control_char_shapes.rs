//! 본문 인라인 개체를 지우면 그 뒤 글자 모양 경계도 개체의 8유닛만큼 당겨진다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// `bold` 글자 범위를 굵게 한 "앞글뒤글" 문단. `object_first` 면 개체를 넣은 뒤 서식을 준다.
fn styled(object: Option<(&str, usize)>, bold: (usize, usize), object_first: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞글뒤글").unwrap();
    let apply_bold = |core: &mut DocumentCore| {
        core.apply_char_format_native(0, 0, bold.0, bold.1, r#"{"bold":true}"#)
            .unwrap();
    };
    if !object_first {
        apply_bold(&mut core);
    }
    match object {
        Some(("picture", offset)) => {
            core.insert_picture_native(
                0,
                0,
                offset,
                &[],
                include_bytes!("../../assets/logo/logo-16.png"),
                4000,
                3000,
                16,
                16,
                "png",
                "지울 그림",
                None,
                None,
            )
            .unwrap();
        }
        Some((_, offset)) => {
            core.insert_equation_native(0, 0, offset, "x+1", 1000, 0)
                .unwrap();
        }
        None => {}
    }
    if object_first {
        apply_bold(&mut core);
    }
    core
}

fn shapes(core: &DocumentCore) -> Vec<(u32, u32)> {
    core.document().sections[0].paragraphs[0]
        .char_shapes
        .iter()
        .map(|cs| (cs.start_pos, cs.char_shape_id))
        .collect()
}

fn reopen(core: &DocumentCore, format: &str) -> DocumentCore {
    let bytes = if format == "HWP" {
        core.export_hwp_native().unwrap()
    } else {
        core.export_hwpx_native().unwrap()
    };
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn check(kind: &str, offset: usize, bold: (usize, usize), object_first: bool) {
    let reference = styled(None, bold, object_first);
    let expected = shapes(&reference);
    let offsets = reference.document().sections[0].paragraphs[0]
        .char_offsets
        .clone();
    let mut core = styled(Some((kind, offset)), bold, object_first);
    let ci = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Picture(_) | Control::Equation(_)))
        .unwrap();
    if kind == "picture" {
        core.delete_picture_control_native(0, 0, ci).unwrap();
    } else {
        core.delete_equation_control_native(0, 0, ci).unwrap();
    }
    let para = &core.document().sections[0].paragraphs[0];
    assert_eq!(para.text, "앞글뒤글");
    assert_eq!(para.char_offsets, offsets);
    assert_eq!(shapes(&core), expected, "{kind} 삭제 뒤 글자 모양 경계");

    // 저장 형식마다 좌표·번호 규약이 달라 개체 없이 만든 같은 문단의 왕복과 비교한다.
    for format in ["HWP", "HWPX"] {
        let reopened = reopen(&core, format);
        assert_eq!(
            reopened.document().sections[0].paragraphs[0].text,
            "앞글뒤글"
        );
        assert_eq!(
            shapes(&reopened),
            shapes(&reopen(&reference, format)),
            "{kind} {format} 저장·재열기"
        );
    }
}

#[test]
fn deleting_inline_picture_pulls_boundary_after_it() {
    // 굵게 끝 경계가 그림 바로 뒤에 있다.
    check("picture", 2, (0, 2), false);
}

#[test]
fn deleting_leading_picture_keeps_later_bold() {
    // 맨 앞 그림을 지워도 뒤 글자의 굵게가 그 글자에 남는다.
    check("picture", 0, (2, 3), false);
}

#[test]
fn deleting_leading_equation_pulls_text_start_boundary() {
    // 문단 맨 앞 수식 바로 뒤 첫 글자에서 굵게가 시작한다.
    check("equation", 0, (0, 2), true);
}
