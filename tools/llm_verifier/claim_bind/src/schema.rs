//! 결속 입력·행 JSON Schema. 기존 봉투 키만 적는다.

pub const BIND_SCHEMA_VERSION: &str = "v-bind.1.0";

pub const CLAIM_BIND_ROW_SCHEMA: &str = include_str!("../schema/claim_bind_row.schema.json");
pub const COORD_BIND_SCHEMA: &str = include_str!("../schema/coord_bind.schema.json");
pub const SEARCH_EXTRACT_ENVELOPE_SCHEMA: &str =
    include_str!("../schema/search_extract_envelope.schema.json");

pub fn schema_parses(raw: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(raw).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_schemas_are_objects() {
        for raw in [
            CLAIM_BIND_ROW_SCHEMA,
            COORD_BIND_SCHEMA,
            SEARCH_EXTRACT_ENVELOPE_SCHEMA,
        ] {
            let v = schema_parses(raw).expect("schema json");
            assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("object"));
        }
    }

    #[test]
    fn row_schema_requires_four_tuple_fields() {
        let v = schema_parses(CLAIM_BIND_ROW_SCHEMA).unwrap();
        let req = v.get("required").and_then(|r| r.as_array()).unwrap();
        for key in ["claimText", "coordsPresent", "fieldSet", "verdict"] {
            assert!(req.iter().any(|x| x.as_str() == Some(key)), "missing {key}");
        }
    }
}
