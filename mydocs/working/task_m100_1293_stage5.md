# Task 1293 Stage 5: 미주 LINE_SEG/vpos 흐름 분석

## 목적

Stage4에서 공식 미주 구분선 시작 높이를 typeset 누적값에 반영했지만, visual sweep의 잔여
frame/red/line flags는 줄지 않았다. 따라서 이번 단계는 구분선 자체가 아니라 미주 내용 내부의
LINE_SEG/vpos 흐름, `미주 사이` 소비 위치, 단 이월 fit 판정과 누적 갱신의 차이를 분석한다.

추가로 작업지시자가 확인한 것처럼, 한컴 UI에서 `구분선 위`를 0으로 바꾸어도 rhwp가 기존
20mm 값을 계속 적용하면 공식 미주 모양 모델을 구현했다고 볼 수 없다. 따라서 `구분선 위=0`
같은 설정 변경이 raw/model/render/API에 모두 반영되는지도 이번 goal의 필수 범위로 포함한다.

## 공식 기준

- `구분선 위`: 본문과 미주 구분선 사이 간격이다.
- `구분선 아래`: 미주 구분선과 첫 미주 내용 사이 간격이다.
- `미주 사이`: 앞 번호 미주 내용 끝과 다음 번호 미주 내용 시작 사이 간격이다.

이 값들은 서로 대체할 수 없다. 특히 `미주 사이`는 같은 번호 미주 내부 줄 사이 gap이나
단/페이지 이월 보정값으로 소비하면 안 된다.

## 참고한 위험 신호

- `typeset_layout_drift_analysis.md`
  - fit 판단과 layout 누적 y가 분리되면 하단 piling/overlap이 발생한다.
- `typeset_fit_accumulation_drift.md`
  - 마지막 줄 spacing은 fit에서 제외될 수 있지만 다음 항목 시작 위치 계산에는 필요한 경우가 있다.
- `2010_01_06_footnote_line_spacing.md`
  - footnote/endnote 내부 paragraph transition gap 누락은 실제 줄간격 불일치로 나타난다.
- `hwpx_lineseg_reflow_trap.md`
  - HWPX lineSeg/reflow 표현은 HWP와 다를 수 있으므로 샘플 간 교차 검증이 필요하다.

## 분석 계획

1. `dump-pages`와 `summary.json`에서 `used`/`hwp_used` 차이가 큰 페이지를 우선순위로 잡는다.
2. 해당 페이지의 render tree bbox와 lineSeg 기반 paragraph 흐름을 비교한다.
3. `미주 사이`가 번호 경계에서만 적용되는지, paragraph 내부 line spacing으로 중복 적용되는지 확인한다.
4. fit 판정에는 제외하고 실제 누적에는 포함해야 하는 trailing advance가 빠진 지점을 찾는다.
5. 공식 의미로 설명 가능한 작은 보정만 구현하고 focused test + sweep으로 회귀 여부를 확인한다.

## Stage5-A: 구분선 위 0 적용 누락

### 원인

`3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`의 HWP5 raw 값은 다음과 같다.

- `separator_margin_top=0`
- `separator_margin_bottom=5669` (20mm)
- `note_spacing=5669` (20mm)

HWPX 조합 샘플은 같은 UI 값을 `separator_margin_top=5669`, `separator_margin_bottom=0`으로
저장한다. 그래서 Stage3에서 `separator_above_margin_hu()`가 top이 0이면 HWP5 fallback 슬롯인
`separator_margin_bottom`을 읽도록 정규화했다.

하지만 rhwp UI/API에서 `separatorMarginTop=0`을 적용할 때는 `separator_margin_top`만 0으로
갱신했다. HWP5 fallback 슬롯의 20mm가 그대로 남기 때문에 getter/render가 다시 20mm로
정규화하는 문제가 있었다.

### 수정

`apply_endnote_shape_native()`에서 `separatorMarginTop`을 받으면 HWPX 슬롯과 HWP5 fallback 슬롯을
함께 갱신하도록 했다. 이제 `구분선 위=0` 적용 시 fallback 슬롯도 0이 되며, `구분선 아래`인
`note_spacing`은 그대로 보존된다.

### 회귀 테스트

`issue_1139_endnote_shape_api_clears_hwp5_separator_above_fallback_slot`을 추가해 다음을 확인한다.

- 적용 전 HWP5 조합 샘플은 fallback raw 슬롯으로 `구분선 위=20mm`를 정규화한다.
- `applyEndnoteShape({"separatorMarginTop":0})` 후 `separator_above_margin_hu()==0`이다.
- `separator_margin_bottom` fallback raw 슬롯도 0으로 지워진다.
- 공식 `구분선 아래=20mm`는 유지된다.

## 우선 조사 대상

- `2024-09-between20` 12쪽
  - Stage4 기준 `frame_overflow_pages: [12]`
  - 구분선/미주 사이 설정이 다른 샘플보다 크고, 단 하단에서 flow drift 후보가 남아 있다.
- `2024-09-below20-above20` 9쪽
  - `구분선 위=20mm`, `구분선 아래=20mm` 조합 샘플의 첫 endnote-heavy 페이지다.
- `2024-09-below20` 22쪽
  - `hwp_used`가 과도하게 크게 잡히는 페이지로, LINE_SEG page-local 기준 처리 여부를 확인한다.

## 검증 대기

- `cargo fmt --all -- --check` — 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_shape_api_clears_hwp5_separator_above_fallback_slot -- --nocapture` — 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` — 52 passed
- `task1274_visual_sweep.py` focused target 재실행

## Stage5 판단

`구분선 위=0`이 적용되지 않는 원인은 renderer가 아니라 API/model 적용 경로의 raw fallback 잔류였다.
해당 기능은 공식 미주 모양 모델의 필수 조건이므로 Stage5에서 먼저 고쳤다.

잔여 visual flags와 하단 overwrap은 계속 남아 있으며, 다음 스테이지에서 LINE_SEG 되감기 문단의
분할/단 이월 판단을 이어서 분석한다.
