//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_025

#[path = "../exam_eng_multicolumn.rs"]
mod exam_eng_multicolumn;

#[path = "../fields_json_contract.rs"]
mod fields_json_contract;

#[path = "../injection_scan_contract.rs"]
mod injection_scan_contract;

#[path = "../issue_1658_page_bottom_fixed_exclusion.rs"]
mod issue_1658_page_bottom_fixed_exclusion;

#[path = "../issue_1771_nested_group_roundtrip.rs"]
mod issue_1771_nested_group_roundtrip;

#[path = "../issue_1914.rs"]
mod issue_1914;

#[path = "../issue_2151_hwp3_ghost_page.rs"]
mod issue_2151_hwp3_ghost_page;

#[path = "../issue_2185_korean_break_unit.rs"]
mod issue_2185_korean_break_unit;

#[path = "../issue_2214_cache_matrix_probe.rs"]
mod issue_2214_cache_matrix_probe;

#[path = "../issue_2222_layer_json_cache.rs"]
mod issue_2222_layer_json_cache;

#[path = "../issue_2230_placeholder_selection.rs"]
mod issue_2230_placeholder_selection;

#[path = "../issue_3492_hwp3_outline_marker_leak.rs"]
mod issue_3492_hwp3_outline_marker_leak;

#[path = "../issue_3565_extract_pages.rs"]
mod issue_3565_extract_pages;

#[path = "../issue_4370_bottom_overflow_reflow.rs"]
mod issue_4370_bottom_overflow_reflow;

#[path = "../issue_530.rs"]
mod issue_530;

#[path = "../mcp_resources_contract.rs"]
mod mcp_resources_contract;

#[path = "../mcp_workspace_contract.rs"]
mod mcp_workspace_contract;

#[path = "../outline_navigation_table_cell_number.rs"]
mod outline_navigation_table_cell_number;

#[path = "../table_extract_json_contract.rs"]
mod table_extract_json_contract;
