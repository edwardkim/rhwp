# Task M100-1133 완료 보고서 — 중첩 표 셀 세로 정렬/높이 산정 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/1133
- 브랜치: `local/task1133`
- 수행계획서: `mydocs/plans/task_m100_1133.md`
- Stage 보고서: `mydocs/working/task_m100_1133_stage1.md`

## 1. 문제 요약

`samples/issue_1133.hwp`, `samples/hwpx/issue_1133.hwpx` page 2의 `지원서 접수` 영역에서,
외부 표 셀 안에 있는 중첩 1x1 표가 외부 셀/행 높이 계산에 충분히 반영되지 않았다.

문제 셀은 외부 표 `s0:pi=29`의 `cell[3]`이며, `valign=Center`로 설정되어 있다. 기존 계산은
마지막 빈 문단의 `line_seg.vpos + line_height`만 콘텐츠 끝점으로 사용했고, 그 문단 안의 중첩 표
실제 높이를 반영하지 못했다. 결과적으로 외부 셀 콘텐츠 높이가 작게 산정되어 아래 행과 겹치는
배치가 발생했다.

## 2. 수정 내용

### 중첩 표 콘텐츠 높이 반영

- `src/renderer/height_measurer.rs`
  - `cell_nested_controls_bottom()` 추가
  - 셀 행 높이 측정과 `MeasuredCell.total_content_height` 산정에 중첩 표 실제 끝점을 반영
- `src/renderer/layout/table_layout.rs`
  - `calc_nested_controls_bottom_height()` 추가
  - 셀 일반 레이아웃과 세로 정렬 계산에 같은 콘텐츠 높이 기준 적용
- `src/renderer/layout/table_partial.rs`
  - partial table 셀 세로 정렬에서도 중첩 표 높이 반영

적용 기준:

```text
cell content bottom = max(existing line/vpos height, paragraph vertical_pos + nested_table_actual_height)
```

### HWPX 빈 앵커 표-표 간격 정합

1차 시각 판정 후 HWPX에서 `s0:pi=28` 표와 `s0:pi=29` 표가 HWP보다 붙어서 출력되는 차이가 발견되었다.
정답은 HWP 출력이다.

- 기존 HWPX: `s0:pi=29 ci=0 8x2 y=431.5`
- 보정 후 HWPX: `s0:pi=29 ci=0 8x2 y=444.3`
- HWP 기준: `s0:pi=29 ci=0 8x2 y=444.3`

원인은 Task #1147의 HWPX 빈 앵커 TopAndBottom 비-TAC 표 예외였다. `표 → 일반 문단`에서는
`host_line_spacing` 억제가 필요하지만, 이번처럼 `빈 앵커 표 → 빈 앵커 표`가 연속될 때는 첫 표의
`line_spacing=960HU`가 실제 표-표 간격 역할을 한다.

- `src/renderer/typeset.rs`
  - 다음 문단도 빈 앵커 TopAndBottom 표이면 HWPX에서도 `host_line_spacing`과 `spacing_before`를 보존
- `src/renderer/layout.rs`
  - 실제 SVG 배치의 표 아래 gap도 typeset과 동일 조건으로 정합

## 3. 테스트

- `cargo test --test issue_1133_nested_table_valign`
- `cargo test --lib`
- `cargo test --test issue_1079_picture_pushdown_vpos`
- `cargo test --test svg_snapshot`
- `git diff --check`

신규 테스트:

- `tests/issue_1133_nested_table_valign.rs`
  - HWP/HWPX 모두 중첩 표 실제 높이가 legacy line segment bottom보다 충분히 큰지 검증
  - HWP/HWPX page 2 debug marker의 `pi=28 → pi=29` 표 간격이 동일한지 검증

## 4. 시각 판정 산출물

중첩 표 높이 보정:

- `output/poc/task1133/fixed-hwp/issue_1133_002.svg`
- `output/poc/task1133/fixed-hwp/issue_1133_002.png`
- `output/poc/task1133/fixed-hwpx/issue_1133_002.svg`
- `output/poc/task1133/fixed-hwpx/issue_1133_002.png`
- `output/poc/task1133/reference-hwpx-page2.png`

HWPX 표-표 간격 보정:

- `output/poc/task1133/fixed-spacing-hwp/issue_1133_002.svg`
- `output/poc/task1133/fixed-spacing-hwp/issue_1133_002.png`
- `output/poc/task1133/fixed-spacing-hwpx/issue_1133_002.svg`
- `output/poc/task1133/fixed-spacing-hwpx/issue_1133_002.png`

작업지시자 시각 판정:

```text
2026-06-07 통과
```

## 5. WASM 빌드

다음 명령으로 rhwp-studio 검증용 WASM 번들을 갱신했다.

```bash
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
Done in 3m 00s
Your wasm pkg is ready to publish at /app/pkg.
```

## 6. 결론

#1133의 핵심 문제였던 중첩 표 높이 산정과 가운데 세로 정렬 배치가 보정되었다. 추가로 발견된
HWPX 빈 앵커 표-표 간격 차이도 HWP 기준과 맞췄다. 공개 샘플 HWP/HWPX 양쪽에서 시각 판정을
통과했고, 관련 회귀 테스트와 WASM 빌드도 완료했다.
