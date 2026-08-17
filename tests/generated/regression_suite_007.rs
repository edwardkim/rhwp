//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_007

#[path = "../hwp3_page_number_pos_fixture.rs"]
mod hwp3_page_number_pos_fixture;

#[path = "../issue_1198_nested_cell_paste.rs"]
mod issue_1198_nested_cell_paste;

#[path = "../issue_1402_enum_token_whitelist.rs"]
mod issue_1402_enum_token_whitelist;

#[path = "../issue_1663.rs"]
mod issue_1663;

#[path = "../issue_1949_giant_cell_render_perf.rs"]
mod issue_1949_giant_cell_render_perf;

#[path = "../issue_2004_cell_image_stack_pagination.rs"]
mod issue_2004_cell_image_stack_pagination;

#[path = "../issue_2027_picture_wrap_toggle_loss.rs"]
mod issue_2027_picture_wrap_toggle_loss;

#[path = "../issue_2279_layout_oracles.rs"]
mod issue_2279_layout_oracles;

#[path = "../issue_2722_table_grid_alloc.rs"]
mod issue_2722_table_grid_alloc;

#[path = "../issue_3385_text_surface_pua.rs"]
mod issue_3385_text_surface_pua;

#[path = "../issue_3565_container_child_ctrl_id.rs"]
mod issue_3565_container_child_ctrl_id;

#[path = "../issue_516.rs"]
mod issue_516;

#[path = "../issue_716.rs"]
mod issue_716;

#[path = "../issue_850_answer_sheet_name_hit_test.rs"]
mod issue_850_answer_sheet_name_hit_test;

#[path = "../mcp_session_changed_pages_contract.rs"]
mod mcp_session_changed_pages_contract;

#[path = "../plan_schema_contract.rs"]
mod plan_schema_contract;

#[path = "../suites/issue_regression_pilot/issue_2373_tac_host_press_pin.rs"]
mod issue_2373_tac_host_press_pin;

#[path = "../wmf_flow_image_emitted_as_svg.rs"]
mod wmf_flow_image_emitted_as_svg;
