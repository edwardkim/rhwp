//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_026

#[path = "../armor_contract.rs"]
mod armor_contract;

#[path = "../doclang_export.rs"]
mod doclang_export;

#[path = "../hwp3_slice_index_oob_no_panic.rs"]
mod hwp3_slice_index_oob_no_panic;

#[path = "../ir_schema_contract.rs"]
mod ir_schema_contract;

#[path = "../issue_1166_landscape.rs"]
mod issue_1166_landscape;

#[path = "../issue_1534_hwpx_form_caption_escape.rs"]
mod issue_1534_hwpx_form_caption_escape;

#[path = "../issue_1562_hwpx_form_caption_display.rs"]
mod issue_1562_hwpx_form_caption_display;

#[path = "../issue_2225_missing_picture_placeholder.rs"]
mod issue_2225_missing_picture_placeholder;

#[path = "../issue_2293_chart_png_text.rs"]
mod issue_2293_chart_png_text;

#[path = "../issue_3380_field_value_equals_guide.rs"]
mod issue_3380_field_value_equals_guide;

#[path = "../issue_3403_split_table_cell_page.rs"]
mod issue_3403_split_table_cell_page;

#[path = "../issue_3528_nested_caption_boundary.rs"]
mod issue_3528_nested_caption_boundary;

#[path = "../issue_3593_cell_para_vpos_anchor.rs"]
mod issue_3593_cell_para_vpos_anchor;

#[path = "../issue_418.rs"]
mod issue_418;

#[path = "../issue_4402_hwp5_guide_residue_roundtrip.rs"]
mod issue_4402_hwp5_guide_residue_roundtrip;

#[path = "../render_p22_web_canvas_contract.rs"]
mod render_p22_web_canvas_contract;

#[path = "../skills_contract.rs"]
mod skills_contract;

#[path = "../suites/issue_regression_pilot/issue_2277_mini_chart_axis.rs"]
mod issue_2277_mini_chart_axis;

#[path = "../table_csv_contract.rs"]
mod table_csv_contract;
