//! 개체 속성 JSON(`--props`)을 setter 에 넘기기 전에 검사한다 (#9019).
//!
//! setter(`apply_picture_props_inner` 등)는 아는 키만 찾아 읽고, 모르는 키나 모르는 열거 값은
//! 건너뛴다. 그래서 오타가 있어도 오류 없이 끝나 호출자가 바뀌지 않은 것을 알 수 없다. 이 검사는
//! CLI·MCP 입구에서만 쓴다. WASM 의 setter 는 조회 결과를 그대로 되먹이는 계약(#6806)이 있어
//! 조회 전용 키가 섞여 들어오므로 여기서 거부하지 않는다.

/// 속성 값의 형식.
#[derive(Clone, Copy)]
pub(crate) enum PropKind {
    Bool,
    /// 정수와 허용 범위(양 끝 포함).
    Int(i64, i64),
    Str,
    /// 허용하는 문자열 값 목록. 대소문자를 구분한다.
    Enum(&'static [&'static str]),
}

pub(crate) struct PropSpec {
    pub key: &'static str,
    pub kind: PropKind,
    /// 같은 JSON 에 `"hasCaption": true` 가 있어야 setter 가 읽는 키.
    pub needs_caption: bool,
}

const fn prop(key: &'static str, kind: PropKind) -> PropSpec {
    PropSpec {
        key,
        kind,
        needs_caption: false,
    }
}

const fn caption_prop(key: &'static str, kind: PropKind) -> PropSpec {
    PropSpec {
        key,
        kind,
        needs_caption: true,
    }
}

const I8: PropKind = PropKind::Int(i8::MIN as i64, i8::MAX as i64);
const I16: PropKind = PropKind::Int(i16::MIN as i64, i16::MAX as i64);
const I32: PropKind = PropKind::Int(i32::MIN as i64, i32::MAX as i64);
const U32: PropKind = PropKind::Int(0, u32::MAX as i64);

const VERT_REL_TO: &[&str] = &["Paper", "Page", "Para"];
const HORZ_REL_TO: &[&str] = &["Paper", "Page", "Column", "Para"];
const VERT_ALIGN: &[&str] = &["Top", "Center", "Bottom"];
const HORZ_ALIGN: &[&str] = &["Left", "Center", "Right"];
const TEXT_WRAP: &[&str] = &[
    "Square",
    "Tight",
    "Through",
    "TopAndBottom",
    "BehindText",
    "InFrontOfText",
];
const IMAGE_EFFECT: &[&str] = &["RealPic", "GrayScale", "BlackWhite", "Pattern8x8"];
const CAPTION_DIRECTION: &[&str] = &["Left", "Right", "Top", "Bottom"];
const CAPTION_VERT_ALIGN: &[&str] = &["Top", "Center", "Bottom"];

/// `apply_picture_props_inner` 가 읽는 키와 값 형식.
pub(crate) const PICTURE_PROPS: &[PropSpec] = &[
    prop("width", U32),
    prop("height", U32),
    prop("treatAsChar", PropKind::Bool),
    prop("vertRelTo", PropKind::Enum(VERT_REL_TO)),
    prop("horzRelTo", PropKind::Enum(HORZ_REL_TO)),
    prop("vertAlign", PropKind::Enum(VERT_ALIGN)),
    prop("horzAlign", PropKind::Enum(HORZ_ALIGN)),
    prop("textWrap", PropKind::Enum(TEXT_WRAP)),
    prop("restrictInPage", PropKind::Bool),
    prop("allowOverlap", PropKind::Bool),
    prop("sizeProtect", PropKind::Bool),
    prop("vertOffset", I32),
    prop("horzOffset", I32),
    prop("brightness", I8),
    prop("contrast", I8),
    prop("transparency", PropKind::Int(0, 100)),
    prop("effect", PropKind::Enum(IMAGE_EFFECT)),
    prop("rotationAngle", I16),
    prop("horzFlip", PropKind::Bool),
    prop("vertFlip", PropKind::Bool),
    prop("cropLeft", I32),
    prop("cropTop", I32),
    prop("cropRight", I32),
    prop("cropBottom", I32),
    prop("paddingLeft", I16),
    prop("paddingTop", I16),
    prop("paddingRight", I16),
    prop("paddingBottom", I16),
    prop("outerMarginLeft", I16),
    prop("outerMarginTop", I16),
    prop("outerMarginRight", I16),
    prop("outerMarginBottom", I16),
    prop("borderColor", U32),
    prop("borderWidth", I32),
    prop("description", PropKind::Str),
    prop("hasCaption", PropKind::Bool),
    caption_prop("captionDirection", PropKind::Enum(CAPTION_DIRECTION)),
    caption_prop("captionVertAlign", PropKind::Enum(CAPTION_VERT_ALIGN)),
    caption_prop("captionWidth", U32),
    caption_prop("captionSpacing", I16),
    caption_prop("captionIncludeMargin", PropKind::Bool),
];

/// 속성 JSON 이 `specs` 에 맞는지 검사한다. 문제가 있으면 사람이 읽을 오류 문장을 돌려준다.
///
/// 빈 객체는 바꿀 것이 없으므로 거부한다. 키 하나라도 맞지 않으면 전체를 거부해, 일부만
/// 적용되고 나머지가 조용히 빠지는 일이 없게 한다.
pub(crate) fn check_props(json: &str, specs: &[PropSpec], what: &str) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("--props 가 올바른 JSON 이 아닙니다 ({e})"))?;
    let Some(object) = value.as_object() else {
        return Err("--props 는 JSON 객체여야 합니다".to_string());
    };
    if object.is_empty() {
        return Err(format!("--props 에 바꿀 {what} 속성이 없습니다"));
    }
    let known_keys = || {
        specs
            .iter()
            .map(|spec| spec.key)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let has_caption = object.get("hasCaption").and_then(|v| v.as_bool()) == Some(true);
    for (key, v) in object {
        let Some(spec) = specs.iter().find(|spec| spec.key == key) else {
            return Err(format!(
                "--props 에 {what} 속성이 아닌 키가 있습니다: {key} (받는 키: {})",
                known_keys()
            ));
        };
        match spec.kind {
            PropKind::Bool => {
                if !v.is_boolean() {
                    return Err(format!("--props 의 {key} 값은 true/false 여야 합니다: {v}"));
                }
            }
            PropKind::Int(min, max) => match v.as_i64() {
                Some(n) if (min..=max).contains(&n) => {}
                _ => {
                    return Err(format!(
                        "--props 의 {key} 값은 {min}~{max} 범위의 정수여야 합니다: {v}"
                    ))
                }
            },
            PropKind::Str => {
                if !v.is_string() {
                    return Err(format!("--props 의 {key} 값은 문자열이어야 합니다: {v}"));
                }
            }
            PropKind::Enum(allowed) => {
                if !v.as_str().is_some_and(|s| allowed.contains(&s)) {
                    return Err(format!(
                        "--props 의 {key} 값이 허용되지 않습니다: {v} (허용: {})",
                        allowed.join(", ")
                    ));
                }
            }
        }
        if spec.needs_caption && !has_caption {
            return Err(format!(
                "--props 에 \"hasCaption\": true 없이 캡션 키가 있습니다: {key}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(json: &str) -> Result<(), String> {
        check_props(json, PICTURE_PROPS, "그림")
    }

    #[test]
    fn accepts_known_keys_and_values() {
        assert!(
            check(r#"{"textWrap":"TopAndBottom","vertRelTo":"Para","horzAlign":"Center"}"#).is_ok()
        );
        assert!(check(r#"{"treatAsChar":true,"width":20000,"brightness":-20}"#).is_ok());
        assert!(
            check(r#"{"hasCaption":true,"captionDirection":"Top","captionSpacing":850}"#).is_ok()
        );
        assert!(check(r#"{"effect":"RealPic","description":"그림 1"}"#).is_ok());
    }

    #[test]
    fn rejects_input_the_setter_would_skip() {
        for json in [
            "abc",
            "[1,2]",
            "{}",
            r#"{"wrap":"TopAndBottom"}"#,
            r#"{"textWrap":"topAndBottom"}"#,
            r#"{"textWrap":"TopAndBottom","vertRelto":"Para"}"#,
            r#"{"horzAlign":"Inside"}"#,
            r#"{"treatAsChar":"true"}"#,
            r#"{"width":-1}"#,
            r#"{"paddingLeft":40000}"#,
            r#"{"vertOffset":1.5}"#,
            r#"{"captionDirection":"Top"}"#,
            r#"{"hasCaption":false,"captionDirection":"Top"}"#,
        ] {
            assert!(check(json).is_err(), "거부해야 함: {json}");
        }
    }

    /// 목록에 적은 키를 setter 가 실제로 읽는지 확인한다. 목록에만 있고 setter 가 읽지 않는 키는
    /// 검사를 통과한 뒤 다시 조용히 건너뛰어지기 때문이다.
    #[test]
    fn every_listed_key_is_read_by_the_setter() {
        let start = include_str!("picture.rs")
            .find("fn apply_picture_props_inner")
            .expect("setter");
        let setter = &include_str!("picture.rs")[start..];
        let end = setter.find("caption_created\n    }").expect("setter end");
        let setter = &setter[..end];
        for spec in PICTURE_PROPS {
            assert!(
                setter.contains(&format!("props_json, \"{}\")", spec.key)),
                "setter 가 읽지 않는 키: {}",
                spec.key
            );
        }
    }
}
