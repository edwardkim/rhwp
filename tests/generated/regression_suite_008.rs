//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_008

#[path = "../batch_fill_contract.rs"]
mod batch_fill_contract;

#[path = "../capabilities_subcommands_contract.rs"]
mod capabilities_subcommands_contract;

#[path = "../issue_1008_gradient.rs"]
mod issue_1008_gradient;

#[path = "../issue_1282_rotated_cell_picture_resize.rs"]
mod issue_1282_rotated_cell_picture_resize;

#[path = "../issue_1285_tac_sequence_right_align.rs"]
mod issue_1285_tac_sequence_right_align;

#[path = "../issue_1389_picture_size_roundtrip.rs"]
mod issue_1389_picture_size_roundtrip;

#[path = "../issue_1488_rowbreak_empty_overlay_pages.rs"]
mod issue_1488_rowbreak_empty_overlay_pages;

#[path = "../issue_1748_rowbreak_straddle_rowspan.rs"]
mod issue_1748_rowbreak_straddle_rowspan;

#[path = "../issue_1835_tac_stale_height.rs"]
mod issue_1835_tac_stale_height;

#[path = "../issue_2158_hwpx_vpos_reset_preserve.rs"]
mod issue_2158_hwpx_vpos_reset_preserve;

#[path = "../issue_2215_selection_page_range.rs"]
mod issue_2215_selection_page_range;

#[path = "../issue_3820_stored_reset_fragment_geometry.rs"]
mod issue_3820_stored_reset_fragment_geometry;

#[path = "../issue_4128_cell_cursor_page_narrowing.rs"]
mod issue_4128_cell_cursor_page_narrowing;

#[path = "../issue_826.rs"]
mod issue_826;

#[path = "../issue_874_ktx_toc_page_number_right_align.rs"]
mod issue_874_ktx_toc_page_number_right_align;

#[path = "../mcp_session_setcell_contract.rs"]
mod mcp_session_setcell_contract;

#[path = "../page_number_propagation.rs"]
mod page_number_propagation;
