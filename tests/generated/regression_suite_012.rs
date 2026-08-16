//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_012

#[path = "../hidden_text_contract.rs"]
mod hidden_text_contract;

#[path = "../issue_1279_picture_rotation_save.rs"]
mod issue_1279_picture_rotation_save;

#[path = "../issue_1392_shape_comment_roundtrip.rs"]
mod issue_1392_shape_comment_roundtrip;

#[path = "../issue_1453_chart_3d_ofpie_routing.rs"]
mod issue_1453_chart_3d_ofpie_routing;

#[path = "../issue_1763_cell_trailing_ls_expand.rs"]
mod issue_1763_cell_trailing_ls_expand;

#[path = "../issue_2070_rowbreak_density.rs"]
mod issue_2070_rowbreak_density;

#[path = "../issue_3236_split_single_cell_table.rs"]
mod issue_3236_split_single_cell_table;

#[path = "../issue_3931_declared_rowbreak.rs"]
mod issue_3931_declared_rowbreak;

#[path = "../issue_4090_hwpx_tail_page_break.rs"]
mod issue_4090_hwpx_tail_page_break;

#[path = "../issue_4515_table_overlap_diag.rs"]
mod issue_4515_table_overlap_diag;

#[path = "../issue_4698_split_cell_fragment_ownership.rs"]
mod issue_4698_split_cell_fragment_ownership;

#[path = "../lineage_contract.rs"]
mod lineage_contract;

#[path = "../mcp_arg_validation_contract.rs"]
mod mcp_arg_validation_contract;

#[path = "../mcp_next_call_contract.rs"]
mod mcp_next_call_contract;

#[path = "../mcp_spec_ledger_contract.rs"]
mod mcp_spec_ledger_contract;

#[path = "../suites/issue_regression_pilot/issue_1035_alignment.rs"]
mod issue_1035_alignment;

#[path = "../suites/issue_regression_pilot/issue_1624_footer_overpush_pagination.rs"]
mod issue_1624_footer_overpush_pagination;

#[path = "../visual_roundtrip_baseline.rs"]
mod visual_roundtrip_baseline;
