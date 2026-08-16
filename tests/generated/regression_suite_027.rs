//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_027

#[path = "../cell_square_picture_anchor.rs"]
mod cell_square_picture_anchor;

#[path = "../hwp5_password_fixture.rs"]
mod hwp5_password_fixture;

#[path = "../issue_1116.rs"]
mod issue_1116;

#[path = "../issue_1161_copy_picture_in_cell.rs"]
mod issue_1161_copy_picture_in_cell;

#[path = "../issue_1171_textbox_picture_cellpath.rs"]
mod issue_1171_textbox_picture_cellpath;

#[path = "../issue_1417_pagination_cursor_render.rs"]
mod issue_1417_pagination_cursor_render;

#[path = "../issue_1638_convert_verify_gate.rs"]
mod issue_1638_convert_verify_gate;

#[path = "../issue_1755_host_heading_pre_emit.rs"]
mod issue_1755_host_heading_pre_emit;

#[path = "../issue_1891.rs"]
mod issue_1891;

#[path = "../issue_2226_cell_flow_pictures_overlap.rs"]
mod issue_2226_cell_flow_pictures_overlap;

#[path = "../issue_2424_pagination_subphase_probe.rs"]
mod issue_2424_pagination_subphase_probe;

#[path = "../issue_3738_hwp_caption_cell_alignment.rs"]
mod issue_3738_hwp_caption_cell_alignment;

#[path = "../issue_4138_split_cell_stale_linesegs.rs"]
mod issue_4138_split_cell_stale_linesegs;

#[path = "../issue_4252_nested_partial_table_cell_path.rs"]
mod issue_4252_nested_partial_table_cell_path;

#[path = "../issue_514.rs"]
mod issue_514;

#[path = "../mcp_session_edit_contract.rs"]
mod mcp_session_edit_contract;

#[path = "../schema_registry_contract.rs"]
mod schema_registry_contract;

#[path = "../unicode_deception_contract.rs"]
mod unicode_deception_contract;
