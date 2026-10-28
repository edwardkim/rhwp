# Task M100 #2277 단계별 완료보고서 — 1단계: 마커 인프라 (× 교체) + scatter 마커 사이클

- 이슈: #2277 (C2a stock HLC 렌더 + 2D fidelity 정합)
- 브랜치: `local/task2277`
- 단계: 1/5
- 작성일: 2026-07-15

## 구현 내용

### `src/ooxml_chart/renderer.rs`

- **헬퍼 `marker_path(si, cx, cy, r) -> (String, bool)` 추출**: `push_line_marker`에 인라인돼
  있던 마커 경로 생성을 순수 함수로 분리 + 반경 `r` 파라미터화 (■/×는 하프폭 `r-0.5` —
  종전 ◆3.5/■3.0 비율 유지, 라인 마커 출력 바이트 보존). 반환 튜플의 bool은
  stroke 기반 여부(× 전용).
- **사이클 4번째 원 폴백 → ×**: `si%4==3`을 두 대각선 열린 경로
  (`M..L.. M..L..`)로 교체 — OHLC 종가 정답지 실측(시가고가저가종가-2022.pdf, 노란 ×).
  ×는 채우면 안 보이므로 `fill="none" stroke={계열색} stroke-width="1.5"`로 방출.
- **헬퍼 `push_marker(svg, class, si, cx, cy, r, color)` 신설**: 채움형(◆■▲)/stroke
  기반(×) 방출 분기 + `class` 파라미터 — 데이터 마커(`hwp-chart-marker`)와 4단계의
  범례 글리프(`hwp-legend-glyph`)를 분리해 issue_2129 마커 카운트 오염을 차단.
  `push_line_marker`는 `push_marker(…, "hwp-chart-marker", …, 3.5, …)` 위임으로 축소.
- **`render_scatter` 마커 사이클화**: `<circle r=3>` → `push_marker(…, si, …, 4.5, …)` —
  정답지 실측(표식만있는분산형-2022.pdf: 계열1 ◆ 파랑/계열2 ■ 주황, 라인보다 큰 표식).
  반경 4.5는 실측 근사로 시각판정(5단계)에서 조정 여지.

### 테스트 (RED→GREEN)

| 테스트 | 단언 | 종별 |
|--------|------|------|
| `test_line_marker_x_series4` | 계열4 마커 skeleton `MLML`(× 열린 경로) + `fill="none"` | 반전(종전 `test_line_marker_circle_fallback_series4` — 원 arc 단언) |
| `test_scatter_markers_use_cycle` | 2계열×3점=마커 6·계열1 ◆(첫 세그먼트 대각)·계열2 ■(수평) | 신규 |
| `test_render_scatter_marker_only` | circle 부재·마커 3·연결선(`data_line_paths`) 부재 | 갱신(종전 circle 단언) |
| `test_render_scatter_line_only` | 연결선 1·마커 0 | 갱신 |
| `test_render_scatter_line_marker` | 연결선 1·마커 3 | 갱신 |
| `test_render_scatter_smooth` | 마커 3·cubic Bézier `C` 존재 | 갱신 |

기존 무회귀 핀 유지: `test_line_marker_shape_cycle`(◆■▲ 바이트 보존),
`test_line_markers_rendered`(12개), `test_line_no_markers_by_default`,
`test_render_scatter_decimal_axis_labels`(0.5 라벨).

## TDD 절차 준수

RED (구현 전) — 5건이 정확한 사유(마커가 아직 circle/원 폴백)로 실패:

```
test_line_marker_x_series4              FAILED (원 arc ≠ MLML)
test_render_scatter_marker_only         FAILED (circle 존재)
test_render_scatter_line_marker         FAILED (marker_ds 0 ≠ 3)
test_render_scatter_smooth              FAILED (marker_ds 0 ≠ 3)
test_scatter_markers_use_cycle          FAILED (marker_ds 0 ≠ 6)
→ 40 passed; 5 failed
```

GREEN (구현 후):

```
$ cargo test --lib ooxml_chart
test result: ok. 73 passed; 0 failed
```

## 검증 결과

- [x] `cargo test --lib ooxml_chart` 73/73
- [x] 차트 통합 4스위트: `issue_2129_line_stacked`(마커 12개 카운트 무회귀 — 3계열이라
  × 무접점, 예측 적중) / `issue_1431_scatter` / `issue_1882_chart_style_gaps` /
  `issue_1453_chart_3d_ofpie_routing` 전부 통과
- [x] `cargo clippy --all-targets -- -D warnings` 무경고 (exit 0)
- [x] **전체 `cargo test` 전수 (최종 소스 상태, 로그 파일 클린 실행)**: 260스위트 중
  259 ok + `svg_snapshot` 2건 실패 → 아래 환경 이슈로 판명·해소 후 **8/8 재통과**.
  최종 집계 **3,156 passed / 0 failed**.
- [x] fmt: 수정 파일(renderer.rs)만 정리, `cargo fmt --check`에서 본 파일 지적 0건.
  (mod.rs/parser.rs 등 타 파일의 "Incorrect newline style"은 Windows 체크아웃
  autocrlf로 인한 기존 환경 노이즈 — 본 작업과 무관, 미변경)

### svg_snapshot 2건 실패 — 선재 환경 이슈 판명 (본 변경 무관)

`issue_267_ktx_toc_page`·`issue_147_aift_page3` 골든 불일치는 **내용 차이 0건**
(`diff --strip-trailing-cr` 무차이) + 두 문서에 차트 요소 자체가 없음(`hwp-ooxml-chart`
0건)으로 본 변경과 무관 확인. 원인: `.gitattributes`의 `tests/golden_svg/**/*.svg text
eol=lf`(#1786) 적용 **이전에 체크아웃된 워킹카피 2개만 CRLF 잔존**(`git ls-files --eol`
= `w/crlf`) → 렌더러 LF 출력과 바이트 비교 실패. 해당 파일 재체크아웃으로 워킹카피
정규화(커밋 대상 아님 — 인덱스 `i/lf` 불변) 후 `svg_snapshot` 8/8 통과.

## 비고

- `render_combo`의 라인 점 circle(r=2.5)은 접점 없음(콤보 범위 외 — 구현계획서 명기).
- scatter 통합 가드(`issue_1431_scatter.rs`)는 placeholder/클래스만 검사해 갱신 불요 —
  구현계획서의 "필요 시 핀 갱신" 항목은 렌더러 유닛 4건 갱신으로 소화.

## 다음 단계

2단계 — stock 2종: 파서 `stockChart`/`hiLowLines`/`upDownBars`/`marker_symbol` +
`OoxmlChartType::Stock` + `render_stock`(고저선/캔들/종가 마커/전용 +1 step 축) +
`tests/issue_2277_stock.rs`.
