//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_005

#[path = "../convert_verify_corpus_ratchet.rs"]
mod convert_verify_corpus_ratchet;

#[path = "../diag_1042_table_row_height.rs"]
mod diag_1042_table_row_height;

#[path = "../hml_serializer.rs"]
mod hml_serializer;

#[path = "../issue_1017.rs"]
mod issue_1017;

#[path = "../issue_1079_picture_pushdown_vpos.rs"]
mod issue_1079_picture_pushdown_vpos;

#[path = "../issue_1167_svg_behindtext_zorder.rs"]
mod issue_1167_svg_behindtext_zorder;

#[path = "../issue_1858.rs"]
mod issue_1858;

#[path = "../issue_1892.rs"]
mod issue_1892;

#[path = "../issue_1921_59043_pagination_pin.rs"]
mod issue_1921_59043_pagination_pin;

#[path = "../issue_1951_table_cell_cursor_clip.rs"]
mod issue_1951_table_cell_cursor_clip;

#[path = "../issue_2740_para_text_space_growth.rs"]
mod issue_2740_para_text_space_growth;

#[path = "../issue_2833_hml_adapter_row_sizes.rs"]
mod issue_2833_hml_adapter_row_sizes;

#[path = "../issue_3353_search_limit_truncation.rs"]
mod issue_3353_search_limit_truncation;

#[path = "../issue_3359_export_family_option_order.rs"]
mod issue_3359_export_family_option_order;

#[path = "../issue_3413_equation_text_extraction.rs"]
mod issue_3413_equation_text_extraction;

#[path = "../issue_header_picture_tac_line_seg_migration.rs"]
mod issue_header_picture_tac_line_seg_migration;

#[path = "../suites/issue_regression_pilot/issue_2098_margin_boundary_split.rs"]
mod issue_2098_margin_boundary_split;
