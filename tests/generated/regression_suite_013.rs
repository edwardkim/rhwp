//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_013

#[path = "../boundary_integrity_contract.rs"]
mod boundary_integrity_contract;

#[path = "../ir_diff_json_contract.rs"]
mod ir_diff_json_contract;

#[path = "../issue_1418_textbox_table_overlap.rs"]
mod issue_1418_textbox_table_overlap;

#[path = "../issue_1686.rs"]
mod issue_1686;

#[path = "../issue_2287_edu_rowspan_block_fragments.rs"]
mod issue_2287_edu_rowspan_block_fragments;

#[path = "../issue_2308_render_normalized_derived_state.rs"]
mod issue_2308_render_normalized_derived_state;

#[path = "../issue_2311_attachment_poster_intra_para_reset.rs"]
mod issue_2311_attachment_poster_intra_para_reset;

#[path = "../issue_2403_provenance_stage1.rs"]
mod issue_2403_provenance_stage1;

#[path = "../issue_3234_active_hf_specificity.rs"]
mod issue_3234_active_hf_specificity;

#[path = "../issue_3315_image_base64_round_trip.rs"]
mod issue_3315_image_base64_round_trip;

#[path = "../issue_3637_nested_table_starts_inside_parent_cell.rs"]
mod issue_3637_nested_table_starts_inside_parent_cell;

#[path = "../issue_4126_cursor_rect_empty_para_pages.rs"]
mod issue_4126_cursor_rect_empty_para_pages;

#[path = "../issue_4396_field_parameters_hwp5_roundtrip.rs"]
mod issue_4396_field_parameters_hwp5_roundtrip;

#[path = "../issue_4896_field_type_identity.rs"]
mod issue_4896_field_type_identity;

#[path = "../issue_501.rs"]
mod issue_501;

#[path = "../issue_554.rs"]
mod issue_554;

#[path = "../suites/issue_regression_pilot/issue_1768_distribution_doc_save.rs"]
mod issue_1768_distribution_doc_save;

#[path = "../wmf_poly_negative_point_count_no_panic.rs"]
mod wmf_poly_negative_point_count_no_panic;
