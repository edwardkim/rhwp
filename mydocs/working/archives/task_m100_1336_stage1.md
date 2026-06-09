# Stage 1 보고서 — 미주 다단 초과 근본원인 (M100 #1336)

구현계획서: `mydocs/plans/task_m100_1336_impl.md`

## 1. 재현

`samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` p22(global_idx=21) 미주 2단,
단 0(35 items)이 body 하단(1092.3)을 초과하여 para 1156~1159 가 y=1102~1174 까지 배치.
`LAYOUT_OVERFLOW` 경고 4건(최대 +82px).

## 2. 불일치 정량

- typeset 누적: 단 0 `used=1010.7px` (bottom ≈ 90.7+1010.7 = 1101.4)
- layout 렌더: 단 0 최하단 ≈ 1174 → **layout 이 ~73px 더 길게 배치**
- 단 0 의 빈/단줄 미주 문단이 많고(1128·1131·1139·1141·1144·1145·1147·1148·1151·1153·
  1155·1158 등 다수 "(빈)"), 줄당 ~2px(line_spacing 분) 과소계상이 35 items 에 누적.

## 3. 근본원인 (코드 위치)

`src/renderer/typeset.rs` 미주 흐름의 `compute_en_metrics`(~2519~2618) 가 미주 다단 fit/
누적 높이를 산출하는데:

- `fit = (metric_advance_px - trailing_ls_px).max(min_h)` — **trailing_ls 차감**.
- `metric_advance_px` 는 휴리스틱(`capped_new_endnote_advance`, `stale_forward_vpos`,
  `inline_object_formatter_overestimate`, `compact_local_rewind`)으로 vpos 실제 advance
  보다 작게 캡되는 경우가 있음.
- 이 값들로 `st.current_height` 가 누적되어 typeset 은 35 items 가 들어간다고 판단하나,
  layout 은 vpos 기반(full spacing)으로 배치해 ~73px 더 길어져 단 0 가 over-fill,
  잔여 항목을 단 1/다음 페이지로 이월하지 못함.

## 4. 핵심 제약 — 극도 fragile 하드튜닝 영역

해당 코드는 **바로 이 문서군에 맞춰 하드코딩 튜닝**됨:

- `en_ref.number 29|30` 분기(`current_default_late_question_title` 등)
- 주석에 `3-09월_교육_통합_2022.hwp 9쪽 문5)`, `2023 12쪽`, `7mm/20mm 미주 구분선 프로파일`
  등 특정 파일·페이지·문항 명시
- `compact_endnote_separator_profile`, `internal_rewind`, `large_vpos_jump` 등 다수의
  특수 케이스
- 메모리 경고: `tech_endnote_tail_backtrack_atomic_vs_text`, `tech_trailing_model_no_ssot`
  (미주 trailing 전면 통일 금지)

→ fit/accumulation 산식을 건드리면 exam 미주 샘플(2022/2023/2024 변형) 회귀 위험 매우 큼.

## 5. 수정 옵션

### 옵션 1 — 안전망 클램프 (범위 최소, 권장)
fit/accumulation 산식은 불변. 미주 다단 컬럼에서 **layout 기준 누적 vpos bottom 이
available + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE 를 초과하면 잔여 항목을 다음 컬럼/
페이지로 이월**하는 최종 안전망만 추가. 기존 튜닝 경로는 보존하고 over-fill 만 차단.
- 위험: 그래도 어느 항목이 넘어가느냐가 exam 페이지 배치를 바꿀 수 있어 전수 회귀 필요.

### 옵션 2 — 보류(won't-fix)
cosmetic(1 페이지 하단 여백 bleed) 대비 회귀 위험 과대. 현행 유지.

## 6. 권고

옵션 1(안전망 클램프)을 **exam 미주 샘플 전수 회귀 게이트** 하에 시도하되, 회귀 발생 시
옵션 2(보류)로 전환. 작업지시자 결정 요청.
