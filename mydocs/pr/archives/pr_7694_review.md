# PR #7694 리뷰 — 기능: 인라인 개체를 논리 캐럿 위치로 원자적으로 이동하는 API 추가

## 최종 판정

**메인터너 보정 후 수용 가능 — 기능: 인라인 개체를 논리 캐럿 위치로 원자적으로 이동하는 API 추가 범위의 로컬 검증 완료.**

원 contributor head의 직접 merge 판정과 메인터너 보정한 통합 후보의 판정을 구분한다. 원격 CI·merge·후속 처리는 실제 수행 후 별도로 기록한다.

## 접수 정보와 적용 계보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#7694](https://github.com/edwardkim/rhwp/pull/7694) / semanticist21 |
| 원 head | `0e7383d03753ac2546b0be74f19a244fe81747ff` |
| 통합 브랜치 | `review/semanticist21-20261019` |
| 검증 코드·테스트 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 고정 upstream/devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 원격 조회 시 상태 | 2026-10-27 조회에서 원 head 불변·open·reviewer jangster77 확인. 원 Draft 여부와 통합 후보 판정은 별개 |

| 원 커밋 | 현재 통합의 대응 |
| --- | --- |
| `f7c0acb2c9580cae3ebc14e09bb503dba261ffaa` | `d1e0e6effb76f008212e35e30d424b3d2bf16859` |
| `3e2a9dc1c6597e357634835208e0efcfca5288c0` | `cbba1142ab875690931e534d0e4dd96613dc99fa` |
| `9dab2711be1156aefc5318a1adb4e98fb0a309fe` | `0ebe2b9c984c71776077526290833410527e86ce` |
| `30a29eaf5481b6ca8b1c283c10b695a735eadc35` | `890cb141fd9cecab6b08a59cd467a83f77aa4fce` |
| `5f7a4fca71f816c2fb72154639c31e3826423a28` | `4b8fa6a3f6c03a9b5d13344632a64741a9a72932` |
| `3e68f620a9c2643bb52a78d305b2f7e95f06a631` | `19472f0d01d239d0fdc04c18e0cbe07fa8519f1a` |
| `55f06f5505d6f6ddd449a3f7b1cb4acd84c29ac2` | `9babbc89eaad590cfba07e9740ac46a7f625384d` |
| `09f91ff16986b6f64db4a3b46d1a63ca0f6e8de5` | `1f9f9a3c6625b309d6801a069a27f19e6451b3c7` |
| `9052f0bde427f93bf3cefe5de09875065a49bfbb` | 원 PR의 devel merge 전용 커밋; 별도 기능 체리픽 없음 |
| `133e09e154d0d496850d64deb7e823b2599cceba` | `a4f46c90d1e602898037cc4aafdffb21a407e8ca` |
| `680f44ae667ba16b8ecc1073ce815286788d138b` | `255357ae0a8852af1298f5b46f3c85a092d17ec1` |
| `0e7383d03753ac2546b0be74f19a244fe81747ff` | `4ae7eacf88977ac7d39d079ddc72ba670ff5178c` |

## 구현 주장·공통 소비 경로·반례

move_inline_control_native는 배포 문서·원본/대상 주소·잠금·logical caret를 먼저 검증하고 영향 구역 복제본에서 remove/insert를 실행한다. 실제 문서 commit 이전에 실패할 수 있는 검사를 끝내는 순서를 확인한다. 같은 위치 no-op은 이벤트를 발행하지 않으며, source 제거 후 sibling 경로와 목적지 offset을 다시 계산한다. inline_control_move의 18개 검사와 분리한 유효 저장본 검사 1개는 Unicode·필드·형광펜·범위 태그·BinData·실제 OOXML 차트 payload 및 공개 HWP/HWPX 저장을 포함한다. 이동 전후 실제 차트의 과거 시각 미달은 뒤의 OLE/차트 공통 보정으로 해결했으며, 원 이동 API와 차트 paint 보정의 범위를 구분한다.

## 원 PR의 실제 회귀 실행

아래 함수와 실제 최종 PASS 행을 대조했다. 계약 검사 통과와 독립 Print의 시각 일치를 구분한다.

- [tests/cases/inline_control_move.rs](../../../tests/cases/inline_control_move.rs): SHA-256 `d85b9ba7f75bed2cdd5c7f942bee95d9c609ca94d0ccf345e62b9c95a9412ea1`.
  - `same_paragraph_forward_and_back_preserve_unicode_order_styles_data_and_undo`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7853`.
  - `adjacent_objects_have_distinct_before_after_carets_and_no_op_emits_nothing`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7840`.
  - `cross_paragraph_move_keeps_paragraphs_and_both_save_formats`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7841`.
  - `moving_after_a_trailing_auto_number_preserves_its_placeholder_and_both_save_formats`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7846`.
  - `zero_length_field_slot_is_removed_exactly_and_field_indices_and_marks_survive`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7856`.
  - `target_sibling_path_reindexes_after_source_removal_and_own_descendant_is_rejected`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7855`.
  - `protected_cells_distribution_invalid_control_and_invalid_caret_are_atomic`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7851`.
  - `header_and_note_destinations_return_existing_owner_forms`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7842`.
  - `picture_binary_identity_and_control_metadata_survive_cross_owner_move`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7850`.
  - `moving_out_of_a_cell_updates_its_owner_after_insertion_before_the_parent`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7847`.
  - `locked_object_and_locked_parent_reject_without_mutation`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7843`.
  - `markpen_object_ranges_move_between_paragraphs_and_preserve_unpaired_metadata`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7845`.
  - `nonempty_field_insertion_keeps_object_inside_after_text_is_deleted`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7849`.
  - `nested_markpen_colors_keep_outer_inner_order_when_moved`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7848`.
  - `table_shape_textbox_and_chart_payloads_are_moved_without_recreation`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7854`.
  - `actual_ooxml_chart_keeps_control_and_binary_payloads_after_move_and_save`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7839`.
  - `public_hwp_export_preserves_inline_chart_with_and_without_move`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7852`.
  - `malformed_source_or_destination_coordinates_reject_without_changes`: PASS, `rhwp::regression_suite_014`, 로컬 `logs/coherent-full-nextest-4697.log:7844`.
- [tests/cases/ooxml_chart_print_frame.rs](../../../tests/cases/ooxml_chart_print_frame.rs): SHA-256 `eed1c3118653f04d739bcc15b19462147d07c59fb59530bfa57c6fe32ad685fc`.
  - `hwp_and_hwpx_moved_charts_keep_print_frame_axes_and_content_relations`: PASS, `rhwp::regression_suite_003`, 로컬 `logs/coherent-full-nextest-4697.log:4780`.
- [tests/cases/valid_inline_move_fixture_contract.rs](../../../tests/cases/valid_inline_move_fixture_contract.rs): SHA-256 `00528471f32b6f7dc3fbab806ad0b814d7b07156be298ef679f2f73d7a48d8aa`.
  - `valid_inline_move_fixtures_preserve_saved_carets_and_chart_data`: PASS, `rhwp::regression_suite_016`, 로컬 `logs/coherent-full-nextest-4697.log:8611`.

## 독립 Print·전쪽 TSV·대표 페이지

이 PR의 기존 검증 입력은 최종 조건 번호 124, 125, 128, 129에 연결한다. 다른 PR의 평균 점수로 이 범위를 대신하지 않는다.

| 조건 | 전체 쪽수 | Native 최저 raw·전체 TSV | fresh WASM 최저 raw·전체 TSV |
| --- | ---: | --- | --- |
| 124 | 1 | [99.80761](../assets/s21_20261027/n/c/124.tsv) | [99.66483](../assets/s21_20261027/w/c/124.tsv) |
| 125 | 1 | [99.71479](../assets/s21_20261027/n/c/125.tsv) | [99.73289](../assets/s21_20261027/w/c/125.tsv) |
| 128 | 1 | [99.80761](../assets/s21_20261027/n/c/128.tsv) | [99.66483](../assets/s21_20261027/w/c/128.tsv) |
| 129 | 1 | [99.71479](../assets/s21_20261027/n/c/129.tsv) | [99.73289](../assets/s21_20261027/w/c/129.tsv) |

[입력·Print 경로와 SHA, producer·DPI·글꼴, base TSV 및 대표 review·overlay](../assets/s21_20261027/README.md)를 각 조건 번호로 대조한다. raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 실제 빈 페이지의 raw 100%는 가시 내용의 정확성 증거로 사용하지 않고, 원본·독립 Print의 쪽수와 소유 계약을 별도로 검사한다.

신규 검사는 문단·줄·개체의 소유·순서·누락·중복·포함·겹침 관계를 기본으로 한다. 독립 Print와 실제 판독을 마치고 Native/fresh WASM이 모두 98% 이상인 완료 페이지만 독립 좌표 ±2px를 허용하며 관계 검사도 유지한다. 원본 HU·크기·비율 계약과 임의 화면 좌표 핀을 구분한다.

## 리베이스와 전체 검증의 경계

새 base #8116의 기존 네 입력은 같은 Print·글꼴·DPI에서 현재 보정 범위와 함께 비교했다. 확정된 중첩 표 원점에서 측정과 배치가 같은 가용 높이를 소비하도록 보정했으며 기존 15개 검사의 기대값을 바꾸지 않았다. 사용자 지시의 페이지별 하락 한계는 0.1%p다. base의 90% 미만은 변경 전 비교 자료로 구분하고 후보는 모든 관련 쪽에서 90% 이상을 유지한다.

필수 로컬 게이트는 최종 실행 결과와 source SHA를 대조해 완료했다. 전체 Rust·Skia·Studio의 실제 종료 집계는 공통 증거의 실행 요약을 따른다.

## 원격 처리와 후속 기록

통합 PR을 실제 템플릿에 맞춰 제출한 뒤 정확한 head의 CI·mergeability를 확인한다. 최신 지시에 따라 code head CI 성공 뒤 같은 통합 PR의 trailing commit으로 self-review·오늘할일을 반영하고 trailing head CI 후 일반 병합한다. 원 PR별 리뷰는 각각 유지하고 실제 통합 merge·devel 포함을 확인한 뒤 존댓말로 원 기여와 보정 범위, CI·merge SHA 및 그 SHA에 고정한 대표 review·overlay를 연결한다. contributor fork·branch와 원 history는 보존한다.

대표 쪽 선정: 차트를 논리 슬롯으로 옮긴 뒤 본문·차트·후속 문단의 순서. 다른 PR의 임의 페이지로 대체하지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 통합 merge와 자산의 devel 포함을 확인한 뒤에만 게시한다. 이 계획을 원 PR의 직접 merge 승인으로 쓰지 않는다.

검증 범위는 위 조건 124, 125, 128, 129 전체 쪽이다. raw 90% 미만·누락·측정 불가 후보는 양쪽 0쪽이며, 대표 페이지와 전체 범위의 최저값을 구분한다. 평균·RGB 진단값으로 raw 미달을 면제하지 않았다.

대표는 조건 125의 1쪽이다. 다음 수치는 실제 새 출력·같은 독립 Print를 대조한 값이며 완전한 RGB 일치의 선언이 아니다.

| 출력 | raw 실루엣 | pixel_match | visual_accuracy_proxy_percent | review / overlay |
| --- | ---: | ---: | ---: | --- |
| native | 99.71479% | 98.86178% | 62.44166% | [review](../assets/s21_20261027/panels/native/125-p001-review.png) / [overlay](../assets/s21_20261027/panels/native/125-p001-overlay.png) |
| wasm | 99.73289% | 99.08741% | 67.58761% | [review](../assets/s21_20261027/panels/wasm/125-p001-review.png) / [overlay](../assets/s21_20261027/panels/wasm/125-p001-overlay.png) |

직접 판독: 옮긴 차트 앞의 차트 새 자리와 차트 뒤 앞뒤 본문, 제목·축·세 계열·범례를 대조했다. 작은 글자와 선 농도 잔차가 남는다.

게시 문안은 원 기여에 감사드리고, 원 PR 번호·원 head와 통합 후보의 보정 범위·실제 CI·merge SHA를 존댓말로 설명한다. 같은 asset의 아래 URL에서 `<merge-commit-sha>`만 실제 merge SHA로 바꾼다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/125-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/125-p001-overlay.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/125-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/125-p001-overlay.png`

실제 LF 본문 파일을 `--body-file`로 게시하고 API로 본문·한글·이미지 URL을 재조회한다. 동일 merge·동일 증거의 기존 comment가 있으면 중복 게시하지 않는다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
