---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1251/README.md
last_verified: 2026-07-16
---

# Task M100-1251 정답 PDF 대비 시각 차이 분석

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **대상 HWP**: `samples/143E433F503322BD33.hwp`
- **정답 PDF**: `pdf-large/hwpx/143E433F503322BD33.pdf`
- **현재 출력**: `output/poc/task1251/hwp/143E433F503322BD33.svg`
- **작성일**: 2026-06-03

## 1. 결론

현재 구현은 `BinData #2` legacy OLE chart object를 generic placeholder 대신 실제 차트로 렌더한다. 그러나 정답 PDF와의 시각 차이는 남아 있다.

주요 원인은 데이터 추출 실패가 아니라 **차트 속성/스타일/object graph를 아직 파싱하지 않는 최소 renderer**라는 점이다. 현재 `OleChart` IR은 title, categories, series values/name만 담고 있으며, 축 scale, tick interval, 색상, plot area, legend 위치, font/spacing, chart border 같은 시각 속성은 복원하지 않는다.

따라서 이번 PR의 완료 기준은 “legacy OLE chart를 감지하고 최소 데이터 기반 차트로 표시한다”로 두고, 정답 PDF 수준의 pixel-level chart fidelity는 후속 이슈로 분리하는 것이 안전하다.

## 2. 비교 방법

정답 PDF를 PNG로 렌더링하고 현재 SVG도 PNG로 변환해 비교했다.

```text
/Users/melee/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/pdfinfo pdf-large/hwpx/143E433F503322BD33.pdf
/Users/melee/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/pdftoppm -png -r 144 pdf-large/hwpx/143E433F503322BD33.pdf tmp/pdfs/task1251/answer
/Users/melee/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node --input-type=module -e "... sharp SVG to PNG ..."
```

분석 시 생성한 임시 산출물 경로:

```text
tmp/pdfs/task1251/answer-1.png
tmp/pdfs/task1251/current-svg.png
tmp/pdfs/task1251/side-by-side.png
tmp/pdfs/task1251/answer-chart.png
tmp/pdfs/task1251/current-chart.png
tmp/pdfs/task1251/diff-amplified.png
```

이 임시 산출물은 PR에 포함하지 않고 작업 후 제거했다. 필요하면 위 절차로 재생성한다.

## 3. 관찰된 주요 차이

| 항목 | 정답 PDF | 현재 Rust SVG renderer | 원인 |
|---|---|---|---|
| y축 범위 | 0-2000 | 0-1702 | `svg_renderer.rs`가 최대값을 그대로 축 상한으로 사용 |
| y축 tick | 0, 500, 1000, 1500, 2000 | 0, 425.5, 851, 1276, 1702 | nice scale/tick interval 미구현 |
| 시리즈 색상 | 하늘색, 붉은색, 노란색 | 파랑, 빨강, 초록 | chart style/palette 미파싱, renderer 기본 팔레트 사용 |
| title | 자간 넓은 일반 weight | 굵은 sans-serif, 상단 밀착 | chart title style/spacing 미파싱 |
| plot area | 좌우 여백과 legend 배치가 한컴 출력에 맞음 | 일반 SVG renderer 기본 margin | chart layout box 미파싱 |
| legend | 하단 중앙, 넓은 spacing | 좌측 기준 고정 spacing | legend style/position 미파싱 |
| border | 얇은 회색 chart frame | `#cbd5e1` 기본 frame | chart area stroke style 미파싱 |
| bar width/placement | 한컴 chart engine 기준 | 간단한 grouped bar 계산 | series gap/category gap 미파싱 |

데이터 자체는 다음 값으로 안정적으로 추출된다.

```text
title: 연금 재정 전망
categories: 2010년, 2020년, 2030년, 2040년
적립금: 328, 812, 1702, 1477
수입: 50, 70, 189, 191
지출: 11, 15, 201, 289
```

## 4. 코드상 직접 원인

- `src/ole_chart/parser.rs`
  - `OleChart` IR은 `chart_type`, `title`, `categories`, `series`만 보유한다.
  - parser는 `VtDataGrid` 구간에서 label과 f64 run을 추출한다.
  - `VtChart`, axis, legend, palette, plot area, text style object는 아직 해석하지 않는다.

- `src/ole_chart/svg_renderer.rs`
  - y축 상한은 `series.values`의 최대값이다.
  - tick은 4등분 고정이다.
  - 팔레트는 renderer 상수 배열이다.
  - margin, title, legend, frame은 모두 renderer 기본값이다.

즉, 현재 차트는 “추출된 데이터의 시각화”이지 “한컴 chart object의 full fidelity replay”가 아니다.

## 5. 이번 PR에서 보강하지 않는 이유

정답 PDF에 맞춰 축 상한을 2000으로 반올림하거나 팔레트를 맞추는 작은 보정은 가능하다. 그러나 이것만 넣으면 #1251 fixture에 과적합된 휴리스틱이 되기 쉽다.

정답 수준의 시각 정합성을 안정적으로 올리려면 다음 정보가 parser/IR로 들어와야 한다.

1. axis min/max/major unit 또는 nice scale 규칙
2. series palette/style
3. chart area/plot area/legend bbox
4. title/font/spacing 속성
5. category/series gap
6. chart object graph record 구조

이는 “placeholder 제거 + 최소 chart 렌더”보다 큰 범위이며, 이번 PR의 리뷰 포인트를 흐릴 수 있다.

## 6. 후속 이슈 후보

1. `VtChart`/axis/legend/style object graph parser 확장
2. `OleChart` IR에 `OleChartStyle`, `OleAxisStyle`, `OleLegendStyle`, `OlePlotArea` 추가
3. axis nice scale 정책 추가 및 fixture별 expected ticks 고정
4. series palette 파싱 또는 한컴 기본 palette fallback 정의
5. chart visual model을 `RawSvg`에서 PageLayerTree `PaintOp` lowering으로 전환
6. PDF/SVG visual comparison harness 추가

## 7. PR 설명에 포함할 요지

- #1251의 직접 문제는 nested OLE에 preview가 없는 legacy `Contents` chart가 generic placeholder로 표시되는 것이다.
- 이번 PR은 해당 chart를 감지하고 데이터 기반 최소 차트로 렌더한다.
- 정답 PDF와의 pixel-level 시각 정합성은 아직 목표가 아니다.
- 차트 데이터는 안정적으로 추출되지만 style/object graph 파싱은 후속 작업으로 분리한다.
