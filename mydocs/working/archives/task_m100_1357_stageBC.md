# Stage B/C 보고서 — Task #1357 정밀 수정 시도 + 회귀 검출 → 롤백

## Stage B — 구현 (col0 tail 렌더-y 예측 advance)
`typeset.rs` 미주 루프에 default-between-notes col0 tail(ep_idx>0) 용 HeightCursor
시뮬레이션 기반 렌더-y 예측 블록 추가. 예측 y + 줄높이가 단 하단 초과 시
`advance_column_or_new_page`. 가장 좁은 게이트(default_between_notes_gap && col0 &&
current_height>0.85·avail && !rewind && split=None && visible) 적용.

## Stage C — 검증 결과: **회귀 발생 → 롤백**

| 항목 | 베이스라인 | 수정 후 | 판정 |
|------|-----------|---------|------|
| 대상 p21 col0 LAYOUT_OVERFLOW | 4건(82px) | **0** | 해소 |
| **다른 페이지 신규 오버플로** | 없음 | **page16 col1(75px), page18 col1(69px), page19 col1(34px)** | **회귀** |
| 전 문서 오버플로 총합 | 50.1px | **72.9px (악화)** | **회귀** |
| issue_1082 (2024 sep20/20) | pass(≤60) | **FAIL(72.9)** | **회귀** |
| issue_1082 타 exam 4종 | pass | pass | 유지 |

## 근본 분석 (실증)
오버플로의 원인은 typeset 누적기 `current_height` 의 **전역 과소누적**(모든 단이 약간씩
과충전). col0 tail 을 다음 단으로 넘기면 그 과충전이 **col1 로 전이**되어, 다른 페이지
col1 이 새로 오버플로한다(p21 해소↔16/18/19 col1 신규). **advance 로는 문제가 이동만
할 뿐 해소되지 않음**이 실증되었다.

→ 안전한 정정은 **누적기를 layout 실제 렌더와 정합**(advance 가 아니라 accumulation
정확도) 하는 것이며, 이는 #1082 가 깊게 다루고 본 변형에서 잔여를 남긴 영역이다.

## 조치
- **롤백 완료**(롤백 기준: 어느 exam 페이지든 악화 시 즉시 revert). 베이스라인 복구 확인
  (page21만, issue_1082 5 passed).
- **수확**: Stage A 시각 회귀 하니스 + issue_1082 가 cascade 회귀를 정확히 검출 →
  결함 있는 변경의 착지를 차단. 회귀 세트의 가치 입증.

## 권고 (갱신)
contained advance 수정은 **실증적으로 불가**(cascade). 남은 경로는 **typeset 누적기
정합(deep #1082)** — 본 변형의 col0 미주 누적이 layout 대비 ~91px 적은 원인을 찾아
누적식을 정정하는 전용 작업. 난도·범위 큼. 현 잔여(50.1px<60px 바운드)는 1줄 수준이라,
누적기 정합 전용 타스크로 분리하거나 바운드 유지를 권고.
