# Task M100-1133 Stage 1 — 중첩 표 높이 산정 진단 및 1차 보정

## 재현

대상 샘플:

- `samples/issue_1133.hwp`
- `samples/hwpx/issue_1133.hwpx`
- `pdf-large/hwpx/issue_1133.pdf`

기준 페이지:

- page 2, `4. 전형절차`
- `지원서 접수` 행의 오른쪽 셀 내부 `※ 지원시 유의사항` 중첩 1x1 표

baseline 출력:

- `output/poc/task1133/baseline-hwp/issue_1133_002.svg`
- `output/poc/task1133/baseline-hwpx/issue_1133_002.svg`

## 원인

외부 표 `pi=29`의 `cell[3]`은 다음 구조이다.

- 외부 셀: `r=1,c=1`, `valign=Center`
- 텍스트 문단 5개 뒤에 중첩 1x1 표가 있는 빈 문단 `p[5]`
- `p[5]`의 line segment: `vpos=8552`, `lh=1400`
- 중첩 표 내부 cell 저장 높이: `282 HU`
- 실제 중첩 표 내부 문단은 5개, 여러 줄로 구성됨

기존 계산은 중첩 표가 있는 문단의 끝 위치를 `line_seg.vpos + line_height` 또는 단순 line height 합으로만 잡았다. 이 때문에 중첩 표의 실제 높이가 외부 셀 콘텐츠 높이에 충분히 반영되지 않았고, `valign=Center` 계산 및 행 높이 계산이 작아진 콘텐츠 높이를 기준으로 수행되었다.

## 보정

다음 경로에서 중첩 표 실제 끝점을 반영했다.

- `src/renderer/height_measurer.rs`
  - `cell_nested_controls_bottom()` 추가
  - 행 높이 측정과 `MeasuredCell.total_content_height` 보정에 적용
- `src/renderer/layout/table_layout.rs`
  - `calc_nested_controls_bottom_height()` 추가
  - 일반 셀 콘텐츠 높이/세로 정렬 높이에 적용
- `src/renderer/layout/table_partial.rs`
  - partial table 셀 세로 정렬 콘텐츠 높이에 같은 보정 적용

보정 기준:

```text
cell content bottom = max(existing line/vpos height, paragraph vertical_pos + nested_table_actual_height)
```

## 산출물

1차 수정 후 출력:

- `output/poc/task1133/fixed-hwp/issue_1133_002.svg`
- `output/poc/task1133/fixed-hwp/issue_1133_002.png`
- `output/poc/task1133/fixed-hwpx/issue_1133_002.svg`
- `output/poc/task1133/fixed-hwpx/issue_1133_002.png`
- `output/poc/task1133/reference-hwpx-page2.png`

## 1차 판정

- HWP/HWPX 모두 `지원시 유의사항` 중첩 표가 아래 행과 겹치지 않음
- HWPX fixed 출력은 제공 PDF page 2의 주요 배치와 유사함
- HWP/HWPX의 page 2 partial row range 차이는 baseline에서도 존재한 포맷별 페이지네이션 차이로 관찰됨

## 추가 발견 — HWPX 빈 앵커 표-표 간격

작업지시자 시각 판정 중 `s0:pi=28` 표와 `s0:pi=29` 표 사이 간격이 HWP/HWPX에서 다르게
출력되는 것을 확인했다. 정답은 HWP 출력이다.

관찰:

- HWP: `s0:pi=28 ci=0 3x6 y=323.7`, `s0:pi=29 ci=0 8x2 y=444.3`
- HWPX: 기존에는 `s0:pi=29 ci=0 8x2 y=431.5`
- 차이 `12.8px`는 `pi=28` 호스트 문단의 `line_seg.line_spacing=960HU`와 일치

원인:

- Task #1147에서 HWPX 빈 앵커 TopAndBottom 비-TAC 표의 `host_line_spacing`을 0으로
  억제하는 예외를 도입했다.
- 이 예외는 `표 → 일반 문단` 경로에서는 필요한 보정이지만, 이번 샘플처럼 `빈 앵커 표 →
  빈 앵커 표`가 연속되는 경우 첫 표의 trailing line spacing이 실제 표-표 간격 역할을 한다.

보정:

- `src/renderer/typeset.rs`
  - 다음 문단이 빈 앵커 TopAndBottom 표인지 검사
  - 다음 문단도 표 앵커이면 HWPX라도 `host_line_spacing`과 `spacing_before`를 HWP처럼 보존
- `src/renderer/layout.rs`
  - 실제 SVG 배치의 표 아래 gap 산식도 typeset과 동일 조건으로 정합
- `tests/issue_1133_nested_table_valign.rs`
  - HWP/HWPX page 2 debug marker의 `pi=28 → pi=29` 표 간격이 같아야 한다는 회귀 테스트 추가

추가 산출물:

- `output/poc/task1133/fixed-spacing-hwp/issue_1133_002.svg`
- `output/poc/task1133/fixed-spacing-hwp/issue_1133_002.png`
- `output/poc/task1133/fixed-spacing-hwpx/issue_1133_002.svg`
- `output/poc/task1133/fixed-spacing-hwpx/issue_1133_002.png`

검증:

- `cargo test --test issue_1133_nested_table_valign`
- `cargo test --lib`
- `cargo test --test issue_1079_picture_pushdown_vpos`
- `cargo test --test svg_snapshot`
- `git diff --check`
