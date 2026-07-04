# task_m100_1660 Stage 4 완료 보고서 — 통합 테스트 + 시각 검증

- 이슈: #1660 (C1b, #1431 Track C)
- 브랜치: `local/task1660`
- 단계: Stage 4 / 4 — 통합 테스트 + 시각 검증

## 변경 내용

1. **신규 통합 테스트** `tests/issue_1431_scatter.rs` (issue_1453 미러):
   분산형 5종 × {hwpx, hwp} = 10파일 각각 page0 SVG가 `차트 (미지원)` 미포함 + `hwp-ooxml-chart"` 포함 + fallback 아님.
2. **시각 검증 기반 0-baseline 튜닝** (`renderer.rs` `scatter_range`):
   한컴 2022 정답지(`pdf/chart/분산형/표식만있는분산형-2022.pdf`) 대조 결과 **X·Y 축이 모두 0부터**(X 0~3, Y 0~5)임을 확인.
   계획서 D항(“PDF가 0을 포함하면 0-clamp로 조정”)에 따라 `scatter_range`에 `if min > 0 { min = 0 }` 추가.
   → 막대/선 축(`value_range_for`)과 동일한 0-baseline으로 **차트 종류 간 일관성**도 확보.
   렌더러 테스트 1개 추가(`test_render_scatter_zero_baseline`), 소수 라벨 테스트 갱신(0-clamp 후 눈금 `>2.4<`).

## 시각 검증 (output/poc/c1b_scatter, pdf/chart/분산형 대조)

5종 SVG 생성·구조 분석 (`<g class="hwp-ooxml-chart">` 기준):

| 샘플 (scatterStyle) | circle | scatter path | bezier C | 판정 |
|---|---|---|---|---|
| 표식만 (marker) | 6 | 0 | 0 | 점만 ✓ |
| 직선이있는 (line) | 0 | 2 | 0 | 직선만 ✓ |
| 직선및표식 (lineMarker) | 6 | 2 | 0 | 직선+점 ✓ |
| 곡선이있는 (smoothMarker) | 6 | 2 | 2 | 곡선+점 ✓ |
| 곡선및표식 (smoothMarker) | 6 | 2 | 2 | 곡선+점 ✓ |

- **데이터·기하**: 정답지와 일치 (Y1 (0.7,2.7)(1.8,3.2)(2.6,0.8), Y2 (0.7,1)(1.8,2)(2.6,4)).
- **축**: 0-baseline 적용 후 X 0~3 (정답지 정확 일치), Y 0~4 (정답지 0~5, +1틱 헤드룸 차이).
- **소수 축 라벨**: 정상 (X 0/0.6/1.2/1.8/2.4/3, Y 0/0.8/1.6/2.4/3.2/4).

### 잔여 스타일 4갭 (C1c, 전 13종 공통 → 분산형도 동일)
① 제목(“차트 제목”) 누락 ② 팔레트(녹색-우선 vs 한컴 파랑/주황) ③ 범례 하단 vs 우측
④ Y축 상한 nice-scale 헤드룸(4 vs 5; 최상단 점이 경계에 붙음). 모두 #1431 C1c 후속이며 C1b(커버리지) 범위 외.

## 검증

```
cargo test -p rhwp ooxml_chart::         → 32 passed; 0 failed (scatter parser 5 + renderer 6 포함)
cargo test --test issue_1431_scatter     → 1 passed (10파일)
cargo test (전체)                         → 0 failed
cargo clippy --all-targets -- -D warnings → Finished (경고 0)
```

## 완료 기준 충족
- 분산형 5종 export-svg **“차트 (미지원)” placeholder 0건** ✓ (C1b 완료기준)
- 데이터·기하 정확 + 합리적 스타일(점/직선/곡선 구분, 0-baseline 수치축) ✓
- 픽셀 parity는 1차 목표 아님(#1251); 스타일 4갭은 C1c 후속.

## 다음
최종 결과보고서(`mydocs/report/task_m100_1660_report.md`) → 승인 후 upstream `devel` PR(`Refs #1431`, `#1660`).
