# 구현계획서 — Task #1370: 미주 측정 SSOT 게이트 재보정

- **이슈**: #1370 (M100) / 브랜치 `local/task1370`
- **수행계획서**: `mydocs/plans/task_m100_1370.md` (승인됨)
- **단계 수**: 5 (Stage 1 진단 → Stage 2~4 그룹별 재보정 → Stage 5 통합검증/승격판단)

## 공통 원칙

- **A3 게이트 내에서만 수정**. 모든 변경은 `ssot_level >= EnSsotLevel::A3` 분기 안. 기본(B) 코드 경로 불변.
- **검증 경로 고정**: `build_page_render_tree`·`dump_page_items`·`dump-pages`·`RHWP_EN_SSOT_DEBUG` 만.
  A3 `export-svg` CLI(158쪽)는 사용 금지.
- **전 exam 동시 green 게이트**: 매 단계 종료 시 다음을 A3 로 동시 실행해 통과 확인.
  ```
  RHWP_EN_SSOT=A3 cargo test --test issue_1082_endnote_overflow_reflow
  RHWP_EN_SSOT=A3 cargo test --test issue_1139_inline_picture_duplicate
  ```
  (+ 기본 B 로도 두 파일 무회귀 확인)
- 각 단계 완료 후 `_stage{N}.md` 보고서 작성 + 소스와 함께 커밋 → 승인 요청.

## Stage 1 — 진단 카탈로그 (코드 무수정)

**목표**: 13건 각각의 발산을 정량화하고 책임 게이트를 매핑, 재보정 그룹을 확정.

**작업**:
1. 13건 각각에 대해:
   - 테스트 단언의 hancom 기대치(y 범위·page-item) 추출
   - A3 실제값(`build_page_render_tree`/`dump_page_items`)과 차이 기록
   - `RHWP_EN_SSOT_DEBUG=1` + `dump-pages -p N` 로 단/쪽 경계 결정 시점의 sim bottom·발화 게이트 포착
2. 책임 게이트 매핑: `split_endnote_to_fit` / `a2_overflow_with_para` / `compact_endnote_own_vpos_span_fits`
   / `advance_large_between_single_line_rewind`(`*0.85`) / `near_bottom_tail`(`*0.90`) /
   `inline_object_formatter_overestimate` / `capped_new_endnote_advance` / `stale_forward_vpos` 중
   각 실패가 어느 분기에서 갈라지는지 확정.
3. 재보정 그룹 확정(수행계획 ①~④ 잠정안 검증·조정).

**산출물**: `mydocs/tech/investigations/issue-1370/task_m100_1370_divergence_catalog.md` (13×{기대/실제/책임게이트/그룹}),
`mydocs/working/task_m100_1370_stage1.md`. **코드 변경 없음** → 커밋은 문서만.

**완료 기준**: 13건 전부 책임 게이트가 1개 이상 매핑되고, Stage 2~4 그룹·순서가 확정.

## Stage 2 — 그룹 A 재보정 (우단/쪽 경계 계열)

**대상(잠정)**: #1·#2 (`page17 question30 우단`, `2023 pages12/13 boundary`) + Stage 1 에서 동류로
판명된 항목.

**작업**: 단/쪽 경계를 결정하는 게이트(`a2_overflow_with_para` 발화 임계, `compact_endnote_own_vpos_span_fits`,
`advance_column_or_new_page` 조건)를 정확 sim bottom 기준으로 교정. 레거시 `available*k` 상수 의존을
sim 직접 판정으로 대체(A3 한정).

**게이트**: 대상 green + 1082 5/5 + 1139 그룹 외 무회귀 + B 무회귀.

**산출물**: `mydocs/working/task_m100_1370_stage2.md` + 소스 커밋.

## Stage 3 — 그룹 B 재보정 (split / full-para 간격 계열)

**대상(잠정)**: #3·#4 (`split titles gap`, `question29 full_para gap`) + 동류.

**작업**: `split_endnote_to_fit` split 위치 계산과 `single_line_tail_split_at_bottom` 필터, full-para
뒤 `미주 사이` gap 보존 로직을 sim 기준으로 교정.

**게이트**: 대상 green + 누적(Stage 2 포함) + 1082 5/5 + B 무회귀.

**산출물**: `mydocs/working/task_m100_1370_stage3.md` + 소스 커밋.

## Stage 4 — 그룹 C 재보정 (수식·질문 tail / late titles / 좌단 tail 계열)

**대상(잠정)**: #5~#13 (1189 formula/question tail, 1284 계열 6건) + 동류. 최대 그룹.

**작업**: `large_between_tail_render_overflows`(`*0.85`)·`near_bottom_tail`(`*0.90`)·
`advance_large_between_single_line_rewind`·`internal_rewind_split` tail 게이트를 sim 기준으로 교정.
late-question-title·좌단 tail 잔류 판정을 sim bottom 으로 단일화. 그룹이 크므로 필요 시 1284 vs
1189 로 나눠 순차 적용하되 **매 수정 전 exam 동시 실행**.

**게이트**: 13건 전부 green + 1082 5/5 + B 무회귀.

**산출물**: `mydocs/working/task_m100_1370_stage4.md` + 소스 커밋.
**비고**: 비단조 cascade 로 일부가 닫히지 않으면 잔여를 명시하고 부분 green + 후속 분리를 제안한다
(임의 단일문서 튜닝 금지).

## Stage 5 — 통합 검증 + A3 기본 승격 판단

**작업**:
1. **전체 cargo test** (`--lib` 아님) A3 / B 양쪽 무회귀 확인.
2. A3 13건 + 1082 5/5 동시 green 최종 확인.
3. 회귀 가드: 재보정으로 의미가 바뀐 주석·게이트 정리, `test_measure_endnote_advance_side_effect_free` 유지.
4. **A3 기본 승격 판단**: 13건+1082+B 전부 green 이면 A3 를 기본으로 승격 가능한지 검토.
   승격은 레거시(per-para) 경로 정리를 수반하므로, 범위가 크면 **별도 후속 타스크로 분리 제안**하고
   본 타스크는 "A3 게이트 13건 재현 완료"로 종료. (승격 여부는 결과에 따라 승인 시점에 결정)

**산출물**: `mydocs/report/task_m100_1370_report.md` + 커밋. 메모리 갱신
(`tech_endnote_overflow_nonmonotonic_gate` 진전 반영).

## 단계 의존성·롤백

- Stage 2~4 는 그룹 격리지만 게이트가 상호결합 → 매 단계 전 exam 게이트로 cascade 즉시 포착.
- 한 단계가 직전 단계를 깨면 직전 게이트 조건을 더 좁혀 양립시키고, 불가 시 해당 그룹을 후속 분리.
- 그룹 경계·단계 수는 Stage 1 진단 결과에 따라 승인 하에 조정 가능(3~6 범위 유지).
