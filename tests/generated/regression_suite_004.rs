//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_004

#[path = "../audit_contract.rs"]
mod audit_contract;

#[path = "../bundle_contract.rs"]
mod bundle_contract;

#[path = "../diag_1042_raw_binary.rs"]
mod diag_1042_raw_binary;

#[path = "../hml_parser.rs"]
mod hml_parser;

#[path = "../issue_1082_endnote_multicolumn_drift.rs"]
mod issue_1082_endnote_multicolumn_drift;

#[path = "../issue_1172_para_margin_roundtrip.rs"]
mod issue_1172_para_margin_roundtrip;

#[path = "../issue_1880.rs"]
mod issue_1880;

#[path = "../issue_1916_tbl_common_attr.rs"]
mod issue_1916_tbl_common_attr;

#[path = "../issue_2137_topbottom_float_anchor_saved_fit.rs"]
mod issue_2137_topbottom_float_anchor_saved_fit;

#[path = "../issue_2550_bin_data_decompression_bomb.rs"]
mod issue_2550_bin_data_decompression_bomb;

#[path = "../issue_3494_char_count_convention.rs"]
mod issue_3494_char_count_convention;

#[path = "../issue_3495_endnote_space_eaten.rs"]
mod issue_3495_endnote_space_eaten;

#[path = "../issue_3547_ole_size_prefix.rs"]
mod issue_3547_ole_size_prefix;

#[path = "../issue_hml_shape_child_attribution.rs"]
mod issue_hml_shape_child_attribution;

#[path = "../schema_version_registry_contract.rs"]
mod schema_version_registry_contract;

#[path = "../suites/issue_regression_pilot/issue_1611_footer_page_bottom_pagination.rs"]
mod issue_1611_footer_page_bottom_pagination;

#[path = "../suites/issue_regression_pilot/issue_1749_saved_bounds_cumulative.rs"]
mod issue_1749_saved_bounds_cumulative;
