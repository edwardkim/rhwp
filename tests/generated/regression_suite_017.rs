//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_017

#[path = "../edit_render_diff_gate.rs"]
mod edit_render_diff_gate;

#[path = "../envelope_integrity_contract.rs"]
mod envelope_integrity_contract;

#[path = "../hwpx_form_roundtrip.rs"]
mod hwpx_form_roundtrip;

#[path = "../insert_image_contract.rs"]
mod insert_image_contract;

#[path = "../issue_1058_textbox_list_header.rs"]
mod issue_1058_textbox_list_header;

#[path = "../issue_1196_hwpx_gutter_left_right.rs"]
mod issue_1196_hwpx_gutter_left_right;

#[path = "../issue_1308_forced_break_hanging_indent.rs"]
mod issue_1308_forced_break_hanging_indent;

#[path = "../issue_1858_bottom_anchor_flush.rs"]
mod issue_1858_bottom_anchor_flush;

#[path = "../issue_1893.rs"]
mod issue_1893;

#[path = "../issue_2015_saved_bounds_rowbreak.rs"]
mod issue_2015_saved_bounds_rowbreak;

#[path = "../issue_3349_export_text_option_order.rs"]
mod issue_3349_export_text_option_order;

#[path = "../issue_3357_capabilities_feature_gate.rs"]
mod issue_3357_capabilities_feature_gate;

#[path = "../issue_3707_hwp3_roundtrip_endnote_columns.rs"]
mod issue_3707_hwp3_roundtrip_endnote_columns;

#[path = "../issue_3739_hwpx_same_char_shape_boundary.rs"]
mod issue_3739_hwpx_same_char_shape_boundary;

#[path = "../issue_4031_enter_latency_probe.rs"]
mod issue_4031_enter_latency_probe;

#[path = "../issue_4397_ruby_hwp5_roundtrip.rs"]
mod issue_4397_ruby_hwp5_roundtrip;

#[path = "../mcp_split_page_base_contract.rs"]
mod mcp_split_page_base_contract;

#[path = "../public_rust_api_vec_compat.rs"]
mod public_rust_api_vec_compat;

#[path = "../suites/issue_regression_pilot/issue_2097_squeeze.rs"]
mod issue_2097_squeeze;
