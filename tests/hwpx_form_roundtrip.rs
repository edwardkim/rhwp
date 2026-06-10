//! 양식 개체(FormObject) HWPX 라운드트립 통합 테스트.
//!
//! `serializer::hwpx::form::write_form` 가 추가되기 전에는 `export_hwpx_native` 가
//! 양식 개체를 전부 누락시켰다(5개 → 0개). 이 테스트는 `samples/hwpx/form-01.hwpx`
//! (5개 폼: btn/checkBtn/radioBtn/comboBox/edit 각 1개)를 열어 저장·재파싱한 뒤
//! 폼 개수와 파서가 보존하는 핵심 필드가 유지되는지 단언한다.

use std::collections::HashMap;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FormObject, FormType};
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;

/// 폼의 라운드트립 비교용 정규화 투영. 파서가 모델에 보존하는 값만 담는다.
/// `FormType` 이 `Eq` 를 파생하지 않으므로 `PartialEq` 만 둔다.
#[derive(Debug, PartialEq)]
struct FormProjection {
    form_type: FormType,
    name: String,
    caption: String,
    text: String,
    value: i32,
    enabled: bool,
    width: u32,
    height: u32,
    fore_color: u32,
    back_color: u32,
    list_items: Vec<String>,
}

impl FormProjection {
    fn from(form: &FormObject) -> Self {
        let mut list_items = Vec::new();
        let mut i = 0;
        while let Some(v) = form.properties.get(&format!("listItem{}", i)) {
            list_items.push(v.clone());
            i += 1;
        }
        FormProjection {
            form_type: form.form_type,
            name: form.name.clone(),
            caption: form.caption.clone(),
            text: form.text.clone(),
            value: form.value,
            enabled: form.enabled,
            width: form.width,
            height: form.height,
            fore_color: form.fore_color,
            back_color: form.back_color,
            list_items,
        }
    }
}

/// 문단의 컨트롤(표 셀 내부 포함)을 재귀적으로 훑어 폼을 수집한다.
fn collect_forms_in_paragraphs(paras: &[Paragraph], out: &mut Vec<FormProjection>) {
    for para in paras {
        for control in &para.controls {
            match control {
                Control::Form(form) => out.push(FormProjection::from(form)),
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect_forms_in_paragraphs(&cell.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

fn collect_forms(doc: &Document) -> Vec<FormProjection> {
    let mut out = Vec::new();
    for section in &doc.sections {
        collect_forms_in_paragraphs(&section.paragraphs, &mut out);
    }
    out
}

/// 이름 → 투영 맵으로 정렬해 순서 무관 비교를 가능케 한다.
fn by_name(forms: Vec<FormProjection>) -> HashMap<String, FormProjection> {
    forms.into_iter().map(|f| (f.name.clone(), f)).collect()
}

#[test]
fn form_objects_survive_hwpx_roundtrip() {
    let bytes = std::fs::read("samples/hwpx/form-01.hwpx").expect("form-01.hwpx 로드 실패");

    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 파싱 실패");
    let original = collect_forms(core.document());

    // 샘플 전제: 5개 폼이 모든 타입을 한 번씩 덮는다. 0개로 떨어지면 비교가
    // 무의미하게 통과하므로 원본 자체를 단언한다.
    assert_eq!(original.len(), 5, "원본 폼 개수: {:?}", original);
    let types: Vec<FormType> = original.iter().map(|f| f.form_type).collect();
    for t in [
        FormType::PushButton,
        FormType::CheckBox,
        FormType::RadioButton,
        FormType::ComboBox,
        FormType::Edit,
    ] {
        assert!(types.contains(&t), "원본에 {:?} 폼 없음: {:?}", t, types);
    }

    let saved = core.export_hwpx_native().expect("HWPX 직렬화 실패");
    let reloaded = DocumentCore::from_bytes(&saved).expect("저장본 재파싱 실패");
    let after = collect_forms(reloaded.document());

    assert_eq!(
        after.len(),
        original.len(),
        "라운드트립 후 폼 개수 불일치: 원본 {} → 저장본 {}",
        original.len(),
        after.len()
    );

    let before_map = by_name(original);
    let after_map = by_name(after);
    for (name, before) in &before_map {
        let after = after_map
            .get(name)
            .unwrap_or_else(|| panic!("저장본에서 폼 '{}' 소실", name));
        assert_eq!(before, after, "폼 '{}' 필드 불일치", name);
    }
}
