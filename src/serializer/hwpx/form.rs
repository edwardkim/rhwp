//! 양식 개체(FormObject) 직렬화 — `<hp:btn>` / `<hp:checkBtn>` / `<hp:radioBtn>` /
//! `<hp:comboBox>` / `<hp:edit>`.
//!
//! `parser::hwpx::section::parse_form_object` 의 역방향이다. 폼 컨트롤은 `<hp:run>` 의
//! 직접 자식으로 위치하므로 (Table/Picture 와 동일, Field 처럼 `<hp:ctrl>` 로 감싸지 않음)
//! `render_control_slot` 에서 이 결과를 그대로 run 안에 출력한다.
//!
//! ## 보존 범위
//!
//! 파서가 읽어 모델에 보존하는 값은 모두 충실히 재현한다. 스칼라 필드:
//! - 요소명(폼 타입), `name`, `caption`(버튼류), `value`(체크 상태), `selectedValue`/
//!   `<hp:text>`(콤보/에디트 텍스트), `foreColor`/`backColor`, `<hp:sz>` 폭·높이, `enabled`.
//!
//! `properties` 에 키-값으로 보존되어 round-trip 하는 속성:
//! - 공통: `GroupName`/`TabStop`/`Editable`/`TabOrder`/`BorderType`/`DrawFrame`/`Printable`/
//!   `Command`/`CharShapeID`/`FollowContext`/`AutoSize`/`WordWrap`
//! - 버튼류: `RadioGroupName`/`TriState`/`BackStyle`
//! - 콤보: `ListBoxRows`/`ListBoxWidth`/`EditEnable`, 항목 `listItem{N}`+`listItemDisplay{N}`
//! - 에디트: `MultiLine`/`PasswordChar`/`MaxLength`/`ScrollBars`/`TabKeyBehavior`/`Number`/
//!   `ReadOnly`/`AlignText`
//! - `<hp:sz>`: `SzWidthRelTo`/`SzHeightRelTo`/`SzProtect`
//! - `<hp:pos>`(앵커링 11속성): `PosTreatAsChar`/`PosAffectLSpacing`/`PosFlowWithText`/
//!   `PosAllowOverlap`/`PosHoldAnchorAndSO`/`PosVertRelTo`/`PosHorzRelTo`/`PosVertAlign`/
//!   `PosHorzAlign`/`PosVertOffset`/`PosHorzOffset`
//! - `<hp:outMargin>`: `OutMarginLeft`/`OutMarginRight`/`OutMarginTop`/`OutMarginBottom`
//!
//! 위 키가 `properties` 에 없으면(예: 새로 생성한 폼) 각 속성은 한컴 기본값으로 출력된다.
//!
//! ## 알려진 손실
//!
//! 위에 열거한 표준 스키마 속성만 보존한다. `parse_form_object` 가 디스패치/열거하지 않는
//! 비표준·확장 속성(있다면)은 모델에 들어오지 않아 출력 시 누락된다.

use std::io::Write;

use quick_xml::Writer;

use crate::model::control::{FormObject, FormType};

use super::utils::{empty_tag, end_tag, start_tag_attrs, text};
use super::SerializeError;

/// 폼 타입 → HWPX 요소명 (parser 디스패치와 정확히 일치해야 함).
fn form_tag(form_type: FormType) -> &'static str {
    match form_type {
        FormType::PushButton => "hp:btn",
        FormType::CheckBox => "hp:checkBtn",
        FormType::RadioButton => "hp:radioBtn",
        FormType::ComboBox => "hp:comboBox",
        FormType::Edit => "hp:edit",
    }
}

/// 모델 색(`0x00BBGGRR`) → HWPX `#RRGGBB`. `parse_color_str` 의 역방향.
fn color_to_hex(color: u32) -> String {
    let r = color & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = (color >> 16) & 0xFF;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn prop<'a>(form: &'a FormObject, key: &str, default: &'a str) -> &'a str {
    form.properties
        .get(key)
        .map(|s| s.as_str())
        .unwrap_or(default)
}

/// 콤보박스 항목(`properties["listItem0"]`, `"listItem1"`, …)을 순서대로 수집한다.
fn combo_items(form: &FormObject) -> Vec<&str> {
    let mut items = Vec::new();
    let mut i = 0;
    while let Some(v) = form.properties.get(&format!("listItem{}", i)) {
        items.push(v.as_str());
        i += 1;
    }
    items
}

/// `<hp:btn>` / `<hp:checkBtn>` / `<hp:radioBtn>` / `<hp:comboBox>` / `<hp:edit>` 를 출력한다.
pub fn write_form<W: Write>(w: &mut Writer<W>, form: &FormObject) -> Result<(), SerializeError> {
    let tag = form_tag(form.form_type);
    let fore = color_to_hex(form.fore_color);
    let back = color_to_hex(form.back_color);
    let enabled = if form.enabled { "1" } else { "0" };
    let checked = if form.value == 1 {
        "CHECKED"
    } else {
        "UNCHECKED"
    };

    // 공통 속성 (모든 폼 타입). 순서는 한컴 산출물을 따른다.
    let mut attrs: Vec<(&str, &str)> = Vec::new();

    // 타입별 선두 속성
    match form.form_type {
        FormType::PushButton | FormType::CheckBox | FormType::RadioButton => {
            attrs.push(("caption", &form.caption));
            attrs.push(("value", checked));
            attrs.push(("radioGroupName", prop(form, "RadioGroupName", "")));
            attrs.push(("triState", prop(form, "TriState", "0")));
            attrs.push(("backStyle", prop(form, "BackStyle", "OPAQUE")));
        }
        FormType::ComboBox => {
            attrs.push(("listBoxRows", prop(form, "ListBoxRows", "4")));
            attrs.push(("listBoxWidth", prop(form, "ListBoxWidth", "0")));
            attrs.push(("editEnable", prop(form, "EditEnable", "1")));
            attrs.push(("selectedValue", &form.text));
        }
        FormType::Edit => {
            attrs.push(("multiLine", prop(form, "MultiLine", "0")));
            attrs.push(("passwordChar", prop(form, "PasswordChar", "")));
            attrs.push(("maxLength", prop(form, "MaxLength", "2147483647")));
            attrs.push(("scrollBars", prop(form, "ScrollBars", "NONE")));
            attrs.push((
                "tabKeyBehavior",
                prop(form, "TabKeyBehavior", "NEXT_OBJECT"),
            ));
            attrs.push(("numOnly", prop(form, "Number", "0")));
            attrs.push(("readOnly", prop(form, "ReadOnly", "0")));
            attrs.push(("alignText", prop(form, "AlignText", "LEFT")));
        }
    }

    // 공통 속성 (AbstractFormObjectType)
    attrs.push(("name", &form.name));
    attrs.push(("foreColor", &fore));
    attrs.push(("backColor", &back));
    attrs.push(("groupName", prop(form, "GroupName", "")));
    attrs.push(("tabStop", prop(form, "TabStop", "1")));
    attrs.push(("editable", prop(form, "Editable", "1")));
    attrs.push(("tabOrder", prop(form, "TabOrder", "0")));
    attrs.push(("enabled", enabled));
    attrs.push(("borderTypeIDRef", prop(form, "BorderType", "0")));
    attrs.push(("drawFrame", prop(form, "DrawFrame", "1")));
    attrs.push(("printable", prop(form, "Printable", "1")));
    attrs.push(("command", prop(form, "Command", "")));

    start_tag_attrs(w, tag, &attrs)?;

    // 자식: formCharPr → (listItem*/text) → sz → pos → outMargin (한컴 순서)
    empty_tag(
        w,
        "hp:formCharPr",
        &[
            ("charPrIDRef", prop(form, "CharShapeID", "0")),
            ("followContext", prop(form, "FollowContext", "0")),
            ("autoSz", prop(form, "AutoSize", "0")),
            ("wordWrap", prop(form, "WordWrap", "0")),
        ],
    )?;

    match form.form_type {
        FormType::ComboBox => {
            for (i, item) in combo_items(form).into_iter().enumerate() {
                let display_key = format!("listItemDisplay{}", i);
                let display = prop(form, &display_key, "");
                empty_tag(
                    w,
                    "hp:listItem",
                    &[("displayText", display), ("value", item)],
                )?;
            }
        }
        FormType::Edit => {
            // 항상 <hp:text> 자식을 둔다 (비어 있어도). 파서가 텍스트 내용을 여기서 읽는다.
            start_tag_attrs(w, "hp:text", &[])?;
            if !form.text.is_empty() {
                text(w, &form.text)?;
            }
            end_tag(w, "hp:text")?;
        }
        _ => {}
    }

    let width = form.width.to_string();
    let height = form.height.to_string();
    empty_tag(
        w,
        "hp:sz",
        &[
            ("width", &width),
            ("widthRelTo", prop(form, "SzWidthRelTo", "ABSOLUTE")),
            ("height", &height),
            ("heightRelTo", prop(form, "SzHeightRelTo", "ABSOLUTE")),
            ("protect", prop(form, "SzProtect", "0")),
        ],
    )?;

    // <hp:pos> — 파서가 보존한 앵커링 값(`Pos*`)을 그대로 출력. 없으면 인라인
    // (treatAsChar=1) 기본값으로 떨어진다.
    empty_tag(
        w,
        "hp:pos",
        &[
            ("treatAsChar", prop(form, "PosTreatAsChar", "1")),
            ("affectLSpacing", prop(form, "PosAffectLSpacing", "0")),
            ("flowWithText", prop(form, "PosFlowWithText", "1")),
            ("allowOverlap", prop(form, "PosAllowOverlap", "1")),
            ("holdAnchorAndSO", prop(form, "PosHoldAnchorAndSO", "0")),
            ("vertRelTo", prop(form, "PosVertRelTo", "PARA")),
            ("horzRelTo", prop(form, "PosHorzRelTo", "COLUMN")),
            ("vertAlign", prop(form, "PosVertAlign", "TOP")),
            ("horzAlign", prop(form, "PosHorzAlign", "LEFT")),
            ("vertOffset", prop(form, "PosVertOffset", "0")),
            ("horzOffset", prop(form, "PosHorzOffset", "0")),
        ],
    )?;

    empty_tag(
        w,
        "hp:outMargin",
        &[
            ("left", prop(form, "OutMarginLeft", "0")),
            ("right", prop(form, "OutMarginRight", "0")),
            ("top", prop(form, "OutMarginTop", "0")),
            ("bottom", prop(form, "OutMarginBottom", "0")),
        ],
    )?;

    end_tag(w, tag)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn to_string<F: FnOnce(&mut Writer<Vec<u8>>) -> Result<(), SerializeError>>(f: F) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        f(&mut w).expect("write");
        String::from_utf8(w.into_inner()).unwrap()
    }

    #[test]
    fn color_inverts_parse_color_str() {
        // parse_color_str("#1A2B3C") = 0x003C2B1A. 역변환은 "#1A2B3C".
        assert_eq!(color_to_hex(0x003C2B1A), "#1A2B3C");
        assert_eq!(color_to_hex(0x00F0F0F0), "#F0F0F0");
    }

    #[test]
    fn checkbox_emits_checked_value_and_caption() {
        let form = FormObject {
            form_type: FormType::CheckBox,
            name: "CheckBox".to_string(),
            caption: "선택 상자".to_string(),
            value: 1,
            enabled: true,
            ..Default::default()
        };
        let xml = to_string(|w| write_form(w, &form));
        assert!(xml.starts_with("<hp:checkBtn"), "{xml}");
        assert!(xml.contains(r#"caption="선택 상자""#), "{xml}");
        assert!(xml.contains(r#"value="CHECKED""#), "{xml}");
        assert!(xml.contains(r#"name="CheckBox""#), "{xml}");
        assert!(xml.contains("</hp:checkBtn>"), "{xml}");
    }

    #[test]
    fn combobox_emits_list_items_and_selected_value() {
        let mut props = HashMap::new();
        props.insert("listItem0".to_string(), "계절 선택".to_string());
        props.insert("listItem1".to_string(), "봄".to_string());
        let form = FormObject {
            form_type: FormType::ComboBox,
            name: "ComboBox".to_string(),
            text: "봄".to_string(),
            enabled: true,
            properties: props,
            ..Default::default()
        };
        let xml = to_string(|w| write_form(w, &form));
        assert!(xml.starts_with("<hp:comboBox"), "{xml}");
        assert!(xml.contains(r#"selectedValue="봄""#), "{xml}");
        assert!(
            xml.contains(r#"<hp:listItem displayText="" value="계절 선택"/>"#),
            "{xml}"
        );
        assert!(
            xml.contains(r#"<hp:listItem displayText="" value="봄"/>"#),
            "{xml}"
        );
    }

    #[test]
    fn edit_emits_text_child() {
        let form = FormObject {
            form_type: FormType::Edit,
            name: "Edit".to_string(),
            text: "입력값".to_string(),
            enabled: true,
            ..Default::default()
        };
        let xml = to_string(|w| write_form(w, &form));
        assert!(xml.starts_with("<hp:edit"), "{xml}");
        assert!(xml.contains("<hp:text>입력값</hp:text>"), "{xml}");
    }

    #[test]
    fn nondefault_pos_outmargin_sz_listdisplay_are_emitted_from_props() {
        // 파서가 보존한 비기본값(`Pos*`/`OutMargin*`/`Sz*`/`listItemDisplay*`)이
        // 하드코딩 기본이 아니라 properties 에서 출력되는지 확인.
        let mut props = HashMap::new();
        props.insert("PosVertOffset".to_string(), "1234".to_string());
        props.insert("PosVertRelTo".to_string(), "PAPER".to_string());
        props.insert("OutMarginLeft".to_string(), "283".to_string());
        props.insert("SzWidthRelTo".to_string(), "RELATIVE".to_string());
        props.insert("SzProtect".to_string(), "1".to_string());
        props.insert("listItem0".to_string(), "v0".to_string());
        props.insert("listItemDisplay0".to_string(), "표시0".to_string());
        let form = FormObject {
            form_type: FormType::ComboBox,
            name: "ComboBox".to_string(),
            properties: props,
            ..Default::default()
        };
        let xml = to_string(|w| write_form(w, &form));
        assert!(xml.contains(r#"vertOffset="1234""#), "{xml}");
        assert!(xml.contains(r#"vertRelTo="PAPER""#), "{xml}");
        assert!(xml.contains(r#"left="283""#), "{xml}");
        assert!(xml.contains(r#"widthRelTo="RELATIVE""#), "{xml}");
        assert!(xml.contains(r#"protect="1""#), "{xml}");
        assert!(
            xml.contains(r#"<hp:listItem displayText="표시0" value="v0"/>"#),
            "{xml}"
        );
    }
}
