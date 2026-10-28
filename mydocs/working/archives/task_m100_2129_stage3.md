# Task M100 #2129 단계별 완료보고서 — 3단계: 표식(마커) 렌더

- 이슈: #2129 (C1d 라인 누적 + 표식)
- 브랜치: `local/task2129`
- 단계: 3/4
- 작성일: 2026-07-09

## 구현 내용

### `src/ooxml_chart/renderer.rs`

- **헬퍼 `push_line_marker(svg, si, cx, cy, color)` 신설** (`polyline_path` 앞):
  계열 인덱스 `si % 4` → ◆ 다이아몬드(r=3.5) / ■ 정사각형(half=3.0) / ▲ 삼각형(r=3.5)
  / 원 폴백(r=3, 계열 4+ — 코퍼스 밖). 정답지 PDF 실측 사이클(표식이있는누적꺽은선형:
  계열1 ◆/계열2 ■/계열3 ▲). 출력: `<path class="hwp-chart-marker" d="…" fill={계열색}
  stroke="#fff" stroke-width="1"/>`.
- **`render_line` 배선**: 시리즈 path 출력 직후 `chart.line_markers`이면 각 점에
  마커 — Stage 2에서 수집한 `points` 좌표 재사용(누적/비누적 모두 선 위 정확 위치).

### 단위 테스트 4건 (+ 헬퍼 `marker_ds`)

| 테스트 | 단언 |
|--------|------|
| `test_line_markers_rendered` | 누적 + line_markers=true → 마커 12개(3계열×4점) |
| `test_line_marker_shape_cycle` | [0]=◆(4각·첫 세그먼트 대각), [4]=■(4각·첫 세그먼트 수평), [8]=▲(3각) |
| `test_line_marker_circle_fallback_series4` | 계열4 → 원(arc) 폴백 |
| `test_line_no_markers_by_default` | 기본값 → `hwp-chart-marker` 부재 (꺽은선형/누적꺽은선형 무회귀) |

## TDD 절차 준수

RED (구현 전): 마커 3건 정확한 사유(마커 코드 부재)로 실패 —

```
test_line_markers_rendered              FAILED
test_line_marker_shape_cycle            FAILED
test_line_marker_circle_fallback_series4 FAILED
→ 0 passed; 3 failed
```

GREEN (구현 후):

```
$ cargo test --lib ooxml_chart
test result: ok. 71 passed; 0 failed
```

## Stage 2 이월 항목 — 전체 스위트 클린 재검증

2단계 보고서에서 파이프 절단으로 불완전했던 전수 로그를 파이프 없는 클린 실행으로
재검증 완료 (Stage 2 커밋 상태 기준):

```
$ cargo test > full_test_clean.log 2>&1
cargo exit: 0
230개 스위트 전부 ok — 2,984 passed / 0 failed
```

## 완료 기준 충족

- [x] `cargo test --lib ooxml_chart` 통과 (71/71)
- [x] 기본값 무마커 → 표식 없는 3종(꺽은선형·누적·백프로) 출력 불변
- [x] 콤보의 `line_markers` 무영향 (`render_combo` 미참조 — 파서 주석 명기)
- [x] `cargo clippy --all-targets -- -D warnings` 무경고 (exit 0)

## 다음 단계

4단계 — 통합 테스트(`tests/issue_2129_line_stacked.rs`, 5종×2포맷 10파일) + 전체
게이트(cargo test/clippy) + `output/poc/chart_c1d/` 시각판정 산출물 + 정답지 대조표.
