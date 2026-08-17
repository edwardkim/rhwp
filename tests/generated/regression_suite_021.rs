//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_021

#[path = "../diag_1042_cfb_check.rs"]
mod diag_1042_cfb_check;

#[path = "../diag_1042_normal_vs_abnormal.rs"]
mod diag_1042_normal_vs_abnormal;

#[path = "../diag_1042_version_check.rs"]
mod diag_1042_version_check;

#[path = "../edit_fill_fields_contract.rs"]
mod edit_fill_fields_contract;

#[path = "../hml_cli.rs"]
mod hml_cli;

#[path = "../hwp5_strikeout_shape_parity.rs"]
mod hwp5_strikeout_shape_parity;

#[path = "../issue_1142.rs"]
mod issue_1142;

#[path = "../issue_1770_hwpx_origin_marker.rs"]
mod issue_1770_hwpx_origin_marker;

#[path = "../issue_1789_exclusion_probe_line_spacing.rs"]
mod issue_1789_exclusion_probe_line_spacing;

#[path = "../issue_2019_floating_form_overpagination.rs"]
mod issue_2019_floating_form_overpagination;

#[path = "../issue_2299_edit_vpos_reset_preserve.rs"]
mod issue_2299_edit_vpos_reset_preserve;

#[path = "../issue_2318_master_page_plane.rs"]
mod issue_2318_master_page_plane;

#[path = "../issue_3358_ingest_unknown_fields.rs"]
mod issue_3358_ingest_unknown_fields;

#[path = "../issue_3385b_text_surface_full_pua.rs"]
mod issue_3385b_text_surface_full_pua;

#[path = "../issue_3544_hwpx_unsigned_offset.rs"]
mod issue_3544_hwpx_unsigned_offset;

#[path = "../issue_3576_char_shape_dedup.rs"]
mod issue_3576_char_shape_dedup;

#[path = "../issue_4224_pua_f02fb_small_right_triangle.rs"]
mod issue_4224_pua_f02fb_small_right_triangle;

#[path = "../issue_937.rs"]
mod issue_937;

#[path = "../render_diff_json_contract.rs"]
mod render_diff_json_contract;
