//! Caller-declared font availability for one rendering session, never document IR.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::HwpError;

/// Explicit final face substitutions shared by layout and paint.
///
/// Keys are exact document font names. Targets are single installed/renderable
/// faces, not CSS fallback chains. Mappings are applied once, not recursively.
/// This declaration does not prove that the backend has the target font.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FontEnvironment {
    id: String,
    substitutions: BTreeMap<String, String>,
    /// 호출자가 실제 출력 프로그램을 확인한 TrueType 최종 face.
    /// 설치 증명은 아니며, 생략하면 기존 HFT 호환 메트릭을 유지한다.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    true_type_faces: Vec<String>,
}

impl FontEnvironment {
    pub fn from_json(json: &str) -> Result<Self, HwpError> {
        if json.len() > 65_536 {
            return Err(HwpError::RenderError(
                "font environment exceeds 64 KiB".into(),
            ));
        }
        let environment: Self = serde_json::from_str(json)
            .map_err(|e| HwpError::RenderError(format!("invalid font environment: {e}")))?;
        environment.validate()?;
        Ok(environment)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn validate(&self) -> Result<(), HwpError> {
        let valid_name = |s: &str| {
            !s.is_empty()
                && s.trim() == s
                && s.len() <= 256
                && !s.chars().any(|c| {
                    c.is_control() || matches!(c, ',' | '\'' | '"' | '\\' | ';' | '{' | '}')
                })
        };
        if !valid_name(&self.id)
            || self.true_type_faces.len() > 256
            || self.true_type_faces.iter().any(|face| !valid_name(face))
            || self
                .true_type_faces
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.true_type_faces.len()
            || self.substitutions.len() > 256
            || self
                .substitutions
                .iter()
                .any(|(from, to)| !valid_name(from) || !valid_name(to))
        {
            return Err(HwpError::RenderError(
                "font environment requires an id and at most 256 single-face substitutions".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn selects_true_type(&self, face: &str) -> bool {
        self.true_type_faces.iter().any(|selected| selected == face)
    }

    pub(crate) fn replacement(&self, face: &str) -> Option<&str> {
        self.substitutions.get(face).map(String::as_str)
    }
}
