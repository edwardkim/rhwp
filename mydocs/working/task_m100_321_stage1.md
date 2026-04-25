# Task #321 Stage 1 — 드리프트 origin 정량화

## 작업 내용

`src/renderer/typeset.rs::typeset_paragraph` 진입부에 `RHWP_TYPESET_DRIFT=1` env 진단 훅을 추가하여, 포맷터 기반 `fmt.total_height`와 LINE_SEG 기반 vpos_h `(last.vpos + last.lh) - first.vpos`를 비교.

## 관측 결과 (21_언어 샘플)

페이지 1 col 1 기준 주요 문단별 drift:

| pi | fmt_total | vpos_h | diff | 비고 |
|----|-----------|--------|------|------|
| 10 | 217.9 | 208.4 | +9.5 | |
| 11 | 145.3 | 135.7 | +9.5 | |
| 12 | 96.9 | 87.3 | +9.5 | |
| 13 | 24.2 | 14.7 | +9.5 | 빈 문단 |
| 14 | 36.2 | 17.3 | +18.8 | |
| 27 | 48.4 | 38.9 | +9.5 | |
| 28 | 48.4 | 38.9 | +9.5 | 오버플로우 대상 |
| 29 | 48.4 | 38.9 | +9.5 | 오버플로우 대상 |
| 30 | 48.4 | 38.9 | +9.5 | **vpos=0 (리셋)** 오버플로우 대상 |
| 31 | 48.4 | 38.9 | +9.5 | col 1에 들어가지 못해 page 2 col 0으로 |

## 해석

1. **per-paragraph diff = +9.5px**: 포맷터가 모든 문단에 **trailing line_spacing (716 HU ≈ 9.5px)**을 포함해 계산. vpos_h는 마지막 줄의 `lh` 까지만 포함하므로 누락.
2. **pi=30의 first_vpos=0**: HWP 원본 LINE_SEG가 pi=30을 **새 페이지/단의 시작 위치**로 기록. pi=29의 last_vpos=89882 에서 갑자기 0으로 리셋.
3. **col 1 누적**:
   - TypesetEngine `current_height` (fmt 합): 1223.1px < available 1226.4px → 들어간다고 판단하여 pi=30까지 배치
   - 실제 Layout `y_offset` 진행 (vpos 보정 포함): pi=28 배치 후 1445.7px → col_bottom 1436.2 초과 9.5px

## 결론

두 가지 요인이 결합:

1. **Trailing line_spacing 이중 계산**: TypesetEngine의 `current_height += total_height`에 포함된 trailing_ls는 실제 렌더링에서는 "다음 문단 spacing_before"와 겹쳐 낭비됨. 하지만 이 메트릭 자체는 **보수적**이므로(과다 계산) 과다 배치 원인은 아님.
2. **inter-paragraph vpos-reset 미탐지**: pi=30의 vpos=0 리셋은 HWP가 pi=30을 새 페이지/단에 배치한다는 의도 신호. TypesetEngine은 이를 무시하고 col 1에 쌓음.
3. **Layout vpos 보정 실패**: pi=10 이후 모든 문단의 first_vpos가 base(86794)보다 작아 Layout의 vpos 보정 경로가 꺼짐 → 순차 y_offset 사용.

## Stage 2 방향

가장 직접적인 원인은 **pi=30의 vpos=0 reset을 pagination 단계에서 강제 분할 신호로 처리하는 것**. Task #311은 동일 접근을 Paginator에서 시도했으나 회귀(19→20쪽). 본 task321은 TypesetEngine 기반이므로 동일 접근의 효과가 다를 수 있음.

Stage 2에서 적용할 규칙:
- 문단 N의 첫 line_seg `vertical_pos == 0` (단순 빈 문단 제외)
- 이전 문단 N-1의 마지막 line_seg `vertical_pos > 5000 HU` (약 1.76mm 이상)
- 단, N이 첫 구역/첫 페이지 시작이 아닐 때
→ 문단 N 배치 전 `st.advance_column_or_new_page()`.

검증:
- 21_언어 1페이지 오버플로우 3건 소거
- 총 페이지 수 15 유지
- 4개 샘플(21_언어, exam_math, exam_kor, exam_eng) 회귀 확인
