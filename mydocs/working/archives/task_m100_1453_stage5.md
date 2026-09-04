# Task M100 #1453 단계별 완료보고서 — 5단계: render_bars 누적/백프로 렌더 + 값축

- 이슈: #1453 (Part B 막대 누적 보정)
- 브랜치: `local/task1453`
- 단계: 5/6
- 작성일: 2026-06-21

## 구현 내용

### `src/ooxml_chart/renderer.rs`

1. **`render_bars` 분기** (`chart.grouping` 기준):
   - **Clustered**(기존): side-by-side, `value_range`.
   - **Stacked**: 카테고리당 단일 막대, 시리즈를 아래/왼쪽부터 누적. 값축 max = `nice_range(0, 카테고리 합의 최대)`.
   - **PercentStacked**: 카테고리 합으로 정규화(누적=플롯 전체 길이=100%). 값축 0~100%.
   - 가로(`horizontal`)도 동일 원리(왼→오 누적).
2. **`category_positive_sum(chart, ci)`** 헬퍼 추가 — 카테고리별 (양수) 시리즈 합. 축·정규화 공용.
3. **`render_value_grid`에 `percent: bool` 파라미터 추가** — true면 라벨을 `{v}%`로. 호출처 4곳 갱신
   (render_bars=percent, render_line/render_combo×2=false). `#[allow(clippy::too_many_arguments)]`.

Pie/Line 경로 무영향. 음수 시리즈는 0으로 클램프(코퍼스 전 양수, 음수 누적은 범위 밖).

## 검증 결과 (시각)

| 종류 | grouping | 렌더 결과 | 값축 |
|------|----------|----------|------|
| 3차원누적세로막대형 | stacked | ✅ 단일 컬럼 누적 (초/파/주 적층) | 0~14 (합계 스케일) |
| 백프로기준누적세로막대형 | percentStacked | ✅ 모든 항목 100% 꽉 참 | **0%~100%** |
| 3차원누적가로막대형 | stacked | ✅ 왼→오 누적 | 0~14 |
| 3차원묶은세로막대형 | clustered | ✅ **여전히 grouped**(무회귀) | 0~5 |

산출물: `output/poc/chart_c1a/{종류}/rhwp.png`.

## 검증 결과 (테스트)

```
$ cargo build --bin rhwp   → 통과
$ cargo clippy --lib       → ooxml_chart 무경고
$ cargo test --test issue_1453_chart_3d_ofpie_routing   → 1 passed (14파일 fallback 0 유지)
$ cargo test --test issue_1156_chart_column_flow        → 2 passed (clustered 무회귀)
$ cargo test --test issue_1251_ole_chart_contents       → 10 passed
$ cargo test --lib ooxml_chart                          → 18 passed
```

## 완료 기준 충족

- [x] stacked → 누적 막대, percentStacked → 100% 꽉 찬 막대
- [x] clustered 무회귀 / pie·line 무영향
- [x] 기존 차트 테스트 회귀 없음 / clippy 무경고

## 다음 단계

6단계 — 누적 기하 회귀 가드(stacked=시리즈 같은 x 공유, percent=전체 높이) + 6종 시각판정 산출 + 전체 스위트.
