/// Versioned root metadata for PageLayerTree JSON exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerTreeSchema {
    /// Major schema version. This remains an integer for v1 compatibility.
    pub schema_version: u32,
    /// Additive schema revision within the current major version.
    pub schema_minor_version: u32,
    /// Major resource table version. This remains an integer for v1 compatibility.
    pub resource_table_version: u32,
    /// Additive resource-table revision within the current major version.
    pub resource_table_minor_version: u32,
    pub unit: &'static str,
    pub coordinate_system: &'static str,
}

pub const LAYER_TREE_SCHEMA: LayerTreeSchema = LayerTreeSchema {
    schema_version: 1,
    schema_minor_version: 15,
    resource_table_version: 1,
    resource_table_minor_version: 4,
    unit: "px",
    coordinate_system: "page-top-left-y-down",
};

pub const PAGE_LAYER_TREE_SCHEMA_VERSION: u32 = LAYER_TREE_SCHEMA.schema_version;
pub const PAGE_LAYER_TREE_SCHEMA_MINOR_VERSION: u32 = LAYER_TREE_SCHEMA.schema_minor_version;
pub const PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION: u32 = LAYER_TREE_SCHEMA.resource_table_version;
pub const PAGE_LAYER_TREE_RESOURCE_TABLE_MINOR_VERSION: u32 =
    LAYER_TREE_SCHEMA.resource_table_minor_version;
pub const PAGE_LAYER_TREE_UNIT: &str = LAYER_TREE_SCHEMA.unit;
pub const PAGE_LAYER_TREE_COORDINATE_SYSTEM: &str = LAYER_TREE_SCHEMA.coordinate_system;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_tree_schema_constants_match_schema() {
        assert_eq!(
            LAYER_TREE_SCHEMA,
            LayerTreeSchema {
                schema_version: PAGE_LAYER_TREE_SCHEMA_VERSION,
                schema_minor_version: PAGE_LAYER_TREE_SCHEMA_MINOR_VERSION,
                resource_table_version: PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION,
                resource_table_minor_version: PAGE_LAYER_TREE_RESOURCE_TABLE_MINOR_VERSION,
                unit: PAGE_LAYER_TREE_UNIT,
                coordinate_system: PAGE_LAYER_TREE_COORDINATE_SYSTEM,
            }
        );
    }
}
