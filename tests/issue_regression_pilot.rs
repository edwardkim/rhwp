//! Issue 회귀 테스트의 링크 단위를 줄이는 첫 통합 suite.
//!
//! 각 case는 독립 모듈이므로 함수·helper 이름이 서로 충돌하지 않는다. nextest는
//! 이 바이너리 안의 테스트도 개별 프로세스로 실행하며, 회귀 추적에는 모듈 경로를
//! 사용한다.

#[path = "suites/issue_regression_pilot/issue_1035_alignment.rs"]
mod issue_1035_alignment;
#[path = "suites/issue_regression_pilot/issue_1608_hwpx_native_no_hwp3_tolerance.rs"]
mod issue_1608_hwpx_native_no_hwp3_tolerance;
#[path = "suites/issue_regression_pilot/issue_1611_footer_page_bottom_pagination.rs"]
mod issue_1611_footer_page_bottom_pagination;
#[path = "suites/issue_regression_pilot/issue_1624_footer_overpush_pagination.rs"]
mod issue_1624_footer_overpush_pagination;
#[path = "suites/issue_regression_pilot/issue_1749_saved_bounds_cumulative.rs"]
mod issue_1749_saved_bounds_cumulative;
#[path = "suites/issue_regression_pilot/issue_1750_split_guard_spacing_before.rs"]
mod issue_1750_split_guard_spacing_before;
#[path = "suites/issue_regression_pilot/issue_1768_distribution_doc_save.rs"]
mod issue_1768_distribution_doc_save;
#[path = "suites/issue_regression_pilot/issue_1937_rowbreak_footnote_overpagination.rs"]
mod issue_1937_rowbreak_footnote_overpagination;
#[path = "suites/issue_regression_pilot/issue_2006_1790387_prep_pagination_pin.rs"]
mod issue_2006_1790387_prep_pagination_pin;
#[path = "suites/issue_regression_pilot/issue_2087_document_core_send.rs"]
mod issue_2087_document_core_send;
#[path = "suites/issue_regression_pilot/issue_2093_1192000_real_doc_pin.rs"]
mod issue_2093_1192000_real_doc_pin;
#[path = "suites/issue_regression_pilot/issue_2097_1730000_real_doc_pin.rs"]
mod issue_2097_1730000_real_doc_pin;
#[path = "suites/issue_regression_pilot/issue_2097_3080901_real_doc_pin.rs"]
mod issue_2097_3080901_real_doc_pin;
#[path = "suites/issue_regression_pilot/issue_2097_squeeze.rs"]
mod issue_2097_squeeze;
#[path = "suites/issue_regression_pilot/issue_2098_margin_boundary_split.rs"]
mod issue_2098_margin_boundary_split;
#[path = "suites/issue_regression_pilot/issue_2277_mini_chart_axis.rs"]
mod issue_2277_mini_chart_axis;
#[path = "suites/issue_regression_pilot/issue_2373_tac_host_press_pin.rs"]
mod issue_2373_tac_host_press_pin;
#[path = "suites/issue_regression_pilot/issue_2559_footnote_footer_band.rs"]
mod issue_2559_footnote_footer_band;
#[path = "suites/issue_regression_pilot/issue_3486_hancom_pua_display.rs"]
mod issue_3486_hancom_pua_display;
#[path = "suites/issue_regression_pilot/issue_546.rs"]
mod issue_546;
