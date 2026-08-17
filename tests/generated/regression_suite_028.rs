//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_028

#[path = "../diag_1042_trailing.rs"]
mod diag_1042_trailing;

#[path = "../issue_1067_shape_rotation.rs"]
mod issue_1067_shape_rotation;

#[path = "../issue_1105.rs"]
mod issue_1105;

#[path = "../issue_1585_caption_floating_image.rs"]
mod issue_1585_caption_floating_image;

#[path = "../issue_1695.rs"]
mod issue_1695;

#[path = "../issue_1733.rs"]
mod issue_1733;

#[path = "../issue_1749_saved_bounds_page_break.rs"]
mod issue_1749_saved_bounds_page_break;

#[path = "../issue_2129_line_stacked.rs"]
mod issue_2129_line_stacked;

#[path = "../issue_2292_chart_png_clip.rs"]
mod issue_2292_chart_png_clip;

#[path = "../issue_2727_equation_line_mode.rs"]
mod issue_2727_equation_line_mode;

#[path = "../issue_3372_gian_form_contract.rs"]
mod issue_3372_gian_form_contract;

#[path = "../issue_3798_page_end_trailing_spill.rs"]
mod issue_3798_page_end_trailing_spill;

#[path = "../issue_4441_ir_sweep_stable_cap.rs"]
mod issue_4441_ir_sweep_stable_cap;

#[path = "../issue_4645_font_lookup_boundary.rs"]
mod issue_4645_font_lookup_boundary;

#[path = "../issue_505.rs"]
mod issue_505;

#[path = "../pr_1136_cell_paragraph_numbering.rs"]
mod pr_1136_cell_paragraph_numbering;

#[path = "../run_plan_cas_contract.rs"]
mod run_plan_cas_contract;

#[path = "../search_dash_query_contract.rs"]
mod search_dash_query_contract;

#[path = "../suites/issue_regression_pilot/issue_1937_rowbreak_footnote_overpagination.rs"]
mod issue_1937_rowbreak_footnote_overpagination;
