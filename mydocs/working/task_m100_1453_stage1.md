# Task M100 #1453 단계별 완료보고서 — 1단계: 파서 라우팅 + 단위 테스트

- 이슈: #1453
- 브랜치: `local/task1453`
- 단계: 1/3 (파서 라우팅 + 단위 테스트)
- 작성일: 2026-06-21

## 구현 내용

`src/ooxml_chart/parser.rs` 에 3D막대·3D원형·ofPie 요소명 라우팅 추가.

### (a) `handle_start` — 요소명 3개 추가 (`pieChart` 분기 직후)

| 요소 | 라우팅 | 비고 |
|------|--------|------|
| `bar3DChart` | `chart_type = Column`, `cur_plot_type = Column` | `barDir` 핸들러가 col/bar 채움 → 후처리(`parser.rs:87-107`)가 Column↔Bar 확정 |
| `pie3DChart` | `chart_type = Pie`, `cur_plot_type = Pie` | 단일 원형 2D 근사 |
| `ofPieChart` | `chart_type = Pie`, `cur_plot_type = Pie` | 보조플롯 미표현(C2), 전 데이터 단일 원형 근사 |

기존 `barChart`/`pieChart` 분기와 동일한 4줄 셋업(`chart_type`/`cur_plot_type`/`ax_ids.clear()`/`series_start`)을 미러.

### (b) `handle_end` — plot-종료 arm 확장

```rust
b"barChart" | b"lineChart" | b"pieChart" | b"bar3DChart" | b"pie3DChart" | b"ofPieChart" => { ... }
```

axId 일괄 할당 로직을 3D/ofPie plot에도 동일 적용.

### (c) 단위 테스트 4건 추가

| 테스트 | 입력 | 기대 |
|--------|------|------|
| `test_parse_bar3d_col` | `bar3DChart` + `barDir=col` | `Column`, values=[100,80] |
| `test_parse_bar3d_bar` | `bar3DChart` + `barDir=bar` | `Bar` |
| `test_parse_pie3d` | `pie3DChart` | `Pie`, values=[30,70] |
| `test_parse_ofpie` | `ofPieChart` + `ofPieType=pie` | `Pie`, values=[40,25,35] |

**렌더러(`renderer.rs`)·데이터 모델(`mod.rs` enum)은 변경 없음.**

## 검증 결과

```
$ cargo test --lib ooxml_chart::parser
test result: ok. 9 passed; 0 failed   (기존 5 + 신규 4)
  test_parse_bar3d_col ... ok
  test_parse_bar3d_bar ... ok
  test_parse_pie3d ... ok
  test_parse_ofpie ... ok

$ cargo clippy --lib
Finished — ooxml_chart 경고 0
```

## 완료 기준 충족

- [x] `cargo test ooxml_chart` 통과 (9/9)
- [x] `cargo build` 통과 (clippy 빌드 동반)
- [x] clippy 무경고 (ooxml_chart 한정)

## 다음 단계

2단계 — 실샘플 회귀 통합 테스트(7종×2포맷=14파일) + `mod.rs` 지원 범위 주석 갱신.
