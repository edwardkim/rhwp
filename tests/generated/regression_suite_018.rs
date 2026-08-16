//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_018

#[path = "../digest_v2_contract.rs"]
mod digest_v2_contract;

#[path = "../issue_1391_memo_field_roundtrip.rs"]
mod issue_1391_memo_field_roundtrip;

#[path = "../issue_2525_single_lineseg_rewrap.rs"]
mod issue_2525_single_lineseg_rewrap;

#[path = "../issue_2743_hml_resource_id_limit.rs"]
mod issue_2743_hml_resource_id_limit;

#[path = "../issue_3366_thumbnail_contract.rs"]
mod issue_3366_thumbnail_contract;

#[path = "../issue_3637_split_cell_nested_table_vpos.rs"]
mod issue_3637_split_cell_nested_table_vpos;

#[path = "../issue_3915_verify_axes_both_reported.rs"]
mod issue_3915_verify_axes_both_reported;

#[path = "../issue_4179_cursor_rect_text_host_para_pages.rs"]
mod issue_4179_cursor_rect_text_host_para_pages;

#[path = "../issue_703.rs"]
mod issue_703;

#[path = "../issue_838_field_set_value.rs"]
mod issue_838_field_set_value;

#[path = "../issue_915_charshape_cell_font_size.rs"]
mod issue_915_charshape_cell_font_size;

#[path = "../issue_rowbreak_chart_overlap.rs"]
mod issue_rowbreak_chart_overlap;

#[path = "../nextcall_cli_contract.rs"]
mod nextcall_cli_contract;

#[path = "../opengov_corpus_snapshot.rs"]
mod opengov_corpus_snapshot;

#[path = "../password_encryption_write_contract.rs"]
mod password_encryption_write_contract;

#[path = "../replay_contract.rs"]
mod replay_contract;

#[path = "../settle_contract.rs"]
mod settle_contract;

#[path = "../wmf_text_negative_string_length_no_panic.rs"]
mod wmf_text_negative_string_length_no_panic;
