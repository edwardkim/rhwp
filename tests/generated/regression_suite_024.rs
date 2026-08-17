//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_024

#[path = "../anchor_contract.rs"]
mod anchor_contract;

#[path = "../batch_extract_data_contract.rs"]
mod batch_extract_data_contract;

#[path = "../capabilities_schema_contract.rs"]
mod capabilities_schema_contract;

#[path = "../hwp3_password_fixture.rs"]
mod hwp3_password_fixture;

#[path = "../issue_1187_textbox_clip.rs"]
mod issue_1187_textbox_clip;

#[path = "../issue_1385_replace_export_roundtrip.rs"]
mod issue_1385_replace_export_roundtrip;

#[path = "../issue_1510.rs"]
mod issue_1510;

#[path = "../issue_1535.rs"]
mod issue_1535;

#[path = "../issue_1772_table_outer_margin_sync.rs"]
mod issue_1772_table_outer_margin_sync;

#[path = "../issue_1929.rs"]
mod issue_1929;

#[path = "../issue_2020.rs"]
mod issue_2020;

#[path = "../issue_2105_rowbreak_table_declared_fits.rs"]
mod issue_2105_rowbreak_table_declared_fits;

#[path = "../issue_2211_nested_table_row_growth.rs"]
mod issue_2211_nested_table_row_growth;

#[path = "../issue_2319_no_lineseg_tac_table_height.rs"]
mod issue_2319_no_lineseg_tac_table_height;

#[path = "../issue_2439.rs"]
mod issue_2439;

#[path = "../issue_4326_partial_table_nested_row_coords.rs"]
mod issue_4326_partial_table_nested_row_coords;

#[path = "../issue_4889_nested_fragment_origin.rs"]
mod issue_4889_nested_fragment_origin;

#[path = "../signing_contract.rs"]
mod signing_contract;

#[path = "../suites/issue_regression_pilot/issue_2087_document_core_send.rs"]
mod issue_2087_document_core_send;
