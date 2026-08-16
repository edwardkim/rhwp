//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_014

#[path = "../edit_field_occurrence_contract.rs"]
mod edit_field_occurrence_contract;

#[path = "../hwpx_roundtrip_integration.rs"]
mod hwpx_roundtrip_integration;

#[path = "../ir_diff_summary_mode.rs"]
mod ir_diff_summary_mode;

#[path = "../issue_1019.rs"]
mod issue_1019;

#[path = "../issue_1143.rs"]
mod issue_1143;

#[path = "../issue_1598_ellipse_geometry_roundtrip.rs"]
mod issue_1598_ellipse_geometry_roundtrip;

#[path = "../issue_2207_cell_overlay_picture_anchor.rs"]
mod issue_2207_cell_overlay_picture_anchor;

#[path = "../issue_3315_image_bytes_by_key.rs"]
mod issue_3315_image_bytes_by_key;

#[path = "../issue_3466_autonum_inline_control_order.rs"]
mod issue_3466_autonum_inline_control_order;

#[path = "../issue_3820_body_top_table_border_clip.rs"]
mod issue_3820_body_top_table_border_clip;

#[path = "../issue_4430_export_content_loss.rs"]
mod issue_4430_export_content_loss;

#[path = "../issue_4444_caption_vertical_alignment.rs"]
mod issue_4444_caption_vertical_alignment;

#[path = "../issue_4494_chart_caption_single_owner.rs"]
mod issue_4494_chart_caption_single_owner;

#[path = "../issue_643.rs"]
mod issue_643;

#[path = "../issue_658_text_selection_rects.rs"]
mod issue_658_text_selection_rects;

#[path = "../issue_884_charshape_diagnostic.rs"]
mod issue_884_charshape_diagnostic;

#[path = "../issue_synam001_visible_float_host_line_overlap.rs"]
mod issue_synam001_visible_float_host_line_overlap;

#[path = "../suites/issue_regression_pilot/issue_2093_1192000_real_doc_pin.rs"]
mod issue_2093_1192000_real_doc_pin;
