//! 스키마 버전과 동봉 JSON Schema 본문.

pub const DECOMP_SCHEMA_VERSION: &str = "v-decomp.1.0";

pub const ATOMIC_CRITERION_SCHEMA: &str = include_str!("../schema/atomic_criterion.schema.json");
pub const DECOMP_ROW_SCHEMA: &str = include_str!("../schema/decomp_row.schema.json");
pub const ENVELOPE_ATOM_SCHEMA: &str = include_str!("../schema/envelope_atom.schema.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_schemas_are_json_objects() {
        for raw in [
            ATOMIC_CRITERION_SCHEMA,
            DECOMP_ROW_SCHEMA,
            ENVELOPE_ATOM_SCHEMA,
        ] {
            let v: serde_json::Value = serde_json::from_str(raw).expect("schema json");
            assert!(v.get("$schema").is_some());
            assert!(v.get("title").is_some());
        }
    }
}
