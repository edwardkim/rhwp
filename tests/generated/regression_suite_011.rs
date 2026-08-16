//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_011

#[path = "../edit_set_cell_contract.rs"]
mod edit_set_cell_contract;

#[path = "../issue_1028_hwpx_textbox_vertical.rs"]
mod issue_1028_hwpx_textbox_vertical;

#[path = "../issue_1271_hwpx_behind_text_table.rs"]
mod issue_1271_hwpx_behind_text_table;

#[path = "../issue_1355_endnote_title_gap_double.rs"]
mod issue_1355_endnote_title_gap_double;

#[path = "../issue_1842.rs"]
mod issue_1842;

#[path = "../issue_1994_behindtext_table_overlap.rs"]
mod issue_1994_behindtext_table_overlap;

#[path = "../issue_2097_band_fill.rs"]
mod issue_2097_band_fill;

#[path = "../issue_2387_prvtext_supplement.rs"]
mod issue_2387_prvtext_supplement;

#[path = "../issue_2524_embedded_font_svg.rs"]
mod issue_2524_embedded_font_svg;

#[path = "../issue_3507_sectiondef_ctrl_data.rs"]
mod issue_3507_sectiondef_ctrl_data;

#[path = "../issue_3693_structure_clause_context.rs"]
mod issue_3693_structure_clause_context;

#[path = "../issue_4388_arc_type_roundtrip.rs"]
mod issue_4388_arc_type_roundtrip;

#[path = "../issue_493_cell_attrs.rs"]
mod issue_493_cell_attrs;

#[path = "../issue_nested_table_border.rs"]
mod issue_nested_table_border;

#[path = "../mcp_result_cursor_contract.rs"]
mod mcp_result_cursor_contract;

#[path = "../run_plan_journal_hash_chain_contract.rs"]
mod run_plan_journal_hash_chain_contract;

#[path = "../suites/issue_regression_pilot/issue_1608_hwpx_native_no_hwp3_tolerance.rs"]
mod issue_1608_hwpx_native_no_hwp3_tolerance;

#[path = "../svg_snapshot.rs"]
mod svg_snapshot;
