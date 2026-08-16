//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_022

#[path = "../batch_parallel_determinism_contract.rs"]
mod batch_parallel_determinism_contract;

#[path = "../center_trailing_ws_alignment.rs"]
mod center_trailing_ws_alignment;

#[path = "../chart_csv_contract.rs"]
mod chart_csv_contract;

#[path = "../explain_contract.rs"]
mod explain_contract;

#[path = "../export_hml_json_contract.rs"]
mod export_hml_json_contract;

#[path = "../hit_test_leading_gap.rs"]
mod hit_test_leading_gap;

#[path = "../issue_1156_chart_column_flow.rs"]
mod issue_1156_chart_column_flow;

#[path = "../issue_1330_bullet_marker_caret_size.rs"]
mod issue_1330_bullet_marker_caret_size;

#[path = "../issue_1375_endnote_rewind_column_overflow.rs"]
mod issue_1375_endnote_rewind_column_overflow;

#[path = "../issue_1950_hwp3_tab_charoffset.rs"]
mod issue_1950_hwp3_tab_charoffset;

#[path = "../issue_2320_vpos_rewind_page_break.rs"]
mod issue_2320_vpos_rewind_page_break;

#[path = "../issue_2428_footnote_fast_reject.rs"]
mod issue_2428_footnote_fast_reject;

#[path = "../issue_4694_chart_list.rs"]
mod issue_4694_chart_list;

#[path = "../issue_775.rs"]
mod issue_775;

#[path = "../issue_938.rs"]
mod issue_938;

#[path = "../issue_insert_para_section_break.rs"]
mod issue_insert_para_section_break;

#[path = "../mcp_session_structure_extract_contract.rs"]
mod mcp_session_structure_extract_contract;

#[path = "../security_corpus_regression.rs"]
mod security_corpus_regression;

#[path = "../suites/issue_regression_pilot/issue_1750_split_guard_spacing_before.rs"]
mod issue_1750_split_guard_spacing_before;
