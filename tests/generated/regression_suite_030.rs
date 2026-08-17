//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_030

#[path = "../agent_profile_router_contract.rs"]
mod agent_profile_router_contract;

#[path = "../field_begin_emission_order.rs"]
mod field_begin_emission_order;

#[path = "../issue_1071_tac_cursor_nav.rs"]
mod issue_1071_tac_cursor_nav;

#[path = "../issue_1113_header_autonum_placeholder.rs"]
mod issue_1113_header_autonum_placeholder;

#[path = "../issue_1692.rs"]
mod issue_1692;

#[path = "../issue_1785_cell_padding_rule_consistency.rs"]
mod issue_1785_cell_padding_rule_consistency;

#[path = "../issue_2063.rs"]
mod issue_2063;

#[path = "../issue_3706_hwp3_convert_file_header_version.rs"]
mod issue_3706_hwp3_convert_file_header_version;

#[path = "../issue_3765_zone_switch_ladder_understates_page.rs"]
mod issue_3765_zone_switch_ladder_understates_page;

#[path = "../issue_3820_rowbreak_rowspan_band.rs"]
mod issue_3820_rowbreak_rowspan_band;

#[path = "../issue_3834_flow_with_text_preserved.rs"]
mod issue_3834_flow_with_text_preserved;

#[path = "../issue_4323_merge_cell_reflow.rs"]
mod issue_4323_merge_cell_reflow;

#[path = "../issue_4488_4495_body_provenance.rs"]
mod issue_4488_4495_body_provenance;

#[path = "../issue_713.rs"]
mod issue_713;

#[path = "../issue_898.rs"]
mod issue_898;

#[path = "../issue_table_vpos_01_page5_cell_hit_test.rs"]
mod issue_table_vpos_01_page5_cell_hit_test;

#[path = "../mcp_session_view_contract.rs"]
mod mcp_session_view_contract;

#[path = "../output_axis_json_contract.rs"]
mod output_axis_json_contract;

#[path = "../render_p37_direct_pdf_export.rs"]
mod render_p37_direct_pdf_export;
