//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_020

#[path = "../batch_axes_contract.rs"]
mod batch_axes_contract;

#[path = "../diag_1042_hwp3_vs_hwp5_paragraph.rs"]
mod diag_1042_hwp3_vs_hwp5_paragraph;

#[path = "../diag_1042_variant_check.rs"]
mod diag_1042_variant_check;

#[path = "../edit_verify_contract.rs"]
mod edit_verify_contract;

#[path = "../issue_1050_footnote_serialize.rs"]
mod issue_1050_footnote_serialize;

#[path = "../issue_1244_tab_extended_fallback.rs"]
mod issue_1244_tab_extended_fallback;

#[path = "../issue_1853.rs"]
mod issue_1853;

#[path = "../issue_2148_degenerate_cell_vpos.rs"]
mod issue_2148_degenerate_cell_vpos;

#[path = "../issue_2308_render_normalized_guard.rs"]
mod issue_2308_render_normalized_guard;

#[path = "../issue_3189_hml_cell_vertical_align.rs"]
mod issue_3189_hml_cell_vertical_align;

#[path = "../issue_3315_flow_image_narrow_query.rs"]
mod issue_3315_flow_image_narrow_query;

#[path = "../issue_3570_record_padding.rs"]
mod issue_3570_record_padding;

#[path = "../issue_3695_structure_auto_policy.rs"]
mod issue_3695_structure_auto_policy;

#[path = "../issue_3738_tac_sibling_shape_line_advance.rs"]
mod issue_3738_tac_sibling_shape_line_advance;

#[path = "../issue_4129_mixed_nested_scan_budget.rs"]
mod issue_4129_mixed_nested_scan_budget;

#[path = "../issue_4493_docinfo_provenance.rs"]
mod issue_4493_docinfo_provenance;

#[path = "../issue_4514_overlay_table_flow.rs"]
mod issue_4514_overlay_table_flow;

#[path = "../issue_4895_soft_hyphen_literal.rs"]
mod issue_4895_soft_hyphen_literal;

#[path = "../mcp_password_contract.rs"]
mod mcp_password_contract;
