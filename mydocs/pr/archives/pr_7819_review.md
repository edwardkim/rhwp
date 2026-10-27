# PR #7819 리뷰 — Fix CharOverlap logical slots and inline note rendering

## 최종 판정

**메인터너 보정 후 수용 가능 — 겹침 글자·미주의 논리 슬롯과 표시 범위의 로컬 검증 완료.**

원 contributor head의 직접 merge 판정과 메인터너 보정한 통합 후보의 판정을 구분한다. 원격 CI·merge·후속 처리는 실제 수행 후 별도로 기록한다.

## 접수 정보와 적용 계보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#7819](https://github.com/edwardkim/rhwp/pull/7819) / semanticist21 |
| 원 head | `36f9d3ac4091cc5c85cd74dee030c93b62c80964` |
| 통합 브랜치 | `review/semanticist21-20261019` |
| 검증 코드·테스트 source | `04c92da78165ccbf63b467855591df183655a98d` |
| 고정 upstream/devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 원격 조회 시 상태 | 2026-10-27 조회에서 원 head 불변·open·reviewer jangster77 확인. 원 Draft 여부와 통합 후보 판정은 별개 |

| 원 커밋 | 현재 통합의 대응 |
| --- | --- |
| `7ded7ca1d468326844ede2e934b10c4add9c7d81` | `0c8c9bc711515b6732bb00f901616ef4777aeb4f` |
| `a3f80b14adcbec5974debb537a02eb84aa061603` | `59986ff5bddcf49b54e0d38782dc9d9ab25ac5c9` |
| `36f9d3ac4091cc5c85cd74dee030c93b62c80964` | `9e38bf704f298d706c7b5c7e880c86367b85a0c4` |

## 구현 주장·공통 소비 경로·반례

CharOverlap과 note의 logical 한 칸을 원문 scalar 길이와 구분한다. model::Control의 논리 슬롯 분류부터 공개 getter·caret·selection·copy/split/merge 소비까지 같은 축을 확인한다. empty overlap·동일 앵커·non-BMP·말미 hit·실제 note 순서가 반례다. 정상 source 문자와 합성 overlap payload를 혼동하지 않으며, px 검사는 issue_7819_caret_axis에 독립 Print와 양쪽 98% 이상을 기록한 해당 페이지로 한정한다.

## 원 PR의 실제 회귀 실행

아래 함수와 실제 최종 PASS 행을 대조했다. 계약 검사 통과와 독립 Print의 시각 일치를 구분한다.

- [tests/cases/char_overlap_caret_axis.rs](../../../tests/cases/char_overlap_caret_axis.rs): SHA-256 `9a680fef8d58f03c6492f58e25f9f57c0195006f1f49b2265b3181c7fdb53c89`.
  - `mixed_overlap_note_uses_one_axis_in_public_getters_and_both_formats`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8918`.
  - `public_caret_and_selection_use_the_rendered_overlap_and_note_boundaries`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8921`.
  - `input_before_and_after_overlap_and_note_preserves_source_slots`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8919`.
  - `logical_copy_excludes_or_includes_the_overlap_slot_without_neighbor_text`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8917`.
  - `split_merge_and_snapshot_restore_keep_the_logical_seam_after_overlap`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8922`.
  - `empty_overlap_and_same_anchor_slots_are_counted_without_claiming_middle_input`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8915`.
  - `floating_and_auto_number_keep_their_existing_caret_contract`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8914`.
  - `non_bmp_text_and_note_keep_source_order_complete_caret_and_hit_boundaries`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8920`.
  - `same_anchor_notes_keep_control_order_and_terminal_overlap_geometry`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8923`.
  - `issue_7819_print_qualified_overlap_note_caret_boundaries_stay_within_two_pixels`: PASS, `rhwp::regression_suite_018`, 로컬 `logs/coherent-full-nextest-4697.log:8916`.
- [tests/cases/issue_7408_stored_row_key_folded_origin.rs](../../../tests/cases/issue_7408_stored_row_key_folded_origin.rs): SHA-256 `5916b148a6b41d49de4b4d14ba626a6c079cb274b88afec4cba224c442963f65`.
  - `footnote_sample_keeps_every_stored_cut`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:7088`.
  - `endnote_sample_keeps_every_stored_cut`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:7087`.
  - `tac_picture_sample_keeps_its_stored_cuts`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:7170`.
  - `edited_paragraph_is_rewrapped`: PASS, `rhwp::regression_suite_011`, 로컬 `logs/coherent-full-nextest-4697.log:7086`.

## 독립 Print·전쪽 TSV·대표 페이지

이 PR의 기존 검증 입력은 최종 조건 번호 080, 081, 082, 083, 084, 085, 086, 087, 088, 089, 090, 091, 092, 093, 094, 095, 096, 097, 098, 099, 100, 101, 102, 103, 104, 105, 106에 연결한다. 다른 PR의 평균 점수로 이 범위를 대신하지 않는다.

| 조건 | 전체 쪽수 | Native 최저 raw·전체 TSV | fresh WASM 최저 raw·전체 TSV |
| --- | ---: | --- | --- |
| 080 | 1 | [100.00000](../assets/s21_20261027/n/c/080.tsv) | [100.00000](../assets/s21_20261027/w/c/080.tsv) |
| 081 | 1 | [100.00000](../assets/s21_20261027/n/c/081.tsv) | [100.00000](../assets/s21_20261027/w/c/081.tsv) |
| 082 | 1 | [100.00000](../assets/s21_20261027/n/c/082.tsv) | [100.00000](../assets/s21_20261027/w/c/082.tsv) |
| 083 | 1 | [100.00000](../assets/s21_20261027/n/c/083.tsv) | [100.00000](../assets/s21_20261027/w/c/083.tsv) |
| 084 | 1 | [100.00000](../assets/s21_20261027/n/c/084.tsv) | [100.00000](../assets/s21_20261027/w/c/084.tsv) |
| 085 | 1 | [100.00000](../assets/s21_20261027/n/c/085.tsv) | [100.00000](../assets/s21_20261027/w/c/085.tsv) |
| 086 | 1 | [100.00000](../assets/s21_20261027/n/c/086.tsv) | [100.00000](../assets/s21_20261027/w/c/086.tsv) |
| 087 | 1 | [100.00000](../assets/s21_20261027/n/c/087.tsv) | [100.00000](../assets/s21_20261027/w/c/087.tsv) |
| 088 | 1 | [100.00000](../assets/s21_20261027/n/c/088.tsv) | [100.00000](../assets/s21_20261027/w/c/088.tsv) |
| 089 | 1 | [100.00000](../assets/s21_20261027/n/c/089.tsv) | [100.00000](../assets/s21_20261027/w/c/089.tsv) |
| 090 | 1 | [100.00000](../assets/s21_20261027/n/c/090.tsv) | [100.00000](../assets/s21_20261027/w/c/090.tsv) |
| 091 | 1 | [100.00000](../assets/s21_20261027/n/c/091.tsv) | [100.00000](../assets/s21_20261027/w/c/091.tsv) |
| 092 | 1 | [100.00000](../assets/s21_20261027/n/c/092.tsv) | [100.00000](../assets/s21_20261027/w/c/092.tsv) |
| 093 | 1 | [100.00000](../assets/s21_20261027/n/c/093.tsv) | [100.00000](../assets/s21_20261027/w/c/093.tsv) |
| 094 | 1 | [100.00000](../assets/s21_20261027/n/c/094.tsv) | [100.00000](../assets/s21_20261027/w/c/094.tsv) |
| 095 | 1 | [100.00000](../assets/s21_20261027/n/c/095.tsv) | [100.00000](../assets/s21_20261027/w/c/095.tsv) |
| 096 | 1 | [100.00000](../assets/s21_20261027/n/c/096.tsv) | [100.00000](../assets/s21_20261027/w/c/096.tsv) |
| 097 | 1 | [100.00000](../assets/s21_20261027/n/c/097.tsv) | [100.00000](../assets/s21_20261027/w/c/097.tsv) |
| 098 | 1 | [100.00000](../assets/s21_20261027/n/c/098.tsv) | [100.00000](../assets/s21_20261027/w/c/098.tsv) |
| 099 | 1 | [100.00000](../assets/s21_20261027/n/c/099.tsv) | [100.00000](../assets/s21_20261027/w/c/099.tsv) |
| 100 | 1 | [100.00000](../assets/s21_20261027/n/c/100.tsv) | [100.00000](../assets/s21_20261027/w/c/100.tsv) |
| 101 | 1 | [100.00000](../assets/s21_20261027/n/c/101.tsv) | [100.00000](../assets/s21_20261027/w/c/101.tsv) |
| 102 | 1 | [100.00000](../assets/s21_20261027/n/c/102.tsv) | [100.00000](../assets/s21_20261027/w/c/102.tsv) |
| 103 | 1 | [100.00000](../assets/s21_20261027/n/c/103.tsv) | [100.00000](../assets/s21_20261027/w/c/103.tsv) |
| 104 | 1 | [100.00000](../assets/s21_20261027/n/c/104.tsv) | [100.00000](../assets/s21_20261027/w/c/104.tsv) |
| 105 | 1 | [100.00000](../assets/s21_20261027/n/c/105.tsv) | [100.00000](../assets/s21_20261027/w/c/105.tsv) |
| 106 | 1 | [95.73216](../assets/s21_20261027/n/c/106.tsv) | [95.27616](../assets/s21_20261027/w/c/106.tsv) |

[입력·Print 경로와 SHA, producer·DPI·글꼴, base TSV 및 대표 review·overlay](../assets/s21_20261027/README.md)를 각 조건 번호로 대조한다. raw는 색상 경계 보정 전 2px 내용 실루엣이며 RGB 완전 일치가 아니다. 실제 빈 페이지의 raw 100%는 가시 내용의 정확성 증거로 사용하지 않고, 원본·독립 Print의 쪽수와 소유 계약을 별도로 검사한다.

신규 검사는 문단·줄·개체의 소유·순서·누락·중복·포함·겹침 관계를 기본으로 한다. 독립 Print와 실제 판독을 마치고 Native/fresh WASM이 모두 98% 이상인 완료 페이지만 독립 좌표 ±2px를 허용하며 관계 검사도 유지한다. 원본 HU·크기·비율 계약과 임의 화면 좌표 핀을 구분한다.

## 리베이스와 전체 검증의 경계

새 base #8116의 기존 네 입력은 같은 Print·글꼴·DPI에서 현재 보정 범위와 함께 비교했다. 확정된 중첩 표 원점에서 측정과 배치가 같은 가용 높이를 소비하도록 보정했으며 기존 15개 검사의 기대값을 바꾸지 않았다. 사용자 지시의 페이지별 하락 한계는 0.1%p다. base의 90% 미만은 변경 전 비교 자료로 구분하고 후보는 모든 관련 쪽에서 90% 이상을 유지한다.

필수 로컬 게이트는 최종 실행 결과와 source SHA를 대조해 완료했다. 전체 Rust·Skia·Studio의 실제 종료 집계는 공통 증거의 실행 요약을 따른다.

## 원격 처리와 후속 기록

통합 PR을 실제 템플릿에 맞춰 제출한 뒤 정확한 head의 CI·mergeability를 확인한다. 최신 지시에 따라 code head CI 성공 뒤 같은 통합 PR의 trailing commit으로 self-review·오늘할일을 반영하고 trailing head CI 후 일반 병합한다. 원 PR별 리뷰는 각각 유지하고 실제 통합 merge·devel 포함을 확인한 뒤 존댓말로 원 기여와 보정 범위, CI·merge SHA 및 그 SHA에 고정한 대표 review·overlay를 연결한다. contributor fork·branch와 원 history는 보존한다.

대표 쪽 선정: 비 BMP 글자 뒤 미주가 있는 실제 출력과 논리 슬롯의 API 검사. 다른 PR의 임의 페이지로 대체하지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 통합 merge와 자산의 devel 포함을 확인한 뒤에만 게시한다. 이 계획을 원 PR의 직접 merge 승인으로 쓰지 않는다.

검증 범위는 위 조건 080, 081, 082, 083, 084, 085, 086, 087, 088, 089, 090, 091, 092, 093, 094, 095, 096, 097, 098, 099, 100, 101, 102, 103, 104, 105, 106 전체 쪽이다. raw 90% 미만·누락·측정 불가 후보는 양쪽 0쪽이며, 대표 페이지와 전체 범위의 최저값을 구분한다. 평균·RGB 진단값으로 raw 미달을 면제하지 않았다.

대표는 조건 082의 1쪽이다. 다음 수치는 실제 새 출력·같은 독립 Print를 대조한 값이며 완전한 RGB 일치의 선언이 아니다.

| 출력 | raw 실루엣 | pixel_match | visual_accuracy_proxy_percent | review / overlay |
| --- | ---: | ---: | ---: | --- |
| native | 100.0% | 99.82377% | 16.08765% | [review](../assets/s21_20261027/panels/native/082-p001-review.png) / [overlay](../assets/s21_20261027/panels/native/082-p001-overlay.png) |
| wasm | 100.0% | 99.83466% | 21.64894% | [review](../assets/s21_20261027/panels/wasm/082-p001-review.png) / [overlay](../assets/s21_20261027/panels/wasm/082-p001-overlay.png) |

직접 판독: 앞 글자·비 BMP 슬롯·미주 번호와 문서 말미 미주 구분선을 대조했다. 캐럿 소속은 실제 API 검사로 별도 확인한다.

게시 문안은 원 기여에 감사드리고, 원 PR 번호·원 head와 통합 후보의 보정 범위·실제 CI·merge SHA를 존댓말로 설명한다. 같은 asset의 아래 URL에서 `<merge-commit-sha>`만 실제 merge SHA로 바꾼다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/082-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/native/082-p001-overlay.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/082-p001-review.png`
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/s21_20261027/panels/wasm/082-p001-overlay.png`

실제 LF 본문 파일을 `--body-file`로 게시하고 API로 본문·한글·이미지 URL을 재조회한다. 동일 merge·동일 증거의 기존 comment가 있으면 중복 게시하지 않는다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
