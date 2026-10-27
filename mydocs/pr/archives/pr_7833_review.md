# PR #7833 리뷰 — Fix inline object positions and boundary cut/paste in cells

## 최종 판정

**메인터너 보정 후 수용 가능 — 표 셀 경계의 인라인 개체·오려 붙이기와 추적 주소 범위의 로컬 검증 완료.**

원 contributor head의 직접 merge 판정과 메인터너 보정한 통합 후보의 판정을 구분한다. 원격 CI·merge·후속 처리는 실제 수행 후 별도로 기록한다.

## 접수 정보와 적용 계보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#7833](https://github.com/edwardkim/rhwp/pull/7833) / semanticist21 |
| 원 head | `255413cdbc74d34f7c1d1be09fb7ad640eac6feb` |
| 통합 브랜치 | `review/semanticist21-20261019` |
| 검증 코드·테스트 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 고정 upstream/devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 원격 조회 시 상태 | 2026-10-27 조회에서 원 head 불변·open·reviewer jangster77 확인. 원 Draft 여부와 통합 후보 판정은 별개 |

| 원 커밋 | 현재 통합의 대응 |
| --- | --- |
| `cbb2d3ee9b702027a8101526fbd2b5f9152603c8` | `d8a31ee40ce242b11ce7671f3e20f2e4230c37f8` |
| `3ae9e3b71ea08fdd1ae06a02aab33b7c31b22ad5` | `d663deb9610da1dcc159d4cbfdd164c1d730e5ba` |
| `a0c72f3cb85347958194a88d0fec4a81f28155ba` | `14e4481d7a92922c48ba1422d9322511c8681faf` |
| `efec7825d325beb5f2eff65ac921f148818367a5` | `18fd8c2eb16964fd2582f88c3b405d238290907c` |
| `c03d1f98ed5b19c091c7028e370089dc6a3dfbc3` | `953d6fbd9af4e50c404f63bf08b06b049dfda836` |
| `79b7dd66f20f89787521175762879f73df478674` | `e7b18279439568ba2020374185ac7e136b953111` |
| `255413cdbc74d34f7c1d1be09fb7ad640eac6feb` | base 동등 구현 `552f5228d2dbfd6046c838f14241704f1bcb8a05` 포함; 현재 강화 `25744c22a0cdf7c48c8724c82bbe55c2b095d28c`와 대조 |

## 구현 주장·공통 소비 경로·반례

cell boundary cut은 문자열만 지우지 않고 첫/끝 문단의 개체 슬롯과 서식을 같은 선택 범위로 제거한다. flat/nested 주소와 source UTF-16·logical 오프셋을 분리하고 Field·range tag·markpen의 미지원 경계는 변경 전에 거절한다. 빈 범위가 끝 개체를 지우지 않는 대조군을 유지한다. 글꼴 trace는 CharOverlap 뒤 확정할 수 없는 원문 주소를 null/unavailable로 내보내며, WASM 검사의 독립 CFB 원문 경계 fixture가 임의 scalar 주소를 주장하지 못하게 한다. 마지막 source의 trace 계약은 새 base에 포함된 동등 구현과 현재 강화 검사를 대조했다. 동등 포함 커밋은 아래 계보 표에 구분한다.

## 원 PR의 실제 회귀 실행

아래 함수와 실제 최종 PASS 행을 대조했다. 계약 검사 통과와 독립 Print의 시각 일치를 구분한다.

- [tests/cases/cell_boundary_object_cut.rs](../../../tests/cases/cell_boundary_object_cut.rs): SHA-256 `409364a7b7993261f42e3c89eee92de12e5e886188decd8545746bf2e406fde7`.
  - `flat_cell_cut_removes_boundary_pictures_and_tables`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8079`.
  - `path_cell_cut_removes_boundary_pictures_and_tables_at_each_depth`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8087`.
  - `same_paragraph_cell_cut_removes_only_selected_object`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8084`.
  - `invalid_cell_ranges_keep_document_and_clipboard`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8081`.
  - `cell_boundary_with_field_is_rejected_before_mutation`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8074`.
  - `empty_cell_ranges_keep_tail_objects_raw_stream_and_events`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8078`.
  - `cell_boundary_range_tags_are_rejected_before_mutation`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8086`.
  - `cell_boundary_markpen_marks_are_rejected_before_mutation`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8077`.
  - `metadata_without_boundary_objects_keeps_existing_deletion_paths`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8082`.
  - `styled_inline_objects_keep_first_and_last_cell_paragraph_logical_offsets`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8102`.
  - `styled_inline_objects_keep_body_run_logical_offsets`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8085`.
  - `styled_inline_objects_keep_logical_offsets_across_line_breaks`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8088`.
  - `styled_inline_objects_keep_prior_slots_on_continuation_pages`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8089`.
  - `cell_cut_paste_restores_text_offsets_after_inline_and_floating_objects`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8091`.
  - `cell_single_control_paste_keeps_same_anchor_objects_before_the_scalar_offset`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8076`.
  - `cell_text_paste_keeps_scalar_fast_path_and_multiline_split_after_inline_objects`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8083`.
  - `flat_table_caption_text_delete_keeps_legacy_address`: PASS, `rhwp::regression_suite_015`, 로컬 `logs/coherent-full-nextest-4697.log:8080`.
- [tests/cases/hwpx_cell_frame_print.rs](../../../tests/cases/hwpx_cell_frame_print.rs): SHA-256 `dfa0af0404fec42ae4c4de18bab429d3f81fc060d876f8a00486f3396d5143e6`.
  - `original_cached_cells_keep_nearest_owner_source_order_and_print_baseline`: PASS, `rhwp::regression_suite_021`, 로컬 `logs/coherent-full-nextest-4697.log:9783`.
  - `implicit_style_inline_tables_keep_print_baseline_and_signed_advance`: PASS, `rhwp::regression_suite_021`, 로컬 `logs/coherent-full-nextest-4697.log:9779`.
- [tests/cases/inline_cell_physical_width_print.rs](../../../tests/cases/inline_cell_physical_width_print.rs): SHA-256 `3f76a1d68cde30c4c57e7bbc552e9e765ee3e10c15e184f85bb3ebcb0b507872`.
  - `printed_inline_cells_keep_padding_content_and_independent_grid_edges`: PASS, `rhwp::regression_suite_001`, 로컬 `logs/coherent-full-nextest-4697.log:4051`.
- [tests/cases/issue_4961_font_decision_trace.rs](../../../tests/cases/issue_4961_font_decision_trace.rs): SHA-256 `289815610b16ab6b0e99bd71f158f5383de79772c20111587581e91d378fa17f`.
  - `font_trace_profiles_follow_source_ownership_and_preserve_font_decisions`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8589`.
  - `stage4_public_trace_limit_and_unsupported_backends_fail_closed`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8407`.
  - `public_fixture_trace_is_bounded_and_deterministic`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8406`.
  - `issue4967_combined_evidence_uses_one_tree_and_preserves_trace`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8403`.
  - `issue4967_combined_evidence_reports_actual_cache_key_rejection`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8402`.
  - `standalone_native_trace_requires_a_prepared_renderer_snapshot`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-native-font-trace-4697.log:10`.
  - `prepared_native_renderer_snapshot_preserves_custom_font_inventory`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-native-font-trace-4697.log:18`.
  - `trace_options_fail_closed`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8409`.
  - `logical_tac_runs_join_exact_scalar_source_in_body_and_nested_cells`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8404`.
  - `synthetic_overlap_cannot_claim_repeated_source_and_real_pua_keeps_source`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8408`.
  - `note_markers_are_transparent_to_body_and_nested_source_glyph_addresses`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8413`.
  - `coanchored_logical_copy_preserves_endpoint_inclusion_and_source_in_both_formats`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8400`.
- [tests/issue_1452_saved_caret.rs](../../../tests/issue_1452_saved_caret.rs): SHA-256 `4a66d919b6da3938fd4654b1ed48df97acac6bb1fc1f480dab1e2a90a805c148`.
  - `transparency_sample_restores_saved_caret_after_second_inline_picture`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8684`.
  - `enter_before_second_tac_picture_splits_inline_controls`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8679`.
  - `repeated_enter_before_second_tac_picture_keeps_picture_after_blank_lines`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8683`.
  - `repeated_enter_after_second_tac_picture_appends_blank_lines`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8681`.
  - `arrow_up_from_second_tac_picture_end_moves_to_first_picture_end`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8677`.
  - `clicking_after_second_tac_picture_hits_picture_end_position`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8678`.
  - `short_inline_picture_hits_follow_styled_empty_carets_in_both_formats`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8689`.
  - `arrow_up_after_enter_before_first_tac_picture_moves_to_previous_picture_end`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8676`.
  - `transparency_test_sample_text_line_boundaries_keep_visual_affinity`: PASS, `rhwp::regression_suite_017`, 로컬 `logs/coherent-full-nextest-4697.log:8685`.

## 독립 Print·전쪽 TSV·대표 페이지

이 PR의 기존 검증 입력은 최종 조건 번호 133, 134, 135, 136, 137, 138, 139, 140, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169에 연결한다. 다른 PR의 평균 점수로 이 범위를 대신하지 않는다.

| 조건 | 전체 쪽수 | Native 최저 raw·전체 TSV | fresh WASM 최저 raw·전체 TSV |
| --- | ---: | --- | --- |
| 133 | 1 | [100.00000](../assets/s21_20261027/n/c/133.tsv) | [100.00000](../assets/s21_20261027/w/c/133.tsv) |
| 134 | 1 | [100.00000](../assets/s21_20261027/n/c/134.tsv) | [100.00000](../assets/s21_20261027/w/c/134.tsv) |
| 135 | 1 | [100.00000](../assets/s21_20261027/n/c/135.tsv) | [100.00000](../assets/s21_20261027/w/c/135.tsv) |
| 136 | 1 | [100.00000](../assets/s21_20261027/n/c/136.tsv) | [100.00000](../assets/s21_20261027/w/c/136.tsv) |
| 137 | 1 | [100.00000](../assets/s21_20261027/n/c/137.tsv) | [100.00000](../assets/s21_20261027/w/c/137.tsv) |
| 138 | 1 | [100.00000](../assets/s21_20261027/n/c/138.tsv) | [100.00000](../assets/s21_20261027/w/c/138.tsv) |
| 139 | 1 | [100.00000](../assets/s21_20261027/n/c/139.tsv) | [100.00000](../assets/s21_20261027/w/c/139.tsv) |
| 140 | 1 | [100.00000](../assets/s21_20261027/n/c/140.tsv) | [100.00000](../assets/s21_20261027/w/c/140.tsv) |
| 154 | 1 | [94.13019](../assets/s21_20261027/n/c/154.tsv) | [94.09928](../assets/s21_20261027/w/c/154.tsv) |
| 155 | 1 | [97.26027](../assets/s21_20261027/n/c/155.tsv) | [97.27333](../assets/s21_20261027/w/c/155.tsv) |
| 156 | 1 | [100.00000](../assets/s21_20261027/n/c/156.tsv) | [100.00000](../assets/s21_20261027/w/c/156.tsv) |
| 157 | 1 | [100.00000](../assets/s21_20261027/n/c/157.tsv) | [100.00000](../assets/s21_20261027/w/c/157.tsv) |
| 158 | 1 | [100.00000](../assets/s21_20261027/n/c/158.tsv) | [100.00000](../assets/s21_20261027/w/c/158.tsv) |
| 159 | 1 | [100.00000](../assets/s21_20261027/n/c/159.tsv) | [100.00000](../assets/s21_20261027/w/c/159.tsv) |
| 160 | 1 | [100.00000](../assets/s21_20261027/n/c/160.tsv) | [100.00000](../assets/s21_20261027/w/c/160.tsv) |
| 161 | 1 | [100.00000](../assets/s21_20261027/n/c/161.tsv) | [100.00000](../assets/s21_20261027/w/c/161.tsv) |
| 162 | 1 | [100.00000](../assets/s21_20261027/n/c/162.tsv) | [100.00000](../assets/s21_20261027/w/c/162.tsv) |
| 163 | 1 | [100.00000](../assets/s21_20261027/n/c/163.tsv) | [100.00000](../assets/s21_20261027/w/c/163.tsv) |
| 164 | 1 | [100.00000](../assets/s21_20261027/n/c/164.tsv) | [100.00000](../assets/s21_20261027/w/c/164.tsv) |
| 165 | 1 | [100.00000](../assets/s21_20261027/n/c/165.tsv) | [100.00000](../assets/s21_20261027/w/c/165.tsv) |
| 166 | 1 | [100.00000](../assets/s21_20261027/n/c/166.tsv) | [100.00000](../assets/s21_20261027/w/c/166.tsv) |
| 167 | 1 | [100.00000](../assets/s21_20261027/n/c/167.tsv) | [100.00000](../assets/s21_20261027/w/c/167.tsv) |
| 168 | 1 | [100.00000](../assets/s21_20261027/n/c/168.tsv) | [100.00000](../assets/s21_20261027/w/c/168.tsv) |
| 169 | 1 | [100.00000](../assets/s21_20261027/n/c/169.tsv) | [100.00000](../assets/s21_20261027/w/c/169.tsv) |

[입력·Print 경로와 SHA, producer·DPI·글꼴, base TSV 및 대표 review·overlay](../assets/s21_20261027/README.md)를 각 조건 번호로 대조한다. raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 실제 빈 페이지의 raw 100%는 가시 내용의 정확성 증거로 사용하지 않고, 원본·독립 Print의 쪽수와 소유 계약을 별도로 검사한다.

신규 검사는 문단·줄·개체의 소유·순서·누락·중복·포함·겹침 관계를 기본으로 한다. 독립 Print와 실제 판독을 마치고 Native/fresh WASM이 모두 98% 이상인 완료 페이지만 독립 좌표 ±2px를 허용하며 관계 검사도 유지한다. 원본 HU·크기·비율 계약과 임의 화면 좌표 핀을 구분한다.

## 리베이스와 전체 검증의 경계

새 base #8116의 기존 네 입력은 같은 Print·글꼴·DPI에서 현재 보정 범위와 함께 비교했다. 확정된 중첩 표 원점에서 측정과 배치가 같은 가용 높이를 소비하도록 보정했으며 기존 15개 검사의 기대값을 바꾸지 않았다. 사용자 지시의 페이지별 하락 한계는 0.1%p다. base의 90% 미만은 변경 전 비교 자료로 구분하고 후보는 모든 관련 쪽에서 90% 이상을 유지한다.

필수 로컬 게이트는 최종 실행 결과와 source SHA를 대조해 완료했다. 전체 Rust·Skia·Studio의 실제 종료 집계는 공통 증거의 실행 요약을 따른다.

## 원격 처리와 후속 기록

통합 PR을 실제 템플릿에 맞춰 제출한 뒤 정확한 head의 CI·mergeability를 확인한다. 최신 지시에 따라 code head CI 성공 뒤 같은 통합 PR의 trailing commit으로 self-review·오늘할일을 반영하고 trailing head CI 후 일반 병합한다. 원 PR별 리뷰는 각각 유지하고 실제 통합 merge·devel 포함을 확인한 뒤 존댓말로 원 기여와 보정 범위, CI·merge SHA 및 그 SHA에 고정한 대표 review·overlay를 연결한다. contributor fork·branch와 원 history는 보존한다.

대표 쪽 선정: 셀 경계 오리기 실행 후 결과; 오리기 전 133은 대조군. 다른 PR의 임의 페이지로 대체하지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 통합 merge와 자산의 devel 포함을 확인한 뒤에만 게시한다. 이 계획을 원 PR의 직접 merge 승인으로 쓰지 않는다.

검증 범위는 위 조건 133, 134, 135, 136, 137, 138, 139, 140, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169 전체 쪽이다. raw 90% 미만·누락·측정 불가 후보는 양쪽 0쪽이며, 대표 페이지와 전체 범위의 최저값을 구분한다. 평균·RGB 진단값으로 raw 미달을 면제하지 않았다.

대표는 조건 134의 1쪽이다. 다음 수치는 실제 새 출력·같은 독립 Print를 대조한 값이며 완전한 RGB 일치의 선언이 아니다.

| 출력 | raw 실루엣 | pixel_match | visual_accuracy_proxy_percent | review / overlay |
| --- | ---: | ---: | ---: | --- |
| native | 100.0% | 99.72274% | 48.12014% | [review](../assets/s21_20261027/panels/native/134-p001-review.png) / [overlay](../assets/s21_20261027/panels/native/134-p001-overlay.png) |
| wasm | 100.0% | 99.74654% | 52.55306% | [review](../assets/s21_20261027/panels/wasm/134-p001-review.png) / [overlay](../assets/s21_20261027/panels/wasm/134-p001-overlay.png) |

직접 판독: 오리기 실행 뒤 첫 셀의 중간·끝 글자와 인라인 개체, 인접 셀의 중간과 표 외곽을 대조했다.

게시 문안은 원 기여에 감사드리고, 원 PR 번호·원 head와 통합 후보의 보정 범위·실제 CI·merge SHA를 존댓말로 설명한다. 같은 asset의 아래 URL에서 `<merge-commit-sha>`만 실제 merge SHA로 바꾼다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/134-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/134-p001-overlay.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/134-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/134-p001-overlay.png`

실제 LF 본문 파일을 `--body-file`로 게시하고 API로 본문·한글·이미지 URL을 재조회한다. 동일 merge·동일 증거의 기존 comment가 있으면 중복 게시하지 않는다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
