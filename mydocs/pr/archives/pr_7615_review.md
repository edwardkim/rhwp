---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-14
---

# PR #7615 리뷰 — 수정: 누름틀 문단 편집이 다른 컨트롤 슬롯과 문단 앞 정의를 지킨다

## 최종 판정

**승인 — 메인터너 보정 후 승인.** 초기 수용 범위가 통합 PR [#7882](https://github.com/edwardkim/rhwp/pull/7882), merge `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`로 실제 병합됐고 `upstream/devel` 포함을 확인했다. 정확한 code/trailing head CI와 최종 정책 status 성공 후 통합했으며 원 PR별 검증·원 기여/메인터너 보정 근거는 아래 기록을 따른다. 초기 head `ed9c2847523c18f658344ec89cb7ed1a426807ae` 이후 #7662 의존 변경은 제외했으므로 원 PR 전체를 닫지 않는다.

## 현재 개별 검토 상태 — 2026-10-14

| 항목 | 실제 확인 결과 |
| --- | --- |
| 브랜치 / 정책 base | `review/semanticist21-20261008` / `22317614b03722e95349a1d3d71c86923363c6f5` |
| 통합 code candidate / 조판 생산 소스 | `0d59e7f73cc366897891329a21d353458db7776e` / `6cbebc7457ea8899c3949e48ed1255d43f40bcb2`; 검증한 Rust 생산1117개 파일·원 PR 계약 source 및 공용 pkg/Studio 바이트 동일성 확인 |
| 이 PR의 실제 계약 실행 | 33/33 PASS·FAIL/SKIP/누락0. `output/pr-review/semanticist21-20261008/complete-row-prefix-v460/original-pr-current-focused-contract-results.json`의 함수·source SHA·focused 실제 관측 연결 |
| 전체 원 PR 집중/중요 문서 검사 | 195 PASS·누락/예상 밖0. 전체10,267 PASS는 이전 ddf17f864 소스의 역사 실행이며 현재 전체 로컬 재수행으로 세지 않음 |
| Native/fresh WASM | 180 TSV·1040쪽/backend에서90% 미만·누락·쪽수 불일치0. 중요 문서 최저값 하락0; 산재 표 HWP/HWPX52쪽 복구. 원 PR의 실제 입력 범위는 아래 개별 표와 TSV manifest를 따름 |
| 최종 source CI | [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38475033894) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38475033796) · [CI](https://github.com/edwardkim/rhwp/actions/runs/38475033776) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38475033750) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38475033733) — 동일 PR/head의 실제 실행 결과. 진행 중·취소·이전 source CI를 대신 사용하지 않음 |
| 통합 완료 | **승인 — 메인터너 보정 후 승인**. #7882 merge `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`의 devel 포함과 최종 정책/CI 성공 확인; 원 PR close는 별도 후속 범위 판단을 따름 |

이 절이 현재 판정이다. 아래 역사 기록의 당시 머지 보류·미검증·옛 base/SHA는 현재 상태로 적용하지 않는다. [최종 로컬 증거 진행238](semanticist21_20261008_integrated_review.md#진행-갱신238--5877-ci-회귀-보정의-최종-로컬-검증-2026-10-14)와 [원문 TSV180개](https://gist.github.com/jangster77/1efd96731b845480bc1b8f38344d6d6e/1d7a7fe25f0960cb43496b28bed1965a68e282cf)를 연결한다. 조판·입력·Print·관계 검사의 구체적 의미와 미검증 범위는 원 PR별 항목 및 원문 manifest에 유지한다.

### 현재 적용 commit

| 원 commit | rebase 후 현재 적용 | 원 기여 대비 처리 |
| --- | --- | --- |
| `f189e5dd59d38c6e4b3c31581a7b55d03c4d20a2` | `2d6d7de9788fbb6b0150845daa5ed33e229e99a1` | 기존 적용과 stable patch-id 동일 |
| `0a76f6fabbd313ddd908da4b3949cbfb0d4db68e` | `d7e612c188689828731b857e2e325a558236da5f` | 기존 적용과 stable patch-id 동일 |
| `c09761d797a13f31c5c6a0ca687c2134a1535c47` | `4a14bc135eb74ea25f33e9b6bdc6081b1a360838` | 기존 적용과 stable patch-id 동일 |
| `1dad94cf07f2be5ffba6e84dc42cc521975bb862` | `666bd8ab3578b341c675d92acc6ebefb40878085` | 기존 적용과 stable patch-id 동일 |
| `4091704afca53dadf067d85be36779c6abf246d5` | `5612fe986f3393b4485e657b0754a0dc9832b127` | 기존 적용과 stable patch-id 동일 |
| `8a17e2e770d98215e0d5f2259868b58a2cf1ee75` | `1b0cc39271d6fb34c0b485471050b4b8864cb66f` | 기존 적용과 stable patch-id 동일 |
| `ae2a058d63b8340d56549097af844697925eacb5` | `00295db6056c512ee4a17621e81eafff86e6bc1a` | 기존 적용과 stable patch-id 동일 |

### 이 PR의 현재 계약 함수

- `auto_number_trailing_placeholder::typing_after_a_trailing_auto_number_in_a_click_here_paragraph_keeps_both` — PASS; `tests/cases/auto_number_trailing_placeholder.rs` SHA-256 `91b4d9f964d94c9025646b94ba5c9304965c371030036d4817b9ee69720596df`.
- `field_paragraph_control_slots::control_fields_only_paragraph_keeps_slots_when_typing` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_outside_field_keeps_footnote_slot` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_inside_active_field_keeps_footnote_slot` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::local_replace_before_field_keeps_field_and_footnote` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::split_keeps_footnote_shape_and_bookmark_slots` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::page_break_keeps_footnote_shape_and_bookmark_slots` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::pasted_footnote_keeps_slot_before_following_field` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_into_first_empty_field_keeps_later_field` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_into_later_field_keeps_first_empty_field_at_start` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_into_first_empty_field_keeps_footnote_and_bookmark_slots` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::breaking_before_empty_field_keeps_both_fields_in_new_paragraph` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::setting_field_value_keeps_later_footnote_slot` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::splitting_inside_filled_field_keeps_later_footnote_slot` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::setting_field_value_keeps_adjacent_field_value` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_after_field_keeps_header_table_and_picture_slots` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_inside_field_keeps_header_table_and_picture_slots` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::breaking_before_last_char_keeps_header_table_and_picture_slots` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::pasting_field_copied_from_header_paragraph_leaves_header_behind` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::pasting_after_field_that_ends_paragraph_keeps_field_end_slot` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_in_cell_keeps_bookmark_and_field_slots` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::merging_after_field_that_ends_paragraph_keeps_field_end_slot` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::merging_paragraph_that_starts_with_empty_field_counts_its_end_slot` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `field_paragraph_control_slots::typing_after_footnote_marker_by_logical_caret_goes_after_it` — PASS; `tests/cases/field_paragraph_control_slots.rs` SHA-256 `340bf89535dfcfd8022a966275b8b68c7fb1b784ad9bc0800f9b29f987941cb5`.
- `leading_definition_controls::typing_before_a_leading_object_keeps_definitions_first` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `leading_definition_controls::composing_before_a_leading_object_keeps_definitions_first` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `leading_definition_controls::splitting_before_a_leading_object_keeps_it_first_in_the_new_paragraph` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `leading_definition_controls::pasting_before_a_leading_object_keeps_definitions_first` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `leading_definition_controls::splitting_before_an_inline_object_mid_paragraph_keeps_it_first` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `leading_definition_controls::splitting_right_before_a_click_here_field_keeps_it_first` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `leading_definition_controls::title_marks_in_the_gap_keep_their_slots` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `leading_definition_controls::typing_at_the_start_of_an_ordinary_first_paragraph_is_unchanged` — PASS; `tests/cases/leading_definition_controls.rs` SHA-256 `63a8b33bfdf7aa8289c36f35f0d448a1195fc0fb73362cc330df6f704fcccfaa`.
- `split_caret_axis::undo_of_enter_right_after_auto_number_restores_the_paragraph` — PASS; `tests/cases/split_caret_axis.rs` SHA-256 `4e8801885c7e2e8586661bdb62c6cf0c3602016d60d281a7fc8d3661744a1350`.

**범위 제한:** 초기 수용 head `ed9c2847523c18f658344ec89cb7ed1a426807ae`까지만 승인한다. 원격 최신 head에 들어간 #7662 의존 후속 변경은 이번에 적용/승인하지 않았으므로 원 PR 전체를 해결 완료로 닫지 않는다.

### 이 PR의 현재 대표 문서·판독 연결

- 입력 `mydocs/pr/assets/semanticist21-20261008/additional-edit-inputs-v94/additional-edited-logical-caret-01-v94.hwpx` SHA-256 `9331dd52058f07e277889f24c6c9c68bc2ed9c79f197c7122c0f94dacf184400`; 독립 Print `pdf/semanticist21-20261008/additional-edit-v94/additional-edited-logical-caret-01-v94-9331dd52058f-2020.pdf` SHA-256 `6851573d1e140b65ae5638fa03cfc909b8b8f9c66921622860f753c42311a261`.
- `original-30-9331dd52058f` 전체1쪽 Native/fresh TSV에서 미달·누락0. 대표p1는 각각100.00000%/100.00000%이며 전체 scope와 구분한다.
- [Native review](../assets/semanticist21-20261008/initial-48-current-proof-v453/original-30-9331dd52058f-native-review.png) · [fresh review](../assets/semanticist21-20261008/initial-48-current-proof-v453/original-30-9331dd52058f-fresh-review.png). 기존 asset은 현재 양 backend의 재출력 PNG와 이전 직접 판독 PNG의 RGBA 동일성을 확인해 재사용한 것으로 기록하며 새 캡처로 가장하지 않는다.
- 이 PR의 실제 명령/편집·소유·내용 검사와 Print의 해당 영역을 함께 판단한다. 점수로 수식·미세 간격 또는 범위 밖 차이를 면제하지 않는다.

## 이전 단계의 검증 기록

### 이전 최종 판정

**머지 보류 — 118쪽 회귀 보정과 현재 전체 Rust 회귀 통과. Rust13게이트 완료. 가맹사업 fresh 전쪽·정식 관계 검사 완료 후 승인.** 사용자 지시에 따라 원 PR 번호별로 작성한 검토 기록이며, 필요한 최종 검증·시각 증적 확인 뒤 판정을 갱신한다. 현재 원 PR 승인 또는 통합 PR 제출을 의미하지 않는다. 모든 필수 테스트·전쪽 Native/fresh 비교·직접 판독이 완료되고 실패/미달이 해결되면 사용자 지시에 따라 **메인터너 보정 후 승인**으로 갱신한다.

### 이전 개별 검토 상태 — 2026-10-13

| 항목 | 확인 결과 |
| --- | --- |
| 작업 브랜치 / 고정 base | `review/semanticist21-20261008` / `552f5228d2dbfd6046c838f14241704f1bcb8a05` |
| 현재 production 보정 | `ddf17f86476293b114b6236bfa6cf1ebc9d9d474` — 원본 전체 행/중첩 셀의 프레임 출처와 정렬 높이 계약 분리 |
| 현재 Native/fresh 재검증 code head | ddf17f864 경계17쪽, 원PR24쌍27쪽 및 초기48입력52쪽 양쪽 전쪽·직접 판독 통과. 직접 문서5종 Native 전쪽 통과, CGMP/KPS/K-water/#157 fresh 전쪽 통과. actual Canvas/Studio 소비 검사와 lint/정책8단계 exit0 |
| 원 기여 적용 | 위 적용 표의 현재 SHA는 모두 HEAD의 조상이며 최초 적용 patch와 stable patch-id가 같다 |
| 이 PR의 실제 계약 실행 | 현재 ddf17f864 전체 nextest에서 33/33 PASS, FAIL/SKIP/미관측0; `v453/original-pr-contract-results.json`에 함수·로그·소스 SHA 연결 |
| 현재 통합 완료 판정 | **머지 보류** — 현재 Rust13게이트 모두 exit0. nextest10,267 PASS·45 SKIP·0 FAIL, Skia lib4,041 PASS·12 IGNORE, placeholder2/직접PDF4/doctest8 PASS. 가맹사업 fresh142쪽 진행; 정식 관계 회귀 등록·최종 관련 게이트 대기 |
| 원격 진행 권한 | 작업지시자의 자동 승인: 모든 필수 테스트·시각 검증 완료 후 통합 PR 생성 → 최신 head CI 확인 → 머지 → 후속 처리 |

이 PR의 입력·호출 경로·계약 함수와 미검증 범위는 아래 개별 항목을 따른다. v447 계약 PASS와
후속 #157 보정의 전2쪽 Native/fresh 통과는 각각 해당 SHA·입력 범위의 결과이며, 이 PR의
편집 저장본 전체나 최종 통합 검증의 통과로 바꾸지 않는다. 아래 이전 진행 문단은 당시 관측이다.
현재 적용 SHA·진행 권한·완료 여부는 이 절을 우선한다. 현재 공통 보정과 순차 검증의 실행 결과는 [최신 PR 준비](semanticist21_20261008_integrated_review.md#진행-갱신208--k-water-fresh-전27쪽-판독-및-최신-대표-png-2026-10-13)에 연결한다. 모든 필수 검증 완료 후 최상단 판정을
**메인터너 보정 후 승인**으로 갱신한다.

이 PR의 검증 함수별 원문 로그 연결은 ignored
`pr-review-resumed-v447/original-pr-contract-results.json`의 PR #7615 항목이고,
리베이스 적용 대조는 `regression-root-cause-batch-v448/individual-pr-current-application-audit.json`이다.
메인터너의 공통 표 프레임 보정·새 base 비교·양 backend 증거는
[누적 기록의 진행173](semanticist21_20261008_integrated_review.md#진행-갱신173--157-양-backend-선행-조건과-정식-회귀-검출-2026-10-13)에 연결한다.

### 이전 검증 기록 — b4ff8c

현재 production `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`의 전체 nextest는 **10267 PASS·45 SKIP·exit0**이다. 이 PR 계약 **33/33 PASS**와 동일 검사 source를 v452 `original-pr-contract-results.json`에서 확인했다. Native Skia3종과 doc는 완료했으며, 최종 전쪽·Canvas와 새 정식 관계 회귀 검증은 아직 남아 **머지 보류**를 유지한다. 아래7f04c 결과는 수정 전 역사 기록이다.

수정 전 production `7f04c48cd395086ed59c08565f343d3f9e1b013d`의 전체 회귀에서 이 PR 계약 33/33개가 PASS했다. 현재 production `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`는 72쪽 배치 차이를 보정한 새 소스이며, 이전 PASS를 현재 소스의 검증으로 대신하지 않는다. Native/fresh 영향 쪽·최종 회귀 및 필수 게이트 완료 전 판정은 **머지 보류**다. 근거와 소비 경로는 [누적 기록의 진행 갱신195](semanticist21_20261008_integrated_review.md#진행-갱신195--전-회귀-통과와-72쪽-정렬-높이-소비-경로-보정-2026-10-13)에 연결한다.


## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 / base | [#7615](https://github.com/edwardkim/rhwp/pull/7615) / semanticist21 / devel |
| 초기 수용 source head | `ed9c2847523c18f658344ec89cb7ed1a426807ae` |
| 2026-10-13 원격 조회 source head / 상태 | `4a6c27c5ffa54e4988a7e0e7956115d933725aca` / OPEN / draft=False |
| 과거 검증 생산 소스 / 당시 base | `940ea1322617ccff5a7b47d6920ae96e94cfffad` / `bfa28a77a2714482f647ee6a7b9aa7f7876d8404` |
| 이번 검토 기록 기준 통합 head | `5cd155886dbcfb19d5f4fa17ae778f278940e08b` |
| 관련 이슈 | [#4680](https://github.com/edwardkim/rhwp/issues/4680), [#7444](https://github.com/edwardkim/rhwp/issues/7444), [#7596](https://github.com/edwardkim/rhwp/issues/7596), [#7598](https://github.com/edwardkim/rhwp/issues/7598), [#7612](https://github.com/edwardkim/rhwp/issues/7612), [#7614](https://github.com/edwardkim/rhwp/issues/7614), [#7662](https://github.com/edwardkim/rhwp/issues/7662) |

## 적용과 검토 범위

원 기여는 초기18개 scope에 이미 체리픽된 기능 commit에 한정한다. 초기 수용 뒤 새 원격 source가 있더라도 추가 체리픽하지 않는다. 메인터너 보정은 같은 review 브랜치에서 분리해 보존하며 최종 code head의 검증으로 판정한다.

| 원 commit | 최초 검토 적용 commit(역사) | 당시 rebase 적용 commit(역사) |
| --- | --- | --- |
| `f189e5dd59d38c6e4b3c31581a7b55d03c4d20a2` | `a2ef9eab3` | `a99ad3618cd518893c9e2f79e01e7e95e333f794` |
| `0a76f6fabbd313ddd908da4b3949cbfb0d4db68e` | `761499928` | `62d2d3b17582b95214eade6ab0198cf4ded57ad1` |
| `c09761d797a13f31c5c6a0ca687c2134a1535c47` | `ebbec90fb` | `60c6bca1e71839d1594e719b034b0ebcc0e79540` |
| `1dad94cf07f2be5ffba6e84dc42cc521975bb862` | `60385438c` | `b0711db04c8e57cacf60433528015148656ef744` |
| `4091704afca53dadf067d85be36779c6abf246d5` | `3addfced3` | `2787fd340012a518fbdfec8d23c28b3cdad1a4df` |
| `8a17e2e770d98215e0d5f2259868b58a2cf1ee75` | `6a8f77dab` | `3f19d7cc812f24c84d88961e4b164ddb1b642387` |
| `ae2a058d63b8340d56549097af844697925eacb5` | `2be87053d` | `c0e29e51207a572e7edc722b8083c422d8192a70` |

최초 검토의 역사 관측: Native 누름틀·정의·슬롯 보존 검사 통과. 편집 저장본 Print 대조 범위 미검증.

위 관측은 최초 후보의 결과이며 현재 code head의 통과/실패로 재사용하지 않는다. 이후 원인별 보정·실제 생산/측정/배치 소비 경로·반례와 증거는 [누적 보정 기록](semanticist21_20261008_integrated_review.md)에 연결한다. 이 PR의 적용/비적용 경로와 필수 개별 검증 결과는 최종 실행 자료를 확인한 뒤 아래에 확정한다.

### 이 PR의 코드·계약 검사

초기 적용 commit의 diff에서 분리한 변경 파일은 다음과 같다. 원 commit SHA가 Git 객체로 남지 않은 경우에도 backup에 보존한 최초 cherry-pick commit의 patch를 사용하며, 최신 원격 diff를 초기 수용 범위로 대체하지 않는다.

- `src/document_core/commands/clipboard.rs`
- `src/document_core/commands/document.rs`
- `src/document_core/commands/text_editing.rs`
- `src/document_core/hyperlink.rs`
- `src/document_core/queries/field_query.rs`
- `src/model/paragraph.rs`
- `tests/cases/auto_number_trailing_placeholder.rs`
- `tests/cases/field_paragraph_control_slots.rs`
- `tests/cases/leading_definition_controls.rs`
- `tests/cases/split_caret_axis.rs`

| 현재 검사 소스 | 이 PR에서 추가한 계약 함수 | 현재 상태 |
| --- | --- | --- |
| `tests/cases/auto_number_trailing_placeholder.rs` | `typing_after_a_trailing_auto_number_in_a_click_here_paragraph_keeps_both` | v345 실제 1 PASS; 최종 통합/시각 gate 대기 |
| `tests/cases/field_paragraph_control_slots.rs` | `control_fields_only_paragraph_keeps_slots_when_typing`, `typing_outside_field_keeps_footnote_slot`, `typing_inside_active_field_keeps_footnote_slot`, `local_replace_before_field_keeps_field_and_footnote`, `split_keeps_footnote_shape_and_bookmark_slots`, `page_break_keeps_footnote_shape_and_bookmark_slots`, `pasted_footnote_keeps_slot_before_following_field`, `typing_into_first_empty_field_keeps_later_field`, `typing_into_later_field_keeps_first_empty_field_at_start`, `typing_into_first_empty_field_keeps_footnote_and_bookmark_slots`, `breaking_before_empty_field_keeps_both_fields_in_new_paragraph`, `setting_field_value_keeps_later_footnote_slot`, `splitting_inside_filled_field_keeps_later_footnote_slot`, `setting_field_value_keeps_adjacent_field_value`, `typing_after_field_keeps_header_table_and_picture_slots`, `typing_inside_field_keeps_header_table_and_picture_slots`, `breaking_before_last_char_keeps_header_table_and_picture_slots`, `pasting_field_copied_from_header_paragraph_leaves_header_behind`, `pasting_after_field_that_ends_paragraph_keeps_field_end_slot`, `typing_in_cell_keeps_bookmark_and_field_slots`, `merging_after_field_that_ends_paragraph_keeps_field_end_slot`, `merging_paragraph_that_starts_with_empty_field_counts_its_end_slot`, `typing_after_footnote_marker_by_logical_caret_goes_after_it` | v345 실제 23 PASS; 최종 통합/시각 gate 대기 |
| `tests/cases/leading_definition_controls.rs` | `typing_before_a_leading_object_keeps_definitions_first`, `composing_before_a_leading_object_keeps_definitions_first`, `splitting_before_a_leading_object_keeps_it_first_in_the_new_paragraph`, `pasting_before_a_leading_object_keeps_definitions_first`, `splitting_before_an_inline_object_mid_paragraph_keeps_it_first`, `splitting_right_before_a_click_here_field_keeps_it_first`, `title_marks_in_the_gap_keep_their_slots`, `typing_at_the_start_of_an_ordinary_first_paragraph_is_unchanged` | v345 실제 8 PASS; 최종 통합/시각 gate 대기 |
| `tests/cases/split_caret_axis.rs` | `undo_of_enter_right_after_auto_number_restores_the_paragraph` | v345 실제 1 PASS; 최종 통합/시각 gate 대기 |

이 목록은 현재 source와 최초 적용 patch를 대조한 결과이며 실행 통과를 뜻하지 않는다. 실제 검사 결과는 이 함수가 실행된 전체 nextest/focused 로그에서 PR별로 추출하고, 최종 입력·Print·Native/fresh 및 확대 판독을 해당 변경 의미에 연결한다. 공통 빌드·회귀 성공만으로 이 PR의 적용 경계나 편집 후 저장 출력이 검증됐다고 판정하지 않는다.

### 현재 원 PR 계약의 실제 실행 확인

2026-10-12 초기 수용 범위와 현재 검사 소스를 다시 대조했다. 위 계약 33건은
v345 전체 회귀의 실제 실행 로그에서 모두 PASS다. 해당 검사 파일은 생산 소스 커밋
`940ea1322617ccff5a7b47d6920ae96e94cfffad`의 바이트와 동일하며, 최근 교육과정 검사 이관은 이 PR의 검사나 생산 코드를 바꾸지 않았다.
로그는 ignored `output/pr-review/semanticist21-20261008/logs/whole-regression-audit-v345-nextest.log`,
정확한 함수·소스 SHA·로그 행의 연결은 `original-pr-contract-readiness-v396.json`에 보존한다.
전체 v345는10254PASS/12FAIL였으므로 이 PR의 PASS를 전체 통합 성공으로 확대하지 않는다.

사용자 지시에 따라 원래 체리픽 문서의 보정과 증거 정리를 우선한다. 기존 초기 편집 입력과
같은 독립 Print48쌍/52쪽의 현재 Native/fresh WASM 비교는 v396에서 실행 중이다.
7개 논리 캐럿 입력은 현재 v175 저장본과 기존 v94 증적 입력의 바이트가 모두 동일함을 확인했다.
같은 생산 소스의 기존 각주·미주/누름틀8쌍(v322/v326) 증거는 입력·Print·소스 SHA와 전쪽 결과를
확인해 해당 범위에만 연결한다. 다른 PR 문서나 미실행 경로의 통과로 대신하지 않는다.
대표 review/overlay의 실제 판독과 이 PR의 적용·비적용 경로를 확정한 뒤 최종 판정을 갱신한다.
원격의 후속 source 변경은 조회만 했으며 추가 체리픽하지 않았다. 현재 원격 진행 권한은 아래 자동 승인 기록을 따른다.

## 검증 입력·결과와 조판 원칙

현재 Native 빌드와 KPS77/가맹사업142/CGMP48 등5원본 쪽수 진단은 통과했다. 가맹사업42쪽 새 Native review는94.27957%이며 표·각주 비겹침을 직접 확인했다. 이 선택 증거는 각 PR의 편집 저장본 Print 계약 또는 전쪽/fresh WASM 통과를 대신하지 않는다.

fmt·Native/WASM32 Clippy와 workspace build는 v172 source에서 통과했다. all-target Clippy는 봉투 계약 검사의 제외 모듈 선언 누락으로 실패하여 `2d6f82fbb`에서 시험지 제외 helper만 연결했다. renderer source는 동일하다. v173 fmt·workspace/all-target Clippy·고정 base manifest/unit 정책은 통과했다. Native/WASM32 Clippy·workspace build의 v172 성공은 동일한 production source의 증거이며, 테스트 모듈 복구 뒤 all-target 경로를 다시 실행했다. fresh WASM 최적화 빌드는 exit0으로 완료됐고 JS/WASM의 pkg·Studio public 동일성을 확인했다. 공개 API runner6종과 #7566의 Studio 실제 클릭도 통과했다. 개별 범위와 byte SHA는 누적 기록140절에 연결했으며 전체 release-test nextest·Native Skia3종·대형 복합 검증의 최종 결과는 대기 중이다. KPS Native 전쪽77쪽은 최저42쪽90.91935%·90% 미만0·누락0·77/77쪽으로 확인했으며, 이 결과를 해당 PR의 편집 저장본 또는 fresh WASM의 시각 증거로 대신하지 않는다. 실제 끝 exit·최저값·누락·쪽수·직접 확대 판독과 입력/기준 PDF SHA를 최종 기록에 연결한다. 조판/회귀/입력 커밋 검토의 미완료 범위는 **미검증**이다. 절대px 기대값·baseline을 통과 목적으로 완화하지 않는다.

## 시각 증적과 남은 범위

최종 Native/fresh TSV와 대표 review/overlay를 같은 입력·쪽·source로 연결한다. 현재 source의 증적은 ignored `output/pr-review/semanticist21-20261008/pr-preparation-native-whole-v172-result.json`, `pr-preparation-wasm-whole-v174-result.json` 및 v171 retry2 기록에서 실제 종료 결과를 확인한다. 미달·구조 결함이 있으면 보정 후 재실행하며 최종 이미지와 URL은 새 head로 확정한다.

## 통합 뒤 원 PR 후속 처리 계획

자동 승인된 범위에서 필수 검증 완료 후 통합 PR과 정확한 head CI를 확인한다. 통합 merge SHA·이 PR의 해결 범위·보정 이유와 원 기여를 구분한 한국어 존댓말 문안을 준비하고, 실제 시각 증적 사용 시 merge SHA 고정 PNG를 첨부한다. 원 PR 종료/댓글은 승인 범위와 현재 상태를 재확인한 뒤 수행한다. 최초18개 밖의 후속 PR은 이번 통합 대상이 아니다.


## 커밋 후 현재 통합 head의 계약 재검증 — 2026-10-13

`review/semanticist21-20261008`의 커밋 `3cdcb2e20a47dc23dfdd6d1b399ad4d247430b44`에서
이 PR의 위 계약 33개가 모두 실제 전체 nextest 실행에서 PASS했다.
고정 base는 `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`이며 fetch 후 SHA가 같아 rebase하지 않았다.
원 PR 최신 head도 기존 접수 기록과 같았다. 원격 조회는 참고값이며 merge 직전 다시 확인한다.
전체 nextest는10258 PASS/12 FAIL/45 skipped·exit100이고, 원 계약 전체78/78 PASS를
통합 PR 전체 성공으로 확대하지 않는다. 실패별 보정과 필수 시각/backend 검증 뒤 최종 판정을 갱신한다.
원문 로그와 함수별 연결은 ignored `pr-review-resumed-v447/full-nextest.log`,
`original-pr-contract-results.json` 및 [누적 기록](semanticist21_20261008_integrated_review.md)에 보존한다.


## Merge 후 contributor PR comment 계획 — #7882

통합 PR의 실제 merge SHA가 devel에 반영되고 asset이 존재한 뒤, 원 기여와 메인터너 보정·이 PR의 초기 수용 범위를 구분하여 한국어 존댓말로 게시한다. 현재 code head CI와 원 PR 계약의 실제 PASS를 연결하며, 동일 입력의 전쪽 Native/fresh90% 이상·변경 영역 판독을 기록한다. 임시 output만 링크하지 않고 merge SHA 고정 review/overlay PNG 및 원문 TSV를 함께 표시한다. 전체10,267 결과는 당시 소스의 기록으로만 설명한다.

- 원 PR 대표 입력 `mydocs/pr/assets/semanticist21-20261008/additional-edit-inputs-v94/additional-edited-logical-caret-01-v94.hwpx`, 독립 Print `pdf/semanticist21-20261008/additional-edit-v94/additional-edited-logical-caret-01-v94-9331dd52058f-2020.pdf`. 대응 페이지·점수 및 전체 범위는 TSV manifest의 `original-30-9331dd52058f` Native/fresh 항목을 인용한다.

- 원 PR 대표 입력 `mydocs/pr/assets/semanticist21-20261008/additional-edit-inputs-v94/additional-edited-logical-caret-02-v94.hwpx`, 독립 Print `pdf/semanticist21-20261008/additional-edit-v94/additional-edited-logical-caret-02-v94-d12cf3413e13-2020.pdf`. 대응 페이지·점수 및 전체 범위는 TSV manifest의 `original-31-d12cf3413e13` Native/fresh 항목을 인용한다.

- 후속 댓글에 사용할 이 PR의 대표는 `original-30-9331dd52058f` p1다. Native 100.00000% /fresh 100.00000% 및 전체1쪽 범위를 구분한다.
- merge SHA로 고정할 실제 asset은 `mydocs/pr/assets/semanticist21-20261008/initial-48-current-proof-v453/original-30-9331dd52058f-native-review.png`(SHA-256 `d628372fd128d739aac0a3d5d3a3fcc023d698035a030de86b9c898c287193b5`) 및 `mydocs/pr/assets/semanticist21-20261008/initial-48-current-proof-v453/original-30-9331dd52058f-fresh-review.png`(SHA-256 `1b1b645b5bd3ca58daa655fbf6ed63d5b8167d191ec941751a9d40cc42161c7d`)이며 두 이미지를 실제 Markdown으로 표시한다. 입력/독립 Print SHA·현재 재출력과 RGBA 동일성·원문 TSV를 함께 연결한다.
- 이후 #7662 의존 변경은 제외했음을 명시하고 원 PR은 열린 상태로 보존한다.


## 통합 merge 확인 — 2026-10-14

통합 PR [#7882](https://github.com/edwardkim/rhwp/pull/7882)는 merge `86a3c88fc89de9a2306a10bdda9a31023f0e24a5`로 실제 병합됐으며 `upstream/devel` 포함을 확인했다. code candidate `0d59e7f73cc366897891329a21d353458db7776e`의 전체 CI, 문서-only trailing head `34cc7ddefa246ecb7780c748dc7eb449bde89e6b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/38478418053)·CodeQL·Render Diff·Adapter·Proptest 및 [정책 감사3](https://github.com/edwardkim/rhwp/actions/runs/38478674616)를 확인했다. 최종 `CI Impact Policy=success`, `MERGEABLE/CLEAN`, 모든 최신 head check 성공/정상 skipped 상태에서 exact-head merge했다. API의 완료 step 메타데이터 지연으로 첫 정책 감사가 실패/대기했으며 실제 step이 모두 종료된 뒤 Controller만 재실행했다. 검사를 완화하거나 전체 로컬 회귀를 반복하지 않았다.

195 집중 PASS·180 Native/fresh TSV·1040쪽/backend의 범위와 독립 입력/Print·생산 SHA는 이전 최종 검증에 유지한다. 이번 후속 기록은 확정 merge SHA만 보완하며 source/test/fixture/workflow·시각 결과는 바꾸지 않는다. 이미 제출된 오늘할일은 반복 갱신하지 않는다.

후속 범위는 CGMP 하위 [#7764](https://github.com/edwardkim/rhwp/issues/7764)의 완료 증거와 원 PR별 contributor comment/supersede 처리다. 원 PR 16개는 머지 후 원격 head 재확인 뒤 처리한다. #7612의 기존 merge는 보존하고 #7615의 초기 수용 이후 변경과 후속6개는 제외한 상태로 유지한다. 대표 #7445 및 Refs #7759/#7807/#4514는 전체 해결로 닫지 않는다. 실행 결과는 원 PR·해당 이슈의 실제 comment에 남기며 이 문단을 미실행 close의 증거로 사용하지 않는다.
