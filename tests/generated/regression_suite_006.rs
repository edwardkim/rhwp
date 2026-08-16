//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_006

#[path = "../cli_json_contract.rs"]
mod cli_json_contract;

#[path = "../digest_macro_contract.rs"]
mod digest_macro_contract;

#[path = "../issue_1434_clickhere_guide_hancom_command.rs"]
mod issue_1434_clickhere_guide_hancom_command;

#[path = "../issue_1882_chart_style_gaps.rs"]
mod issue_1882_chart_style_gaps;

#[path = "../issue_2097_none_table_declared_fits.rs"]
mod issue_2097_none_table_declared_fits;

#[path = "../issue_2277_legend_order.rs"]
mod issue_2277_legend_order;

#[path = "../issue_2342_cell_merge_para_meta.rs"]
mod issue_2342_cell_merge_para_meta;

#[path = "../issue_2470_masking_page_pins.rs"]
mod issue_2470_masking_page_pins;

#[path = "../issue_3375_field_guide_print_profile.rs"]
mod issue_3375_field_guide_print_profile;

#[path = "../issue_3546_chart_preserved_on_save.rs"]
mod issue_3546_chart_preserved_on_save;

#[path = "../issue_3592_row_filter_valign.rs"]
mod issue_3592_row_filter_valign;

#[path = "../issue_4090_square_table_left_wrap.rs"]
mod issue_4090_square_table_left_wrap;

#[path = "../issue_493_hwpx_cell_field_name.rs"]
mod issue_493_hwpx_cell_field_name;

#[path = "../issue_598_footnote_marker_nav.rs"]
mod issue_598_footnote_marker_nav;

#[path = "../issue_948.rs"]
mod issue_948;

#[path = "../issue_hwp3_bookmark_native.rs"]
mod issue_hwp3_bookmark_native;

#[path = "../mcp_session_arg_typing_contract.rs"]
mod mcp_session_arg_typing_contract;

#[path = "../suites/issue_regression_pilot/issue_2559_footnote_footer_band.rs"]
mod issue_2559_footnote_footer_band;
