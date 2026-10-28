---
kind: working
status: active
issue: 5590
---

# 행별 선언 열 구획 보존 (#5590)

작업 브랜치: `fix/5590-per-row-declared-columns`
대상: `src/renderer/layout/border_rendering.rs` · `src/renderer/layout/table_layout.rs` ·
`tests/cases/issue_5590_per_row_column_widths.rs` · `samples/issue5590_per_row_column_widths.hwpx`

## 한 줄

행마다 다른 열 구획을 선언한 표를 **전역 열 grid 하나**로 그리면서, 전역 grid 가 표 선언
폭과 어긋난 표에서 어느 행의 셀 폭이 원본과 달라졌다. 그런 표에 한해 **자기 구획을 완결한
행**은 선언 구획대로 그린다.

## 이슈가 요구한 것

- 00288(약장 배치표): 모든 행의 셀 폭 합 = 표 폭인데도 마지막 열이 1,006HU(13.4px) 깎여
  표 안에 빈 띠가 남고 격자가 어긋난다.
- 코퍼스 표본 1,255표 중 62표(4.9%) · 38문서(14.2%)가 같은 축.

## 원인

`resolve_column_widths` 는 전역 열 폭 벡터 하나를 만든다. 행별 선언 구획이 서로 어긋나면
어느 행인가는 반드시 진다. 앞 열들이 다른 행 기준으로 풀리고 남은 폭이 마지막 열로 몰리면
그 열만 좁아진다(= 보고된 증상).

`build_row_col_x` 에 행별 독립 grid 경로가 이미 있으나 두 군데서 막힌다.

1. Studio 런타임 힌트(`local_resize_rows`) 또는 그 추론(`inferred_local_resize_rows`)이
   있는 표만 대상이다.
2. 폴백 경로의 `cell_width_grid` 는 **`col_span == 1` 셀만** 담는다. 00288 처럼 병합 셀로만
   이루어진 행은 재구성되지 않아 전역 grid 로 되돌아간다.

## 수정

`build_row_col_x` 에 행별 선언 구획 경로를 하나 더 둔다(`declared_row_col_x`).
한 행이 (1) 0열부터 빈틈없이 (2) 마지막 열까지 덮고 (3) 선언 폭 합이 표 폭과 일치하면,
그 행은 자기 구획을 완결한 것으로 보고 선언대로 x 경계를 세운다. 병합 셀은 폭 그대로 쓰고,
안쪽 열 경계만 span 비율로 나눈다(기존 local-resize 경로와 같은 규약).

게이트 둘을 함께 둔다.

- **전역 grid 가 표 선언 폭과 이미 맞는 표는 건드리지 않는다.** 이 결함은 전역 grid 가
  선언 폭과 어긋난 표에서만 나타난다. (근거: 아래 form-002 판정)
- Studio 명시 힌트(`local_resize_rows`) 표는 종전 경로 유지 — 그쪽은 `local_resize_cell_widths`
  라는 별도 폭 원본을 쓴다.

표 상자 폭도 함께 맞춘다(`table_layout`): 모든 행이 선언 폭에 맞춰 완결됐고 전역 col_x 합이
0.5px 넘게 크면, 표 폭은 선언 폭이다. 안 그러면 행 오른쪽 끝과 표 오른쪽 테두리가 어긋난다.

## 한컴 정합 판정 — form-002 (게이트를 좁힌 근거)

게이트 없이 적용한 1차 구현은 SVG 골든 4건을 깼다. 요소 단위로 좌표를 정규화해 비교하니
**실질 변경은 단 1건**이고 나머지는 부동소수 끝자리 표기 차이였다.

```
form-002/page-0
  golden : <line x1="75.59" y1="581.20" x2="258.04" …/>
  1차 수정: <line x1="75.59" y1="581.20" x2="146.21" …/>   ← 부분 가로선이 짧아짐
```

한컴 오라클(`pdf/hwpx/form-002-2022.pdf`)을 SVG 로 변환해 그 부분 가로선을 실측했다.

```
한컴 수평선 (pt → px = pt/0.75)
  y=402.6pt → 536.8px, x 59.4..196.5pt → 79.2..262.0px   ← 부분 가로선 (폭 182.8px)
  y=423.1pt, y=466.6pt → 전폭 79.2..714.8px
rhwp golden 부분 가로선: x 75.6..258.0px (폭 182.5px)
```

폭이 182.8 vs 182.5 로 사실상 일치한다 — **한컴도 이 선을 길게 그린다.** 즉 1차 구현이
한컴 정합을 깼다. 그래서 "전역 grid 가 선언 폭과 어긋난 표에서만" 게이트를 추가했고, 그
뒤 골든 8건 전부 바이트 동일로 통과한다.

## 재현 픽스처

`samples/issue5590_per_row_column_widths.hwpx` (4.4KB, 합성). 6열 2행, 표 폭 36,000HU(480px).
두 행 모두 선언 폭 합 = 480px 로 자기 구획을 완결하지만 구획이 서로 어긋난다.

| 행 | 선언 |
|----|------|
| row0 | c0(span2) 160 · c2 80 · c3 80 · c4 80 · c5 80 |
| row1 | c0 60 · c1(span2) 200 · c3 80 · c4(span2) 140 |

## 검증 실측

```
rhwp export-render-tree samples/issue5590_per_row_column_widths.hwpx
  수정 전: row0 c2 = 100.0 (선언 80) · row1 c4 = 160.0 (선언 140) · 표 상자 500.0 (선언 480)
  수정 후: 모든 셀이 선언대로 · 표 상자 480.0

rhwp export-svg  — 가로 테두리 오른쪽 끝 579.4 → 559.4 (= 79.4 + 480)
```

로컬 `samples/*.hwpx` 전수 셀 폭 대조(원본 `hp:cellSz` ↔ 렌더 `Cell.bbox.w`): 수정 전후
불일치 목록 **동일**(7건, 모두 행 합 ≠ 표 폭이라 게이트에 걸리지 않음) — 회귀 0.

## 시험 명령

```
cargo test --profile release-test --test regression_suite_030 issue_5590   # 신규 가드
cargo test --profile release-test --test regression_suite_028 svg_snapshot # 골든 8건
cargo test --profile release-test --tests --no-fail-fast                   # 전체
```

신규 가드는 수정 전 코드에서 실패(`셀 (r0, c2) 선언 80.0, 렌더 100.0`), 수정 후 통과.

## fmt 게이트

```
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

## 환경 · 한계

Linux 6.17 · rhwp v0.8.4. 원 보고 문서(admrul 00288)는 이 환경에 없어, 보고서의 표 기하로
합성 픽스처를 만들어 재현했다. 보고서에 실린 두 행만으로는 결함이 재현되지 않는다(그 둘만
쓰면 전역 grid 도 같은 답을 낸다) — 어긋남을 만드는 나머지 행이 원문서에 있다. 따라서 이
PR 은 **결함의 형상**(행별 구획 충돌 + 전역 grid 가 선언 폭과 어긋남)을 닫으며, 00288 자체
확인은 원본으로 한 번 더 하는 것이 좋다.

## PR 메모

`gh pr create --base devel --body-file ...`, 제목·본문 한국어, `closes #5590`.
