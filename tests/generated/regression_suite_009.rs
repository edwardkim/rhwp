//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_009

#[path = "../cli_exit_codes_hwp5_inventory_anchor.rs"]
mod cli_exit_codes_hwp5_inventory_anchor;

#[path = "../diag_1042_hwp_summary.rs"]
mod diag_1042_hwp_summary;

#[path = "../dump_pages_cli.rs"]
mod dump_pages_cli;

#[path = "../edit_fit_check_contract.rs"]
mod edit_fit_check_contract;

#[path = "../ffi_c_surface_contract.rs"]
mod ffi_c_surface_contract;

#[path = "../hwpx_roundtrip_baseline.rs"]
mod hwpx_roundtrip_baseline;

#[path = "../issue_1267_hwpx_tab_and_diagonal.rs"]
mod issue_1267_hwpx_tab_and_diagonal;

#[path = "../issue_1452_saved_caret.rs"]
mod issue_1452_saved_caret;

#[path = "../issue_1639.rs"]
mod issue_1639;

#[path = "../issue_1915_hwp3_pagedef.rs"]
mod issue_1915_hwp3_pagedef;

#[path = "../issue_2189_cell_text_clip.rs"]
mod issue_2189_cell_text_clip;

#[path = "../issue_2813_para_float_stack_anchor_line.rs"]
mod issue_2813_para_float_stack_anchor_line;

#[path = "../issue_301.rs"]
mod issue_301;

#[path = "../issue_3676_hwp3_convert_hancom_openable.rs"]
mod issue_3676_hwp3_convert_hancom_openable;

#[path = "../issue_4490_4491_anchor_flow.rs"]
mod issue_4490_4491_anchor_flow;

#[path = "../issue_929.rs"]
mod issue_929;

#[path = "../mcp_server_contract.rs"]
mod mcp_server_contract;

#[path = "../mcp_tool_annotations_contract.rs"]
mod mcp_tool_annotations_contract;
