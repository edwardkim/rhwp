//! 코퍼스 한 행. `(task, criterion_id, envelope_field, atom_pass, holistic_would_hide)`.

use crate::atom::Expected;
use crate::field::is_allowed_envelope_field;
use crate::verdict::FailKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecompRow {
    pub row_id: String,
    pub task: String,
    pub criterion_id: String,
    pub envelope_field: String,
    pub atom_pass: bool,
    pub holistic_would_hide: bool,
    pub command: String,
    pub bundle_pass_count: u64,
    pub bundle_total: u64,
    pub expected: Expected,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_kind: Option<FailKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default)]
    pub holistic_only: bool,
}

impl DecompRow {
    pub fn tuple_key(&self) -> (String, String, String, bool, bool) {
        (
            self.task.clone(),
            self.criterion_id.clone(),
            self.envelope_field.clone(),
            self.atom_pass,
            self.holistic_would_hide,
        )
    }

    pub fn field_allowed_or_invented_fail(&self) -> bool {
        is_allowed_envelope_field(&self.envelope_field)
            || self.fail_kind == Some(FailKind::InventedField)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_hide_row() {
        let raw = r#"{
            "rowId":"CD-000001",
            "task":"기획재정부 과업지시서 누름틀 2칸을 edit fill-fields --verify 로 검증한다.",
            "criterionId":"C-fill-identical-000001",
            "envelopeField":"verify.identical",
            "atomPass":false,
            "holisticWouldHide":true,
            "command":"edit fill-fields",
            "bundlePassCount":4,
            "bundleTotal":5,
            "expected":{"kind":"bool","value":true},
            "observed":false,
            "failKind":"atom_mismatch"
        }"#;
        let row: DecompRow = serde_json::from_str(raw).expect("row");
        assert!(!row.atom_pass);
        assert!(row.holistic_would_hide);
        assert_eq!(row.envelope_field, "verify.identical");
        assert!(row.field_allowed_or_invented_fail());
    }
}
