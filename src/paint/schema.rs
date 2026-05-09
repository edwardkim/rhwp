/// Versioned root metadata for PageLayerTree JSON exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerTreeSchema {
    pub schema_version: u32,
    pub resource_table_version: u32,
    pub unit: &'static str,
    pub coordinate_system: &'static str,
}

pub const LAYER_TREE_SCHEMA: LayerTreeSchema = LayerTreeSchema {
    schema_version: 1,
    resource_table_version: 1,
    unit: "px",
    coordinate_system: "page-top-left",
};

pub const PAGE_LAYER_TREE_SCHEMA_VERSION: u32 = LAYER_TREE_SCHEMA.schema_version;
pub const PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION: u32 = LAYER_TREE_SCHEMA.resource_table_version;
pub const PAGE_LAYER_TREE_UNIT: &str = LAYER_TREE_SCHEMA.unit;
pub const PAGE_LAYER_TREE_COORDINATE_SYSTEM: &str = LAYER_TREE_SCHEMA.coordinate_system;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_tree_schema_contract_is_stable() {
        assert_eq!(LAYER_TREE_SCHEMA.schema_version, 1);
        assert_eq!(LAYER_TREE_SCHEMA.resource_table_version, 1);
        assert_eq!(LAYER_TREE_SCHEMA.unit, "px");
        assert_eq!(LAYER_TREE_SCHEMA.coordinate_system, "page-top-left");
    }
}
