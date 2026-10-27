//! 양식 개체만 든 문단은 rhwp가 합성한 줄을 HWPX에 저장하지 않는다. 다시 열면 줄 정보가
//! 없어도 개체를 그리고, 개체 높이만큼 줄을 차지해야 한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormObject, FormType};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

const KINDS: [(FormType, &str, u32); 5] = [
    (FormType::PushButton, "명령 단추", 1984),
    (FormType::CheckBox, "선택 상자", 1984),
    (FormType::RadioButton, "라디오 단추", 1984),
    (FormType::ComboBox, "봄", 1450),
    (FormType::Edit, "입력", 1984),
];

fn forms(node: &RenderNode, out: &mut Vec<(FormType, BoundingBox)>) {
    if let RenderNodeType::FormObject(form) = &node.node_type {
        out.push((form.form_type, node.bbox.clone()));
    }
    for child in &node.children {
        forms(child, out);
    }
}

/// "가" / 양식 개체 다섯 문단 / "나" 문서. 수식으로 넣은 글자처럼 취급 자리를 양식 개체로 바꾼다.
fn document() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나").unwrap();
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    for (i, (form_type, label, height)) in KINDS.into_iter().enumerate() {
        let para = i + 1;
        core.split_paragraph_native(0, para, 0, None).unwrap();
        core.insert_equation_native(0, para, 0, "x", 1000, 0)
            .unwrap();
        let mut form = FormObject {
            form_type,
            width: 7087,
            height,
            enabled: true,
            ..Default::default()
        };
        match form_type {
            FormType::ComboBox | FormType::Edit => form.text = label.into(),
            _ => form.caption = label.into(),
        }
        form.common.treat_as_char = true;
        form.common.width = 7087;
        form.common.height = height;
        let slot = &mut core.document_mut().sections[0].paragraphs[para].controls[0];
        assert!(matches!(slot, Control::Equation(_)));
        *slot = Control::Form(Box::new(form));
    }
    core
}

#[test]
fn form_only_paragraph_draws_after_hwpx_reopen_without_lineseg() {
    let core = DocumentCore::from_bytes(&document().export_hwpx_native().unwrap()).unwrap();
    let paras = &core.document().sections[0].paragraphs;
    assert!(
        (1..=KINDS.len()).all(|i| paras[i].line_segs.is_empty()),
        "합성 줄은 HWPX에 저장하지 않는다"
    );

    let mut drawn = Vec::new();
    forms(&core.build_page_render_tree(0).unwrap().root, &mut drawn);
    assert_eq!(
        drawn.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
        KINDS.map(|k| k.0),
        "다시 연 HWPX에서 양식 개체를 그린다"
    );
    // 한컴 재조판처럼 개체 높이를 줄 높이로 쓰고 문단 줄간격(160% → 600HU)을 더해 쌓는다.
    let px = |hu: u32| hwpunit_to_px(hu as i32, DEFAULT_DPI);
    for (i, pair) in drawn.windows(2).enumerate() {
        let (prev, next) = (&pair[0].1, &pair[1].1);
        assert!(
            (prev.height - px(KINDS[i].2)).abs() < 0.01,
            "{i}번 개체 높이"
        );
        assert!(
            (next.y - prev.y - px(KINDS[i].2 + 600)).abs() < 0.01,
            "{i}번 개체 줄이 개체 높이만큼 자리를 차지한다: {prev:?} -> {next:?}"
        );
    }
}
