# ±1px 회귀검사 함수 목록

- base `ad4bf269ffef5adf2142b33004a9981fad7ce804`, 조사 head `23acf787e3`.
- 테스트 함수 단위로 집계하며 반복 쪽·assertion 수와 구분한다.
- 고정 좌표·크기·영역에 ±1px을 사용하는 함수와 상대 관계만 ±1px로 검사하는 함수는 분리한다.
- Linux에서 시스템 글꼴 경로를 격리한 관련 32개 모듈 89건은 모두 통과했다. 실행 환경과 범위는 README를 참조한다.

| 파일 | 변경 상태 | 고정형 함수 |
| --- | --- | --- |
| `tests/cases/issue_8153_all_pages.rs` | A | `hwp_all_pages_keep_independent_print_anchors`<br>`hwpx_all_pages_keep_independent_print_anchors` |
| `tests/cases/issue_8153_centered_condense_fit.rs` | A | `hwp_centered_condense_keeps_independent_glyph_and_space_advances`<br>`hwpx_centered_condense_keeps_independent_glyph_and_space_advances` |
| `tests/cases/issue_8153_centered_picture_fill.rs` | A | `hwp_centered_cell_picture_keeps_print_clip`<br>`hwpx_centered_cell_picture_keeps_print_clip` |
| `tests/cases/issue_8153_centered_tracking.rs` | A | `hwp_centers_visible_sender_glyphs_without_terminal_tracking`<br>`hwpx_centers_visible_sender_glyphs_without_terminal_tracking` |
| `tests/cases/issue_8153_decorated_tac_pen.rs` | A | `same_saved_tac_rows_keep_their_pen_across_hwp_and_hwpx` |
| `tests/cases/issue_8153_fragment_physical_band.rs` | A | `hwp_fragment_keeps_the_source_blank_band_and_following_content`<br>`hwpx_fragment_keeps_the_source_blank_band_and_following_content` |
| `tests/cases/issue_8153_group_bitmap_clip.rs` | A | `hwp_p131_grouped_picture_print_window`<br>`hwp_p140_grouped_picture_print_window`<br>`hwpx_p131_grouped_picture_print_window`<br>`hwpx_p140_grouped_picture_print_window` |
| `tests/cases/issue_8153_handbook_print_boundaries.rs` | A | `hwp_keeps_independent_print_owners_picture_frames_and_nested_rows`<br>`hwpx_keeps_independent_print_owners_picture_frames_and_nested_rows` |
| `tests/cases/issue_8153_hanja_source_advance.rs` | A | `hwp_original_hanja_keeps_next_style_origin`<br>`hwpx_original_hanja_keeps_next_style_origin` |
| `tests/cases/issue_8153_inline_picture_row_frame.rs` | A | `hwp_inline_picture_and_padding_own_the_same_print_row`<br>`hwpx_inline_picture_and_padding_own_the_same_print_row` |
| `tests/cases/issue_8153_justified_final_line_compression.rs` | A | `hwp_p148_justified_final_line_keeps_the_print_word_and_glyph_contract`<br>`hwp_p327_justified_final_line_keeps_the_print_word_and_glyph_contract`<br>`hwpx_p148_justified_final_line_keeps_the_print_word_and_glyph_contract`<br>`hwpx_p327_justified_final_line_keeps_the_print_word_and_glyph_contract` |
| `tests/cases/issue_8153_justified_visible_last_line.rs` | A | `hwp_plain_terminal_space_keeps_the_fitting_visible_last_line`<br>`hwpx_plain_terminal_space_keeps_the_fitting_visible_last_line` |
| `tests/cases/issue_8153_kopub_missing_dot_supplier.rs` | A | `hwp_missing_kopub_dot_uses_declared_supplier_and_keeps_spaces`<br>`hwpx_missing_kopub_dot_uses_declared_supplier_and_keeps_spaces` |
| `tests/cases/issue_8153_opaque_mixed_axis_image_sampling.rs` | A | `hwp_original_frame_corner_and_bottom_ink_are_preserved`<br>`hwpx_original_frame_corner_and_bottom_ink_are_preserved` |
| `tests/cases/issue_8153_ordinary_row_closing_body.rs` | A | `hwp_paragraph_local_restart_does_not_extend_completed_table_into_following_body`<br>`hwpx_paragraph_local_restart_does_not_extend_completed_table_into_following_body` |
| `tests/cases/issue_8153_ordinary_row_continuation_frame.rs` | A | `hwp_continuing_ordinary_row_keeps_source_cut_and_physical_end`<br>`hwpx_continuing_ordinary_row_keeps_source_cut_and_physical_end` |
| `tests/cases/issue_8153_ordinary_row_opening_frame.rs` | A | `hwp_p316_ordinary_row_opening_owns_physical_frame`<br>`hwp_p326_ordinary_row_opening_owns_physical_frame`<br>`hwp_p344_ordinary_row_opening_owns_physical_frame`<br>`hwp_p352_ordinary_row_opening_owns_physical_frame`<br>`hwpx_p316_ordinary_row_opening_owns_physical_frame`<br>`hwpx_p326_ordinary_row_opening_owns_physical_frame`<br>`hwpx_p344_ordinary_row_opening_owns_physical_frame`<br>`hwpx_p352_ordinary_row_opening_owns_physical_frame` |
| `tests/cases/issue_8153_ordinary_row_physical_remainder.rs` | A | `hwp_ordinary_row_physical_remainder_survives_multiple_content_frames`<br>`hwpx_ordinary_row_physical_remainder_survives_multiple_content_frames` |
| `tests/cases/issue_8153_overlay_picture_line_owner.rs` | A | `hwp_overlay_picture_keeps_its_owned_line`<br>`hwpx_overlay_picture_keeps_its_owned_line` |
| `tests/cases/issue_8153_picture_resource_resampling.rs` | A | `hwp_picture_resource_preserves_both_axes_and_owner`<br>`hwpx_picture_resource_preserves_both_axes_and_owner` |
| `tests/cases/issue_8153_positive_source_tracking.rs` | A | `hwp_terminal_underline_ends_after_the_last_digit`<br>`hwpx_terminal_underline_ends_after_the_last_digit` |
| `tests/cases/issue_8153_positive_word_slack.rs` | A | `hwp_positive_word_slack_preserves_glyphs_and_next_line_content`<br>`hwpx_positive_word_slack_preserves_glyphs_and_next_line_content` |
| `tests/cases/issue_8153_print_alpha_mask.rs` | A | `hwp_print_alpha_preserves_hidden_color_source_owner_and_bounds`<br>`hwpx_print_alpha_preserves_hidden_color_source_owner_and_bounds` |
| `tests/cases/issue_8153_reflowed_inline_shape_spacing.rs` | A | `hwp_reflowed_shape_row_owns_frame_and_signed_spacing`<br>`hwpx_reflowed_shape_row_owns_frame_and_signed_spacing` |
| `tests/cases/issue_8153_row_cell_frame_ownership.rs` | A | `hwp_signature_dates_use_their_source_cell_frame`<br>`hwpx_signature_dates_use_their_source_cell_frame` |
| `tests/cases/issue_8153_textbox_saved_line_frame.rs` | A | `hwp_stored_textbox_uses_its_original_line_frame`<br>`hwpx_stored_textbox_uses_its_original_line_frame` |
| `tests/cases/issue_8160_all_pages.rs` | M | `issue_8160_hwp2022_all_64_pages_preserve_independent_print_content_and_owners`<br>`issue_8160_hwp2024_all_64_pages_preserve_independent_print_content_and_owners`<br>`issue_8160_hwpx_all_64_pages_preserve_independent_print_content_and_owners` |
| `tests/cases/issue_8160_hwp2022_paragraph_relations.rs` | M | `issue_8160_sample16_2022_bcp_keeps_both_authored_rows_and_owned_successor`<br>`issue_8160_sample16_2022_compound_page_border_keeps_both_print_coverage_bands` |
| `tests/cases/issue_8160_saved_page_boundaries.rs` | M | `issue_8160_sample16_hwp5_2022_server_requirements_page_matches_hancom` |
| `tests/cases/issue_8160_stored_shape_host_physical_frame.rs` | M | `issue_8160_hwpx_completed_page_protects_print_heading_and_table_boundaries` |
| `tests/cases/issue_8160_terminal_style_cell_frame.rs` | M | `terminal_style_preserves_printed_cell_baselines_and_source_ownership` |

고정형 함수 69건: 새 파일 61건, 기존 파일 변경 8건.
별도 관계형 함수 2건: `hwp_centered_inline_picture_ignores_trailing_space_width`, `hwpx_centered_inline_picture_ignores_trailing_space_width`.

함수 호출과 수치 표기를 추적하고 잉크 영역 및 허용 오차 인자 사용을 수동 확인했다. 함수별 다수의 assertion과 원본별 반복은 중복 계수하지 않는다.
