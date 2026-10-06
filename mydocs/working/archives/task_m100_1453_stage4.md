# Task M100 #1453 단계별 완료보고서 — 4단계: 파서 grouping 파싱 + 모델 필드

- 이슈: #1453 (Part B 막대 누적 보정)
- 브랜치: `local/task1453`
- 단계: 4/6
- 작성일: 2026-06-21

## 구현 내용

### (a) `src/ooxml_chart/mod.rs` — 모델 확장

- `OoxmlChart`에 `grouping: BarGrouping` 필드 추가 (막대 렌더러만 사용, line/pie 무관).
- 신규 enum:
  ```rust
  pub enum BarGrouping { #[default] Clustered, Stacked, PercentStacked }
  ```
  `clustered`/`standard` → Clustered 흡수.

### (b) `src/ooxml_chart/parser.rs` — `c:grouping` 파싱

- import에 `BarGrouping` 추가.
- `handle_start`에 `b"grouping"` 분기: **막대(bar/bar3D) plot 내에서만** 채택
  (`cur_plot_type`이 Column/Bar일 때). line의 grouping은 무시(C1d 후속).
  `stacked`→Stacked, `percentStacked`→PercentStacked, 그 외→Clustered.

### (c) 단위 테스트 4건

| 테스트 | 입력 | 기대 |
|--------|------|------|
| `test_parse_grouping_stacked` | barChart + grouping=stacked | `Stacked` |
| `test_parse_grouping_percent_stacked` | bar3DChart + grouping=percentStacked | `PercentStacked` |
| `test_parse_grouping_clustered_default` | clustered 명시 / grouping 없음 | `Clustered` |
| `test_parse_grouping_line_ignored` | lineChart + grouping=stacked | `Clustered`(막대 미반영) |

## 검증 결과

```
$ cargo test --lib ooxml_chart::parser
test result: ok. 13 passed; 0 failed   (기존 9 + 신규 4)

$ cargo clippy --lib   → ooxml_chart 무경고
```

### 실샘플 grouping 확인 (HWPX Chart XML)

| 샘플 | grouping | barDir |
|------|----------|--------|
| 3차원누적세로막대형 | stacked | col |
| 3차원묶은세로막대형 | clustered | col |
| 누적세로막대형 (2D) | stacked | col |
| 백프로기준누적세로막대형 (2D) | percentStacked | col |
| 3차원누적가로막대형 | stacked | bar |
| 백프로기준누적가로막대형 (2D) | percentStacked | bar |

→ 6종 모두 예상 grouping 보유. 5단계 렌더 분기 입력 확정.

## 완료 기준 충족

- [x] `cargo test ooxml_chart` 통과 (13/13)
- [x] 빌드/clippy 무경고
- [x] 실샘플 grouping 값 확인

## 다음 단계

5단계 — `render_bars`에 Stacked(누적)/PercentStacked(100%) 분기 + 값축 grouping-aware.
