//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_031

#[path = "../diag_1042_2024_reverse.rs"]
mod diag_1042_2024_reverse;

#[path = "../diagnostics_flag_contract.rs"]
mod diagnostics_flag_contract;

#[path = "../hwpx_password_fixture.rs"]
mod hwpx_password_fixture;

#[path = "../issue_1133_nested_table_valign.rs"]
mod issue_1133_nested_table_valign;

#[path = "../issue_1270_caption_inline_image.rs"]
mod issue_1270_caption_inline_image;

#[path = "../issue_1916.rs"]
mod issue_1916;

#[path = "../issue_1917.rs"]
mod issue_1917;

#[path = "../issue_2069_ole_object_selection.rs"]
mod issue_2069_ole_object_selection;

#[path = "../issue_2243.rs"]
mod issue_2243;

#[path = "../issue_2322_fullpage_form_table_pair.rs"]
mod issue_2322_fullpage_form_table_pair;

#[path = "../issue_3206_hf_edit_target.rs"]
mod issue_3206_hf_edit_target;

#[path = "../issue_3552_table_common_attr_save.rs"]
mod issue_3552_table_common_attr_save;

#[path = "../issue_4438_table_ctrl_data_item_ids.rs"]
mod issue_4438_table_ctrl_data_item_ids;

#[path = "../issue_4657_distribute_alignment.rs"]
mod issue_4657_distribute_alignment;

#[path = "../issue_919_textbox_hit_test.rs"]
mod issue_919_textbox_hit_test;

#[path = "../knowledge_map_field_dictionary_contract.rs"]
mod knowledge_map_field_dictionary_contract;

#[path = "../render_p23_pdf_export_contract.rs"]
mod render_p23_pdf_export_contract;

#[path = "../suites/issue_regression_pilot/issue_2097_3080901_real_doc_pin.rs"]
mod issue_2097_3080901_real_doc_pin;

#[path = "../threat_scan_contract.rs"]
mod threat_scan_contract;
