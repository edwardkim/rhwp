# task_m100_1660 최종 결과보고서 — C1b 분산형(scatter) 차트 렌더링

- 이슈: **#1660** (C1b, #1431 Track C 하위)
- 브랜치: `local/task1660` (from `local/devel`)
- 마일스톤: M100
- 상태: 구현·검증 완료, 시각 정합 작업지시자 확인 완료

## 1. 목표 / 결과

분산형 차트 5종이 `c:xVal`/`c:yVal` 미파싱 + `scatterChart` 미인식으로 "차트 (미지원)" placeholder로
렌더되던 문제를 해소하여 **렌더 커버리지**(placeholder 0건) + 합리적 스타일을 달성했다.
(픽셀 parity는 1차 목표 아님 — #1251.)

| 항목 | 결과 |
|---|---|
| 분산형 5종 × {hwpx, hwp} = 10파일 | placeholder 0건, 정상 차트 렌더 |
| 데이터·기하 | 한컴 2022 정답지와 일치 |
| 스타일 | scatterStyle로 표식/직선/곡선 4종 구분 |
| 축 | X·Y 수치축, 0-baseline, 소수 라벨 |

## 2. 설계 결정

- **IR: `OoxmlSeries` 확장** (`x_values: Vec<f64>` 추가, `values`=Y 유지). 신규 타입 분리는 현재 소비처 없음 →
  bubble·stock HLC가 등장하는 **C2 시점**에 `SeriesData` 합타입으로 재평가. legend/색상/이름/`is_combo` 전부 재사용.
- **스타일: `c:scatterStyle` 단독 구분** — marker/line/lineMarker/smoothMarker(곡선 2종은 동일 XML이라 동일 렌더).
  `c:smooth`/`c:marker` 파싱 불요(실측 확인).
- **0-baseline (C1b 편입)**: 한컴 PDF가 축을 0부터 그림을 확인 → `scatter_range`에 `min>0→0` clamp.
  막대/선 축(`value_range_for`)이 이미 하던 동작과 **일관**시킨 것이라 새 스타일 작업 아님.

## 3. 단계별 요약

| 단계 | 내용 | 커밋 |
|---|---|---|
| Stage 1 | 모델: `OoxmlChartType::Scatter`, `ScatterStyle`, `OoxmlSeries.x_values`, `OoxmlChart.scatter_style` | `24dc238e` |
| Stage 2 | 파서: `scatterChart`/`xVal`/`yVal`/`scatterStyle` + 축 분류 가드(보조축 오분류 차단) + 테스트 5 | `d66207dd` |
| Stage 3 | 렌더러: `render_scatter`(2 수치축·점/직선/Catmull-Rom 곡선/표식), `format_axis_num`+`render_value_grid(decimal)`, `scatter_range` + 테스트 5 | `6ca389fc` |
| Stage 4 | 통합 테스트(`issue_1431_scatter.rs`, 10파일) + 시각 검증 → 0-baseline 튜닝 + 테스트 | `773423b7` |

수정 파일: `src/ooxml_chart/{mod,parser,renderer}.rs` + 신규 `tests/issue_1431_scatter.rs`.
shape_layout 등 배선 무수정(자동 흐름). 무관 rustfmt churn 없음.

## 4. 검증

```
cargo test -p rhwp ooxml_chart::          → 32 passed, 0 failed (scatter parser 5 + renderer 6 포함)
cargo test --test issue_1431_scatter      → 1 passed (10파일 placeholder 회귀 가드)
cargo test (전체)                          → 0 failed
cargo clippy --all-targets -- -D warnings  → 경고 0
```

**시각 검증** (`output/poc/c1b_scatter/` SVG/PNG ↔ `pdf/chart/분산형/*-2022.pdf`):

| 샘플 (scatterStyle) | 렌더 | 정답지 정합 |
|---|---|---|
| 표식만 (marker) | 점만 | 데이터·기하 일치 |
| 직선이있는 (line) | 직선만 | 일치 |
| 직선및표식 (lineMarker) | 직선+점 | 일치 |
| 곡선이있는 / 곡선및표식 (smoothMarker) | Catmull-Rom 곡선+점 | 일치 |

- 축: X 0~3 (정답지 정확 일치), Y 0~4. 작업지시자 studio(최신 WASM) 시각 확인 완료.

## 5. 잔여 (스타일 4갭, C1c 이관 — 전 13종 공통)

① 제목 누락 ② 팔레트(녹색-우선 vs 한컴 파랑/주황) ③ 범례 하단 vs 우측
④ **Y축 max nice-scale headroom** (현 0~4 vs 한컴 0~5).

> ④는 공용 `nice_range` 정책 갭으로 막대·선도 동일하게 data max에 붙는다(`[1,2,3]`→max 3, headroom 0).
> 작업지시자 결정(2026-06-29)에 따라 **scatter만 따로 고치지 않고 C1c에서 전 차트 종류 일괄 보정**한다.

## 6. 후속

- 본 보고서 승인 후 origin(fork) push → upstream `devel` PR (`Refs #1431`, `#1660`).
- C1c: 스타일 4갭(제목·팔레트·범례 위치·Y축 nice-scale headroom) 전 차트 공통 보정.
- #1456: studio 캔버스 rawSvg 첫로드 공백(별도 인프라 결함, 본 작업과 무관).
