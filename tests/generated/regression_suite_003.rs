//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_003

#[path = "../audit_standard_contract.rs"]
mod audit_standard_contract;

#[path = "../did_you_mean_contract.rs"]
mod did_you_mean_contract;

#[path = "../edit_replace_text_contract.rs"]
mod edit_replace_text_contract;

#[path = "../issue_1073_nested_table_split.rs"]
mod issue_1073_nested_table_split;

#[path = "../issue_1197_svg_object_zorder.rs"]
mod issue_1197_svg_object_zorder;

#[path = "../issue_1431_scatter.rs"]
mod issue_1431_scatter;

#[path = "../issue_1753_deferred_table_fill_ahead.rs"]
mod issue_1753_deferred_table_fill_ahead;

#[path = "../issue_2075_shape_offpage_restrict_loss.rs"]
mod issue_2075_shape_offpage_restrict_loss;

#[path = "../issue_241.rs"]
mod issue_241;

#[path = "../issue_3311_malformed_cfb_no_panic.rs"]
mod issue_3311_malformed_cfb_no_panic;

#[path = "../issue_3595_nested_split_row_identity.rs"]
mod issue_3595_nested_split_row_identity;

#[path = "../issue_3738_rowbreak_table_footnote_fragment.rs"]
mod issue_3738_rowbreak_table_footnote_fragment;

#[path = "../issue_4097_mini_cfb_root_clsid.rs"]
mod issue_4097_mini_cfb_root_clsid;

#[path = "../issue_717_table_cell_hit_test.rs"]
mod issue_717_table_cell_hit_test;

#[path = "../suites/issue_regression_pilot/issue_546.rs"]
mod issue_546;

#[path = "../tab_cross_run.rs"]
mod tab_cross_run;
