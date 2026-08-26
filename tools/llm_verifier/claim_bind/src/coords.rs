//! 문서 좌표. 키 이름은 search/extract-data 봉투가 주는 것만 쓴다.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

/// 축 3 필수 좌표. 하나라도 없으면 주장은 미결속이다.
pub const REQUIRED_COORD_FIELDS: &[&str] = &["section", "paragraph", "page", "charOffset"];

/// 봉투가 줄 수 있는 좌표 키. 이 밖은 발명이다.
pub const ALLOWED_COORD_FIELDS: &[&str] = &[
    "section",
    "paragraph",
    "page",
    "charOffset",
    "length",
    "cell",
    "textbox",
];

/// 전략가 좌표 규칙이 금지하는 발명 키. 여기 있으면 무조건 실패.
pub const INVENTED_COORD_FIELDS: &[&str] = &["line", "column", "pdfPage", "humanPage", "offset"];

/// `search`/`extract-data` 매치 한 건에서 복사한 좌표.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCoords {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub char_offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textbox: Option<Value>,
}

impl DocumentCoords {
    pub fn from_envelope_value(v: &Value) -> Self {
        Self {
            section: as_u64(v.get("section")),
            paragraph: as_u64(v.get("paragraph")),
            page: as_u64(v.get("page")),
            char_offset: as_u64(v.get("charOffset")),
            length: as_u64(v.get("length")),
            cell: v.get("cell").cloned().filter(|c| !c.is_null()),
            textbox: v.get("textbox").cloned().filter(|c| !c.is_null()),
        }
    }

    pub fn present_fields(&self) -> BTreeSet<&'static str> {
        let mut set = BTreeSet::new();
        if self.section.is_some() {
            set.insert("section");
        }
        if self.paragraph.is_some() {
            set.insert("paragraph");
        }
        if self.page.is_some() {
            set.insert("page");
        }
        if self.char_offset.is_some() {
            set.insert("charOffset");
        }
        if self.length.is_some() {
            set.insert("length");
        }
        if self.cell.is_some() {
            set.insert("cell");
        }
        if self.textbox.is_some() {
            set.insert("textbox");
        }
        set
    }

    pub fn field_set(&self) -> Vec<String> {
        self.present_fields()
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    /// 필수 4키가 모두 있다.
    pub fn coords_present(&self) -> bool {
        self.section.is_some()
            && self.paragraph.is_some()
            && self.page.is_some()
            && self.char_offset.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.present_fields().is_empty()
    }

    pub fn required_missing(&self) -> Vec<&'static str> {
        REQUIRED_COORD_FIELDS
            .iter()
            .copied()
            .filter(|k| !self.present_fields().contains(k))
            .collect()
    }

    /// 필수 4키만 비교. 선택 키(length/cell/textbox)는 결속에 쓰지 않는다.
    pub fn required_tuple(&self) -> Option<(u64, u64, u64, u64)> {
        Some((
            self.section?,
            self.paragraph?,
            self.page?,
            self.char_offset?,
        ))
    }
}

pub fn field_set_of(coords: Option<&DocumentCoords>) -> Vec<String> {
    coords.map(DocumentCoords::field_set).unwrap_or_default()
}

pub fn invented_keys_in(value: &Value) -> Vec<String> {
    let Some(obj) = value.as_object() else {
        return Vec::new();
    };
    obj.keys()
        .filter(|k| INVENTED_COORD_FIELDS.contains(&k.as_str()))
        .cloned()
        .collect()
}

pub fn unknown_coord_keys_in(value: &Value) -> Vec<String> {
    let Some(obj) = value.as_object() else {
        return Vec::new();
    };
    obj.keys()
        .filter(|k| {
            is_coord_like(k)
                && !ALLOWED_COORD_FIELDS.contains(&k.as_str())
                && !INVENTED_COORD_FIELDS.contains(&k.as_str())
        })
        .cloned()
        .collect()
}

fn is_coord_like(key: &str) -> bool {
    matches!(
        key,
        "section"
            | "paragraph"
            | "page"
            | "charOffset"
            | "length"
            | "cell"
            | "textbox"
            | "line"
            | "column"
            | "pdfPage"
            | "humanPage"
            | "offset"
            | "char_offset"
            | "pdf_page"
            | "human_page"
    )
}

fn as_u64(v: Option<&Value>) -> Option<u64> {
    match v? {
        Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_i64().and_then(|i| u64::try_from(i).ok())),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn missing_page_is_not_coords_present() {
        let c = DocumentCoords::from_envelope_value(&json!({
            "section": 0,
            "paragraph": 4,
            "charOffset": 8
        }));
        assert!(!c.coords_present());
        assert_eq!(c.required_missing(), ["page"]);
    }

    #[test]
    fn full_search_hit_is_coords_present() {
        let c = DocumentCoords::from_envelope_value(&json!({
            "section": 0,
            "paragraph": 4,
            "page": 1,
            "charOffset": 8,
            "length": 7
        }));
        assert!(c.coords_present());
        assert_eq!(
            c.field_set(),
            ["charOffset", "length", "page", "paragraph", "section"]
        );
    }

    #[test]
    fn invented_keys_are_detected() {
        let keys = invented_keys_in(&json!({"page": 1, "pdfPage": 2, "humanPage": 3}));
        assert_eq!(keys, ["humanPage", "pdfPage"]);
    }

    #[test]
    fn page_zero_is_valid_envelope_page() {
        let c = DocumentCoords::from_envelope_value(&json!({
            "section": 0,
            "paragraph": 0,
            "page": 0,
            "charOffset": 0
        }));
        assert!(c.coords_present());
        assert_eq!(c.page, Some(0));
    }
}
