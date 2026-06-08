# 최종 결과보고서 — 미주 다단 body 초과 (M100 #1336)

- 이슈: edwardkim/rhwp#1336 (선행 #1335 조사 분리)
- 브랜치: `local/task1336` (base: stream/devel)
- 결정: **근본 정정 보류 + 바운드 회귀 테스트로 추적** (오버플로우 허용 범위 내)
- 관련: `plans/task_m100_1336.md`, `plans/task_m100_1336_impl.md`,
  `working/task_m100_1336_stage1.md`

## 1. 이슈 요지

`samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` p22 미주 2단 단 0(35 items)이
body 하단을 초과 배치(`LAYOUT_OVERFLOW`). #1335 에서 별개 버그로 분리.

## 2. Stage 1 근본원인

미주 다단 fit/accumulation(`typeset.rs` `compute_en_metrics`)이 layout 의 vpos 기반
배치보다 줄당 과소 계상 → 단 0 over-fill, 이월 미발동.
- `en_advance`(=`metric_advance_px`)가 `capped_new_endnote_advance`/`stale_forward_vpos`/
  `inline_object_formatter_overestimate` 등 **개별 exam 페이지에 하드튜닝된 캡**으로 축소.
- generic 이월 트리거(`large_between_tail_render_overflows` 등 ~6개)는 모두
  `!default_between_notes_gap` 게이트 → 이 변형(default gap)에선 비활성.

## 3. Stage 2 — 결정적 발견

1. **오버플로우가 프로젝트 자체 한계 이내**: `issue_1082` 회귀 메트릭(text baseline y가
   페이지 높이 초과분 합산) 으로 측정 시 대상 파일 **총 50.1px** (worst p22). 기존
   `REG_LIMIT_PX = 60` 이내. 다른 미주 샘플(2022/2023/2024 미주사이20·구분선아래20 등)은
   모두 0.0px.
2. **안전한 수정 불가**: under-count 의 원천(누적 캡)이 exam 페이지별로 하드튜닝되어
   있고, 모든 generic 이월 트리거가 이 변형에선 게이트로 막혀 있다. 안전망을 추가하려면
   튜닝된 캡/예측 로직을 우회·복제해야 하며, 0-overflow 인 2022/2023 exam 파일 회귀
   위험이 크다. (메모리 `tech_endnote_tail_backtrack_atomic_vs_text`,
   `tech_trailing_model_no_ssot`)

## 4. 결정 (작업지시자 승인)

**근본 정정 보류(won't-fix) + 바운드 회귀 테스트 추가.**
- fragile 미주 다단 코드는 **무수정**.
- `tests/issue_1082_endnote_multicolumn_drift.rs` 에 2024 변형 케이스 추가
  (`exam_3_09_2024_sep2020_hwp_endnote_drift_capped`): 오버플로우가 `REG_LIMIT_PX`(60px)
  이내로 유지되는지 회귀 가드. 현재 50.1px → 통과. 추후 회귀(수백 px) 시 검출.

## 5. 산출물

- 미주 다단 소스 변경 **없음**.
- `tests/issue_1082_endnote_multicolumn_drift.rs`: 2024 변형 회귀 테스트 1건 추가 (5 passed).
- 문서: 수행/구현 계획서, Stage 1 보고서, 본 최종 보고서.

## 6. 후속

- #1336: 종료(won't-fix, 허용 범위 내 + 바운드 추적).
- 미주 다단 fit/accumulation 캡의 일반화(exam별 하드튜닝 제거)는 대규모 리팩터링 과제로,
  필요 시 별도 마일스톤에서 다룬다.
