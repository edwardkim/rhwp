# Task M100-1251 Stage 7 완료 보고

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **브랜치**: `task-1251-ole-chart`
- **일자**: 2026-06-03
- **상태**: 완료

## 1. 목표

정답 PDF(`pdf-large/hwpx/143E433F503322BD33.pdf`) 대비 현재 렌더링의 시각 차이를 분석하고, 이번 PR에서 정합성 보강까지 진행할지 또는 추적 가능한 문서화에 집중할지 결정한다.

## 2. 분석 결과

차트 데이터 추출은 안정적이다.

```text
title: 연금 재정 전망
categories: 2010년, 2020년, 2030년, 2040년
적립금: 328, 812, 1702, 1477
수입: 50, 70, 189, 191
지출: 11, 15, 201, 289
```

시각 차이는 주로 다음 원인에서 발생한다.

1. y축 nice scale 미구현: 정답은 0-2000/500 tick, 현재는 0-1702/4등분 tick
2. chart palette 미복원: 정답은 한컴 기본색, 현재는 Rust renderer 기본 palette
3. chart object style 미파싱: title spacing, legend 위치, plot margin, border, bar gap
4. parser IR 범위 제한: `OleChart`가 아직 데이터 중심 필드만 포함
5. 일반 문서 layout 차이: chart 외에도 일부 column/text flow 차이가 존재

## 3. 결정

이번 PR에서는 정답 PDF 수준의 시각 정합성 보강을 추가로 진행하지 않는다.

이유:

- 이번 PR의 핵심은 preview가 없는 legacy OLE chart를 generic placeholder 대신 렌더하는 것이다.
- 시각 정합성 보강은 axis/style/object graph parser 확장까지 포함하는 별도 큰 작업이다.
- 단기 휴리스틱으로 정답 PDF에 맞추면 #1251 fixture에 과적합될 위험이 있다.
- 업스트림 리뷰에서는 기능 추가 범위, renderer 선택 이유, known visual gap을 명확히 공유하는 편이 더 추적 가능하다.

## 4. 산출물

- `mydocs/tech/investigations/issue-1251/hwp_ole_chart_visual_diff_against_hancom_pdf_1251.md`
- `mydocs/tech/hwp_ole_chart_renderer_architecture_decision_1251.md`

## 5. 후속 작업 후보

1. legacy OLE chart style/object graph parser 확장
2. axis nice scale 및 tick interval 복원
3. palette/series style 복원
4. legend/title/plot area 복원
5. chart visual model의 PageLayerTree `PaintOp` lowering
6. PDF/SVG visual comparison harness 추가
