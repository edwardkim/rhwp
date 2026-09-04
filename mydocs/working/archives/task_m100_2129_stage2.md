# Task M100 #2129 단계별 완료보고서 — 2단계: render_line 누적/백프로 기하 + 축

- 이슈: #2129 (C1d 라인 누적 + 표식)
- 브랜치: `local/task2129`
- 단계: 2/4
- 작성일: 2026-07-09

## 구현 내용

### `src/ooxml_chart/renderer.rs` — `render_line` 재작성

- **flags**: `stacked`/`percent`를 `chart.line_grouping`에서 도출 (render_bars 관용구 미러).
- **값축 분기** (render_bars 누적 정책 미러, 새 축 기계장치 없음):
  - percent → `(0.0, 100.0, 20.0)` 고정 (정답지: 0%~100% step 20%)
  - stacked → `nice_axis(0, max category_positive_sum, VERTICAL_AXIS_TICKS)` (정답지:
    합 12.3 → 0~15 step 5)
  - 비누적 → `value_range` 현행 유지
- `render_value_grid`에 `percent` 플래그 전달 → % 축 라벨 재사용.
- **누적 기하**: `cum: vec![0.0; max_len]` 값공간 누적 — `cum[i] += v.max(0.0)`
  (음수 clamp, render_bars 동일 정책). percent는 `cum/카테고리합*100`, 합≤0→0.0 가드
  (막대의 `denom=1.0` 가드와 동등한 출력).
- **path 조립 정리**: 인라인 문자열 조립 → `points: Vec<(f64,f64)>` 수집 +
  `polyline_path` 재사용 (동일 `{:.2}` 포맷 = 비누적 출력 바이트 동일, 3단계 마커가
  같은 좌표 사용 예정).

### 단위 테스트 6건 (+ 헬퍼 `line_chart`/`data_line_paths`/`path_points`)

`line_chart(grouping)`: 3계열×4카테고리, 카테고리 합 8.7/8.9/8.3/12.3 (코퍼스 라인
샘플과 동일 스케일), 개별값 최대 5.0 → 비누적 축 0~6 vs 누적 축 0~15로 판별 가능.

| 테스트 | 단언 |
|--------|------|
| `test_line_stacked_axis_from_category_sum` | `>15<` 존재, `>6<` 부재(개별값 축 미사용), `>14<` 부재(headroom 미발동) |
| `test_line_stacked_series_order` | 시리즈2 첫 점(누적 6.7) y < 시리즈1(4.3) y — 위에 쌓임 |
| `test_line_percent_axis_labels` | `100%`·`20%` 라벨 |
| `test_line_percent_top_series_flat` | 최상위 시리즈 y 4점 전부 동일(100% 수평선 — 정답지 정합) |
| `test_line_percent_zero_sum_category_no_nan` | 합 0 카테고리 → NaN 부재 |
| `test_line_clustered_unchanged` | 비누적 무회귀 핀: `>6<` 존재·`>15<` 부재·시리즈1이 위 |

## TDD 절차 준수

RED (구현 전): 누적/percent 동작 필요 4건이 정확한 사유로 실패, 현행 고정 2건
(clustered/zero-sum)은 통과 —

```
test_line_stacked_axis_from_category_sum   FAILED
test_line_stacked_series_order             FAILED
test_line_percent_axis_labels              FAILED
test_line_percent_top_series_flat          FAILED  (y: 188→188→143→53 — 독립 선)
→ 2 passed; 4 failed
```

GREEN (구현 후):

```
$ cargo test --lib ooxml_chart
test result: ok. 67 passed; 0 failed          ← 2단계 게이트

$ cargo clippy --all-targets -- -D warnings   → 무경고 (exit 0)
```

전체 스위트: 1차 실행에서 lib 포함 2,283건·30개 스위트 통과(실패 0) 관찰 + 226개
테스트 바이너리 전부 순차 완주 확인. 단, 로그 수집 명령의 파이프 절단(head)으로
전수 로그가 불완전하여 **파이프 없는 클린 재실행을 백그라운드로 진행 중** — 결과는
3단계 착수 전 확인하고 3단계 보고서에 기재한다. (전수 게이트 자체는 4단계 완료 기준.)

## 완료 기준 충족

- [x] `cargo test --lib ooxml_chart` 통과 (67/67 — 막대/scatter/콤보 회귀 없음)
- [x] `cargo clippy --all-targets -- -D warnings` 무경고
- [x] 비누적 경로 출력 불변 (`polyline_path` 동일 포맷 + `test_line_clustered_unchanged` 핀)
- [ ] 전체 스위트 클린 로그 — 재실행 중 (3단계 착수 전 확인)

## 다음 단계

3단계 — 표식(마커) 렌더: `push_line_marker` 헬퍼(◆■▲+원 폴백) + `line_markers` 배선.
