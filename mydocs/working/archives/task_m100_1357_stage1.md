# Stage 1 조사보고서 — Task #1357 (정밀 수정 시도 → 근본원인 확정)

## 계측 결과 (p21 col 0, en_para_idx 1156~1159)

| pi | ep_idx | col | cur_h | avail | ratio | dbn_gap | en_fit | split |
|----|--------|-----|-------|-------|-------|---------|--------|-------|
| 1156 | 27 | 0/2 | 920.7 | 1001.6 | 0.92 | **true** | 12.0 | None |
| 1157 | 28 | 0/2 | 938.7 | 1001.6 | 0.94 | true | 14.4 | None |
| 1158 | 29 | 0/2 | 959.1 | 1001.6 | 0.96 | true | 27.6 | None |
| 1159 | 30 | 0/2 | 992.7 | 1001.6 | 0.99 | true | 12.0 | None |

- 이 미주는 `default_between_notes_gap = true`(작은 between-notes; "구분선아래/위20"은
  **구분선 마진**이 20mm이고 between-notes 는 별개로 작음)
- **typeset `current_height`(920.7)가 실제 layout 렌더 위치(col top 기준 ~1011px)보다
  ~91px 과소누적**. en_fit(12px)이 작아 fit 판정 → 배치 → 본문 하단 초과.

## 근본 원인 확정
미주 다단 흐름에서 typeset 누적기(`current_height`, vpos-delta 기반)가 layout 의 실제
렌더 y(HeightCursor lazy_base + 줄간격 + between-notes 보정 반영)보다 누적 ~91px 적게
쌓인다. 누적이 정확하면 `current_height + en_fit > available` 로 기존 split/advance 가
정상 발화하나, 과소누적으로 발화하지 못한다. (#1082 vpos-delta↔layout 정합의 잔여)

## 정밀 수정 실현 가능성 분석

### 후보 수정
정확한 렌더-y 예측이 필요. 본 함수는 케이스별로 **HeightCursor 시뮬레이션(~90줄)**을
중복 배치해 예측한다(`large_between_title_tail_render_overflows` 2824,
`large_between_last_column_visual_split` 3001 등). default-between-notes 의 **col0
tail(ep_idx>0)** 케이스만 이 정확한 예측이 없어, 동형 블록을 1개 더 추가하면 발화 가능.

### 위험 (보류 사유)
- 회귀 가드 exam 2022/2023/2011 도 **default_between_notes** 라 새 예측/advance 가
  그 문서들의 col0 에도 평가된다.
- 위험 회귀는 **과소충전(under-fill)** — 한컴이 채우는 col0 를 우리가 일찍 넘김. 이는
  `issue_1082` 의 **오버플로 px 메트릭으로 잡히지 않는다**(넘김은 오버플로를 줄임).
  → 테스트 통과가 무회귀를 보장하지 못함. exam 다문서 **페이지별 시각 비교**가 필수.
- 즉 #1336 이 "exam별 하드튜닝"으로 보류한 본질과 동일.

## 결론·권고
정밀 수정 경로는 확인됐으나(동형 예측 블록 추가), 안전한 착지는 **exam 4종 전 페이지
시각 회귀 검증**을 동반하는 전용 작업이어야 한다(오버플로 바운드만으로는 under-fill
회귀 미포착). 현 잔여는 1줄/50.1px 로 `issue_1082` 60px 바운드 이내.

- **권고**: 본 변경을 단독·성급히 착지시키지 말 것. 전용 시각 검증 세트(exam별 기준
  PDF 페이지 매칭) 구축 후 진행하거나, 바운드 유지.
- 본 보고서로 근본 원인·수정 지점·검증 요건을 확정 기록(다음 착수 시 재조사 불필요).
