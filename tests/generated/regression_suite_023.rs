//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_023

#[path = "../cli_exit_codes_diagnostic_commands.rs"]
mod cli_exit_codes_diagnostic_commands;

#[path = "../cli_exit_codes_dump_diag.rs"]
mod cli_exit_codes_dump_diag;

#[path = "../diag_1042_2022.rs"]
mod diag_1042_2022;

#[path = "../gate_contract.rs"]
mod gate_contract;

#[path = "../issue_1100_exam_social_hwpx_header.rs"]
mod issue_1100_exam_social_hwpx_header;

#[path = "../issue_1156_rowbreak_fragment_fit.rs"]
mod issue_1156_rowbreak_fragment_fit;

#[path = "../issue_1403_pic_shape_caption_roundtrip.rs"]
mod issue_1403_pic_shape_caption_roundtrip;

#[path = "../issue_2093_saved_single_line_spacing_after.rs"]
mod issue_2093_saved_single_line_spacing_after;

#[path = "../issue_2212_nested_cell_path_bbox.rs"]
mod issue_2212_nested_cell_path_bbox;

#[path = "../issue_3315_image_key.rs"]
mod issue_3315_image_key;

#[path = "../issue_3545_clickhere_dirty_roundtrip.rs"]
mod issue_3545_clickhere_dirty_roundtrip;

#[path = "../issue_4668_pic_offset_preserved.rs"]
mod issue_4668_pic_offset_preserved;

#[path = "../issue_630.rs"]
mod issue_630;

#[path = "../issue_702.rs"]
mod issue_702;

#[path = "../issue_centered_cell_vpos_after_tac_shape.rs"]
mod issue_centered_cell_vpos_after_tac_shape;

#[path = "../mcp_session_query_contract.rs"]
mod mcp_session_query_contract;

#[path = "../pr_2219_hml_middle_anchor.rs"]
mod pr_2219_hml_middle_anchor;

#[path = "../redact_sanitize_contract.rs"]
mod redact_sanitize_contract;

#[path = "../render_manifest_json_contract.rs"]
mod render_manifest_json_contract;
